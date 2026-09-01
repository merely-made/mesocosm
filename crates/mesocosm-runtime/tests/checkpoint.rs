// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! **The individual checkpoint, driven.** PE1's runtime half: a birth involving
//! the critter under your hand stops the world, one recorded choice resumes it,
//! and a body ending offers the line rather than the wall.
//!
//! Every birth below is the ordinary breeding transaction. Nothing here forces
//! one: the run takes control of a founder the adult-mass gate has already
//! cleared, and the next tick does what it was going to do anyway.
//!
//! The load-bearing negative is the first test. An enclosure nobody is touching
//! must never be interrupted — that is TD4's ruling and it is also why every
//! headless fixture in the workspace, the population instrument included, keeps
//! its exact timing across this phase.

use mesocosm_core::snapshot::encode;
use mesocosm_core::{Intent, OrganismId, World, state_hash};
use mesocosm_runtime::{Checkpoint, Occasion, Runtime};

/// How many ticks this body is from its next brood, or `None` if the ordinary
/// adult-mass gate would refuse it whatever its clock said.
///
/// **Nobody is ever observably "ready" between ticks**: the tick's aging pass
/// carries an organism over its gestation and the breed pass immediately after
/// spends it, so `can_reproduce` is true only inside a tick. The gate is
/// therefore asked of a *copy* with its clock wound forward, and then bisected
/// to find the exact tick it turns over. That is the ecology's own gate,
/// answered about a clone; the world is not touched and the birth, when it
/// comes, is the ordinary one.
fn ticks_to_brood(organism: &mesocosm_core::Organism) -> Option<u32> {
    if organism.stage != mesocosm_core::Stage::Mature {
        return None;
    }
    let mut probe = organism.clone();
    probe.since_offspring = u32::MAX;
    if !probe.can_reproduce() {
        return None;
    }
    let (mut lo, mut hi) = (0u32, u32::MAX);
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        probe.since_offspring = mid;
        if probe.can_reproduce() {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    Some(lo.saturating_sub(organism.since_offspring))
}

/// The founder nearest its next brood that this world would also let anyone
/// inhabit, and how long that is.
fn nearest_breeder(world: &World) -> Option<(OrganismId, u32)> {
    world
        .living()
        .filter(|organism| world.is_eligible(organism.id))
        .filter_map(|organism| ticks_to_brood(organism).map(|wait| (wait, organism.id)))
        .min()
        .map(|(wait, id)| (id, wait))
}

/// Drives to the first checkpoint, answering nothing. `Resume` rather than
/// `Idle` so the hand stays on the critter — which is what a player watching
/// their creature amounts to, and what the questions below require.
fn drive_to_a_checkpoint(rt: &mut Runtime, within: u64) -> &Checkpoint {
    for _ in 0..within {
        if rt.checkpoint().is_some() {
            break;
        }
        rt.queue(Intent::Resume);
        rt.step(1);
    }
    rt.checkpoint().expect("a checkpoint within the horizon")
}

/// Takes the founder closest to its next brood and holds it until the brood
/// arrives. Returns the parent and the checkpoint's own birth record.
fn run_to_a_birth(seed: u64) -> (Runtime, OrganismId, mesocosm_runtime::Birth) {
    let mut rt = Runtime::new(seed, 60, 10);
    // An enclosure of hundred-milligram founders has nobody heavy enough to
    // breed at genesis; the ecology has to grow one first. So: run it as an
    // ant farm until somebody is nearly there, then put a hand on them.
    let (breeder, wait) = loop {
        if let Some(found) = nearest_breeder(rt.world()).filter(|(_, wait)| *wait <= 40) {
            break found;
        }
        assert!(
            rt.step(1) == 1 && rt.world().tick < 4_000,
            "nobody in this enclosure grew up to breed"
        );
    };
    rt.queue(Intent::TakeControl { organism: breeder });
    rt.step(1);

    // Its own gestation plus a little, and no policy at all: this run does
    // nothing but hold the creature and let the ecology happen to it.
    let checkpoint = drive_to_a_checkpoint(&mut rt, u64::from(wait) + 20);
    let Occasion::Birth(birth) = checkpoint.occasion else {
        panic!(
            "the held critter was expected to breed, not {:?}",
            checkpoint.occasion
        );
    };
    assert_eq!(birth.parent, breeder);
    (rt, breeder, birth)
}

/// **The ant farm is never interrupted.** An enclosure with no hand on it runs
/// exactly as it always did — which is the ruling, and the reason no existing
/// fixture's timing moved.
#[test]
fn an_idle_terrarium_is_never_asked_anything() {
    let mut rt = Runtime::new(4_242, 60, 10);
    let ran = rt.step(1_200);
    assert_eq!(ran, 1_200, "every step ran");
    assert_eq!(rt.trace().len(), 1_200);
    assert!(
        rt.checkpoint().is_none(),
        "nobody was holding it, so nobody was asked"
    );
    assert_eq!(
        rt.world().tick,
        1_200,
        "and the world advanced a tick per step throughout"
    );
}

/// Parent, offspring, cost and descent, all four off the question itself — and
/// the cost read out of PE0's flow record rather than recomputed.
#[test]
fn a_birth_under_the_hand_opens_a_bounded_checkpoint() {
    let (rt, parent, birth) = run_to_a_birth(11);
    let checkpoint = rt.checkpoint().expect("standing").clone();
    assert_eq!(birth.parent, parent);
    assert!(birth.cost_mg() > 0, "a birth costs its parent something");
    assert_eq!(
        birth.cost_mg(),
        birth.substance_mg + birth.reserve_mg,
        "both accounts, and nothing else"
    );

    let child = rt
        .world()
        .living()
        .find(|o| o.id == birth.offspring)
        .expect("the offspring is in the enclosure");
    assert_eq!(
        child.biomass_mg(),
        birth.substance_mg,
        "the number on the question is the number the ledger reconciled"
    );
    assert_eq!(child.energy_mg, birth.reserve_mg);
    assert_eq!(child.species, birth.lineage, "and descent is on it");

    // One offering, not a brood. The other children of this line are out in
    // the enclosure and are reached the ordinary way.
    assert_eq!(checkpoint.heirs, vec![birth.offspring]);
}

/// Bounded: the world stops, one choice restarts it, and nothing in between
/// consumes an intent or advances a tick.
#[test]
fn the_world_holds_until_the_question_is_answered() {
    let (mut rt, _, _) = run_to_a_birth(11);
    let held_at = rt.world().tick;
    let trace = rt.trace().len();
    assert_eq!(rt.step(10), 0, "an unanswered question is a stopped world");
    assert_eq!(rt.world().tick, held_at);
    assert_eq!(rt.trace().len(), trace, "and nothing entered the record");

    // A verb is not an answer, and is not spent trying to be one.
    rt.queue(Intent::Deposit { mass_mg: 5 });
    assert_eq!(rt.step(4), 0);
    assert_eq!(rt.queued_len(), 1, "the intent is still there, unspent");
    assert!(rt.checkpoint().is_some());
    rt.queue(Intent::Resume);

    // The deposit is still at the front, so the answer behind it does not
    // sneak the world forward either.
    assert_eq!(rt.step(4), 0);
    assert!(rt.checkpoint().is_some());
}

/// One recorded choice resumes play, and it is in the trace afterwards.
#[test]
fn one_recorded_choice_resumes_play() {
    let (mut rt, _, _) = run_to_a_birth(11);
    let held_at = rt.world().tick;

    rt.queue(Intent::Resume);
    assert_eq!(rt.step(1), 1, "the answer let the world go");
    assert!(rt.checkpoint().is_none());
    assert_eq!(rt.world().tick, held_at + 1);
    assert_eq!(rt.trace().last(), Some(&Intent::Resume));

    assert_eq!(rt.step(5), 5, "and play carries on");
}

/// The other answer. Taking the offspring is the ordinary control intent
/// through the ordinary eligibility gate, and it moves the run into the body
/// the parent just paid for.
#[test]
fn taking_the_offspring_is_the_other_answer() {
    let (mut rt, parent, _) = run_to_a_birth(11);
    let heir = rt
        .checkpoint()
        .and_then(|checkpoint| checkpoint.heir())
        .expect("a newborn to take");

    rt.queue(Intent::TakeControl { organism: heir });
    assert_eq!(rt.step(1), 1);
    assert!(rt.checkpoint().is_none());
    assert_eq!(rt.world().controlled_id(), Some(heir));
    assert_ne!(heir, parent);
    assert!(
        rt.world().living().any(|o| o.id == parent),
        "and the parent is still an organism in the enclosure, not a discard"
    );
}

/// **The hold is not in the world.** A run that stopped to ask, and a bare
/// world handed the same trace, are the same world — body, control holder,
/// history, readings and state hash. That is what keeps a pause out of the
/// determinism story entirely.
#[test]
fn a_paused_run_replays_to_the_same_everything() {
    let (mut rt, _, _) = run_to_a_birth(11);
    rt.queue(Intent::Resume);
    rt.step(1);
    drive_to_a_checkpoint(&mut rt, 3_000);
    let heir = rt.checkpoint().and_then(|c| c.heir());
    rt.queue(match heir {
        Some(organism) => Intent::TakeControl { organism },
        None => Intent::Resume,
    });
    rt.step(1);
    rt.step(20);

    let replayed = Runtime::replayed(11, 60, rt.trace());
    assert_eq!(state_hash(&replayed.world), rt.state_hash());
    assert_eq!(
        replayed.world.controlled_id(),
        rt.world().controlled_id(),
        "the same control holder"
    );
    assert_eq!(
        replayed.world.body().map(|body| body.len()),
        rt.world().body().map(|body| body.len()),
        "the same body"
    );
    assert_eq!(&replayed.history, rt.history(), "the same past");
    assert_eq!(
        encode(&replayed.readings).expect("encodable"),
        encode(rt.windows()).expect("encodable"),
        "and the same readings, byte for byte"
    );
}

/// Runs one enclosure from a birth to the parent's death, answering every
/// question by carrying on. `None` when the line did not outlive its founder —
/// a fact about that terrarium, not about the checkpoint.
fn run_to_a_loss(seed: u64) -> Option<(Runtime, OrganismId)> {
    let (mut rt, parent, _) = run_to_a_birth(seed);
    rt.queue(Intent::Resume);
    rt.step(1);

    // Nothing but answers from here: a critter that never eats spends its rent
    // down and the enclosure takes it back, which is the ordinary way a run
    // ends and the one this phase is about.
    loop {
        match rt.checkpoint().map(|checkpoint| checkpoint.occasion) {
            Some(Occasion::Loss(loss)) => {
                assert_eq!(loss.organism, parent);
                assert!(!rt.world().is_embodied(), "the body is gone");
                let heir = rt.checkpoint().expect("standing").heir()?;
                return Some((rt, heir));
            }
            Some(Occasion::Birth(_)) => {
                rt.queue(Intent::Resume);
                rt.step(1);
            }
            None => {
                rt.queue(Intent::Resume);
                assert_eq!(rt.step(1), 1, "the world kept running");
                assert!(
                    rt.world().tick < 8_000,
                    "the played critter outlived the horizon"
                );
            }
        }
    }
}

/// **Death is a hinge.** The body ends, the question names who is left, and the
/// run continues in a descendant instead of refusing every input forever.
///
/// Several enclosures, because whether a newborn survives its parent is the
/// terrarium's business and not this phase's: a juvenile in a busy enclosure is
/// food like anything else, and a line that leaves nobody behind is a real
/// outcome the checkpoint reports honestly. The claim under test is what
/// happens when one *does* survive.
#[test]
fn a_death_under_the_hand_offers_the_line() {
    for seed in [11u64, 3, 7, 21] {
        let Some((mut rt, heir)) = run_to_a_loss(seed) else {
            continue;
        };
        rt.queue(Intent::TakeControl { organism: heir });
        assert_eq!(rt.step(1), 1);
        assert_eq!(rt.world().controlled_id(), Some(heir));
        assert!(rt.checkpoint().is_none(), "and play resumed");
        assert_eq!(rt.step(20), 20, "in a body that goes on living");
        return;
    }
    panic!("no line outlived its founder in any of the seeds tried");
}

/// Declining is an answer too. The wall is still reachable — it is just a
/// choice now rather than the only thing that happens.
#[test]
fn declining_the_line_resumes_disembodied() {
    let (mut rt, _, _) = run_to_a_birth(11);
    let answer = rt.checkpoint().expect("standing").default_answer();
    assert_eq!(answer, Intent::Resume, "the world default changes nothing");
    rt.queue(answer);
    assert_eq!(rt.step(1), 1);
    assert!(rt.checkpoint().is_none());
}
