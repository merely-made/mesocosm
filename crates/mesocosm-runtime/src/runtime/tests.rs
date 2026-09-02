// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The driver's own claims: frame delivery does not reach the simulation, a
//! step cap defers work rather than dropping it, and a trace replays to the
//! world it recorded.
//!
//! Split out of `runtime.rs` at the 600-line ceiling when PE3b added the
//! review. What the *checkpoint* and the *board* do to stepping are driven
//! claims and live in `tests/checkpoint.rs` and `tests/review.rs`.

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
