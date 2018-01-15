// gate_demo, a demo game built using the "gate" game library.
// Copyright (C) 2017-2018  Matthew D. Michelotti
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

mod background;
mod builder;
mod button;
mod cell;
mod effect;
mod step_queue;
mod piece_profile;
mod platform;
mod player_enum;
mod player;
mod star;
mod util;
mod warp;

use std::collections::HashMap;
use std::f64;

use gate::Audio;
use gate::renderer::Renderer;

use collider::{Collider, HbId, HbVel, HbEvent, HbProfile};
use collider::geom::{v2, Vec2, Card, CardMask, Shape};

use game_input::{InputEvent, HorizDir};
use asset_id::{AssetId, SoundId};
use self::piece_profile::{PieceKind, PieceProfile};
use self::player_enum::{PlayerEnum, PlayerComplete, PlayerWarping};
use self::player::Player;
use self::step_queue::{StepQueue, Step};
use self::cell::{Cell, CellKind};
use self::effect::Effect;
use self::star::Star;
use self::platform::Platform;
use self::button::ButtonAction;
use self::util::{IdGen, idx_to_vec, vec_to_affine, card_offset};
use self::warp::Lasor;

pub use self::builder::GameBoardBuilder;
pub use self::platform::PlatformKind;
pub use self::warp::{WarpColor, LasorKind};

pub type Idx2 = (i32, i32);

const CELL_LEN: i32 = 8;
pub const SCREEN_PIXELS_HEIGHT: f64 = CELL_LEN as f64 * 24.;

pub struct GameBoard {
    id_gen: IdGen,
    collider: Collider<PieceProfile>,
    move_dir: Option<HorizDir>,
    player: PlayerEnum,
    star: Star,
    grid: HashMap<Idx2, Cell>,
    room_dims: Idx2,
    platforms: HashMap<HbId, Platform>,
    step_queue: StepQueue,
    buttons: HashMap<HbId, ButtonAction>,
    effects: Vec<Effect>,
    warps: HashMap<HbId, WarpColor>,
    respawns: HashMap<WarpColor, Vec2>,
    lasors: Vec<Lasor>,
}

impl GameBoard {
    pub fn builder(dims: Idx2) -> GameBoardBuilder { GameBoardBuilder::new(dims) }
    pub fn is_done(&self) -> bool { self.time() > self.star.level_end_time() }

    fn room_pixels(&self) -> Vec2 { v2((self.room_dims.0 * CELL_LEN) as f64, (self.room_dims.1 * CELL_LEN) as f64) }
    fn player_pos(&self) -> Vec2 { self.player.pos(&self.collider) }
    fn time(&self) -> f64 { self.collider.time() }

    pub fn input(&mut self, event: InputEvent) {
        match event {
            InputEvent::UpdateMovement(dir) => self.move_dir = dir,
            _ => {},
        }
        if let PlayerEnum::Normal(ref mut player) = self.player {
            match event {
                InputEvent::UpdateMovement(dir) => player.set_movement(dir),
                InputEvent::PressJump => player.press_jump(),
                InputEvent::ReleaseJump => player.release_jump(),
            }
        }
    }

    pub fn advance(&mut self, elapsed: f64, audio: &mut Audio<AssetId>) {
        let end_time = self.time() + elapsed;
        if self.time() == 0.0 && end_time > 0.0 { audio.play_sound(SoundId::Clear); }
        while self.time() < end_time {
            let collider_time = self.collider.next_time();
            let event_time = self.step_queue.peek();
            let player_transition_time = self.player.transition_time();
            let time = collider_time.min(event_time).min(end_time).min(player_transition_time);
            self.collider.set_time(time);
            if let PlayerEnum::Normal(ref mut player) = self.player { player.set_time(time); }
            if time == event_time {
                match self.step_queue.pop() {
                    Step::Player => self.player_step(audio),
                    Step::Platform => self.platform_step(),
                    Step::WarpEffectSpawn => self.warp_effect_step(),
                    Step::LasorFire => self.lasors_step(audio),
                }
            } else if time == player_transition_time {
                self.player_transition(audio);
            } else if let Some((hb_event, p_1, p_2)) = self.collider.next() {
                self.handle_hb_event_asym(hb_event, p_1, p_2, audio);
                self.handle_hb_event_asym(hb_event, p_2, p_1, audio);
            }
        }
    }

