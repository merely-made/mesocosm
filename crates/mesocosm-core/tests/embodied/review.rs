// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! PE3b: the played line's own turn, in a world.
//!
//! The scorer's claims are `src/world/adapt/tests.rs`', the review's own
//! arithmetic is `src/world/review/tests.rs`', and what a round does on its own
//! is next door in `round.rs`. What lives here is the join: a line that has
//! come to something, the table it is offered, and the price a birth then
//! actually pays.

use mesocosm_core::discovery::HUNGER_TICKS;
use mesocosm_core::history::Event;
use mesocosm_core::rules::{EpochRule, WorldRules};
use mesocosm_core::{
    Appendage, ConditionId, Intent, OrganismId, Outcome, Recipe, Role, SpeciesId, Stage, Tagma,
    Unexpressed, Untakeable, World,
};

use super::bulk_world;
use super::discovery::{endure, hunger};
use super::gland::{frond_on, gland};

/// A recipe whose bodies grow one plate: the only shape that admits a gland.
fn plate_recipe() -> Recipe {
    Recipe::of(vec![Tagma::new(1, Appendage::Plate)])
}

/// Stands a world at its lineage checkpoint, on a budget a test can afford.
fn at_the_checkpoint(world: World) -> World {
    let mut world = world.with_rules(
        WorldRules::native()
            .ending(EpochRule::Timed { ticks: 1 })
            .scoring_over(4),
    );
    world.apply(Intent::Idle);
    assert!(world.at_boundary(), "a one-tick budget is spent every tick");
    world
}

/// A world whose played line has come through the starvation horizon, so it
/// holds the gland candidate — standing at its lineage checkpoint.
fn discovered_world(seed: u64) -> World {
    let mut world = bulk_world(seed, 24);
    frond_on(&mut world);
    endure(&mut world, HUNGER_TICKS + 1);
    assert!(world.discovered(hunger()), "the line came through");
    at_the_checkpoint(world)
}

fn played(world: &World) -> SpeciesId {
    world.controlled().expect("embodied").species
}

/// The row for one candidate.
fn row(world: &World, candidate: Option<ConditionId>) -> mesocosm_core::Offer {
    world
        .offers(played(world))
        .into_iter()
        .find(|offer| offer.candidate == candidate)
        .unwrap_or_else(|| panic!("no row for {candidate:?}"))
}

/// Puts a body past the ordinary breeding gate, which is itself untouched.
fn ready_to_breed(world: &mut World, id: OrganismId) {
    let organism = world
        .organisms
        .iter_mut()
        .find(|o| o.id == id)
        .expect("in the roster");
    organism.stage = Stage::Mature;
    organism.since_offspring = u32::MAX;
    assert!(organism.can_reproduce(), "the ecology's own gate");
}

#[test]
fn the_table_offers_the_status_quo_and_everything_the_line_came_to() {
    // The review's shape, in a world that has actually discovered something:
    // the status quo leads, the candidate follows, and both carry figures.
    let world = discovered_world(7_007);
    let offers = world.offers(played(&world));

    assert_eq!(
        offers.len(),
        2,
        "the status quo and one candidate: {offers:?}"
    );
    assert_eq!(offers[0].candidate, None);
    assert_eq!(offers[1].candidate, Some(hunger()));
    for offer in &offers {
        assert_eq!(
            offer.score.ticks, 4,
            "scored over the window the rules name"
        );
    }
    assert!(
        world.lineage_budget(played(&world)) > 0,
        "and a founder has something to develop with"
    );
}

#[test]
fn an_untakeable_candidate_stays_on_the_table_with_its_reason() {
    // **PE2's residue, answered.** A bulk line has nowhere to put a gland, so
    // the row is offered and says exactly that rather than disappearing —
    // which is the difference between "this world has nothing for me" and
    // "this body is the wrong shape for it, yet".
    let world = discovered_world(7_007);
    let offer = row(&world, Some(hunger()));

    assert_eq!(
        offer.why_not,
        Some(Untakeable::Unexpressed(Unexpressed::NoSite {
            role: Role::Plate
        })),
        "a founder of a bulk line grows no plate"
    );
    assert!(!offer.takeable());
    assert_eq!(
        offer.why_not.expect("a reason").words(),
        "nowhere on this body is a plate"
    );
}

