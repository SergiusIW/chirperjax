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

use collider::geom::{Shape, PlacedShape, v2};

use asset_id::{AssetId, SpriteId};
use super::{CELL_LEN, Idx2};
use super::platform::PlatformKind;
use super::util::idx_to_vec;

const WIDTH: f64 = CELL_LEN as f64 - 0.1;
const HEIGHT: f64 = 1.;
const Y_OFFSET: f64 = -0.5 * CELL_LEN as f64 + 0.5 * HEIGHT;

pub struct ButtonAction { pub unlock_cells: Vec<Idx2>, pub platforms: Vec<(Idx2, PlatformKind)> }

pub fn shape(pos: Idx2) -> PlacedShape {
    Shape::rect(v2(WIDTH, HEIGHT)).place(idx_to_vec(pos) + v2(0., Y_OFFSET))
}

pub fn draw(renderer: &mut SpriteRenderer<AssetId>, affine: Affine) {
    renderer.draw(&affine.pre_translate(0., -Y_OFFSET), SpriteId::TileR1C2);
}
