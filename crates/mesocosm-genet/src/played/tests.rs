// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What the recorded demo has to contain, asserted against a recording.
//!
//! Split out of `played.rs` at the 600-line ceiling when P3's branch transfer
//! added a claim. Writing the script and checking what it produces are two
//! jobs, and only the first belongs beside the trace types.

use std::sync::LazyLock;

use super::*;
use mesocosm_core::{INSTINCT_IDLE_TICKS, OrganismId, SpeciesId};
use mesocosm_runtime::{Birth, Loss};

/// Steps and founders the assertions below record with.
///
/// **Not the shipped demo's.** Recording 3,100 steps of the shipping cohort
/// takes minutes unoptimized, and a workspace test run should not. These
/// tests are about the *script* — that it walks, eats, digs, deposits, puts
/// its hands down, and replays — and a few hundred ticks says all of that.
/// The claim the shipped length exists for, that the run reaches a birth
/// and a death, is receipted by the recording itself and by
/// `mesocosm-runtime`'s checkpoint tests.
const TEST_STEPS: u64 = 400;
const TEST_FOUNDERS: u32 = 60;

/// One recording, shared. Each of these tests used to pay for its own.
static TRACE: LazyLock<PlayedTrace> =
    LazyLock::new(|| record_demo(DEMO_SEED, TEST_FOUNDERS, 10, TEST_STEPS));

#[test]
fn the_demo_trace_exercises_every_verb_the_slice_claims() {
    let trace = &*TRACE;
    assert_eq!(trace.intents.len(), TEST_STEPS as usize);

    let moves: std::collections::BTreeSet<_> = trace
        .intents
        .iter()
        .filter_map(|intent| match intent {
            Intent::Move { delta } => Some([delta[0].signum(), delta[2].signum()]),
            _ => None,
        })
        .collect();
    assert!(
        moves.len() >= 4,
        "the trace walks in more than one direction: {moves:?}"
    );
    for verb in ["Metabolize", "Deposit", "Carve", "Idle"] {
        assert!(
            trace
                .intents
                .iter()
                .any(|intent| format!("{intent:?}").starts_with(verb)),
            "the trace contains a {verb}"
        );
    }
}

/// The claim `--replay` makes: the recorded hash is reachable from the
/// seed and the trace alone, with no host in the loop.
#[test]
fn the_demo_trace_replays_to_its_recorded_hash() {
    let trace = &*TRACE;
    let (world, _) = Runtime::replay(trace.seed, trace.organisms, &trace.intents);
    assert_eq!(mesocosm_core::state_hash(&world), trace.state_hash);
}

/// TD4's other half, receipted: the recorded run must actually let go, or
/// the fixture proves nothing about instincts under idleness.
#[test]
fn the_demo_trace_puts_its_hands_down_long_enough_to_lose_the_body() {
    let trace = &*TRACE;
    let longest = trace
        .intents
        .iter()
        .fold((0u32, 0u32), |(run, best), intent| {
            let run = if matches!(intent, Intent::Idle) {
                run + 1
            } else {
                0
            };
            (run, best.max(run))
        })
        .1;
    assert!(
        longest >= INSTINCT_IDLE_TICKS,
        "the demo idles {longest} in a row, short of the {INSTINCT_IDLE_TICKS} \
         it takes for the ecology to take the body back"
    );

    let (world, _) = Runtime::replay(trace.seed, trace.organisms, &trace.intents);
    assert!(
        world.controlled_id().is_none() || world.idle_run() < INSTINCT_IDLE_TICKS,
        "and the run ends back under the hand, so the capture is of a played world"
    );
}

/// The script's whole answer to a checkpoint, in one place: continue at a
/// birth, succeed at a death, decline only when there is nothing left to
/// continue through. Pure, so it is asserted directly rather than fished
/// out of a long recording.
#[test]
fn the_demo_continues_at_a_birth_and_succeeds_at_a_death() {
    let heirless = Checkpoint {
        tick: 10,
        occasion: Occasion::Loss(Loss {
            organism: OrganismId(4),
            lineage: SpeciesId(1),
        }),
        heirs: Vec::new(),
    };
    assert_eq!(answer(&heirless), Intent::Resume);

    let carried = Checkpoint {
        heirs: vec![OrganismId(9), OrganismId(12)],
        ..heirless.clone()
    };
    assert_eq!(
        answer(&carried),
        Intent::TakeControl {
            organism: OrganismId(9)
        },
        "the eldest descendant, and only it"
    );

    let birth = Checkpoint {
        tick: 20,
        occasion: Occasion::Birth(Birth {
            parent: OrganismId(4),
            offspring: OrganismId(9),
            lineage: SpeciesId(1),
            substance_mg: 500,
            reserve_mg: 500,
        }),
        heirs: vec![OrganismId(9)],
    };
    assert_eq!(
        answer(&birth),
        Intent::Resume,
        "a birth keeps the body the run has been growing"
    );
}