#[test]
fn a_line_that_grows_the_shape_can_take_it() {
    // The other half of the same claim: give the line a recipe that grows a
    // plate and the identical candidate becomes takeable, with a price and a
    // preview beside it.
    let mut world = discovered_world(7_007);
    let species = played(&world);
    world.lineages_mut().set_recipe(species, plate_recipe());

    let offer = row(&world, Some(hunger()));
    assert_eq!(offer.why_not, None, "nothing stops it now");
    assert!(offer.takeable());
    assert!(offer.price_mg > 0, "and it costs something to grow");
    assert_ne!(offer.preview, row(&world, None).preview, "a different body");
    assert_ne!(
        offer.program,
        row(&world, None).program,
        "under a new program"
    );
}

/// Commits the gland on the played line and returns what the next descendant
/// of it was actually charged.
fn quoted_then_paid(mut world: World) -> (u64, u64) {
    let species = played(&world);
    world.lineages_mut().set_recipe(species, plate_recipe());
    let me = world.controlled_id().expect("embodied");
    let quoted = row(&world, Some(hunger())).price_mg;

    match world.apply(Intent::Revise {
        condition: hunger(),
    }) {
        Outcome::Revised { .. } => {}
        other => panic!("the commit was refused: {other:?}"),
    }
    // The parent the quote was taken against, past the breeding gate, and
    // nothing else about the enclosure touched.
    ready_to_breed(&mut world, me);

    for _ in 0..40 {
        world.apply(Intent::Idle);
        for recorded in world.drain_events() {
            if let Event::Inherited {
                organism, cost_mg, ..
            } = recorded.record
                && world
                    .living()
                    .any(|o| o.id == organism && o.species == species)
            {
                return (quoted, cost_mg);
            }
        }
    }
    panic!("no descendant expressed the program in forty ticks");
}

#[test]
fn the_price_is_the_filial_cost_the_birth_then_pays() {
    // **The load-bearing one.** The number on the table is not an estimate of a
    // development price: it is that price, read off the same
    // `program::express` the birth pass runs, against the same founder the
    // birth pass would provision. On ordinary ground the quote and the charge
    // are one number.
    //
    // The critter walks away from where it endured before the boundary, which
    // is what makes them one number: a body that stands still returns its
    // upkeep into the column under it, so a hundred ticks of enduring leaves
    // the ground under it richer than the ground its offspring will disperse
    // onto. See the companion test below — that gap is the ruled dormancy rule
    // doing its job, not a disagreement about the price.
    let mut world = bulk_world(4_242, 24);
    frond_on(&mut world);
    endure(&mut world, HUNGER_TICKS + 1);
    assert!(world.discovered(hunger()));
    for _ in 0..6 {
        world.apply(Intent::Move { delta: [3, 0, 0] });
    }

    let (quoted, paid) = quoted_then_paid(at_the_checkpoint(world));
    assert!(quoted > 0, "there is a price to check");
    assert_eq!(
        paid, quoted,
        "the milligrams the table quoted are the milligrams the birth charged"
    );
}

#[test]
fn richer_ground_under_the_parent_quotes_more_than_a_dispersed_birth_pays() {
    // The companion, and the honest half of the same claim. A founder preview
    // is realized under **declared** conditions — the ground your line is
    // standing on — and a descendant that disperses onto ordinary soil grows
    // only what that column can charge. The quote is still the full price of
    // the site; what changes is how much of the site the ground affords, which
    // is `Conditions::affords` and nothing else.
    let world = discovered_world(4_242);
    let (quoted, paid) = quoted_then_paid(world);
    assert!(
        paid < quoted,
        "a hundred ticks of standing still enriched the column: {paid} against {quoted}"
    );
    assert_eq!(
        quoted % paid,
        0,
        "and the difference is whole cells of one site, not a different price"
    );
}

#[test]
fn a_commit_is_admitted_at_the_boundary_and_refused_outside_it() {
    // The verb the board's key sends, and the one gate it passes. Bodies change
    // between epochs and not during them.
    let mut world = discovered_world(3_300);
    let species = played(&world);
    world.lineages_mut().set_recipe(species, plate_recipe());
    assert!(world.revision_admitted_now(), "standing at the checkpoint");

    match world.apply(Intent::Revise {
        condition: hunger(),
    }) {
        Outcome::Revised { .. } => {}
        other => panic!("the commit was refused: {other:?}"),
    }

    // One ordinary tick, on a budget that does not end an epoch, and the door
    // is shut again.
    let mut world = world.with_rules(WorldRules::native());
    world.apply(Intent::Idle);
    assert!(!world.revision_admitted_now());
    assert_eq!(
        world.apply(Intent::Revise {
            condition: hunger()
        }),
        Outcome::Rejected(mesocosm_core::Rejection::Unrevised(
            mesocosm_core::Unrevised::NotYet
        ))
    );
}

