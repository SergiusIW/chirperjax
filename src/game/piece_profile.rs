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

use collider::{HbId, HbProfile};

use super::{Idx2, CELL_LEN};
use super::cell::CellKind;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PieceKind { Wall, Floor, Player, Platform, Star, Button, Warp }

impl From<CellKind> for PieceKind {
    fn from(kind: CellKind) -> PieceKind {
        match kind {
            CellKind::Wall => PieceKind::Wall,
            CellKind::Floor => PieceKind::Floor,
        }
    }
}

#[derive(Copy, Clone)]
pub struct PieceProfile { pub id: HbId, pub index: Option<Idx2>, pub kind: PieceKind }

impl PieceProfile {
    pub fn new(id: HbId, kind: PieceKind) -> PieceProfile {
        PieceProfile { id, kind, index: None }
    }

    pub fn cell(id: HbId, index: Idx2, kind: CellKind) -> PieceProfile {
        PieceProfile { id, index: Some(index), kind: kind.into() }
    }

    fn can_interact_asym(&self, other: &PieceProfile) -> bool {
        match self.kind {
            PieceKind::Player => match other.kind {
                PieceKind::Wall | PieceKind::Floor | PieceKind::Platform | PieceKind::Button | PieceKind::Warp | PieceKind::Star => true,
                _ => false,
            },
            PieceKind::Warp => match other.kind {
                PieceKind::Floor | PieceKind::Platform => true,
                PieceKind::Wall => other.index.is_some(),
                _ => false,
            },
            _ => false,
        }
    }
}

impl HbProfile for PieceProfile {
    fn id(&self) -> HbId { self.id }
    fn can_interact(&self, other: &Self) -> bool {
        self.can_interact_asym(other) || other.can_interact_asym(self)
    }

    fn cell_width() -> f64 { CELL_LEN as f64 }
    fn padding() -> f64 { 0.025 }
}
