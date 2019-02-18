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

use gate::renderer::{Renderer, SpriteRenderer, Affine};

use collider::geom::{v2, Vec2};

use crate::asset_id::{AssetId, SpriteId};
use super::SCREEN_PIXELS_HEIGHT;

const COLOR: (u8, u8, u8) = (203, 219, 255);
const PERIOD: f64 = 10.;
const SEPARATION: f64 = 60.;

pub fn draw(renderer: &mut Renderer<AssetId>, camera: Vec2, room_pixels: Vec2, time: f64, screen_pixels_width: f64) {
    renderer.clear(COLOR);

    let mut renderer = renderer.sprite_mode();
    let time = time + 0.125 * PERIOD;
    let offset = (room_pixels * 0.5 - (camera + 0.5 * v2(screen_pixels_width, SCREEN_PIXELS_HEIGHT))) * 0.5;

    draw_bg_piece_grid(&mut renderer, offset, time, screen_pixels_width);
    draw_bg_piece_grid(&mut renderer, offset + v2(SEPARATION, SEPARATION), time + 0.25 * PERIOD, screen_pixels_width);
    draw_bg_piece_grid(&mut renderer, offset + v2(2. * SEPARATION, 0.), time + 0.5 * PERIOD, screen_pixels_width);
    draw_bg_piece_grid(&mut renderer, offset + v2(SEPARATION, -SEPARATION), time + 0.75 * PERIOD, screen_pixels_width);
}

fn draw_bg_piece_grid(renderer: &mut SpriteRenderer<AssetId>, center: Vec2, time: f64, screen_pixels_width: f64) {
    let time = time % PERIOD;
    let angle = 0.25 * (time * 15.).sin() * (-3. * time).exp();
    let pre_affine = Affine::scale(1.25).pre_rotate(angle);
    let separation = SEPARATION * 2.;

    let max_x = (screen_pixels_width + SEPARATION) * 0.5 + 3.;
    let max_y = (SCREEN_PIXELS_HEIGHT + SEPARATION) * 0.5 + 3.;

    let start_x_idx = ((-max_x - center.x) / separation).ceil() as i32;
    let end_x_idx = ((max_x - center.x) / separation).ceil() as i32;
    let start_y_idx = ((-max_y - center.y) / separation).ceil() as i32;
    let end_y_idx = ((max_y - center.y) / separation).ceil() as i32;

    let x_off = 0.5 * screen_pixels_width;
    let y_off = 0.5 * SCREEN_PIXELS_HEIGHT;

    for x_idx in start_x_idx..end_x_idx {
        for y_idx in start_y_idx..end_y_idx {
            if (x_idx + y_idx) % 2 == 0 {
                let pos = center + v2(separation * x_idx as f64, separation * y_idx as f64);
                renderer.draw(&pre_affine.post_translate(pos.x + x_off, pos.y + y_off), SpriteId::BgPattern);
            }
        }
    }
}