    fn player_transition(&mut self, audio: &mut Audio<AssetId>) {
        if let PlayerEnum::Warping(_) = self.player { audio.play_sound(SoundId::Warp) }
        let pos = self.player_pos();
        let (player, shape) = Player::new(self.id_gen.next(), pos, self.collider.time(), self.move_dir);
        let hitbox = shape.still_until(self.step_queue.peek_specific(Step::Player));
        let overlaps = self.collider.add_hitbox(PieceProfile::new(player.id(), PieceKind::Player), hitbox);
        assert!(overlaps.iter().all(|p| p.kind == PieceKind::Platform), "unexpected overlap with new player");
        self.player = PlayerEnum::Normal(player);
    }

    fn handle_hb_event_asym(&mut self, event: HbEvent, p_1: PieceProfile, p_2: PieceProfile, audio: &mut Audio<AssetId>) {
        match p_1.kind {
            PieceKind::Player => match p_2.kind {
                PieceKind::Wall | PieceKind::Floor | PieceKind::Platform => self.update_player_barriers(),
                PieceKind::Button if event == HbEvent::Collide => self.press_button(p_2.id(), audio),
                PieceKind::Warp if event == HbEvent::Collide => self.warp(p_2.id(), audio),
                PieceKind::Star if event == HbEvent::Collide => self.obtain_star(audio),
                _ => {},
            },
            PieceKind::Warp if event == HbEvent::Collide => match p_2.kind {
                PieceKind::Wall => self.warp_hits_wall(p_1.id, p_2.id, CardMask::full()),
                PieceKind::Floor | PieceKind::Platform => self.warp_hits_wall(p_1.id, p_2.id, Card::PlusY.into()),
                _ => {},
            },
            _ => {},
        }
    }

    fn update_player_barriers(&mut self) {
        let is_near_ground = self.check_player_near_ground();
        if let PlayerEnum::Normal(ref mut player) = self.player {
            let player_shape = self.collider.get_hitbox(player.id()).value;
            let barrier_prs = self.collider.get_overlaps(player.id());
            let (grid, collider) = (&self.grid, &self.collider);
            let barriers = barrier_prs.iter().filter_map(|pr| match pr.kind {
                PieceKind::Floor | PieceKind::Platform => Some((collider.get_hitbox(pr.id), Card::PlusY.into())),
                PieceKind::Wall if pr.index.is_some() => {
                    let wall_hitbox = collider.get_hitbox(pr.id);
                    let player_above_wall = player_shape.masked_normal_from(
                        &wall_hitbox.value, Card::PlusY.into()).len() < PieceProfile::padding();
                    let mask = wall_card_mask(grid, pr.index.unwrap(), player_above_wall);
                    if mask == CardMask::empty() { None } else { Some((wall_hitbox, mask)) }
                },
                PieceKind::Wall => Some((collider.get_hitbox(pr.id), CardMask::full())),
                _ => None,
            });
            player.update_barriers(player_shape, is_near_ground, barriers);
        } else {
            unreachable!();
        }
        self.update_player_hitbox_vel();
    }

    fn check_player_near_ground(&self) -> bool {
        if let PlayerEnum::Normal(ref player) = self.player {
            let player_shape = self.collider.get_hitbox(player.id()).value;
            let padding = PieceProfile::padding();
            let test_shape = Shape::rect(Vec2::zero())
                                   .place(player_shape.pos + v2(0., -0.5 * player_shape.dims().y - padding));
            let mut overlaps = self.collider.query_overlaps(&test_shape, &PieceProfile::new(player.id(), PieceKind::Player));
            let result = overlaps.drain(..).any(|p| {
                p.kind == PieceKind::Wall || p.kind == PieceKind::Floor
            });
            result
        } else {
            unreachable!()
        }
    }

    fn update_player_hitbox_vel(&mut self) {
        if let PlayerEnum::Normal(ref player) = self.player {
            let next_time = self.step_queue.peek_specific(Step::Player);
            self.collider.set_hitbox_vel(player.id(), HbVel::moving_until(player.vel(), next_time));
        }
    }

