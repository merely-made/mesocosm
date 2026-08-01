// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! P0: where a meal goes.
//!
//! Before this, eating granted a part **and** half the mass as energy, so the
//! game's central verb asked the player nothing. These tests pin the tradeoff
//! that replaces it: the same meal, two destinations, and no way to have both.
//!
//! The done-condition from the phenotype plan is three things — mutually
//! exclusive receipts, consistent venom, and identical replay — and the fourth
//! is Mark's judgment about whether the choice is tense or clerical, which no
//! test can supply.

use mesocosm_core::{Intent, Outcome, Rejection, Placement, Route, World, snapshot, state_hash};

/// A world with something in reach, and the id of what to eat.
fn fed() -> (World, mesocosm_core::OrganismId) {
    let mut world = World::new(4_242, 24);

    // Walk to the nearest organism so the meal is legal.
    for _ in 0..400 {
        let Some((prey, at)) = world
            .organisms
            .iter()
            .filter(|o| o.mass_mg > 0)
            .map(|o| (o.id, o.position))
            .min_by_key(|(_, at): &(_, [i32; 3])| {
                (0..3).map(|a| (at[a] - world.position[a]).abs()).max().unwrap_or(0)
            })
        else {
            break;
        };
        let step = [0, 1, 2].map(|a| (at[a] - world.position[a]).signum());
        if step == [0, 0, 0] {
            return (world, prey);
        }
        world.apply(Intent::Move { delta: step });
    }
    panic!("nothing came within reach");
}

#[test]
fn burning_gives_energy_and_no_body() {
    let (mut world, prey) = fed();
    let parts = world.body.len();
    let energy = world.energy_mg;

    let outcome = world.apply(Intent::Metabolize { organism: prey, route: Route::Burn });

    assert!(matches!(outcome, Outcome::Burned { .. }), "got {outcome:?}");
    assert_eq!(world.body.len(), parts, "burning grows nothing");
    assert!(world.energy_mg > energy, "and it pays now");
}

#[test]
fn incorporating_gives_body_and_no_energy() {
    // The half of the tradeoff that used to be free. Growth is the slow
    // answer, and a meal cannot be both meals.
    let (mut world, prey) = fed();
    let parts = world.body.len();
    let energy = world.energy_mg;

    let outcome = world.apply(Intent::Metabolize { organism: prey, route: Route::Incorporate { placement: Placement::Planned } });

    assert!(
        matches!(outcome, Outcome::Incorporated { .. } | Outcome::IncorporatedPair { .. }),
        "got {outcome:?}"
    );
    assert!(world.body.len() > parts, "the body grew");
    assert!(world.energy_mg <= energy, "and it paid nothing immediately");
}

#[test]
fn the_same_meal_cannot_be_spent_twice() {
    // Mutually exclusive receipts, which is the done-condition's exact words.
    // Whichever route is taken, the organism is consumed and the other route
    // is no longer available.
    for route in [Route::Burn, Route::Incorporate { placement: Placement::Planned }] {
        let (mut world, prey) = fed();
        let before = world.organisms.len();

        world.apply(Intent::Metabolize { organism: prey, route });
        assert_eq!(world.organisms.len(), before - 1, "the meal is gone");

        let again = world.apply(Intent::Metabolize { organism: prey, route: Route::Burn });
        assert_eq!(again, Outcome::Rejected(Rejection::NoSuchOrganism(prey)));
    }
}

#[test]
fn burning_and_growing_diverge_from_one_world() {
    // The choice has to *matter*, not merely exist. Two worlds identical up to
    // the meal end up in different states, and the difference is exactly the
    // one the design claims: mass or budget.
    let (burned_world, prey) = fed();
    let mut burned = burned_world;
    let mut grown = burned.clone();

    burned.apply(Intent::Metabolize { organism: prey, route: Route::Burn });
    grown.apply(Intent::Metabolize { organism: prey, route: Route::Incorporate { placement: Placement::Planned } });

    assert_ne!(state_hash(&burned), state_hash(&grown), "the routes are not the same act");
    assert!(burned.energy_mg > grown.energy_mg, "one lives now");
    assert!(grown.body.len() > burned.body.len(), "the other grows later");
}

#[test]
fn venom_is_charged_whatever_the_meal_becomes() {
    // The defect this replaces: the explicit editor path subtracted venom and
    // the automatic path did not, so the safe-looking verb was the dangerous
    // one. A warning signal is only worth reading if believing it changes what
    // happens, on every route.
    let mut world = World::new(4_242, 24);

    // Give everything in reach a real venom load, then walk to one.
    for organism in world.organisms.iter_mut() {
        organism.venom_mg = 40;
    }
    let (mut world, prey) = {
        for _ in 0..400 {
            let Some((_id, at)) = world
                .organisms
                .iter()
                .map(|o| (o.id, o.position))
                .min_by_key(|(_, at): &(_, [i32; 3])| {
                    (0..3).map(|a| (at[a] - world.position[a]).abs()).max().unwrap_or(0)
                })
            else {
                break;
            };
            let step = [0, 1, 2].map(|a| (at[a] - world.position[a]).signum());
            if step == [0, 0, 0] {
                break;
            }
            world.apply(Intent::Move { delta: step });
        }
        let prey = world
            .organisms
            .iter()
            .find(|o| {
                (0..3).all(|a| (o.position[a] - world.position[a]).abs() <= 8)
            })
            .map(|o| o.id)
            .expect("something is in reach");
        (world, prey)
    };

    let venom = world.organisms.iter().find(|o| o.id == prey).map(|o| o.venom_mg).unwrap();
    assert!(venom > 0, "the fixture is actually venomous");

    let mut grown = world.clone();
    let before = world.energy_mg;

    let burned_energy = {
        let mass = world.organisms.iter().find(|o| o.id == prey).map(|o| o.mass_mg).unwrap();
        world.apply(Intent::Metabolize { organism: prey, route: Route::Burn });
        // **Gains before costs.** The earlier version subtracted venom first,
        // which this assertion enshrined; with low energy the floor erased part
        // of the toxin before the meal paid out.
        assert_eq!(world.energy_mg, (before + mass).saturating_sub(venom));
        world.energy_mg
    };

    grown.apply(Intent::Metabolize {
        organism: prey,
        route: Route::Incorporate { placement: Placement::Planned },
    });
    assert_eq!(grown.energy_mg, before.saturating_sub(venom), "growing pays it too");
    assert!(burned_energy > grown.energy_mg);
}

