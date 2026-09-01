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

use mesocosm_core::{History, Intent, Outcome, Reading, Trend, World, state_hash};

use crate::clock::Clock;
use crate::readings::FlowWindows;
use crate::succession::Checkpoint;

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
    /// What happened, drained tick by tick.
    ///
    /// The world buffers one tick and a caller drains it, so somebody has to be
    /// that caller or a shipped run has no past at all. This is the caller: the
    /// driver already records every intent, and recording every consequence
    /// belongs beside it.
    history: History,
    /// The bounded ecology windows, reduced from the same two streams.
    ///
    /// Here rather than in the world for the reason the history is here: it is
    /// derivable from a seed and a trace, so keeping it in the snapshot would
    /// put a presentation reading inside the replay hash. `Runtime::replayed`
    /// rebuilds it, which is that claim made executable.
    readings: FlowWindows,
    /// The question the world is holding at, if it is holding at one.
    ///
    /// **The pause lives here and nowhere else.** Stepping is the driver's job,
    /// so not stepping is too; the world stays a pure function of its seed and
    /// its trace and never learns that anybody stopped to think. See
    /// [`crate::succession`].
    checkpoint: Option<Checkpoint>,
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
            history: History::new(),
            readings: FlowWindows::new(),
            checkpoint: None,
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
    ///
    /// **A held world does not bank time.** While a [`checkpoint`] stands and
    /// nothing queued answers it, the elapsed microseconds are dropped rather
    /// than accumulated in the clock — otherwise a player who took ten seconds
    /// over the question would be answered by ten seconds of ecology sprinting
    /// past in one frame.
    ///
    /// [`checkpoint`]: Self::checkpoint
    pub fn advance(&mut self, elapsed_us: u64) -> u64 {
        if self.held_at_a_question() {
            self.last.clear();
            return 0;
        }
        let advance = self.clock.advance(elapsed_us, self.max_steps);
        self.last.clear();
        let mut taken = 0;
        for _ in 0..advance.steps {
            if !self.step_once() {
                break;
            }
            taken += 1;
        }
        taken
    }

    /// Whether the world is stopped at a question nothing queued answers.
    fn held_at_a_question(&self) -> bool {
        self.checkpoint.as_ref().is_some_and(|checkpoint| {
            !self
                .queued
                .front()
                .is_some_and(|intent| checkpoint.answers(intent))
        })
    }

    /// Runs up to `steps` steps, ignoring the clock. Returns how many ran, which
    /// is fewer than asked when a checkpoint holds the world.
    pub fn step(&mut self, steps: u64) -> u64 {
        self.last.clear();
        let mut taken = 0;
        for _ in 0..steps {
            if !self.step_once() {
                break;
            }
            taken += 1;
        }
        taken
    }

    /// One tick, unless the world is holding at a question this tick's intent
    /// does not answer. `false` means nothing happened and nothing was consumed.
    fn step_once(&mut self) -> bool {
        if let Some(checkpoint) = &self.checkpoint {
            // An unanswered question is not a slow tick; it is a stopped world.
            // Note what is *queued* rather than substituting an answer: putting
            // one in on the player's behalf is exactly what "one recorded
            // choice" rules out.
            match self.queued.front() {
                Some(intent) if checkpoint.answers(intent) => self.checkpoint = None,
                _ => return false,
            }
        }
        // The hand, read before the tick: a critter that dies this tick is held
        // by nobody after it — and if it was *eaten* it is not even in the
        // roster to be asked what lineage it was. That is the moment the
        // question matters most, so both facts are taken while they still exist.
        let hand = self
            .world
            .held()
            .and_then(|id| self.world.controlled().map(|held| (id, held.species)));
        let intent = self.queued.pop_front().unwrap_or(Intent::Idle);
        let outcome = self.world.apply(intent.clone());
        self.trace.push(intent);
        self.absorb(hand);
        self.last.push(outcome);
        true
    }

    /// Takes the tick's two records: the causal half into the past, both halves
    /// into the windows, and both again to see whether this tick asked the
    /// player something. One call, because they are one tick's worth and a
    /// caller that drained one and forgot the other would have a past and a
    /// reading that disagreed.
    fn absorb(&mut self, hand: Option<(mesocosm_core::OrganismId, mesocosm_core::SpeciesId)>) {
        let events = self.world.drain_events();
        let flows = self.world.drain_flows();
        self.readings.absorb(&events, &flows);
        // Recorded before the question is put, so a line that gained a
        // descendant on the same tick it lost its body can still be continued
        // through that descendant.
        self.history.record_all(events.iter().copied());
        if self.checkpoint.is_none() {
            self.checkpoint =
                crate::succession::opened(&self.world, &self.history, hand, &events, &flows);
        }
    }

    /// The question the world is holding at, if any.
    pub fn checkpoint(&self) -> Option<&Checkpoint> {
        self.checkpoint.as_ref()
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    /// Takes the ground bricks changed since the last drain, so a host's
    /// projection can upload regions rather than the whole map.
    ///
    /// The only `&mut World` route a host gets, and it is not a world change:
    /// the dirty set is outside the snapshot and outside equality, so a host
    /// that drains and a headless replay that never does still agree.
    pub fn drain_ground_dirty(&mut self) -> Vec<[i16; 3]> {
        self.world.drain_ground_dirty()
    }

    /// What this run has seen happen.
    pub fn history(&self) -> &History {
        &self.history
    }

    /// The bounded ecology windows over this run's flows.
    pub fn windows(&self) -> &FlowWindows {
        &self.readings
    }

    /// What those windows currently read: facts and the windows they cover.
    pub fn trend(&self) -> Trend {
        self.readings.trend()
    }

    /// What each lineage has done, without noting any of it.
    ///
    /// The reading half on its own, so a host can show a standing without
    /// ending an epoch to find it out.
    pub fn readings(&self) -> Vec<Reading> {
        mesocosm_core::readings(&self.world, &self.history)
    }

    /// Ends the epoch and writes what it came to into the world's record.
    pub fn end_epoch(&mut self) -> Vec<Reading> {
        self.world.end_epoch(&self.history)
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
            // The trace is one entry per step actually applied, so it is the
            // step count whether the run was clocked or stepped by hand — and
            // it stays honest across a checkpoint, where the clock may have
            // authorised steps the held world never took.
            steps: self.trace.len() as u64,
            state_hash: self.state_hash(),
        }
    }

    /// Rebuilds a run from a seed and trace, without any host at all. Two hosts
    /// agree exactly when their traces replay to the same hash here.
    ///
    /// Returns the past as well as the world, because it has to: a driven run
    /// drains its events every tick and a replay that did not would end holding
    /// a tick of undrained ones, which is a difference in the snapshot and so a
    /// difference in the hash. Reproducing the history rather than discarding it
    /// is also the claim that keeps it out of the snapshot, made executable.
    pub fn replay(seed: u64, organisms: u32, trace: &[Intent]) -> (World, History) {
        let replayed = Self::replayed(seed, organisms, trace);
        (replayed.world, replayed.history)
    }

    /// The same replay, keeping the readings it rebuilt.
    ///
    /// **The done-condition made runnable.** The windows are not in the
    /// snapshot, so nothing forces them to agree; reducing the replay's own
    /// streams through the same reducer and comparing the encodings is what
    /// shows that a replayed run reads the same as the run it replays.
    pub fn replayed(seed: u64, organisms: u32, trace: &[Intent]) -> Replayed {
        let mut world = World::new(seed, organisms);
        let mut history = History::new();
        let mut readings = FlowWindows::new();
        for intent in trace {
            world.apply(intent.clone());
            let events = world.drain_events();
            readings.absorb(&events, &world.drain_flows());
            history.record_all(events);
        }
        Replayed {
            world,
            history,
            readings,
        }
    }
}