    fn obtain_star(&mut self, audio: &mut Audio<AssetId>) {
        let time = self.time();
        self.star.obtain(time);
        let pos = self.player_pos();
        let (tex, mirror) = if let PlayerEnum::Normal(ref mut player) = self.player {
            self.collider.remove_hitbox(player.id());
            player.tex_and_mirror()
        } else {
            unreachable!()
        };

        self.player = PlayerEnum::Complete(PlayerComplete::new(pos, time, tex, mirror));
        audio.play_sound(SoundId::Clear);
    }

    fn warp(&mut self, warp_id: HbId, audio: &mut Audio<AssetId>) {
        audio.play_sound(SoundId::Warp);
        let color = *self.warps.get(&warp_id).unwrap();
        if self.collider.get_hitbox(warp_id).vel.value != Vec2::zero() {
            self.collider.remove_hitbox(warp_id);
            self.warps.remove(&warp_id);
        }
        let start_pos = self.player_pos();
        let end_pos = self.respawns[&color];
        if let PlayerEnum::Normal(ref player) = self.player { self.collider.remove_hitbox(player.id()); } else { unreachable!() }
        self.player = PlayerEnum::Warping(PlayerWarping::new(start_pos, end_pos, color, self.time()));
    }

    fn warp_hits_wall(&mut self, warp_id: HbId, wall_id: HbId, card_mask: CardMask) {
        let warp_shape = self.collider.get_hitbox(warp_id).value;
        let wall_shape = self.collider.get_hitbox(wall_id).value;
        let normal = warp_shape.masked_normal_from(&wall_shape, card_mask);
        if normal.len() < PieceProfile::padding() {
            self.warps.remove(&warp_id);
            self.collider.remove_hitbox(warp_id);
            let pos = warp_shape.pos - normal.dir() * 0.5 * warp_shape.dims().x;
            let angle = normal.dir().y.atan2(normal.dir().x) - 0.5 * f64::consts::PI;
            self.effects.push(effect::puff(pos, self.collider.time(), angle));
        }
    }

    fn press_button(&mut self, button_id: HbId, audio: &mut Audio<AssetId>) {
        audio.play_sound(SoundId::Button);
        self.collider.remove_hitbox(button_id);
        let mut actions = self.buttons.remove(&button_id).unwrap();
        for pos in actions.unlock_cells.drain(..) { self.remove_cell(pos); }
        for (pos, kind) in actions.platforms.drain(..) { self.add_platform(pos, kind); }
    }

    fn remove_cell(&mut self, pos: Idx2) {
        let cell = self.grid.remove(&pos).unwrap();
        let overlaps = self.collider.remove_hitbox(cell.id());
        assert!(overlaps.is_empty(), "unexpected overlap with removed cell");
        self.effects.push(effect::square_fade(idx_to_vec(pos), self.collider.time()));
    }

    fn add_platform(&mut self, pos: Idx2, kind: PlatformKind) {
        let update_time = self.step_queue.peek_specific(Step::Platform);
        for (platform, hitbox) in Platform::new(kind, pos, self.time(), update_time) {
            let id = self.id_gen.next();
            self.platforms.insert(id, platform);
            self.collider.add_hitbox(PieceProfile::new(id, PieceKind::Platform), hitbox);
        }
    }

    fn add_warp(&mut self, pos: Vec2, color: WarpColor, vel: Vec2, audio: Option<&mut Audio<AssetId>>) {
        let id = self.id_gen.next();
        self.warps.insert(id, color);
        let hitbox = warp::shape().place(pos).moving(vel);
        let overlaps = self.collider.add_hitbox(PieceProfile::new(id, PieceKind::Warp), hitbox);
        let mut warping = false;
        for overlap in overlaps {
            match overlap.kind {
                PieceKind::Platform | PieceKind::Floor => {},
                PieceKind::Player => warping = true,
                _ => panic!("unexpected overlap with warp"),
            }
        }
        if warping { self.warp(id, audio.expect("unexpected warping with audio unavailable")) };
    }

    fn player_step(&mut self, audio: &mut Audio<AssetId>) {
        if let PlayerEnum::Normal(ref mut player) = self.player { player.step(audio); }
        self.update_player_hitbox_vel();
    }

