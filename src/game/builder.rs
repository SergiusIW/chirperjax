// gate_demo, a demo game built using the "gate" game library.
// Copyright (C) 2017  Matthew D. Michelotti
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::collections::HashMap;
use std::mem;

use collider::{Collider, HbId};
use collider::geom::{Shape, v2, Vec2, Card};

use super::{GameBoard, Idx2, PlatformKind, CELL_LEN};
use super::player_enum::PlayerEnum;
use super::star::Star;
use super::piece_profile::{PieceProfile, PieceKind};
use super::step_queue::StepQueue;
use super::cell::Cell;
use super::button::{self, ButtonAction};
use super::warp::{WarpColor, LasorKind, Lasor};
use super::util::{IdGen, idx_to_vec, card_offset};

#[derive(Copy, Clone, PartialEq, Eq)]
enum PendingCell { Wall, Floor, Gate, Spawn(WarpColor, bool) }

pub struct GameBoardBuilder {
    id_gen: IdGen,
    collider: Collider<PieceProfile>,
    room_dims: Idx2,
    player: Option<PlayerEnum>,
    star: Option<Star>,
    platforms: Vec<(Idx2, PlatformKind)>,
    grid: HashMap<Idx2, PendingCell>,
    buttons: HashMap<u32, (Option<Idx2>, ButtonAction)>,
    warps: Vec<(Idx2, WarpColor)>,
    respawns: HashMap<WarpColor, Vec2>,
    lasors: Vec<(Idx2, LasorKind, WarpColor)>,
}

impl GameBoardBuilder {
    pub fn new(room_dims: Idx2) -> GameBoardBuilder {
        GameBoardBuilder {
            id_gen: IdGen::new(),
            collider: Collider::new(),
            room_dims,
            player: None,
            star: None,
            grid: HashMap::new(),
            platforms: Vec::new(),
            buttons: HashMap::new(),
            warps: Vec::new(),
            respawns: HashMap::new(),
            lasors: Vec::new(),
        }
    }

    fn button_mut(&mut self, index: u32) -> &mut (Option<Idx2>, ButtonAction) {
        self.buttons.entry(index).or_insert((None, ButtonAction { unlock_cells: Vec::new(), platforms: Vec::new() }))
    }

    pub fn add_player(&mut self, pos: Idx2) { self.player = Some(PlayerEnum::Start(idx_to_vec(pos))); }
    pub fn add_star(&mut self, pos: Idx2) {
        let (star, hitbox) = Star::new(self.id_gen.next(), pos);
        let overlaps = self.collider.add_hitbox(PieceProfile::new(star.id(), PieceKind::Star), hitbox);
        assert!(overlaps.is_empty(), "unexpected overlap with star hitbox");
        self.star = Some(star);
    }

    pub fn add_wall(&mut self, pos: Idx2) { self.grid.insert(pos, PendingCell::Wall); }
    pub fn add_floor(&mut self, pos: Idx2) { self.grid.insert(pos, PendingCell::Floor); }

    pub fn add_platform(&mut self, pos: Idx2, kind: PlatformKind, index: Option<u32>) {
        if let Some(index) = index {
            self.button_mut(index).1.platforms.push((pos, kind));
        } else {
            self.platforms.push((pos, kind));
        }
    }

    pub fn add_gate(&mut self, pos: Idx2, index: u32) {
        self.grid.insert(pos, PendingCell::Gate);
        self.button_mut(index).1.unlock_cells.push(pos);
    }

    pub fn add_button(&mut self, pos: Idx2, index: u32) { self.button_mut(index).0 = Some(pos) }

    pub fn add_warp(&mut self, pos: Idx2, color: WarpColor) { self.warps.push((pos, color)); }
    pub fn add_respawn(&mut self, pos: Idx2, color: WarpColor) {
        self.grid.insert(pos, PendingCell::Spawn(color, false));
        self.grid.insert((pos.0 + 1, pos.1), PendingCell::Spawn(color, true));
        self.respawns.insert(color, idx_to_vec(pos) + v2(0.5 * CELL_LEN as f64, 11.));
    }

    pub fn add_lasor(&mut self, pos: Idx2, kind: LasorKind, color: WarpColor) { self.lasors.push((pos, kind, color)); }

