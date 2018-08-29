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

use gate::renderer::{SpriteRenderer, Affine};

use collider::HbId;
use collider::geom::Vec2;

use asset_id::{AssetId, SpriteId};
use super::warp::WarpColor;

#[derive(Copy, Clone)]
enum CellTransform { Id, Turn90, Turn180, Turn270, Mirror }

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CellKind { Wall, Floor }

pub struct Cell { id: HbId, kind: CellKind, tile: SpriteId, transform: CellTransform }

impl Cell {
    // neighbors flags start at the top-left neighbor and circles clockwise
    pub fn wall(id: HbId, neighbors: [bool; 8]) -> Cell {
        let (tile, transform) = wall_tile_and_transform(neighbors);
        Cell { id, kind: CellKind::Wall, tile, transform }
    }

    // neighbors flags are for left and right neighbors respectively
    pub fn floor(id: HbId, neighbors: [bool; 2]) -> Cell {
        let (tile, transform) = match (neighbors[0], neighbors[1]) {
            (false, true) => (SpriteId::TileR0C0, CellTransform::Id),
            (true, false) => (SpriteId::TileR0C0, CellTransform::Mirror),
            (true, true) => (SpriteId::TileR0C1, CellTransform::Id),
            _ => panic!("no suitable floor tile to display given surrounding tiles"),
        };
        Cell { id, kind: CellKind::Floor, tile, transform }
    }

    pub fn gate(id: HbId) -> Cell {
        Cell { id, kind: CellKind::Wall, tile: SpriteId::TileR1C3, transform: CellTransform::Id }
    }

    pub fn spawn(id: HbId, color: WarpColor, mirrored: bool) -> Cell {
        let tile = match color {
            WarpColor::Green => SpriteId::TileR2C0,
            WarpColor::Blue => SpriteId::TileR2C1,
            WarpColor::Pink => SpriteId::TileR2C2,
        };
        let transform = if mirrored { CellTransform::Mirror } else { CellTransform::Id };
        Cell { id, kind: CellKind::Wall, tile, transform }
    }

    pub fn id(&self) -> HbId { self.id }
    pub fn kind(&self) -> CellKind { self.kind }

    pub fn draw(&self, renderer: &mut SpriteRenderer<AssetId>, pos: Vec2) {
        let affine = Affine::translate(pos.x, pos.y);
        let affine = match self.transform {
            CellTransform::Id => affine,
            CellTransform::Turn90 => affine.pre_rotate(-90_f64.to_radians()),
            CellTransform::Turn180 => affine.pre_rotate(-180_f64.to_radians()),
            CellTransform::Turn270 => affine.pre_rotate(-270_f64.to_radians()),
            CellTransform::Mirror => affine.pre_scale_axes(-1., 1.),
        };
        renderer.draw(&affine, self.tile);
    }
}

fn wall_tile_and_transform(neighbors: [bool; 8]) -> (SpriteId, CellTransform) {
    let transform_map = [CellTransform::Id, CellTransform::Turn90, CellTransform::Turn180, CellTransform::Turn270];
    for turns in 0..4 {
        if let Some(tile) = wall_tile(neighbors, turns * 2) {
            return (tile, transform_map[turns]);
        }
    }
    panic!("no suitable wall tile to display given surrounding tiles")
}

fn wall_tile(neighbors: [bool; 8], neighbors_shift: usize) -> Option<SpriteId> {
    let n = |idx| neighbors[(idx + neighbors_shift) % 8];
    match (n(0), n(1), n(2), n(3), n(4), n(5), n(6), n(7)) {
        (_, false, _, true, true, true, _, false) => Some(SpriteId::TileR0C2),
        (true, true, true, true, false, true, true, true) => Some(SpriteId::TileR0C3),
        (_, false, _, true, true, true, true, true) => Some(SpriteId::TileR1C0),
        (true, true, true, true, true, true, true, true) => Some(SpriteId::TileR1C1),
        _ => None,
    }
}
