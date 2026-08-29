// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! **Total matter is conserved.** TD6's load-bearing invariant.
//!
//! The enclosure has a finite matter budget: producers draw out of the soil
//! column they stand on, rent and travel and decay put it back where the body
//! is, a death releases the reserve it was carrying, and the player's deposit
//! enriches the ground. Light is the one open input, and light is not matter.
//! So the sum of soil, living substance, carrion, and banked reserves is a
//! constant of a run, and mass cannot run away because it has to be somewhere.
//!
//! # Exceptions
//!
//! None. There is no documented sink or source in the tick or in any intent —
//! that is what the checks below assert, milligram-exact, tick by tick, rather
//! than within a tolerance that would hide one.
//!
//! Two things deliberately outside the account, because they are not matter:
//!
//! - **Light.** A producer's income is drawn from soil; the free energy that
//!   powers the drawing never enters the ledger. This is the ruling.
//! - **Ground bricks.** [`Ground`](mesocosm_core::places::Ground) is what is
//!   solid and walkable; the soil store is what can be eaten out of the floor.
//!   Carving air changes the first and not the second, which is why
//!   `Intent::Carve` moves no matter and is checked here saying so.
//!
//! # The instrument is proved, not assumed
//!
//! An absence is evidence only beside a positive control in the same run, so
//! [`conserved`] — the exact check the long runs use — is also handed a world
//! with a conjured milligram and a leaked one, and must report both.

use mesocosm_core::{Intent, Placement, World};

/// The ledger, or what is wrong with it. `Ok` is silence; `Err` is the message
/// a failing conservation assertion prints.
///
/// One function, used by the runs that must pass **and** by the controls that
/// must fail, because a check that only ever gets shown conserved worlds has
/// not been shown to detect anything.
fn conserved(world: &World, expected_mg: u64, at: &str) -> Result<(), String> {
    let actual = world.total_matter_mg();
    if actual == expected_mg {
        return Ok(());
    }
    let soil = world.soil().total_mg();
    Err(format!(
        "matter is not conserved {at}: {actual} mg against {expected_mg} mg at genesis \
         ({} mg {}); soil {soil} mg, {} bodies holding {} mg",
        actual.abs_diff(expected_mg),
        if actual > expected_mg {
            "conjured"
        } else {
            "leaked"
        },
        world.organisms.len(),
        actual - soil.min(actual),
    ))
}

#[test]
fn matter_is_conserved_across_a_long_run() {
    // Four seeds across four thousand idle ticks each: births, deaths,
    // grazing, predation, scavenging, decay, dispersal, and the founding
    // cohort dying of old age all happen inside this window.
    for seed in [1u64, 4, 7, 4_242] {
        let mut world = World::new(seed, 60);
        let opening = world.total_matter_mg();
        assert!(opening > 0, "seed {seed} founded an empty enclosure");

        for tick in 1..=4_000 {
            world.apply(Intent::Idle);
            if let Err(why) = conserved(&world, opening, &format!("on tick {tick} of seed {seed}"))
            {
                panic!("{why}");
            }
        }
    }
}

#[test]
fn matter_is_conserved_through_the_played_verbs() {
    // The tick is not the only thing that moves matter. Every acting intent
    // that touches a ledger is exercised here: a meal (burned or built, the
    // body decides), a deposit into the ground, movement paid in substance,
    // and a carve, which must move none at all.
    let mut world = World::new(11, 40);
    let opening = world.total_matter_mg();

    let mut trace = vec![
        Intent::Deposit { mass_mg: 60 },
        Intent::Move { delta: [1, 0, 0] },
        Intent::Move { delta: [-1, 0, 1] },
        Intent::Carve {
            at: world.position().expect("a played critter"),
            radius: 1,
        },
    ];
    // Whatever the played critter can reach, in id order, so the meal is a
    // real one rather than a rejection.
    let me = world.controlled_id().expect("a played critter");
    for organism in world.living().map(|o| o.id).collect::<Vec<_>>() {
        if organism != me {
            trace.push(Intent::Metabolize {
                organism,
                placement: Placement::Planned,
            });
        }
    }

    for (step, intent) in trace.into_iter().enumerate() {
        world.apply(intent);
        conserved(&world, opening, &format!("after played step {step}")).expect("conserved");
    }
}

#[test]
fn the_check_catches_income_conjured_the_way_it_used_to_be() {
    // **The broken control.** Before TD6 a producer's income was a number the
    // world minted: `earn` credited the budget or the body with
    // `producer_income_for_mass(...)` and nothing anywhere was debited. Replay
    // exactly that on top of a conserved tick — one producer, one tick's worth
    // of the old free income — and the check must say so.
    let mut world = World::new(1, 60);
    let opening = world.total_matter_mg();
    world.apply(Intent::Idle);
    conserved(&world, opening, "before the control").expect("the run itself conserves");

    let producer = world
        .living()
        .find(|o| o.kingdom() == mesocosm_core::Kingdom::Producer)
        .map(|o| o.id)
        .expect("seed 1 founds producers");
    let conjured = 11; // a ~300 mg producer's old per-tick fixing income
    world
        .organisms
        .iter_mut()
        .find(|o| o.id == producer)
        .expect("still on the roster")
        .energy_mg += conjured;

    let complaint = conserved(&world, opening, "after conjuring a producer's old income")
        .expect_err("a conjured milligram must not pass");
    assert!(
        complaint.contains("conjured"),
        "the check must name the direction: {complaint}"
    );
    assert!(
        complaint.contains(&conjured.to_string()),
        "and the size of the discrepancy: {complaint}"
    );
}

#[test]
fn the_check_catches_a_leak() {
    // The other direction, and the one the round actually had to close in
    // several places: matter spent and never put anywhere.
    let mut world = World::new(1, 60);
    let opening = world.total_matter_mg();
    world.apply(Intent::Idle);

    world.organisms[0].spend_mass(25);

    let complaint = conserved(&world, opening, "after burning mass into nothing")
        .expect_err("a leaked milligram must not pass");
    assert!(
        complaint.contains("leaked"),
        "the check must name the direction: {complaint}"
    );
}