#[test]
fn being_nearly_starved_does_not_make_venom_safer() {
    // The hazard the previous ordering created. Subtracting venom before adding
    // the meal meant a critter at low energy lost part of the toxin to the zero
    // floor and then collected the full mass, so approaching death was a
    // discount on poison. Gains land first, so the toxin is paid in full.
    let (mut world, prey) = fed();

    let (mass, venom) = {
        let o = world.organisms.iter_mut().find(|o| o.id == prey).unwrap();
        o.venom_mg = 500;
        (o.mass_mg, o.venom_mg)
    };
    world.energy_mg = 10; // nearly starved
    assert!(venom > world.energy_mg + mass, "the toxin outweighs the meal");

    world.apply(Intent::Metabolize { organism: prey, route: Route::Burn });

    assert_eq!(world.energy_mg, 0, "a meal that cannot cover its venom leaves nothing");
}

#[test]
fn a_full_critter_pays_the_same_toxin_as_a_starving_one() {
    // The same claim from the other side: the cost of a venomous meal is a
    // property of the meal, not of how desperate you were when you ate it.
    let (base, prey) = fed();

    let mut rich = base.clone();
    let mut poor = base;
    for world in [&mut rich, &mut poor] {
        world.organisms.iter_mut().find(|o| o.id == prey).unwrap().venom_mg = 60;
    }
    rich.energy_mg = 5_000;
    poor.energy_mg = 1_000;

    let (rich_before, poor_before) = (rich.energy_mg, poor.energy_mg);
    rich.apply(Intent::Metabolize { organism: prey, route: Route::Burn });
    poor.apply(Intent::Metabolize { organism: prey, route: Route::Burn });

    assert_eq!(
        rich.energy_mg - rich_before,
        poor.energy_mg - poor_before,
        "the same meal nets the same amount at any starting energy"
    );
}

#[test]
fn a_refused_placement_costs_neither_the_meal_nor_its_venom() {
    // One transaction. An earlier cut removed the organism and charged venom
    // before the attachment was known to succeed, so a refusal ate the meal and
    // poisoned you for it.
    let (mut world, prey) = fed();
    world.organisms.iter_mut().find(|o| o.id == prey).unwrap().venom_mg = 90;

    let roster = world.organisms.len();
    let energy = world.energy_mg;
    let parts = world.body.len();

    let outcome = world.apply(Intent::Metabolize {
        organism: prey,
        route: Route::Incorporate {
            placement: Placement::Explicit {
                parent: mesocosm_core::PartId(9_999),
                offset: [2, 0, 0],
                yaw: mesocosm_core::Yaw::Zero,
            },
        },
    });

    assert_eq!(outcome, Outcome::Rejected(Rejection::NoSuchParent(mesocosm_core::PartId(9_999))));
    assert_eq!(world.energy_mg, energy, "no venom was charged");
    assert_eq!(world.body.len(), parts, "nothing was grown");
    assert!(
        world.organisms.iter().any(|o| o.id == prey),
        "and the meal is still there to eat"
    );
    assert!(world.organisms.len() >= roster.saturating_sub(1));
}

#[test]
fn a_refused_meal_leaves_the_world_untouched() {
    // Placement is resolved before the organism is consumed, so a body with
    // nowhere to put a part does not lose the meal as well.
    let (mut world, _) = fed();
    let absent = mesocosm_core::OrganismId(9_999);
    let roster = world.organisms.len();
    let parts = world.body.len();
    let energy = world.energy_mg;

    let outcome = world.apply(Intent::Metabolize { organism: absent, route: Route::Incorporate { placement: Placement::Planned } });

    assert_eq!(outcome, Outcome::Rejected(Rejection::NoSuchOrganism(absent)));
    assert_eq!(world.body.len(), parts, "nothing was grown");
    assert_eq!(world.energy_mg, energy, "and nothing was charged");
    // The ecology still stepped, so the roster may change by birth or death,
    // but no organism was eaten by this refusal.
    assert!(world.organisms.len() >= roster.saturating_sub(1));
}

#[test]
fn both_routes_replay_identically() {
    // The determinism boundary the wing rests on. Routing is part of the
    // recorded intent, so a trace that chooses differently replays differently
    // and a trace that chooses the same replays the same.
    for route in [Route::Burn, Route::Incorporate { placement: Placement::Planned }] {
        let (world, prey) = fed();

        let mut straight = world.clone();
        straight.apply(Intent::Metabolize { organism: prey, route });
        straight.apply(Intent::Move { delta: [1, 0, 0] });

        let mut forked = world.clone();
        forked.apply(Intent::Metabolize { organism: prey, route });
        let bytes = snapshot(&forked).unwrap();
        let mut resumed = mesocosm_core::restore(&bytes).unwrap();
        resumed.apply(Intent::Move { delta: [1, 0, 0] });

        assert_eq!(state_hash(&straight), state_hash(&resumed), "route {route:?} replays");
    }
}
