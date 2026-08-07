use super::*;
use crate::body::{Origin, VolumeRef};
use crate::organism::{Kingdom, Stage};

/// Walks the critter to its nearest neighbour and returns it.
fn near_organism(world: &mut World) -> OrganismId {
    for _ in 0..400 {
        let here = world.position().expect("embodied");
        let Some((id, at)) = world
            .organisms
            .iter()
            .filter(|m| Some(m.id) != world.controlled_id() && m.is_alive())
            .map(|m| (m.id, m.position))
            .min_by_key(|(_, at): &(_, [i32; 3])| {
                (0..3).map(|a| (at[a] - here[a]).abs()).max().unwrap_or(0)
            })
        else {
            break;
        };
        if world.in_reach(at) {
            return id;
        }
        let step = [0, 1, 2].map(|a| (at[a] - here[a]).signum());
        world.apply(Intent::Move { delta: step });
    }
    panic!("nothing came within reach")
}

#[test]
fn same_seed_builds_the_same_world() {
    assert_eq!(World::new(1234, 12), World::new(1234, 12));
}

#[test]
fn different_seeds_build_different_worlds() {
    assert_ne!(World::new(1, 12).organisms, World::new(2, 12).organisms);
}

#[test]
fn metabolize_grows_mass_and_collision() {
    let mut world = World::new(99, 24);
    let target = near_organism(&mut world);
    let mass_before = world.total_mass_mg();
    let box_before = world.collision().unwrap();

    let outcome = world.apply(Intent::Metabolize {
        organism: target,
        route: Route::Incorporate {
            placement: Placement::Explicit {
                parent: world.body().unwrap().root,
                offset: [5, 0, 0],
                yaw: Yaw::Zero,
            },
        },
    });

    assert!(matches!(outcome, Outcome::Incorporated { .. }));
    assert!(world.total_mass_mg() > mass_before);
    assert!(world.collision().unwrap().extent()[0] > box_before.extent()[0]);
}

#[test]
fn metabolize_records_where_the_part_came_from() {
    let mut world = World::new(7, 24);
    let target = near_organism(&mut world);
    let eaten_species = world
        .organisms
        .iter()
        .find(|m| m.id == target)
        .map(|m| m.species)
        .unwrap();

    let Outcome::Incorporated { part } = world.apply(Intent::Metabolize {
        organism: target,
        route: Route::Incorporate {
            placement: Placement::Explicit {
                parent: world.body().unwrap().root,
                offset: [4, 0, 0],
                yaw: Yaw::Zero,
            },
        },
    }) else {
        panic!("expected incorporation");
    };

    assert_eq!(
        world.body().unwrap().part(part).unwrap().provenance.origin,
        Origin::Incorporated {
            from_species: eaten_species,
            from_part: PartId(0)
        }
    );
}

#[test]
fn out_of_reach_organisms_are_refused() {
    let mut world = World::new(5, 4);
    world.organisms.push(Organism {
        stage: Stage::Mature,
        ..Organism::founding(
            OrganismId(900),
            SpeciesId(3),
            Kingdom::Producer,
            VolumeRef::from_tag(2),
            [1, 1, 1],
            [500, 0, 0],
            100,
        )
    });
    let outcome = world.apply(Intent::Metabolize {
        organism: OrganismId(900),
        route: Route::Incorporate {
            placement: Placement::Explicit {
                parent: world.body().unwrap().root,
                offset: [1, 0, 0],
                yaw: Yaw::Zero,
            },
        },
    });
    assert!(
        matches!(
            outcome,
            Outcome::Rejected(Rejection::OutOfReach(crate::process::Unmet::TooFar {
                distance: 500,
                ..
            }))
        ),
        "got {outcome:?}"
    );
}

#[test]
fn rejected_intents_still_advance_the_tick() {
    let mut world = World::new(11, 2);
    let before = world.tick;
    let outcome = world.apply(Intent::Metabolize {
        organism: OrganismId(4242),
        route: Route::Incorporate {
            placement: Placement::Explicit {
                parent: world.body().unwrap().root,
                offset: [0, 0, 0],
                yaw: Yaw::Zero,
            },
        },
    });
    assert_eq!(
        outcome,
        Outcome::Rejected(Rejection::NoSuchOrganism(OrganismId(4242)))
    );
    assert_eq!(world.tick, before + 1);
}

#[test]
fn movement_spends_the_budget() {
    let mut world = World::new(3, 2);
    let before = world.energy_mg().unwrap();
    let upkeep = world.controlled().unwrap().upkeep_mg();
    world.apply(Intent::Move { delta: [3, 0, -2] });
    assert_eq!(world.energy_mg().unwrap(), before - 5 - upkeep);
}

#[test]
fn a_bigger_body_costs_more_to_carry() {
    let world = World::new(3, 2);
    let small = world.controlled().unwrap().upkeep_mg();
    let mut grown = world.clone();
    let me = grown.controlled_id().unwrap();
    grown
        .organisms
        .iter_mut()
        .find(|o| o.id == me)
        .unwrap()
        .gain_mass(5_000);

    assert!(
        grown.controlled().unwrap().upkeep_mg() > small,
        "a heavier body pays more rent: {} vs {}",
        grown.controlled().unwrap().upkeep_mg(),
        small
    );
}

#[test]
fn deposit_returns_matter_to_the_enclosure() {
    let mut world = World::new(3, 2);
    let count = world.organisms.len();
    let outcome = world.apply(Intent::Deposit { mass_mg: 200 });
    assert!(matches!(outcome, Outcome::Deposited { .. }));
    assert_eq!(world.organisms.len(), count + 1);
}
