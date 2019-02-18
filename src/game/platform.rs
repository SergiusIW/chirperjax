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

use collider::{Hitbox, HbVel};
use collider::geom::{Vec2, v2, Shape};

use asset_id::{AssetId, SpriteId};
use super::util::idx_to_vec;
use super::Idx2;

#[derive(Copy, Clone)]
pub enum PlatformKind { Circle, ReverseCircle, UpDown, DownUp, RightLeft, LeftRight }

impl PlatformKind {
    fn radii(self) -> Vec2 {
        match self {
            PlatformKind::Circle => v2(32., 32.),
            PlatformKind::ReverseCircle => v2(32., -32.),
            PlatformKind::UpDown => v2(0., 40.),
            PlatformKind::DownUp => v2(0., -40.),
            PlatformKind::RightLeft => v2(40., 0.),
            PlatformKind::LeftRight => v2(-40., 0.),
        }
    }

    fn count(self) -> u32 {
        match self {
            PlatformKind::Circle | PlatformKind::ReverseCircle => 4,
            _ => 1,
        }
    }

    fn phase(self, index: u32) -> f64 { (index as f64 / self.count() as f64) * (2. * f64::consts::PI) }

    fn offset(self, index: u32, time: f64) -> Vec2 {
        let radii = self.radii();
        let angle = time * 1.1 + self.phase(index);
        v2(radii.x * angle.cos(), radii.y * angle.sin())
    }
}

pub struct Platform { kind: PlatformKind, index: u32, center: Vec2, fade_in_time: f64 }

impl Platform {
    pub fn new(kind: PlatformKind, pos: Idx2, time: f64, end_time: f64) -> Vec<(Platform, Hitbox)> {
        let fade_in_time = if time == 0. { f64::NEG_INFINITY } else { time };
        let center = idx_to_vec(pos);
        (0..kind.count()).map(|index| {
            let platform = Platform { kind, index, fade_in_time, center };
            let pos = platform.position_at_time(time);
            let vel = platform.step(pos, time, end_time);
            (platform, Hitbox::new(Shape::rect(v2(24., 8.)).place(pos), vel))
        }).collect()
    }

    pub fn step(&self, pos: Vec2, time: f64, end_time: f64) -> HbVel {
        let delta_time = end_time - time;
        let vel = if delta_time > 0.01 {
            let target_pos = self.position_at_time(end_time);
            (target_pos - pos) * (1. / delta_time)
        } else {
            v2(0., 0.)
        };
        HbVel::moving_until(vel, end_time)
    }

    fn position_at_time(&self, time: f64) -> Vec2 { self.center + self.kind.offset(self.index, time) }

    pub fn draw(&self, renderer: &mut SpriteRenderer<AssetId>, affine: Affine, time: f64) {
        let time = time - self.fade_in_time;
        let flash_ratio = 1.0 - time;
        renderer.draw_flash(&affine, SpriteId::Platform, flash_ratio);
    }
}
