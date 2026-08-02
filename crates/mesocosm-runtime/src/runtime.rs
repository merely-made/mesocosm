// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The host-neutral driver.
//!
//! A host owns the window, the device, and the frame loop. It hands elapsed
//! time in, queues intents, and reads the world back to draw it. It never
//! touches world state, and every step it causes is recorded, so any run can
//! be replayed from its seed and trace alone.

use std::collections::VecDeque;

use mesocosm_core::{Intent, Outcome, World, state_hash};

use crate::clock::Clock;

/// Default ceiling on steps authorised by one `advance` call. A stalled host
/// resuming after a long pause catches up over several frames rather than in
/// one burst that would itself cause another stall.
pub const DEFAULT_MAX_STEPS_PER_ADVANCE: u64 = 8;

/// Drives a [`World`] at a fixed rate from a host's uneven frame delivery.
pub struct Runtime {
    world: World,
    clock: Clock,
    queued: VecDeque<Intent>,
    /// The ordered trace of every intent actually applied.
    trace: Vec<Intent>,
    last: Vec<Outcome>,
    max_steps: u64,
    seed: u64,
    organisms: u32,
}

impl Runtime {
    pub fn new(seed: u64, organisms: u32, ticks_per_second: u32) -> Self {
        Self {
            world: World::new(seed, organisms),
            clock: Clock::new(ticks_per_second),
            queued: VecDeque::new(),
            trace: Vec::new(),
            last: Vec::new(),
            max_steps: DEFAULT_MAX_STEPS_PER_ADVANCE,
            seed,
            organisms,
        }
    }

    pub fn with_max_steps(mut self, max_steps: u64) -> Self {
        assert!(max_steps > 0, "at least one step per advance");
        self.max_steps = max_steps;
        self
    }

    /// Queues an intent for the next step. Intents are consumed in order, one
    /// per step; a step with nothing queued runs [`Intent::Idle`], so the
    /// simulation advances at a fixed rate whether or not the player acts.
    pub fn queue(&mut self, intent: Intent) {
        self.queued.push_back(intent);
    }

    pub fn queued_len(&self) -> usize {
        self.queued.len()
    }

    /// Runs whatever steps the elapsed time authorises. Returns how many ran.
    pub fn advance(&mut self, elapsed_us: u64) -> u64 {
        let advance = self.clock.advance(elapsed_us, self.max_steps);
        self.last.clear();
        for _ in 0..advance.steps {
            let intent = self.queued.pop_front().unwrap_or(Intent::Idle);
            let outcome = self.world.apply(intent.clone());
            self.trace.push(intent);
            self.last.push(outcome);
        }
        advance.steps
    }

    /// Runs exactly `steps` steps, ignoring the clock. For tests and for a
    /// host that wants to step manually.
    pub fn step(&mut self, steps: u64) {
        self.last.clear();
        for _ in 0..steps {
            let intent = self.queued.pop_front().unwrap_or(Intent::Idle);
            let outcome = self.world.apply(intent.clone());
            self.trace.push(intent);
            self.last.push(outcome);
        }
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    /// The ordered trace of applied intents. Together with the seed and organism
    /// count this reproduces the run exactly.
    pub fn trace(&self) -> &[Intent] {
        &self.trace
    }

    /// Outcomes from the most recent `advance` or `step`.
    pub fn last_outcomes(&self) -> &[Outcome] {
        &self.last
    }

    pub fn state_hash(&self) -> u64 {
        state_hash(&self.world)
    }

    /// The receipt a host probe compares: what this run was, and where it
    /// ended up.
    pub fn receipt(&self) -> Receipt {
        Receipt {
            seed: self.seed,
            organisms: self.organisms,
            steps: self.clock.steps_taken().max(self.trace.len() as u64),
            state_hash: self.state_hash(),
        }
    }

    /// Rebuilds a world from a seed and trace, without any host at all. Two
    /// hosts agree exactly when their traces replay to the same hash here.
    pub fn replay(seed: u64, organisms: u32, trace: &[Intent]) -> World {
        let mut world = World::new(seed, organisms);
        world.apply_all(trace);
        world
    }
}

/// A run's identity, for comparing hosts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub seed: u64,
    pub organisms: u32,
    pub steps: u64,
    pub state_hash: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use mesocosm_core::{PartId, Placement, Route, Yaw};

