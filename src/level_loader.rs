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

use crate::game::{GameBoard, LasorKind, PlatformKind, WarpColor, Idx2};

pub const LEVEL_COUNT: usize = 7;

const LEVELS: [&'static str; LEVEL_COUNT] = [
    include_str!("levels/level0.txt"),
    include_str!("levels/level1.txt"),
    include_str!("levels/level2.txt"),
    include_str!("levels/level3.txt"),
    include_str!("levels/level4.txt"),
    include_str!("levels/level5.txt"),
    include_str!("levels/level6.txt"),
];

const LEVELS_INDEX: [&'static str; LEVEL_COUNT] = [
    include_str!("levels/level0_index.txt"),
    include_str!("levels/level1_index.txt"),
    include_str!("levels/level2_index.txt"),
    include_str!("levels/level3_index.txt"),
    include_str!("levels/level4_index.txt"),
    include_str!("levels/level5_index.txt"),
    include_str!("levels/level6_index.txt"),
];

pub fn load(level_num: usize) -> GameBoard {
    let level = LevelFile::new(LEVELS[level_num]);
    let level_index = LevelFile::new(LEVELS_INDEX[level_num]);

    let mut board = GameBoard::builder(level.dims);
    for y in 0..level.dims.1 {
        for x in 0..level.dims.0 {
            let pos = (x, y);
            match (level.get(pos), digit(level_index.get(pos))) {
                ('P', None) => board.add_player(pos),
                ('@', None) => board.add_star(pos),
                ('-', None) => board.add_wall(pos),
                ('+', None) => board.add_floor(pos),
                ('I', Some(idx)) => board.add_gate(pos, idx),
                ('C', idx) => board.add_platform(pos, PlatformKind::Circle, idx),
                ('c', idx) => board.add_platform(pos, PlatformKind::ReverseCircle, idx),
                ('A', idx) => board.add_platform(pos, PlatformKind::UpDown, idx),
                ('V', idx) => board.add_platform(pos, PlatformKind::DownUp, idx),
                ('>', idx) => board.add_platform(pos, PlatformKind::RightLeft, idx),
                ('<', idx) => board.add_platform(pos, PlatformKind::LeftRight, idx),
                ('L', Some(idx)) => board.add_lasor(pos, LasorKind::Still, index_to_color(idx)),
                ('H', Some(idx)) => board.add_lasor(pos, LasorKind::Aiming, index_to_color(idx)),
                ('B', Some(idx)) => board.add_button(pos, idx),
                ('w', Some(idx)) => board.add_respawn(pos, index_to_color(idx)),
                ('W', Some(idx)) => board.add_warp(pos, index_to_color(idx)),
                (' ', None) => {},
                _ => panic!("error reading level {}, position {:?}", level_num, pos),
            }
        }
    }

    board.build()
}

fn digit(c: char) -> Option<u32> {
    if c >= '0' && c <= '9' { Some(c as u32 - '0' as u32) } else { None }
}

fn index_to_color(index: u32) -> WarpColor {
    match index {
        0 => WarpColor::Green,
        1 => WarpColor::Blue,
        2 => WarpColor::Pink,
        _ => panic!("invalid warp index"),
    }
}

struct LevelFile { dims: Idx2, grid: Vec<Vec<char>> }

impl LevelFile {
    fn new(file_contents: &str) -> LevelFile {
        let grid: Vec<Vec<char>> = file_contents.lines().map(|s| s.chars().collect())
                                                        .filter(|s: &Vec<char>| !s.is_empty())
                                                        .collect();
        LevelFile { dims: (grid[0].len() as i32 - 1, grid.len() as i32), grid }
    }

    fn get(&self, pos: Idx2) -> char {
        self.grid[(self.dims.1 - 1 - pos.1) as usize][pos.0 as usize]
    }
}
