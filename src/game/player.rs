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

use collider::geom::{Card, CardMask, Vec2, PlacedShape, Shape, v2};
use collider::{Hitbox, HbProfile, HbId};

use gate::Audio;
use gate::renderer::{SpriteRenderer, Affine};

use super::PieceProfile;
use super::util::nearest_card;
use game_input::HorizDir;
use asset_id::{AssetId, SpriteId, SoundId};

pub const STEP_PERIOD: f64 = 1. / 60.;

const MOVE_ACCEL: f64 = 150. * STEP_PERIOD;
const STOP_ACCEL: f64 = 350. * STEP_PERIOD;
const FALL_ACCEL: f64 = 360. * STEP_PERIOD;
const JUMP_SPEED: f64 = 80.;
const MAX_FALL_SPEED: f64 = 120.;
const MAX_MOVE_SPEED: f64 = 80.;
fn jump_duration(x_speed: f64) -> f64 { 0.21 + 0.10 * (x_speed / MAX_MOVE_SPEED) }

const GRAPHIC_STEP_DURATION: f64 = 0.16;
const GRAPHIC_CHIRP_DELAY: f64 = 0.8;
const GRAPHIC_CHIRP_DURATION: f64 = 0.125;

pub struct Player {
    id: HbId,
    time: f64,
    dir: HorizDir,
    moving: bool,
    state_start_time: f64,
    on_ground: bool,
    jump_held: bool,
    queued_jump: bool,
    jump_transition_time: f64,
    blocked_cards: CardMask,
    vel: Vec2,
    floor_vel: Vec2,
}

impl Player {
    pub fn new(id: HbId, pos: Vec2, time: f64, move_dir: Option<HorizDir>) -> (Player, PlacedShape) {
        let mut player = Player {
            id,
            time,
            dir: HorizDir::Right,
            moving: false,
            state_start_time: time,
            on_ground: false,
            jump_held: false,
            queued_jump: false,
            jump_transition_time: time,
            blocked_cards: CardMask::empty(),
            vel: Vec2::zero(),
            floor_vel: Vec2::zero(),
        };
        player.set_movement(move_dir);
        (player, Shape::rect(v2(3.5, 11.)).place(pos))
    }

    pub fn set_time(&mut self, time: f64) { self.time = time; }
    pub fn id(&self) -> HbId { self.id }
    pub fn vel(&self) -> Vec2 { self.vel }

    pub fn update_platform_vel(&mut self, player_shape: &PlacedShape, platform_hb: &Hitbox) {
        let normal = player_shape.masked_normal_from(&platform_hb.value, Card::PlusY.into());
        if normal.len() < PieceProfile::padding() && self.on_ground {
            let vel = platform_hb.vel.value;
            let rel_vel_x = self.vel.x - self.floor_vel.x;
            self.floor_vel = vel;
            self.vel.y = vel.y;
            self.vel.x = vel.x + rel_vel_x;
        }
    }

    pub fn set_movement(&mut self, movement: Option<HorizDir>) {
        let (moving, dir) = if let Some(dir) = movement { (true, dir) } else { (false, self.dir) };
        if self.moving != moving || self.dir != dir { self.state_start_time = self.time; }
        self.moving = moving;
        self.dir = dir;
    }

    pub fn press_jump(&mut self) {
        if self.on_ground && !self.queued_jump {
            self.queued_jump = true;
            self.jump_held = true;
        }
    }

    pub fn release_jump(&mut self) { self.jump_held = false; }

    pub fn step(&mut self, audio: &mut Audio<AssetId>) {
        let stop_accel = if self.on_ground { STOP_ACCEL } else { MOVE_ACCEL };
        let rel_vel_x = self.vel.x - self.floor_vel.x;
        let rel_vel_x = if self.moving {
            let accel = if self.dir.signum() == rel_vel_x.signum() { MOVE_ACCEL } else { stop_accel };
            rel_vel_x + self.dir.signum() * accel
        } else if self.on_ground {
            if rel_vel_x.abs() > stop_accel { rel_vel_x - rel_vel_x.signum() * stop_accel } else { 0.0 }
        } else {
            rel_vel_x
        };
        self.vel.x = self.floor_vel.x + rel_vel_x;
        if self.queued_jump {
            self.jump(audio);
        } else if self.time > self.jump_transition_time || !self.jump_held {
            self.jump_held = false;
            self.vel.y -= FALL_ACCEL;
        }
        self.bound_vel();
        self.update_on_ground(false);
    }