/// PE2's two claims, in the recorded loop rather than only in a fixture.
///
/// **A non-food discovery**, reached by the script putting the food down
/// long enough to come through the starvation horizon; and **a meal that
/// refuses an incompatible candidate** — every one of the demo's meals is
/// an observation, and the record says the endurance condition could not be
/// reached by any of them because it never declared that lane.
#[test]
fn the_demo_reaches_a_non_food_discovery_and_a_meal_that_unlocks_nothing() {
    let trace = &*TRACE;
    let (world, _) = Runtime::replay(trace.seed, trace.organisms, &trace.intents);

    let discoveries = world.discoveries();
    assert_eq!(
        discoveries.len(),
        1,
        "the recorded run comes through exactly one condition: {discoveries:?}"
    );
    let discovery = discoveries[0];
    assert_eq!(
        discovery.route,
        mesocosm_core::Input::Endurance,
        "and it is not a meal that taught it"
    );
    assert!(matches!(
        discovery.evidence,
        mesocosm_core::Evidence::Endured { .. }
    ));
    assert!(
        world.last_observation().is_some(),
        "and the run's last evidence is on the record either way"
    );
}

/// The other half, asserted where the routing happens rather than only at
/// the end of a run: a meal's evidence cannot reach the endurance
/// condition, and the observation says so in those words.
#[test]
fn a_recorded_meal_is_observed_and_unlocks_nothing() {
    let mut runtime = Runtime::new(DEMO_SEED, TEST_FOUNDERS, 10);
    let volumes = crate::fixture::volumes();
    let mut script = Script::default();
    let mut seen = None;
    for step in 0..TEST_STEPS {
        let intent = match runtime.checkpoint() {
            Some(checkpoint) => answer(checkpoint),
            None => demo_intent(runtime.world(), &volumes, step, &mut script),
        };
        let ate = matches!(intent, Intent::Metabolize { .. });
        runtime.queue(intent);
        runtime.step(1);
        if ate && let Some(observation) = runtime.world().last_observation() {
            seen = Some(observation.clone());
            break;
        }
    }
    let observation = seen.expect("the script eats early and often");
    assert_eq!(observation.route, mesocosm_core::Input::Meal);
    assert!(
        observation
            .missed
            .iter()
            .any(|(_, miss)| matches!(miss, mesocosm_core::Miss::UndeclaredInput)),
        "a meal cannot be offered to a condition that never asked about \
         meals: {observation:?}"
    );
}

/// P3's receipt, in the recorded loop rather than only in a fixture: the
/// demo takes one branch off a carcass, and the world remembers the terms.
///
/// **Its own recording, at the shipping density.** Whether a carcass
/// carrying a branch is ever within reach is the enclosure's business, and
/// the 60-founder world the tests above share is fifteen times sparser than
/// the one the demo ships in — it offers none inside the window. So this
/// records the cohort the claim is actually about, and stops as soon as the
/// window has closed.
#[test]
fn the_demo_takes_one_branch_off_a_carcass() {
    let trace = record_demo(DEMO_SEED, mesocosm_core::world::FOUNDERS, 10, 360);
    let grafts = trace
        .intents
        .iter()
        .filter(|intent| matches!(intent, Intent::Graft { .. }))
        .count();
    assert_eq!(grafts, 1, "one branch, taken once");

    let (world, _) = Runtime::replay(trace.seed, trace.organisms, &trace.intents);
    let graft = world
        .last_graft()
        .expect("the transfer landed, or the intent above was a refusal");
    assert!(graft.parts.len() >= 2, "a branch, not an organ: {graft:?}");
    assert!(graft.mass_mg > 0);
    // Every part of it names the part it came off, which is the whole of
    // what a transfer keeps that growing does not.
    let body = world.body().expect("still embodied");
    for part in &graft.parts {
        assert!(
            matches!(
                body.part(*part).map(|found| &found.provenance.origin),
                Some(mesocosm_core::Origin::Incorporated { .. })
            ),
            "part {part:?} lost its provenance"
        );
    }
}

/// A carve that removed nothing would leave the section with no dirty
/// bricks to drain, and the whole refresh path untested by the receipt.
#[test]
fn the_demo_trace_changes_the_ground() {
    let trace = &*TRACE;
    let (world, _) = Runtime::replay(trace.seed, trace.organisms, &trace.intents);
    assert!(world.ground().revision() > 0, "the digging removed voxels");
}
