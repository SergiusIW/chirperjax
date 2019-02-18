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

use gate::KeyCode;

#[derive(PartialEq, Eq, Copy, Clone, Hash)]
pub enum HorizDir { Left, Right }

impl HorizDir {
    pub fn signum(self) -> f64 {
        match self {
            HorizDir::Left => -1.,
            HorizDir::Right => 1.,
        }
    }

    fn from_key(key: KeyCode) -> Option<HorizDir> {
        match key {
            KeyCode::Left => Some(HorizDir::Left),
            KeyCode::Right => Some(HorizDir::Right),
            _ => None,
        }
    }
}

pub enum InputEvent {
    UpdateMovement(Option<HorizDir>),
    PressJump,
    ReleaseJump,
}

pub struct GameInput { held_dirs: Vec<HorizDir> }

impl GameInput {
    pub fn new() -> GameInput { GameInput { held_dirs: Vec::new() } }

    pub fn key_down(&mut self, key: KeyCode) -> Option<InputEvent> {
        if let Some(dir) = HorizDir::from_key(key) {
            self.held_dirs.push(dir);
            Some(InputEvent::UpdateMovement(Some(dir)))
        } else if key == KeyCode::Up {
            Some(InputEvent::PressJump)
        } else {
            None
        }
    }

    pub fn key_up(&mut self, key: KeyCode) -> Option<InputEvent> {
        if let Some(dir) = HorizDir::from_key(key) {
            self.held_dirs.retain(|&d| d != dir);
            Some(InputEvent::UpdateMovement(self.held_dir()))
        } else if key == KeyCode::Up {
            Some(InputEvent::ReleaseJump)
        } else {
            None
        }
    }

    pub fn held_dir(&self) -> Option<HorizDir> { self.held_dirs.last().cloned() }
}
