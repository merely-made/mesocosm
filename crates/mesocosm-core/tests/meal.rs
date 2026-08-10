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

use mesocosm_core::{Intent, Outcome, Placement, Rejection, Route, World, snapshot, state_hash};

/// Sets the played critter's budget. Energy lives on the organism now, so a
/// test that wants a starving critter has to say which one.
fn set_energy(world: &mut World, energy_mg: u64) {
    let id = world.controlled_id().unwrap();
    world
        .organisms
        .iter_mut()
        .find(|o| o.id == id)
        .unwrap()
        .energy_mg = energy_mg;
}

/// A world with something in reach, and the id of what to eat. The meal tests
/// exercise the two metabolize receipts, so the fixture places a live target
/// directly in reach instead of spending hundreds of ecology ticks chasing a
/// moving organism.
fn fed() -> (World, mesocosm_core::OrganismId) {
    let mut world = World::new(4_242, 24);
    let here = world.position().unwrap();
    let prey = world
        .organisms
        .iter()
        .filter(|o| o.biomass_mg() > 0 && Some(o.id) != world.controlled_id())
        .min_by_key(|o| {
            (0..3)
                .map(|a| (o.position[a] - here[a]).abs())
                .max()
                .unwrap_or(0)
        })
        .map(|o| o.id)
        .expect("the fixture scatters organisms");
    world
        .organisms
        .iter_mut()
        .find(|o| o.id == prey)
        .expect("the selected organism remains in the roster")
        .position = here;
    (world, prey)
}

#[test]
fn burning_gives_energy_and_no_body() {
    let (mut world, prey) = fed();
    let parts = world.body().unwrap().len();
    let energy = world.energy_mg().unwrap();

    let outcome = world.apply(Intent::Metabolize {
        organism: prey,
        route: Route::Burn,
    });

    assert!(matches!(outcome, Outcome::Burned { .. }), "got {outcome:?}");
    assert_eq!(world.body().unwrap().len(), parts, "burning grows nothing");
    assert!(world.energy_mg().unwrap() > energy, "and it pays now");
}

#[test]
fn incorporating_gives_body_and_no_energy() {
    // The half of the tradeoff that used to be free. Growth is the slow
    // answer, and a meal cannot be both meals.
    let (mut world, prey) = fed();
    let parts = world.body().unwrap().len();
    let energy = world.energy_mg().unwrap();

    let outcome = world.apply(Intent::Metabolize {
        organism: prey,
        route: Route::Incorporate {
            placement: Placement::Planned,
        },
    });

    assert!(
        matches!(
            outcome,
            Outcome::Incorporated { .. } | Outcome::IncorporatedPair { .. }
        ),
        "got {outcome:?}"
    );
    assert!(world.body().unwrap().len() > parts, "the body grew");
    assert!(
        world.energy_mg().unwrap() <= energy,
        "and it paid nothing immediately"
    );
}

#[test]
fn the_same_meal_cannot_be_spent_twice() {
    // Mutually exclusive receipts, which is the done-condition's exact words.
    // Whichever route is taken, the organism is consumed and the other route
    // is no longer available.
    for route in [
        Route::Burn,
        Route::Incorporate {
            placement: Placement::Planned,
        },
    ] {
        let (mut world, prey) = fed();
        let before = world.organisms.len();

        world.apply(Intent::Metabolize {
            organism: prey,
            route,
        });
        assert_eq!(world.organisms.len(), before - 1, "the meal is gone");

        let again = world.apply(Intent::Metabolize {
            organism: prey,
            route: Route::Burn,
        });
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

    burned.apply(Intent::Metabolize {
        organism: prey,
        route: Route::Burn,
    });
    grown.apply(Intent::Metabolize {
        organism: prey,
        route: Route::Incorporate {
            placement: Placement::Planned,
        },
    });

    assert_ne!(
        state_hash(&burned),
        state_hash(&grown),
        "the routes are not the same act"
    );
    assert!(
        burned.energy_mg().unwrap() > grown.energy_mg().unwrap(),
        "one lives now"
    );
    assert!(
        grown.body().unwrap().len() > burned.body().unwrap().len(),
        "the other grows later"
    );
}

#[test]
fn venom_is_charged_whatever_the_meal_becomes() {
    // The defect this replaces: the explicit editor path subtracted venom and
    // the automatic path did not, so the safe-looking verb was the dangerous
    // one. A warning signal is only worth reading if believing it changes what
    // happens, on every route.
    let (mut world, prey) = fed();
    let venom = 40;
    {
        let o = world.organisms.iter_mut().find(|o| o.id == prey).unwrap();
        o.venom_mg = venom;
    }

    let mass = world
        .organisms
        .iter()
        .find(|o| o.id == prey)
        .map(|o| o.biomass_mg())
        .unwrap();
    let before = world.energy_mg().unwrap();
    // Upkeep comes out of the same budget in the same tick, and it scales with
    // the body, so the grown critter pays more of it than the burnt one.
    let upkeep = world.controlled().unwrap().upkeep_mg();
    let mut grown = world.clone();

    world.apply(Intent::Metabolize {
        organism: prey,
        route: Route::Burn,
    });
    // Gains before costs. The earlier version subtracted venom first, which
    // this assertion enshrined; with low energy the floor erased part of the
    // toxin before the meal paid out.
    assert_eq!(
        world.energy_mg().unwrap(),
        (before + mass).saturating_sub(venom).saturating_sub(upkeep)
    );

    grown.apply(Intent::Metabolize {
        organism: prey,
        route: Route::Incorporate {
            placement: Placement::Planned,
        },
    });
    let grown_upkeep = grown.controlled().unwrap().upkeep_mg();
    assert!(grown_upkeep >= upkeep, "growing raised the rent");
    assert!(
        world.energy_mg().unwrap() > grown.energy_mg().unwrap(),
        "burning banked more"
    );
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
        (o.biomass_mg(), o.venom_mg)
    };
    set_energy(&mut world, 10); // nearly starved
    assert!(
        venom > world.energy_mg().unwrap() + mass,
        "the toxin outweighs the meal"
    );

    world.apply(Intent::Metabolize {
        organism: prey,
        route: Route::Burn,
    });

    assert_eq!(
        world.energy_mg().unwrap(),
        0,
        "a meal that cannot cover its venom leaves nothing"
    );
}

