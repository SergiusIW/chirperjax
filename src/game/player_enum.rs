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

use std::f64;

use gate::renderer::{SpriteRenderer, Affine};

use collider::Collider;
use collider::geom::Vec2;

use asset_id::{AssetId, SpriteId};
use super::player::Player;
use super::star;
use super::piece_profile::PieceProfile;
use super::warp::WarpColor;

const START_FADE_VEL: f64 = 1. / 0.6;
const START_DELAY: f64 = 0.7;
const WARP_SPEED: f64 = 180.;

pub enum PlayerEnum {
    Start(Vec2), Normal(Player), Warping(PlayerWarping), Complete(PlayerComplete)
}

impl PlayerEnum {
    pub fn transition_time(&self) -> f64 {
        match *self {
            PlayerEnum::Warping(ref w) => w.end_time,
            PlayerEnum::Start(_) => START_DELAY,
            _ => f64::INFINITY,
        }
    }

    pub fn draw(&self, renderer: &mut SpriteRenderer<AssetId>, affine: Affine, time: f64) {
        match *self {
            PlayerEnum::Normal(ref player) => player.draw(renderer, affine),
            PlayerEnum::Complete(ref player) => {
                if time < player.complete_time + star::OBTAIN_VANISH_DELAY {
                    let fade_time = time - player.complete_time;
                    let fade = if fade_time > 0. { star::OBTAIN_FADE_VEL * fade_time } else { 0. };
                    let affine = if player.mirror { affine.pre_scale_axes(-1., 1.) } else { affine };
                    renderer.draw_flash(&affine, player.tex, fade);
                }
            },
            PlayerEnum::Start(_) => {
                let fade = (START_DELAY - time) * START_FADE_VEL;
                if fade <= 1. { renderer.draw_flash(&affine, SpriteId::PlayerRun, 0.5 + 0.6 * fade) }
            },
            PlayerEnum::Warping(_) => {},
        }
    }

    pub fn pos(&self, collider: &Collider<PieceProfile>) -> Vec2 {
        match *self {
            PlayerEnum::Normal(ref player) => collider.get_hitbox(player.id()).value.pos,
            PlayerEnum::Start(pos) => pos,
            PlayerEnum::Complete(ref player) => player.pos,
            PlayerEnum::Warping(ref player) => player.end_pos - player.vel * (player.end_time - collider.time()),
        }
    }
}

pub struct PlayerComplete { pos: Vec2, complete_time: f64, tex: SpriteId, mirror: bool }

impl PlayerComplete {
    pub fn new(pos: Vec2, complete_time: f64, tex: SpriteId, mirror: bool) -> PlayerComplete {
        PlayerComplete { pos, complete_time, tex, mirror }
    }
}

pub struct PlayerWarping { end_time: f64, color: WarpColor, end_pos: Vec2, vel: Vec2 }

impl PlayerWarping {
    pub fn new(start_pos: Vec2, end_pos: Vec2, color: WarpColor, time: f64) -> PlayerWarping {
        let delta_pos = end_pos - start_pos;
        let dir = delta_pos.normalize().unwrap_or(Vec2::zero());
        PlayerWarping { color, end_pos, vel: dir * WARP_SPEED, end_time: time + delta_pos.len() / WARP_SPEED }
    }

    pub fn color(&self) -> WarpColor { self.color }
}
