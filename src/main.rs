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

extern crate gate;
extern crate collider;

mod level_loader;
mod game_input;
mod game;
mod asset_id { include!(concat!(env!("OUT_DIR"), "/asset_id.rs")); }

use gate::{App, AppContext, AppInfo, KeyCode};
use gate::renderer::{Renderer, Affine};

use game_input::{GameInput, InputEvent};
use game::GameBoard;
use asset_id::{AssetId, MusicId, SpriteId};
use level_loader::LEVEL_COUNT;

fn main() {
    let info = AppInfo::with_app_height(game::SCREEN_PIXELS_HEIGHT)
                       .title("Gate Demo Game")
                       .print_workload_info()
                       .print_gl_info();
    gate::run(info, GameApp::new());
}

struct GameApp { input: GameInput, level: usize, board: GameBoard }

impl GameApp {
    pub fn new() -> GameApp {
        GameApp { input: GameInput::new(), level: 0, board: level_loader::load(0) }
    }

    fn load_next_level(&mut self) {
        self.level = (self.level + 1) % LEVEL_COUNT;
        self.board = level_loader::load(self.level);
        if let Some(held_dir) = self.input.held_dir() {
            self.board.input(InputEvent::UpdateMovement(Some(held_dir)));
        }
    }
}

impl App<AssetId> for GameApp {
    fn start(&mut self, ctx: &mut AppContext<AssetId>) { ctx.audio.loop_music(MusicId::BgMusic); }

    fn render(&mut self, renderer: &mut Renderer<AssetId>, ctx: &AppContext<AssetId>) {
        self.board.draw(renderer, ctx.dims().0);
        if self.level == 0 {
            renderer.sprite_mode().draw(&Affine::scale(2.), SpriteId::Instructions);
        }
    }

    fn advance(&mut self, seconds: f64, ctx: &mut AppContext<AssetId>) {
        self.board.advance(seconds, &mut ctx.audio);
        if self.board.is_done() {
            self.load_next_level();
        }
    }

    fn key_down(&mut self, key: KeyCode, _: &mut AppContext<AssetId>) {
        if let Some(event) = self.input.key_down(key) {
            self.board.input(event);
        }
    }

    fn key_up(&mut self, key: KeyCode, _: &mut AppContext<AssetId>) {
        if let Some(event) = self.input.key_up(key) {
            self.board.input(event);
        }
    }
}