    fn platform_step(&mut self) {
        let time = self.time();
        let next_time = self.step_queue.peek_specific(Step::Platform);
        for (&id, platform) in self.platforms.iter() {
            let mut hitbox = self.collider.get_hitbox(id);
            hitbox.vel = platform.step(hitbox.value.pos, time, next_time);
            self.collider.set_hitbox_vel(id, hitbox.vel.clone());

            if let PlayerEnum::Normal(ref mut player) = self.player {
                if self.collider.is_overlapping(id, player.id()) {
                    let next_player_step_time = self.step_queue.peek_specific(Step::Player);
                    let player_shape = self.collider.get_hitbox(player.id()).value;
                    player.update_platform_vel(&player_shape, &hitbox);
                    self.collider.set_hitbox_vel(player.id(), HbVel::moving_until(player.vel(), next_player_step_time));
                }
            }
        }
    }

    fn warp_effect_step(&mut self) {
        let color = match self.player {
            PlayerEnum::Warping(ref player) => Some(player.color()),
            _ => None,
        };
        if let Some(color) = color {
            let pos = self.player_pos();
            self.effects.push(effect::color_fade(pos, self.collider.time(), color));
        }
    }

    fn lasors_step(&mut self, audio: &mut Audio<AssetId>) {
        if self.lasors.len() > 0 {
            audio.play_sound(SoundId::Lasor);
            let player_pos = self.player_pos();
            let mut new_warps: Vec<_> = self.lasors.iter().map(|l| l.fire(player_pos)).collect();
            for (warp_pos, warp_color, warp_vel) in new_warps.drain(..) {
                self.add_warp(warp_pos, warp_color, warp_vel, Some(audio));
            }
        }
    }

    // TODO consider only drawing tiles that are on-screen?
    pub fn draw(&mut self, renderer: &mut Renderer<AssetId>) {
        let time = self.time();
        let player_pos = self.player_pos();
        let camera = self.camera_pos(renderer.app_width());
        let screen_pixels_width = renderer.app_width();
        background::draw(renderer, camera, self.room_pixels(), time, screen_pixels_width);
        {
            let renderer = &mut renderer.tiled_mode(camera.x, camera.y);
            for (&pos, cell) in self.grid.iter() { cell.draw(renderer, pos); }
            for &button_id in self.buttons.keys() {
                button::draw(renderer, vec_to_affine(self.hb_pos(button_id)));
            }
            for lasor in &self.lasors { lasor.draw_support(renderer); }
        }
        {
            let renderer = &mut renderer.sprite_mode();
            for (&platform_id, platform) in self.platforms.iter() {
                platform.draw(renderer, vec_to_affine(self.hb_pos(platform_id) - camera), time);
            }
            let next_lasor_fire_time = self.step_queue.peek_specific(Step::LasorFire);
            for lasor in &self.lasors {
                lasor.draw(renderer, camera, time, next_lasor_fire_time, player_pos);
            }
            for (&warp_id, &warp_color) in self.warps.iter() {
                warp_color.draw_warp(renderer, vec_to_affine(self.hb_pos(warp_id) - camera), time);
            }
            self.star.draw(renderer, vec_to_affine(self.hb_pos(self.star.id()) - camera), time);
            self.effects.retain(|e| e.draw(renderer, camera, time));
            self.player.draw(renderer, vec_to_affine(player_pos - camera), time);
        }
    }

    fn camera_pos(&self, screen_pixels_width: f64) -> Vec2 {
        fn coord(player: f64, room_pixels: f64, screen_pixels: f64) -> f64 {
            player.max(0.5 * screen_pixels).min(room_pixels - 0.5 * screen_pixels)
        }
        let player = self.player_pos();
        let room_pixels = self.room_pixels();
        v2(coord(player.x, room_pixels.x, screen_pixels_width), coord(player.y, room_pixels.y, SCREEN_PIXELS_HEIGHT))
    }

    fn hb_pos(&self, id: HbId) -> Vec2 { self.collider.get_hitbox(id).value.pos }
}

fn wall_card_mask(grid: &HashMap<Idx2, Cell>, index: Idx2, player_above_wall: bool) -> CardMask {
    let mut card_mask = CardMask::empty();
    for &card in Card::values().iter() {
        let offset = card_offset(card);
        let neighbor_kind = grid.get(&(index.0 + offset.0, index.1 + offset.1)).map(|cell| cell.kind());
        card_mask[card] = match neighbor_kind {
            Some(CellKind::Wall) => false,
            Some(CellKind::Floor) => match card {
                Card::PlusY | Card::MinusY => true,
                Card::PlusX | Card::MinusX => !player_above_wall,
            },
            _ => true,
        };
    }
    card_mask
}
