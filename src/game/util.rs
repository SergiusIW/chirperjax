// gate_demo, a demo game built using the "gate" game library.
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

use gate::renderer::Affine;

use collider::geom::{Vec2, v2, Card};

use super::Idx2;

pub struct IdGen { next: u64 }

impl IdGen {
    pub fn new() -> IdGen { IdGen { next: 0 } }
    pub fn next(&mut self) -> u64 {
        let id = self.next;
        self.next += 1;
        id
    }
}

pub fn idx_to_vec(idx: Idx2) -> Vec2 {
    fn idx_to_f64(idx: i32) -> f64 { (idx * 8 + 4) as f64 }
    v2(idx_to_f64(idx.0), idx_to_f64(idx.1))
}

pub fn vec_to_affine(vec: Vec2) -> Affine { Affine::translate(vec.x, vec.y) }

pub fn card_offset(card: Card) -> Idx2 {
    match card {
        Card::PlusX => (1, 0),
        Card::PlusY => (0, 1),
        Card::MinusX => (-1, 0),
        Card::MinusY => (0, -1),
    }
}

pub fn nearest_card(vector: Vec2) -> Card {
    if vector.x.abs() > vector.y.abs() {
        if vector.x > 0.0 { Card::PlusX } else { Card::MinusX }
    } else {
        if vector.y > 0.0 { Card::PlusY } else { Card::MinusY }
    }
}
