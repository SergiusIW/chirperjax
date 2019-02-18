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

use gate::renderer::{SpriteRenderer, Affine};

use collider::geom::Vec2;

use asset_id::{AssetId, SpriteId};
use super::util::vec_to_affine;
use super::warp::WarpColor;

pub struct Effect { value: Box<InternalEffect>, pos: Vec2, start_time: f64 }

impl Effect {
    pub fn draw(&self, renderer: &mut SpriteRenderer<AssetId>, camera: Vec2, time: f64) -> bool {
        self.value.draw(renderer, &vec_to_affine(self.pos - camera), time - self.start_time)
    }
}

trait InternalEffect {
    fn draw(&self, renderer: &mut SpriteRenderer<AssetId>, affine: &Affine, time: f64) -> bool;
}

struct SquareFade;

impl InternalEffect for SquareFade {
    fn draw(&self, renderer: &mut SpriteRenderer<AssetId>, affine: &Affine, time: f64) -> bool {
        let scale = 1. - time / 0.3;
        if scale > 0. {
            renderer.draw(&affine.pre_scale(scale), SpriteId::WhiteSquare);
            true
        } else {
            false
        }
    }
}

pub fn square_fade(pos: Vec2, start_time: f64) -> Effect {
    Effect { pos, start_time, value: Box::new(SquareFade) }
}

struct ColorFade { color: WarpColor }

impl InternalEffect for ColorFade {
    fn draw(&self, renderer: &mut SpriteRenderer<AssetId>, affine: &Affine, time: f64) -> bool {
        let scale = 1.0 - time / 0.3;
        if scale > 0.0 {
            let tex = match self.color {
                WarpColor::Green => SpriteId::GreenFade,
                WarpColor::Blue => SpriteId::BlueFade,
                WarpColor::Pink => SpriteId::PinkFade,
            };
            renderer.draw(&affine.pre_scale(scale), tex);
            true
        } else {
            false
        }
    }
}

pub fn color_fade(pos: Vec2, start_time: f64, color: WarpColor) -> Effect {
    Effect { pos, start_time, value: Box::new(ColorFade { color }) }
}

struct Puff { angle: f64, }

impl InternalEffect for Puff {
    fn draw(&self, renderer: &mut SpriteRenderer<AssetId>, affine: &Affine, time: f64) -> bool {
        let affine = affine.pre_rotate(self.angle).pre_translate(0., 2.);
        let ratio = time / 0.15;
        if ratio < 1. {
            let dist = 1.5 + 3. * ratio;
            renderer.draw(&affine.pre_translate(dist, 0.), SpriteId::Puff);
            renderer.draw(&affine.pre_translate(-dist, 0.).pre_scale_axes(-1., 1.), SpriteId::Puff);
            true
        } else {
            false
        }
    }
}

pub fn puff(pos: Vec2, start_time: f64, angle: f64) -> Effect {
    Effect { pos, start_time, value: Box::new(Puff { angle }) }
}