#[test]
fn after_a_commit_the_review_shows_the_revision_as_current() {
    // The board re-reads: the committed candidate is gone from the table
    // because the line now holds it, and the status quo it is compared against
    // is the new program.
    let mut world = discovered_world(7_007);
    let species = played(&world);
    world.lineages_mut().set_recipe(species, plate_recipe());

    let before = row(&world, None).program;
    world.apply(Intent::Revise {
        condition: hunger(),
    });

    let after = world.offers(species);
    assert_eq!(after.len(), 1, "nothing left to weigh: {after:?}");
    assert_eq!(after[0].candidate, None);
    assert_ne!(
        after[0].program, before,
        "and the status quo moved under it"
    );
    assert!(
        world
            .lineages()
            .get(species)
            .expect("the line")
            .program()
            .current()
            .is_some(),
        "the line is born under a revision now"
    );
}

#[test]
fn one_rival_lineage_responds_to_the_change() {
    // **PE3's "watches one rival lineage respond".** The player commits at a
    // boundary; at the next one an unplayed line takes its turn, and it scores
    // its candidates against a world the commit has already changed. The claim
    // is made by counterfactual: the same seed, the same ticks, one with the
    // commit and one without, and a rival's own figures differ.
    let round_after = |commit: bool| {
        let mut world = discovered_world(4_242);
        let species = played(&world);
        world.lineages_mut().set_recipe(species, plate_recipe());
        // One unplayed line grown up, so a scoring window sees births of it and
        // the candidate is worth something either way.
        let grown = world
            .initiative()
            .into_iter()
            .filter(|line| *line != species)
            .filter(|line| world.candidates(*line).len() > 1)
            .max_by_key(|line| world.living().filter(|o| o.species == *line).count())
            .expect("an unplayed line can carry a gland");
        for organism in world.organisms.iter_mut() {
            if organism.species == grown {
                organism.stage = Stage::Mature;
            }
        }
        if commit {
            world.apply(Intent::Revise {
                condition: hunger(),
            });
        }
        let me = world.controlled_id().expect("embodied");
        ready_to_breed(&mut world, me);
        // On to the next boundary, with the round it fires.
        for _ in 0..24 {
            world.apply(Intent::Idle);
        }
        (grown, world.last_round().clone())
    };

    let (rival, unchanged) = round_after(false);
    let (also_rival, changed) = round_after(true);
    assert_eq!(rival, also_rival, "the same rival in both runs");

    let turn_of = |round: &mesocosm_core::world::Round| {
        round
            .turn(rival)
            .unwrap_or_else(|| panic!("the rival took a turn: {round:?}"))
            .clone()
    };
    let before = turn_of(&unchanged);
    let after = turn_of(&changed);
    assert_ne!(
        before.considered, after.considered,
        "the rival weighed a different world because the player changed it"
    );
}

#[test]
fn the_gland_the_review_prices_is_the_one_a_descendant_expresses() {
    // The row is about a body, not a number: the founder the price was quoted
    // for is the founder that grows the organ.
    let mut world = discovered_world(555);
    let species = played(&world);
    world.lineages_mut().set_recipe(species, plate_recipe());
    let me = world.controlled_id().expect("embodied");
    assert!(row(&world, Some(hunger())).takeable());

    world.apply(Intent::Revise {
        condition: hunger(),
    });
    ready_to_breed(&mut world, me);

    let mut child = None;
    for _ in 0..40 {
        world.apply(Intent::Idle);
        for recorded in world.drain_events() {
            if let Event::Inherited { organism, .. } = recorded.record {
                child = Some(organism);
            }
        }
        if child.is_some() {
            break;
        }
    }
    let child = child.expect("a descendant expressed the program");
    assert!(
        world
            .living()
            .find(|o| o.id == child)
            .expect("alive")
            .phenotype
            .expresses(gland()),
        "and it is born with the organ the table previewed"
    );
}