/// What a replay reproduced: the world, its past, and its readings.
pub struct Replayed {
    pub world: World,
    pub history: History,
    pub readings: FlowWindows,
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
    use mesocosm_core::{PartId, Placement, Yaw};

    fn scripted() -> Vec<Intent> {
        vec![
            Intent::Move { delta: [1, 0, 0] },
            Intent::Idle,
            Intent::Move { delta: [0, 0, 2] },
            Intent::Deposit { mass_mg: 25 },
            Intent::Metabolize {
                organism: mesocosm_core::OrganismId(0),
                placement: Placement::Explicit {
                    parent: PartId(0),
                    offset: [4, 0, 0],
                    yaw: Yaw::Zero,
                },
            },
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

        assert_eq!(
            steady.trace(),
            ragged.trace(),
            "same intents in the same order"
        );
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

        let (replayed, past) = Runtime::replay(555, 20, rt.trace());
        assert_eq!(state_hash(&replayed), rt.state_hash());
        assert_eq!(&past, rt.history(), "and the same run has the same past");
    }

    #[test]
    fn a_driven_run_keeps_its_past() {
        // The world buffers one tick and drops it if nobody drains. Before the
        // driver recorded, every shipped run had a present and no history.
        let mut rt = Runtime::new(4_242, 40, 60);
        rt.step(200);

        assert!(!rt.history().is_empty(), "two hundred ticks left a record");
        assert!(!rt.readings().is_empty(), "and it comes to something");
    }

    #[test]
    fn ending_an_epoch_notes_what_the_run_did() {
        let mut rt = Runtime::new(4_242, 40, 60);
        rt.step(200);

        assert_eq!(rt.world().record().filled(), 0);
        let readings = rt.end_epoch();
        assert!(!readings.is_empty());
        assert!(rt.world().record().filled() > 0, "the record has it now");
        assert_eq!(rt.world().epoch, 1);
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
