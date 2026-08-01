// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! P1: one organism model, and control as a pointer.
//!
//! Before this the played critter was a `BodyDocument`, a position, and an
//! energy budget living on `World`, beside a vector of scalar organisms that
//! had none of those things. Anatomy could not constrain an unplayed creature,
//! prey had no parts to lose, and switching lineage would have meant rebuilding
//! state.
//!
//! The claim these tests pin is that **the played critter is an ordinary
//! organism that happens to be pointed at**, and that nothing in the rules can
//! tell the difference.

use mesocosm_core::{
    Intent, Outcome, Placement, Rejection, Route, World, restore, snapshot, state_hash,
};

/// The nearest organism that is not the player.
fn prey(world: &World) -> mesocosm_core::OrganismId {
    world
        .organisms
        .iter()
        .filter(|o| o.id != world.controlled_id())
        .map(|o| (o.id, o.position))
        .min_by_key(|(_, at)| {
            (0..3).map(|a| (at[a] - world.position()[a]).abs()).max().unwrap_or(0)
        })
        .expect("the fixture scatters organisms")
        .0
}

#[test]
fn the_player_is_an_ordinary_organism() {
    let world = World::new(4_242, 24);
    let me = world.controlled().expect("somebody is being played");

    assert!(world.organisms.iter().any(|o| o.id == me.id), "in the roster like anything else");
    assert!(!me.body.is_empty(), "with a body");
    assert!(me.energy_mg > 0, "and a budget");
}

#[test]
fn every_organism_has_a_body_now() {
    // The thing that unblocks damage, branch transfer, and phenotype-granted
    // action: there is no scalar organism left to special-case.
    let world = World::new(11, 24);
    for organism in &world.organisms {
        assert!(!organism.body.is_empty(), "{:?} has anatomy", organism.id);
        assert_eq!(
            organism.half_extent(),
            organism.body.part(organism.body.root).unwrap().half_extent,
            "and its shape is read off that anatomy rather than stored beside it"
        );
    }
}

#[test]
fn control_is_a_pointer_and_moving_it_rebuilds_nothing() {
    // The core claim. Taking control of another organism changes which body
    // the world reports and nothing else about the world's contents.
    let mut world = World::new(4_242, 24);
    let other = prey(&world);

    let roster_before = world.organisms.clone();
    let mine = world.body().clone();

    assert!(world.take_control(other));

    assert_eq!(world.organisms, roster_before, "no state was rebuilt or copied");
    assert_eq!(world.controlled_id(), other);
    assert_ne!(world.body(), &mine, "but the played body is somebody else's now");
}

#[test]
fn control_refuses_an_organism_that_does_not_exist() {
    let mut world = World::new(3, 8);
    let before = world.controlled_id();
    assert!(!world.take_control(mesocosm_core::OrganismId(9_999)));
    assert_eq!(world.controlled_id(), before, "and control did not move");
}

#[test]
fn serialization_does_not_distinguish_the_played_critter() {
    // Law C at the level of the simulation rather than the file format. Two
    // worlds identical except for which organism is pointed at must serialize
    // to the same length and restore identically, because the only difference
    // is one id.
    let world = World::new(4_242, 24);
    let other = prey(&world);

    let mut moved = world.clone();
    moved.take_control(other);

    let a = snapshot(&world).unwrap();
    let b = snapshot(&moved).unwrap();
    assert_eq!(a.len(), b.len(), "control costs no representation");

    assert_eq!(restore(&a).unwrap(), world);
    assert_eq!(restore(&b).unwrap(), moved);
}

#[test]
fn the_ecology_treats_the_played_critter_like_everything_else() {
    // Nothing exempts the player. If it did, that exemption would be a rule
    // branching on who is playing, which is the marker Law C forbids.
    let mut played = World::new(77, 24);
    let mut unplayed = played.clone();
    unplayed.take_control(prey(&unplayed));

    for _ in 0..40 {
        played.apply(Intent::Idle);
        unplayed.apply(Intent::Idle);
    }

    assert_eq!(
        played.organisms, unplayed.organisms,
        "forty ticks of ecology ran identically regardless of who was inhabited"
    );
}

#[test]
fn a_critter_cannot_eat_itself() {
    // Only expressible since P1 put the player in the roster.
    let mut world = World::new(5, 12);
    let me = world.controlled_id();
    assert_eq!(
        world.apply(Intent::Metabolize { organism: me, route: Route::Burn }),
        Outcome::Rejected(Rejection::Itself)
    );
    assert_eq!(
        world.apply(Intent::Metabolize {
            organism: me,
            route: Route::Incorporate { placement: Placement::Planned },
        }),
        Outcome::Rejected(Rejection::Itself)
    );
}

#[test]
fn a_world_can_outlive_whoever_was_in_it() {
    // Nobody home is a state, not a crash. It is what "leave the world
    // running" has to mean, and what lineage switching passes through.
    let mut world = World::new(13, 16);
    let me = world.controlled_id();
    world.organisms.retain(|o| o.id != me);

    assert!(!world.is_embodied());
    assert_eq!(world.controlled(), None);
    assert_eq!(world.position(), [0, 0, 0], "and the accessors still answer");
    assert_eq!(world.energy_mg(), 0);

    assert_eq!(
        world.apply(Intent::Move { delta: [1, 0, 0] }),
        Outcome::Rejected(Rejection::Disembodied),
        "acting refuses rather than panicking"
    );
    assert_eq!(world.apply(Intent::Idle), Outcome::Idled, "and time still passes");
}

#[test]
fn replay_holds_within_the_new_schema() {
    // The determinism guarantee, restated honestly: a build replays its own
    // traces. The migration moved hashes because the schema changed, and that
    // is a version boundary rather than a broken promise.
    let trace = [
        Intent::Move { delta: [1, 0, 1] },
        Intent::Idle,
        Intent::Deposit { mass_mg: 50 },
        Intent::Move { delta: [-2, 0, 0] },
    ];

    let mut straight = World::new(77, 12);
    straight.apply_all(&trace);

    let mut forked = World::new(77, 12);
    forked.apply_all(&trace[..2]);
    let mut resumed = restore(&snapshot(&forked).unwrap()).unwrap();
    resumed.apply_all(&trace[2..]);

    assert_eq!(state_hash(&straight), state_hash(&resumed));
}

#[test]
fn the_same_trace_gives_the_same_result_whoever_runs_it() {
    // The property that actually matters across the migration, and the one
    // hash stability was standing in for: the rules are about bodies and
    // intents, not about which body is special.
    let base = World::new(31, 24);
    let other = prey(&base);

    let trace = [Intent::Move { delta: [1, 0, 0] }, Intent::Idle, Intent::Move { delta: [0, 0, 1] }];

    let mut first = base.clone();
    let start_first = first.position();
    first.apply_all(&trace);
    let moved_first = [
        first.position()[0] - start_first[0],
        first.position()[1] - start_first[1],
        first.position()[2] - start_first[2],
    ];

    let mut second = base;
    second.take_control(other);
    let start_second = second.position();
    second.apply_all(&trace);
    let moved_second = [
        second.position()[0] - start_second[0],
        second.position()[1] - start_second[1],
        second.position()[2] - start_second[2],
    ];

    assert_eq!(moved_first, moved_second, "the same intents move whoever is inhabited");
}