#[test]
fn a_full_critter_pays_the_same_toxin_as_a_starving_one() {
    // The same claim from the other side: the cost of a venomous meal is a
    // property of the meal, not of how desperate you were when you ate it.
    let (base, prey) = fed();

    let mut rich = base.clone();
    let mut poor = base;
    for world in [&mut rich, &mut poor] {
        world
            .organisms
            .iter_mut()
            .find(|o| o.id == prey)
            .unwrap()
            .venom_mg = 60;
    }
    set_energy(&mut rich, 5_000);
    set_energy(&mut poor, 1_000);

    let (rich_before, poor_before) = (rich.energy_mg().unwrap(), poor.energy_mg().unwrap());
    rich.apply(Intent::Metabolize {
        organism: prey,
        route: Route::Burn,
    });
    poor.apply(Intent::Metabolize {
        organism: prey,
        route: Route::Burn,
    });

    assert_eq!(
        rich.energy_mg().unwrap() - rich_before,
        poor.energy_mg().unwrap() - poor_before,
        "the same meal nets the same amount at any starting energy"
    );
}

#[test]
fn a_refused_placement_costs_neither_the_meal_nor_its_venom() {
    // One transaction. An earlier cut removed the organism and charged venom
    // before the attachment was known to succeed, so a refusal ate the meal and
    // poisoned you for it.
    let (mut world, prey) = fed();
    world
        .organisms
        .iter_mut()
        .find(|o| o.id == prey)
        .unwrap()
        .venom_mg = 90;

    let roster = world.organisms.len();
    let energy = world.energy_mg().unwrap();
    let upkeep = world.controlled().unwrap().upkeep_mg();
    let parts = world.body().unwrap().len();

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

    assert_eq!(
        outcome,
        Outcome::Rejected(Rejection::NoSuchParent(mesocosm_core::PartId(9_999)))
    );
    assert_eq!(
        world.energy_mg().unwrap(),
        energy - upkeep,
        "rent only: no venom was charged"
    );
    assert_eq!(world.body().unwrap().len(), parts, "nothing was grown");
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
    let parts = world.body().unwrap().len();
    let energy = world.energy_mg().unwrap();

    let upkeep = world.controlled().unwrap().upkeep_mg();
    let outcome = world.apply(Intent::Metabolize {
        organism: absent,
        route: Route::Incorporate {
            placement: Placement::Planned,
        },
    });

    assert_eq!(
        outcome,
        Outcome::Rejected(Rejection::NoSuchOrganism(absent))
    );
    assert_eq!(world.body().unwrap().len(), parts, "nothing was grown");
    // Rent still came due, but the refusal itself cost nothing.
    assert_eq!(
        world.energy_mg().unwrap(),
        energy - upkeep,
        "no venom, no meal, just rent"
    );
    // The ecology still stepped, so the roster may change by birth or death,
    // but no organism was eaten by this refusal.
    assert!(world.organisms.len() >= roster.saturating_sub(1));
}

#[test]
fn both_routes_replay_identically() {
    // The determinism boundary the wing rests on. Routing is part of the
    // recorded intent, so a trace that chooses differently replays differently
    // and a trace that chooses the same replays the same.
    for route in [
        Route::Burn,
        Route::Incorporate {
            placement: Placement::Planned,
        },
    ] {
        let (world, prey) = fed();

        let mut straight = world.clone();
        straight.apply(Intent::Metabolize {
            organism: prey,
            route,
        });
        straight.apply(Intent::Move { delta: [1, 0, 0] });

        let mut forked = world.clone();
        forked.apply(Intent::Metabolize {
            organism: prey,
            route,
        });
        let bytes = snapshot(&forked).unwrap();
        let mut resumed = mesocosm_core::restore(&bytes).unwrap();
        resumed.apply(Intent::Move { delta: [1, 0, 0] });

        assert_eq!(
            state_hash(&straight),
            state_hash(&resumed),
            "route {route:?} replays"
        );
    }
}
