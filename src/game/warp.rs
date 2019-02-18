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

use collider::geom::{Shape, Vec2, v2, Card};

use crate::asset_id::{AssetId, SpriteId};
use super::Idx2;
use super::util::{idx_to_vec, vec_to_affine};

const SPIN_VEL: f64 = -4.;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum WarpColor { Green, Blue, Pink }

impl WarpColor {
    pub fn draw_warp(self, renderer: &mut SpriteRenderer<AssetId>, affine: Affine, time: f64) {
        renderer.draw(&affine.pre_rotate(time * SPIN_VEL), self.tex());
    }

    fn tex(self) -> SpriteId {
        match self {
            WarpColor::Green => SpriteId::GreenWarp,
            WarpColor::Blue => SpriteId::BlueWarp,
            WarpColor::Pink => SpriteId::PinkWarp,
        }
    }
}

pub fn shape() -> Shape { Shape::circle(7.) }

#[derive(Copy, Clone)]
pub enum LasorKind { Still, Aiming }

impl LasorKind {
    fn max_angle(self) -> f64 {
        match self {
            LasorKind::Still => 0.,
            LasorKind::Aiming => 30_f64.to_radians(),
        }
    }
}

const LASOR_FIRE_SPEED: f64 = 60.;
const LASOR_FIRE_OFFSET: f64 = 7.;

fn angle_to_vec(angle: f64) -> Vec2 { v2(angle.cos(), angle.sin()) }

pub struct Lasor { pos: Vec2, card: Card, max_angle: f64, color: WarpColor }

impl Lasor {
    pub fn new(pos: Idx2, kind: LasorKind, color: WarpColor, card: Card) -> Lasor {
        Lasor { card, color, pos: idx_to_vec(pos), max_angle: kind.max_angle() }
    }

    pub fn fire(&self, player_pos: Vec2) -> (Vec2, WarpColor, Vec2) {
        let angle = self.angle(player_pos);
        (self.fire_pos(angle), self.color, angle_to_vec(angle) * LASOR_FIRE_SPEED)
    }

    fn fire_pos(&self, angle: f64) -> Vec2 {
        self.pos + angle_to_vec(angle) * LASOR_FIRE_OFFSET
    }

    fn support_angle(&self) -> f64 {
        let card_vec: Vec2 = self.card.into();
        card_vec.y.atan2(card_vec.x)
    }

    fn angle(&self, player_pos: Vec2) -> f64 {
        let rel_player = player_pos - self.pos;
        let support_angle = self.support_angle();
        let angle_delta = (rel_player.y.atan2(rel_player.x) - support_angle) % (2. * f64::consts::PI);
        let angle_delta = if angle_delta > f64::consts::PI {
            angle_delta - 2. * f64::consts::PI
        } else if angle_delta < -f64::consts::PI {
            angle_delta + 2. * f64::consts::PI
        } else {
            angle_delta
        };
        let angle_delta = if angle_delta > 0.5 * f64::consts::PI {
            f64::consts::PI - angle_delta
        } else if angle_delta < -0.5 * f64::consts::PI {
            -f64::consts::PI - angle_delta
        } else {
            angle_delta
        };
        let angle_delta = angle_delta.max(-self.max_angle).min(self.max_angle);
        support_angle + angle_delta
    }

    pub fn draw(&self, renderer: &mut SpriteRenderer<AssetId>, camera: Vec2, time: f64, next_fire_time: f64, player_pos: Vec2) {
        renderer.draw(&vec_to_affine(self.pos - camera).pre_rotate(self.support_angle() + f64::consts::PI), SpriteId::TileR2C3);
        let angle = self.angle(player_pos);
        let lasor_affine = vec_to_affine(self.pos - camera).pre_rotate(angle + f64::consts::PI);
        renderer.draw(&lasor_affine, SpriteId::Lasor);
        let ratio = 1. - (next_fire_time - time) * 2.5;
        if ratio > 0. {
            let scale = 0.2 + 0.8 * ratio;
            let flash = 1.0 - 0.5 * ratio;
            let affine = vec_to_affine(self.fire_pos(angle) - camera).pre_scale(scale).pre_rotate(time * SPIN_VEL);
            renderer.draw_flash(&affine, self.color.tex(), flash);
        }
    }
}