    fn scripted() -> Vec<Intent> {
        vec![
            Intent::Move { delta: [1, 0, 0] },
            Intent::Idle,
            Intent::Move { delta: [0, 0, 2] },
            Intent::Deposit { mass_mg: 25 },
            Intent::Metabolize { organism: mesocosm_core::OrganismId(0), route: Route::Incorporate { placement: Placement::Explicit { parent: PartId(0), offset: [4, 0, 0], yaw: Yaw::Zero } } },
            Intent::Move { delta: [-1, 0, -1] },
        ]
    }

    #[test]
    fn uneven_frames_do_not_change_the_simulation() {
        let steady = {
            let mut rt = Runtime::new(2024, 24, 60).with_max_steps(u64::MAX);
            for intent in scripted() {
                rt.queue(intent);
            }
            for _ in 0..6 {
                rt.advance(16_666);
            }
            rt
        };

        let ragged = {
            let mut rt = Runtime::new(2024, 24, 60).with_max_steps(u64::MAX);
            for intent in scripted() {
                rt.queue(intent);
            }
            // The same total time, delivered badly: a stall, a burst, a crawl.
            for chunk in [33u64, 1, 79_998, 12, 19_952] {
                rt.advance(chunk);
            }
            rt
        };

        assert_eq!(steady.trace(), ragged.trace(), "same intents in the same order");
        assert_eq!(steady.state_hash(), ragged.state_hash());
    }

    #[test]
    fn a_step_cap_delays_work_without_changing_it() {
        let uncapped = {
            let mut rt = Runtime::new(77, 16, 60).with_max_steps(u64::MAX);
            for intent in scripted() {
                rt.queue(intent);
            }
            rt.advance(100_000);
            rt
        };

        let capped = {
            let mut rt = Runtime::new(77, 16, 60).with_max_steps(2);
            for intent in scripted() {
                rt.queue(intent);
            }
            // One big frame, then idle frames while it catches up.
            rt.advance(100_000);
            for _ in 0..8 {
                rt.advance(0);
            }
            rt
        };

        assert_eq!(uncapped.trace().len(), capped.trace().len());
        assert_eq!(uncapped.state_hash(), capped.state_hash());
    }

    #[test]
    fn empty_queue_idles_rather_than_stalling() {
        let mut rt = Runtime::new(3, 4, 60).with_max_steps(u64::MAX);
        rt.advance(16_666 * 4);
        assert_eq!(rt.trace().len(), 3);
        assert!(rt.trace().iter().all(|i| matches!(i, Intent::Idle)));
    }

    #[test]
    fn trace_replays_to_the_same_world() {
        let mut rt = Runtime::new(555, 20, 60).with_max_steps(u64::MAX);
        for intent in scripted() {
            rt.queue(intent);
        }
        rt.advance(200_000);

        let replayed = Runtime::replay(555, 20, rt.trace());
        assert_eq!(state_hash(&replayed), rt.state_hash());
    }

    #[test]
    fn receipts_match_when_runs_match() {
        let run = |max: u64, chunks: &[u64]| {
            let mut rt = Runtime::new(8, 12, 60).with_max_steps(max);
            for intent in scripted() {
                rt.queue(intent);
            }
            for c in chunks {
                rt.advance(*c);
            }
            rt.receipt()
        };
        let a = run(u64::MAX, &[100_000]);
        let b = run(2, &[100_000, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(a, b);
    }

    #[test]
    fn manual_stepping_matches_clocked_stepping() {
        let clocked = {
            let mut rt = Runtime::new(41, 10, 60).with_max_steps(u64::MAX);
            for intent in scripted() {
                rt.queue(intent);
            }
            // Exactly six steps at 60 Hz is 100_000 us, not 16_666 * 6. The
            // clock is drift-free, so the shortfall would run five.
            rt.advance(100_000);
            rt.state_hash()
        };
        let manual = {
            let mut rt = Runtime::new(41, 10, 60);
            for intent in scripted() {
                rt.queue(intent);
            }
            rt.step(6);
            rt.state_hash()
        };
        assert_eq!(clocked, manual);
    }
}
