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

use super::player;

#[derive(Copy, Clone)]
pub enum Step { Player, Platform, WarpEffectSpawn, LasorFire }
const STEP_COUNT: usize = 4;
const STEPS: [Step; STEP_COUNT] = [Step::Player, Step::Platform, Step::WarpEffectSpawn, Step::LasorFire ];

impl Step {
    fn period(self) -> f64 {
        match self {
            Step::Player => player::STEP_PERIOD,
            Step::Platform => 0.18,
            Step::WarpEffectSpawn => 1. / 30.,
            Step::LasorFire => 1.,
        }
    }
}

pub struct StepQueue { times: [f64; STEP_COUNT] }

// queue of steps that repeat periodically
impl StepQueue {
    pub fn new() -> StepQueue {
        let mut times = [0.; STEP_COUNT];
        for &step in STEPS.iter() { times[step as usize] = step.period(); }
        StepQueue { times }
    }

    // returns time of next step
    pub fn peek(&self) -> f64 {
        self.times.iter().cloned().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()
    }

    // returns time that step of given type will be returned next
    pub fn peek_specific(&self, step: Step) -> f64 {
        self.times[step as usize]
    }

    // pops the step to occur at time `self.peek()`
    pub fn pop(&mut self) -> Step {
        let step = STEPS.iter().cloned()
                        .min_by(|&a, &b| self.times[a as usize].partial_cmp(&self.times[b as usize]).unwrap())
                        .unwrap();
        self.times[step as usize] += step.period();
        step
    }
}