    fn jump(&mut self, audio: &mut Audio<AssetId>) {
        audio.play_sound(SoundId::Jump);
        self.on_ground = false;
        self.queued_jump = false;
        self.state_start_time = self.time;
        self.jump_transition_time = self.time + jump_duration((self.vel.x - self.floor_vel.x).abs());
        self.vel.y = JUMP_SPEED + self.floor_vel.y.max(0.0);
        self.vel.x = self.vel.x - self.floor_vel.x;
        self.floor_vel = Vec2::zero();
    }

    pub fn update_barriers<I: Iterator<Item=(Hitbox, CardMask)>>(&mut self, shape: PlacedShape, near_ground: bool, barriers: I) {
        let mut barrier_count = 0;
        self.blocked_cards = CardMask::empty();
        let mut floor_vel = None;
        for (hitbox, mask) in barriers {
            let normal = shape.masked_normal_from(&hitbox.value, mask);
            if normal.len() < PieceProfile::padding() {
                if floor_vel.is_none() {
                    floor_vel = Some(Vec2::zero());
                }
                let dir = nearest_card(normal.dir()).flip();
                self.blocked_cards[dir] = true;
                if hitbox.vel.value != Vec2::zero() {
                    assert!(dir == Card::MinusY, "only floors can be moving barriers");
                    floor_vel = Some(hitbox.vel.value);
                }
                barrier_count += 1;
            }
        }
        if let Some(floor_vel) = floor_vel {
            self.floor_vel = floor_vel;
        }
        assert!(self.floor_vel == Vec2::zero() || barrier_count <= 1,
                "cannot touch multiple barriers with moving floor");
        self.bound_vel();
        self.update_on_ground(near_ground);
    }

    fn bound_vel(&mut self) {
        self.vel.x = self.vel.x.max(-MAX_MOVE_SPEED + self.floor_vel.x);
        if self.blocked_cards[Card::MinusX] { self.vel.x = self.vel.x.max(0.0); }
        self.vel.x = self.vel.x.min(MAX_MOVE_SPEED + self.floor_vel.x);
        if self.blocked_cards[Card::PlusX] { self.vel.x = self.vel.x.min(0.0); }
        self.vel.y = self.vel.y.max(-MAX_FALL_SPEED);
        if self.blocked_cards[Card::PlusY] {
            self.vel.y = self.vel.y.min(0.0);
            self.jump_held = false;
        }
        if self.blocked_cards[Card::MinusY] { self.vel.y = self.vel.y.max(self.floor_vel.y); }
    }

    fn update_on_ground(&mut self, near_ground: bool) {
        let on_ground = self.blocked_cards[Card::MinusY] && self.vel.y == self.floor_vel.y;
        match (self.on_ground, on_ground, near_ground) {
            (false, true, _) => {
                self.state_start_time = self.time - GRAPHIC_STEP_DURATION; // lands with legs together
                self.queued_jump = false;
                self.jump_held = false;
                self.on_ground = true;
                self.bound_vel();
            },
            (true, false, false) => {
                self.floor_vel = Vec2::zero();
                self.state_start_time = self.time;
                self.queued_jump = false;
                self.jump_held = false;
                self.on_ground = false;
                self.bound_vel();
            },
            (true, false, true) => {
                self.vel.y = -MAX_FALL_SPEED;
                self.floor_vel.y = -MAX_FALL_SPEED;
            },
            _ => {},
        }
    }

    pub fn tex_and_mirror(&self) -> (SpriteId, bool) {
        let time = self.time - self.state_start_time;
        let tex = if !self.on_ground {
            SpriteId::PlayerRun
        } else if self.moving {
            let time = time % (2. * GRAPHIC_STEP_DURATION);
            if time < GRAPHIC_STEP_DURATION { SpriteId::PlayerRun } else { SpriteId::PlayerStill }
        } else {
            let time = time % (GRAPHIC_CHIRP_DELAY + GRAPHIC_CHIRP_DURATION);
            if time < GRAPHIC_CHIRP_DELAY { SpriteId::PlayerStill } else { SpriteId::PlayerChirp }
        };
        (tex, self.dir == HorizDir::Left)
    }

    pub fn draw(&self, renderer: &mut SpriteRenderer<AssetId>, affine: Affine) {
        let (tex, mirror) = self.tex_and_mirror();
        let affine = if mirror { affine.pre_scale_axes(-1., 1.) } else { affine };
        renderer.draw(&affine, tex);
    }
}
