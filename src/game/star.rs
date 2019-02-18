// chirperjax, a demo game built using the "gate" game library.
// Copyright (C) 2017-2019  Matthew D. Michelotti
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

use std::f64;

use gate::renderer::{SpriteRenderer, Affine};

use collider::{HbId, Hitbox};
use collider::geom::Shape;

use crate::asset_id::{AssetId, SpriteId};
use super::Idx2;
use super::util::idx_to_vec;

pub const OBTAIN_FADE_VEL: f64 = 1.8;
pub const OBTAIN_VANISH_DELAY: f64 = 0.85;

pub struct Star { id: HbId, obtain_time: f64 }

impl Star {
    pub fn new(id: HbId, pos: Idx2) -> (Star, Hitbox) {
        (Star { id, obtain_time: f64::INFINITY }, Shape::circle(13.).place(idx_to_vec(pos)).still())
    }

    pub fn id(&self) -> HbId { self.id }

    pub fn obtain(&mut self, time: f64) { self.obtain_time = time; }

    pub fn level_end_time(&self) -> f64 { self.obtain_time + 1.85 }

    pub fn draw(&self, renderer: &mut SpriteRenderer<AssetId>, affine: Affine, time: f64) {
        if time < self.obtain_time + OBTAIN_VANISH_DELAY {
            let fade_time = time - self.obtain_time;
            let fade = if fade_time > 0. { OBTAIN_FADE_VEL * fade_time } else { 0. };
            let angle = Star::angle(time);
            let lag_angle = Star::angle(time - 0.21);
            renderer.draw_flash(&affine.pre_rotate(lag_angle), SpriteId::Star, 1.);
            renderer.draw_flash(&affine.pre_rotate(angle), SpriteId::Star, fade);
        }
    }

    fn angle(time: f64) -> f64 { 0.3 * (5. * (time - 0.21)).sin() }
}
