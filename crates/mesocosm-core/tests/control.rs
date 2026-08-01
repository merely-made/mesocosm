// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! P1: one organism model, and control as a recorded pointer.
//!
//! Before this the played critter was a `BodyDocument`, a position, and an
//! energy budget living on `World`, beside a vector of scalar organisms that
//! had none of those things. Anatomy could not constrain an unplayed creature,
//! prey had no parts to lose, and switching lineage would have meant rebuilding
//! state.
//!
//! The claim these tests pin is that **the played critter is an ordinary
//! organism that happens to be pointed at**, that nothing in the rules can tell
//! the difference, and that the pointer moves only through a recorded intent.

use mesocosm_core::{
    Intent, Outcome, OrganismId, Placement, Rejection, Route, World, restore, snapshot,
    state_hash,
};

/// The nearest organism that is not the player.
fn prey(world: &World) -> OrganismId {
    let here = world.position().expect("somebody is embodied");
    world
        .organisms
        .iter()
        .filter(|o| Some(o.id) != world.controlled_id() && o.is_alive())
        .map(|o| (o.id, o.position))
        .min_by_key(|(_, at)| (0..3).map(|a| (at[a] - here[a]).abs()).max().unwrap_or(0))
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
fn control_moves_only_through_a_recorded_intent() {
    // Ordered intents are the only mutation path. A control change made
    // outside that path would replay every fact about a run except who was
    // living it, and lineage switching is gameplay.
    let mut world = World::new(4_242, 24);
    let other = prey(&world);

    let roster_before = world.organisms.clone();
    let mine = world.body().unwrap().clone();

    let outcome = world.apply(Intent::TakeControl { organism: other });

    assert_eq!(outcome, Outcome::Inhabited { organism: other });
    assert_eq!(world.controlled_id(), Some(other));
    assert_ne!(world.body().unwrap(), &mine, "the played body is somebody else's now");
    // The ecology ran a tick, so the roster is not frozen; what matters is
    // that nothing was rebuilt or copied for the sake of control.
    assert!(roster_before.len().abs_diff(world.organisms.len()) < 5);
}

#[test]
fn a_control_change_replays() {
    // The hole this closes: a trace containing a lineage switch must reproduce
    // who was inhabited, not merely what happened to the world.
    let trace = [
        Intent::Move { delta: [1, 0, 0] },
        Intent::Idle,
        Intent::Move { delta: [0, 0, 1] },
    ];

    let mut straight = World::new(4_242, 24);
    let other = prey(&straight);
    straight.apply(Intent::TakeControl { organism: other });
    straight.apply_all(&trace);

    let mut forked = World::new(4_242, 24);
    forked.apply(Intent::TakeControl { organism: other });
    forked.apply_all(&trace[..1]);
    let mut resumed = restore(&snapshot(&forked).unwrap()).unwrap();
    resumed.apply_all(&trace[1..]);

    assert_eq!(state_hash(&straight), state_hash(&resumed));
    assert_eq!(resumed.controlled_id(), Some(other), "and control survived the snapshot");
}

#[test]
fn control_refuses_an_organism_that_cannot_be_played() {
    let mut world = World::new(3, 8);
    let before = world.controlled_id();

    let absent = OrganismId(9_999);
    assert_eq!(
        world.apply(Intent::TakeControl { organism: absent }),
        Outcome::Rejected(Rejection::NoSuchOrganism(absent))
    );
    assert_eq!(world.controlled_id(), before, "control did not move");
}

#[test]
fn serialization_does_not_distinguish_the_played_critter() {
    // Law C at the level of the simulation rather than the file format. Two
    // worlds identical except for which organism is pointed at must serialize
    // to the same length and restore identically, because the only difference
    // is one id.
    let mut world = World::new(4_242, 24);
    let other = prey(&world);

    let mut moved = world.clone();
    moved.apply(Intent::TakeControl { organism: other });
    world.apply(Intent::Idle);

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
    let other = prey(&unplayed);

    unplayed.apply(Intent::TakeControl { organism: other });
    played.apply(Intent::Idle);

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
    // Only expressible since P1 put the player in the roster. This forbids
    // targeting yourself as *prey*; consuming one of your own parts during
    // starvation or metamorphosis is a different, part-addressed operation and
    // is not ruled out here.
    let mut world = World::new(5, 12);
    let me = world.controlled_id().unwrap();

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
fn dying_ends_control_rather_than_the_world() {
    // Killed through the ecology, not by removing the row. Natural death makes
    // an organism carrion, which lingers until it is spent, so an earlier cut
    // that only checked for the id kept a decomposing critter walking around.
    let mut world = World::new(13, 16);
    let me = world.controlled_id().unwrap();

    // Starve it. Upkeep does the rest, and nothing intervenes on its behalf.
    world.organisms.iter_mut().find(|o| o.id == me).unwrap().mass_mg = 1;

    let mut ticks = 0;
    while world.is_embodied() && ticks < 500 {
        world.apply(Intent::Idle);
        ticks += 1;
    }

    assert!(!world.is_embodied(), "the played critter died like anything else");
    assert_eq!(world.control_lost(), Some(me), "and the world says whose body was lost");
    assert_eq!(world.controlled_id(), None, "the pointer was released, not left stale");
}

#[test]
fn a_carcass_cannot_be_played() {
    // The specific defect: carrion is still in the roster, so an id check
    // alone reports it as embodied.
    let mut world = World::new(21, 16);
    let me = world.controlled_id().unwrap();
    world.organisms.iter_mut().find(|o| o.id == me).unwrap().stage =
        mesocosm_core::Stage::Carrion;

    assert!(world.organisms.iter().any(|o| o.id == me), "the row is still there");
    assert!(!world.is_embodied(), "and it is not somebody you can be");
    assert_eq!(
        world.apply(Intent::Move { delta: [1, 0, 0] }),
        Outcome::Rejected(Rejection::Disembodied),
        "a corpse does not walk"
    );
}

#[test]
fn a_world_can_outlive_whoever_was_in_it() {
    // Nobody home is a state, not a crash, and it is the seam where
    // witnessing, adaptation, and choosing another critter happen.
    let mut world = World::new(13, 16);
    let me = world.controlled_id().unwrap();
    world.organisms.retain(|o| o.id != me);
    world.apply(Intent::Idle);

    assert!(!world.is_embodied());
    assert_eq!(world.controlled(), None);
    assert_eq!(world.body(), None, "no ghost body at the origin");
    assert_eq!(world.position(), None);
    assert_eq!(world.energy_mg(), None);

    assert_eq!(
        world.apply(Intent::Move { delta: [1, 0, 0] }),
        Outcome::Rejected(Rejection::Disembodied),
        "acting refuses rather than panicking"
    );
    assert_eq!(world.apply(Intent::Idle), Outcome::Idled, "and time still passes");
}

#[test]
fn a_disembodied_world_can_be_inhabited_again() {
    // What disembodiment is *for*: it is a seam, not a dead end.
    let mut world = World::new(29, 24);
    let me = world.controlled_id().unwrap();
    let heir = prey(&world);

    world.organisms.retain(|o| o.id != me);
    world.apply(Intent::Idle);
    assert!(!world.is_embodied());

    assert_eq!(
        world.apply(Intent::TakeControl { organism: heir }),
        Outcome::Inhabited { organism: heir }
    );
    assert!(world.is_embodied());
    assert_eq!(world.controlled_id(), Some(heir));
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
fn reproduction_does_not_manufacture_body_mass() {
    // A parent used to pay a quarter of its scalar mass while its offspring
    // received a clone of its whole anatomy, so a forty-part parent produced a
    // forty-part child out of nothing. An offspring now starts at exactly what
    // was paid for.
    //
    // Scoped to newborns on purpose. The ecology moves `mass_mg` by grazing
    // and upkeep without touching anatomy, so the two ledgers diverge with
    // age; reconciling them is P2's work and this test must not pretend it is
    // already done.
    let mut world = World::new(4_242, 40);

    let mut checked = 0;
    for _ in 0..600 {
        world.apply(Intent::Idle);
        for newborn in world.organisms.iter().filter(|o| o.age == 0) {
            assert_eq!(
                newborn.body.total_mass_mg(),
                newborn.mass_mg,
                "{:?} was born with anatomy it did not pay for",
                newborn.id
            );
            assert_eq!(newborn.body.len(), 1, "an offspring starts as one part");
            checked += 1;
        }
        if checked > 3 {
            break;
        }
    }

    assert!(checked > 0, "the fixture actually reproduced");
}