    pub fn build(mut self) -> GameBoard {
        let mut grid_positions: Vec<_> = self.grid.keys().cloned().collect();
        let grid = grid_positions.drain(..).map(|pos| (pos, self.form_grid_cell(pos))).collect();
        self.add_border(false);
        self.add_border(true);

        let mut builder_buttons = HashMap::new();
        mem::swap(&mut self.buttons, &mut builder_buttons);
        let buttons = builder_buttons.drain().map(|(_, (pos, action))| {
            let id = self.form_button(pos.expect("button position must be set"));
            (id, action)
        }).collect();

        let mut builder_lasors = Vec::new();
        mem::swap(&mut self.lasors, &mut builder_lasors);
        let lasors = builder_lasors.drain(..).map(|(pos, kind, color)| self.form_lasor(pos, kind, color)).collect();

        let mut board = GameBoard {
            id_gen: self.id_gen,
            collider: self.collider,
            move_dir: None,
            player: self.player.unwrap(),
            star: self.star.unwrap(),
            grid,
            room_dims: self.room_dims,
            platforms: HashMap::new(),
            step_queue: StepQueue::new(),
            buttons,
            effects: Vec::new(),
            warps: HashMap::new(),
            respawns: self.respawns,
            lasors,
        };
        for (pos, kind) in self.platforms.drain(..) { board.add_platform(pos, kind); }
        for (pos, color) in self.warps.drain(..) { board.add_warp(idx_to_vec(pos), color, Vec2::zero(), None); }
        board
    }

    fn form_button(&mut self, pos: Idx2) -> HbId {
        let id = self.id_gen.next();
        let hitbox = button::shape(pos).still();
        let profile = PieceProfile::new(id, PieceKind::Button);
        let overlaps = self.collider.add_hitbox(profile, hitbox);
        assert!(overlaps.is_empty(), "unexpected overlap with button");
        id
    }

    fn form_lasor(&mut self, pos: Idx2, kind: LasorKind, color: WarpColor) -> Lasor {
        let all_cards = Card::values();
        let card = all_cards.iter().cloned().find(|&c| {
            self.neighbor(pos, card_offset(c.flip())) == Some(PendingCell::Wall)
        }).expect("lasor was not adjacent to a wall");
        Lasor::new(pos, kind, color, card)
    }

    fn add_border(&mut self, right: bool) {
        let (width, height) = (self.room_dims.0 as f64 * 8., self.room_dims.1 as f64 * 8.);
        let shape = Shape::rect(v2(8., height));
        let x = if right { width } else { 0. };
        let shape = shape.place(v2(x, 0.5 * height));
        let pr = PieceProfile::new(self.id_gen.next(), PieceKind::Wall);
        let overlaps = self.collider.add_hitbox(pr, shape.still());
        assert!(overlaps.is_empty(), "unexpected border overlap");
    }

    fn form_grid_cell(&mut self, pos: Idx2) -> Cell {
        let kind = self.grid[&pos];
        let id = self.id_gen.next();
        let cell = match kind {
            PendingCell::Wall => {
                let mut neighbors = [false; 8];
                let neighbor_offsets = [(-1, 1), (0, 1), (1, 1), (1, 0), (1, -1), (0, -1), (-1, -1), (-1, 0)];
                for (idx, &offset) in neighbor_offsets.iter().enumerate() {
                    neighbors[idx] = match self.neighbor(pos, offset) {
                        Some(PendingCell::Wall) | Some(PendingCell::Spawn(_, _)) => true,
                        _ => false,
                    }
                }
                Cell::wall(id, neighbors)
            },
            PendingCell::Floor => {
                let neighbors = [self.neighbor(pos, (-1, 0)).is_some(), self.neighbor(pos, (1, 0)).is_some()];
                Cell::floor(id, neighbors)
            },
            PendingCell::Gate => Cell::gate(id),
            PendingCell::Spawn(color, mirror) => Cell::spawn(id, color, mirror),
        };
        let hitbox = Shape::square(CELL_LEN as f64).place(idx_to_vec(pos)).still();
        let overlaps = self.collider.add_hitbox(PieceProfile::cell(id, pos, cell.kind()), hitbox);
        assert!(overlaps.is_empty(), "unexpected overlap with grid cell");
        cell
    }

    fn neighbor(&mut self, pos: Idx2, offset: Idx2) -> Option<PendingCell> {
        let pos = (pos.0 + offset.0, pos.1 + offset.1);
        if pos.0 < 0 || pos.1 < 0 || pos.0 >= self.room_dims.0 || pos.1 >= self.room_dims.1 {
            Some(PendingCell::Wall)
        } else {
            self.grid.get(&pos).cloned()
        }
    }
}
