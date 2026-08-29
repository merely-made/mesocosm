use super::*;
use crate::body::{Attachment, Origin, Provenance, VolumeRef};
use crate::history::History;
use crate::organism::{Kingdom, LastSeen, Stage};
use crate::places::{
    Places, Tier, WALKER_HEIGHT, WalkerShape, route_step, spot, spot_for, step, surface_stance_for,
};

/// Walks the critter to its nearest neighbour and returns it.
/// Lets the hand go, so the next tick's ecology drives the controlled body
/// like any other. TD4 spares a *held* critter its instincts, and the tests
/// below are about an uncommanded one; this is how they say so without
/// spending thirty ticks idling to get there.
fn let_go(world: &mut World) {
    world.idle_run = INSTINCT_IDLE_TICKS;
}

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

fn assert_grounded_near(world: &World) {
    for organism in world
        .organisms
        .iter()
        .filter(|organism| organism.is_alive() && organism.tier == Tier::Near)
    {
        assert!(
            organism
                .walker_shape()
                .stands(world.ground(), organism.position),
            "near organism {:?} left footing at {:?}",
            organism.id,
            organism.position
        );
    }
}

/// The generated entrance route for the first nest, shared with generation
/// rather than reconstructed as a second terrain fixture.
fn generated_nest_entry(seed: u64) -> Vec<[i32; 3]> {
    let grown = Places::grown(seed ^ PLACE_SALT, PLACE_SIDE, ENCLOSURE);
    let nest = grown.nests.first().expect("seed grows a nest");
    crate::places::nest_entry(&grown, ENCLOSURE, *nest)
        .expect("seed's nest has a generated entry")
        .route
}

fn generated_sight_split(
    ground: &crate::places::Ground,
    compact: WalkerShape,
    tall: WalkerShape,
    target_shape: WalkerShape,
) -> ([i32; 3], [i32; 3]) {
    const DIRECTIONS: [[i32; 2]; 8] = [
        [1, 0],
        [-1, 0],
        [0, 1],
        [0, -1],
        [1, 1],
        [1, -1],
        [-1, 1],
        [-1, -1],
    ];
    const RANGE: i32 = 8;
    for z in -40..40 {
        for x in -40..40 {
            let Some(observer) = surface_stance_for(ground, compact, [x, 0, z]) else {
                continue;
            };
            if !tall.stands(ground, observer) {
                continue;
            }
            for distance in 3..=RANGE {
                for [dx, dz] in DIRECTIONS {
                    let Some(target) = surface_stance_for(
                        ground,
                        target_shape,
                        [x + dx * distance, 0, z + dz * distance],
                    ) else {
                        continue;
                    };
                    if !spot_for(ground, compact, observer, target_shape, target, RANGE)
                        && spot_for(ground, tall, observer, target_shape, target, RANGE)
                    {
                        return (observer, target);
                    }
                }
            }
        }
    }
    panic!("generated Ground offered no compact/tall terrain sight split");
}

#[test]
fn hunter_and_player_cross_one_generated_entry_and_place_boundary() {
    // Re-pinned 0 -> 129 by S1, and the reason is the finding: `PLACE_SIDE`
    // stayed 3 while the enclosure went 16 -> 64, so a region is now 43 voxels
    // across and a generated nest entry is 5 to 8 voxels long. An entry that
    // crosses a place boundary went from ordinary to rare — 30 of the first
    // thousand seeds still have one, and this is the first whose crossing sits
    // at the same step the pinned run always asserted **and** joins two places
    // the grown graph actually links — an unlinked pair is two hops apart, which
    // is `demote_hops`, so the hunter would answer by teleporting a region
    // rather than following a stance.
    const SEED: u64 = 172;
    let grown = Places::grown(SEED ^ PLACE_SALT, PLACE_SIDE, ENCLOSURE);
    let (route, boundary_step, from_place, to_place) = grown
        .nest_entries(ENCLOSURE)
        .find_map(|(_, entry)| {
            let route = entry.route;
            let boundary_step = route
                .windows(2)
                .position(|step| grown.places.at(step[0]) != grown.places.at(step[1]))?;
            Some((
                route.clone(),
                boundary_step,
                grown.places.at(route[boundary_step])?,
                grown.places.at(route[boundary_step + 1])?,
            ))
        })
        .expect("seed has a generated entry crossing a place boundary");
    assert_ne!(from_place, to_place);
    assert_eq!(boundary_step, 2, "the pinned run moved its place edge");
    assert!(route.len() > boundary_step + 2);

    let mut world = World::new(SEED, 0);
    world.organisms = vec![
        Organism::founding(
            OrganismId(0),
            SpeciesId(3),
            Kingdom::Producer,
            VolumeRef::from_tag(18),
            [1, 1, 1],
            route[1],
            300,
        ),
        Organism::founding(
            OrganismId(900),
            SpeciesId(2),
            Kingdom::Consumer,
            VolumeRef::from_tag(16),
            [3, 1, 1],
            route[0],
            300,
        ),
    ];
    let mut twin = world.clone();
    let mut history = History::new();
    let mut twin_history = History::new();
    let mut player_positions = vec![route[1]];
    let mut hunter_positions = vec![route[0]];
    let trace = route
        .windows(2)
        .skip(1)
        .map(|step| Intent::Move {
            delta: [0, 1, 2].map(|axis| step[1][axis] - step[0][axis]),
        })
        .collect::<Vec<_>>();

    for (index, intent) in trace.iter().enumerate() {
        let target = route[index + 2];
        let player = world.position().unwrap();
        assert_eq!(step(world.ground(), player, target), target);
        let outcome = world.apply(intent.clone());
        let twin_outcome = twin.apply(intent.clone());
        history.record_all(world.drain_events());
        twin_history.record_all(twin.drain_events());
        assert_eq!(outcome, twin_outcome, "replay changed an outcome");
        assert!(matches!(outcome, Outcome::Moved));
        assert_eq!(
            world.position(),
            Some(target),
            "player stuttered on the entry"
        );
        let hunter = world
            .organisms
            .iter()
            .find(|organism| organism.id == OrganismId(900))
            .unwrap();
        assert_eq!(
            hunter.position,
            route[index + 1],
            "hunter did not follow one generated stance behind"
        );
        assert_eq!(
            hunter
                .last_fauna_decision
                .as_ref()
                .map(|decision| decision.selected_drive),
            Some(crate::organism::FaunaDrive::Pursue)
        );
        assert!(world.ground().stands(hunter.position, WALKER_HEIGHT));
        player_positions.push(target);
        hunter_positions.push(hunter.position);
    }

    assert_eq!(grown.places.at(player_positions[0]), Some(from_place));
    assert_eq!(
        grown.places.at(*player_positions.last().unwrap()),
        Some(to_place)
    );
    assert_eq!(grown.places.at(hunter_positions[0]), Some(from_place));
    assert_eq!(
        grown.places.at(*hunter_positions.last().unwrap()),
        Some(to_place)
    );
    assert_eq!(hunter_positions[1], route[1], "hunter missed the threshold");
    assert_eq!(
        player_positions.iter().collect::<BTreeSet<_>>().len(),
        player_positions.len(),
        "player stuttered"
    );
    assert_eq!(
        hunter_positions.iter().collect::<BTreeSet<_>>().len(),
        hunter_positions.len(),
        "hunter stuttered"
    );
    assert_eq!(
        crate::snapshot::state_hash(&world),
        crate::snapshot::state_hash(&twin)
    );
    assert_eq!(history, twin_history);
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
    let eaten = world
        .organisms
        .iter()
        .find(|organism| organism.id == target)
        .unwrap()
        .clone();
    let mut preview = world.body().unwrap().clone();
    preview
        .attach(
            eaten.volume(),
            eaten.biomass_mg(),
            eaten.half_extent(),
            Attachment {
                parent: preview.root,
                offset: [5, 0, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .unwrap();
    let stance = crate::places::surface_stance_for(
        world.ground(),
        crate::places::WalkerShape::from_aabb(preview.aabb()),
        world.position().unwrap(),
    )
    .expect("the generated surface has room for the previewed body");
    world.controlled_mut().unwrap().position = stance;
    world
        .organisms
        .iter_mut()
        .find(|organism| organism.id == target)
        .unwrap()
        .position = stance;
    let mass_before = world.total_mass_mg();
    let box_before = world.collision().unwrap();

    let outcome = world.apply(Intent::Metabolize {
        organism: target,
        placement: Placement::Explicit {
            parent: world.body().unwrap().root,
            offset: [5, 0, 0],
            yaw: Yaw::Zero,
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
        placement: Placement::Explicit {
            parent: world.body().unwrap().root,
            offset: [4, 0, 0],
            yaw: Yaw::Zero,
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
        placement: Placement::Explicit {
            parent: world.body().unwrap().root,
            offset: [1, 0, 0],
            yaw: Yaw::Zero,
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
        placement: Placement::Explicit {
            parent: world.body().unwrap().root,
            offset: [0, 0, 0],
            yaw: Yaw::Zero,
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
    let mut world = World::new(3, 0);
    let from = world.position().unwrap();
    let before = world.energy_mg().unwrap();
    let upkeep = world.controlled().unwrap().upkeep_mg();
    world.apply(Intent::Move { delta: [3, 0, -2] });
    let after = world.position().unwrap();
    // A Move is now one legal kinematic step, even if its caller supplied a
    // larger offset. The ecology's later upkeep is still charged for the
    // whole tick.
    assert!(
        (after[0] - from[0]).abs() <= 1 && (after[2] - from[2]).abs() <= 1,
        "movement teleported to {after:?}"
    );
    let distance = u64::from((after[0] - from[0]).unsigned_abs())
        + u64::from((after[2] - from[2]).unsigned_abs());
    assert_eq!(world.energy_mg().unwrap(), before - distance - upkeep);
}

#[test]
fn founders_begin_on_footing() {
    let world = World::new(4_242, 24);
    for organism in &world.organisms {
        assert!(
            organism
                .walker_shape()
                .stands(world.ground(), organism.position),
            "founder {:?} begins without footing at {:?}",
            organism.id,
            organism.position
        );
    }
}

#[test]
fn a_near_consumer_descends_a_generated_roofed_nest_entry() {
    const SEED: u64 = 4_242;
    let route = generated_nest_entry(SEED);
    let from = route[0];
    let inside = *route.last().unwrap();
    let mut world = World::new(SEED, 0);
    assert!(world.ground().stands(from, WALKER_HEIGHT));
    assert!(world.ground().stands(inside, WALKER_HEIGHT));
    assert!(world.ground().solid([inside[0], inside[1] + 2, inside[2]]));
    assert!(spot(world.ground(), from, inside, 8));

    world.organisms = vec![
        Organism::founding(
            OrganismId(0),
            SpeciesId(3),
            Kingdom::Producer,
            VolumeRef::from_tag(18),
            [1, 1, 1],
            inside,
            300,
        ),
        Organism::founding(
            OrganismId(900),
            SpeciesId(2),
            Kingdom::Consumer,
            VolumeRef::from_tag(16),
            [3, 1, 1],
            from,
            300,
        ),
    ];
    let mut twin = world.clone();
    assert!(matches!(world.apply(Intent::Idle), Outcome::Idled));
    twin.apply(Intent::Idle);

    let hunter = world
        .organisms
        .iter()
        .find(|organism| organism.id == OrganismId(900))
        .unwrap()
        .position;
    assert!(
        route.contains(&hunter),
        "hunter left the generated entry: {hunter:?}"
    );
    assert!(
        hunter[1] < from[1],
        "hunter did not descend: {from:?} -> {hunter:?}"
    );
    assert!(world.ground().stands(hunter, WALKER_HEIGHT));
    assert_eq!(
        world
            .organisms
            .iter()
            .find(|organism| organism.id == OrganismId(900))
            .and_then(|organism| organism.last_seen),
        Some(LastSeen {
            target: OrganismId(0),
            position: inside,
            ticks_left: 8,
        }),
        "direct near-tier perception must become replayable organism state"
    );
    assert_eq!(
        crate::snapshot::state_hash(&world),
        crate::snapshot::state_hash(&twin)
    );
}

#[test]
fn the_same_pursuit_selects_between_bodies_at_a_generated_threshold() {
    const SEED: u64 = 0;
    const PREY: OrganismId = OrganismId(0);
    const HUNTER: OrganismId = OrganismId(900);
    let route = generated_nest_entry(SEED);
    let setup = |hunter_extent| {
        let mut world = World::new(SEED, 0);
        world.organisms = vec![
            Organism::founding(
                PREY,
                SpeciesId(3),
                Kingdom::Producer,
                VolumeRef::from_tag(18),
                [1, 1, 1],
                route[2],
                300,
            ),
            Organism::founding(
                HUNTER,
                SpeciesId(2),
                Kingdom::Consumer,
                VolumeRef::from_tag(16),
                hunter_extent,
                route[0],
                300,
            ),
        ];
        world
    };
    let mut compact = setup([1, 1, 1]);
    let mut broad = setup([3, 1, 3]);
    let mut compact_twin = compact.clone();
    let mut broad_twin = broad.clone();

    compact.apply(Intent::Idle);
    broad.apply(Intent::Idle);
    compact_twin.apply(Intent::Idle);
    broad_twin.apply(Intent::Idle);

    fn hunter(world: &World, id: OrganismId) -> &Organism {
        world
            .organisms
            .iter()
            .find(|organism| organism.id == id)
            .unwrap()
    }
    for world in [&compact, &broad] {
        assert_eq!(
            hunter(world, HUNTER)
                .last_fauna_decision
                .as_ref()
                .map(|decision| decision.selected_drive),
            Some(crate::organism::FaunaDrive::Pursue),
            "body shape must constrain the shared pursuit rather than select it"
        );
        assert!(
            hunter(world, HUNTER)
                .walker_shape()
                .stands(world.ground(), hunter(world, HUNTER).position)
        );
    }
    assert_eq!(hunter(&compact, HUNTER).walker_shape().radius(), 0);
    assert_eq!(hunter(&broad, HUNTER).walker_shape().radius(), 1);
    assert_eq!(hunter(&compact, HUNTER).position, route[1]);
    assert_ne!(
        hunter(&broad, HUNTER).position,
        route[1],
        "the broad body entered the one-voxel generated threshold"
    );
    assert_eq!(
        crate::snapshot::state_hash(&compact),
        crate::snapshot::state_hash(&compact_twin)
    );
    assert_eq!(
        crate::snapshot::state_hash(&broad),
        crate::snapshot::state_hash(&broad_twin)
    );
}

#[test]
fn live_height_changes_perception_and_behavior_over_the_same_ground() {
    const SEED: u64 = 0;
    const PREY: OrganismId = OrganismId(0);
    const HUNTER: OrganismId = OrganismId(900);
    const RANGE: i32 = 8;
    let make_hunter = |tall: bool, position| {
        let mut hunter = Organism::founding(
            HUNTER,
            SpeciesId(2),
            Kingdom::Consumer,
            VolumeRef::from_tag(16),
            [3, 1, 1],
            position,
            300,
        );
        if tall {
            hunter
                .body
                .attach(
                    VolumeRef::from_tag(17),
                    1,
                    [1, 5, 1],
                    Attachment {
                        parent: hunter.body.root,
                        offset: [0, 6, 0],
                        yaw: Yaw::Zero,
                    },
                    Provenance::founding(),
                )
                .unwrap();
        }
        hunter
    };
    let make_prey = |position| {
        Organism::founding(
            PREY,
            SpeciesId(3),
            Kingdom::Producer,
            VolumeRef::from_tag(18),
            [1, 1, 1],
            position,
            300,
        )
    };
    let compact_hunter = make_hunter(false, [0; 3]);
    let tall_hunter = make_hunter(true, [0; 3]);
    let prey_shape = make_prey([0; 3]).walker_shape();
    let compact_shape = compact_hunter.walker_shape();
    let tall_shape = tall_hunter.walker_shape();
    assert_eq!(compact_hunter.feeding_mode(), tall_hunter.feeding_mode());
    assert_eq!((compact_shape.radius(), compact_shape.height()), (0, 1));
    assert_eq!((tall_shape.radius(), tall_shape.height()), (0, 3));

    let terrain = World::new(SEED, 0);
    let (observer, target) =
        generated_sight_split(terrain.ground(), compact_shape, tall_shape, prey_shape);
    let setup = |hunter: Organism| {
        let mut world = World::new(SEED, 0);
        let mut hunter = hunter;
        hunter.position = observer;
        world.organisms = vec![make_prey(target), hunter];
        world
    };
    let mut compact = setup(compact_hunter);
    let mut tall = setup(tall_hunter);
    let mut compact_twin = compact.clone();
    let mut tall_twin = tall.clone();

    assert!(!spot_for(
        compact.ground(),
        compact_shape,
        observer,
        prey_shape,
        target,
        RANGE,
    ));
    assert!(spot_for(
        tall.ground(),
        tall_shape,
        observer,
        prey_shape,
        target,
        RANGE,
    ));

    compact.apply(Intent::Idle);
    tall.apply(Intent::Idle);
    compact_twin.apply(Intent::Idle);
    tall_twin.apply(Intent::Idle);

    let decision = |world: &World| {
        world
            .organisms
            .iter()
            .find(|organism| organism.id == HUNTER)
            .and_then(|organism| organism.last_fauna_decision.as_ref())
            .map(|decision| decision.selected_drive)
    };
    assert_eq!(decision(&compact), None);
    assert_eq!(decision(&tall), Some(crate::organism::FaunaDrive::Pursue));
    assert_eq!(
        crate::snapshot::state_hash(&compact),
        crate::snapshot::state_hash(&compact_twin)
    );
    assert_eq!(
        crate::snapshot::state_hash(&tall),
        crate::snapshot::state_hash(&tall_twin)
    );
}

#[test]
fn incorporation_fits_on_the_surface_and_refuses_inside_the_same_burrow() {
    const SEED: u64 = 0;
    const PREY: OrganismId = OrganismId(900);
    let route = generated_nest_entry(SEED);
    let setup = |eater_at, prey_at| {
        let mut world = World::new(SEED, 0);
        world.organisms = vec![
            Organism::founding(
                OrganismId(0),
                SpeciesId(1),
                Kingdom::Consumer,
                VolumeRef::from_tag(1),
                [1, 1, 1],
                eater_at,
                300,
            ),
            Organism::founding(
                PREY,
                SpeciesId(3),
                Kingdom::Producer,
                VolumeRef::from_tag(2),
                [3, 1, 3],
                prey_at,
                100,
            ),
        ];
        world
    };
    let meal = Intent::Metabolize {
        organism: PREY,
        placement: Placement::Explicit {
            parent: PartId(0),
            offset: [0, 2, 0],
            yaw: Yaw::Zero,
        },
    };

    let mut surface = setup(route[0], route[1]);
    assert!(matches!(
        surface.apply(meal.clone()),
        Outcome::Incorporated { .. }
    ));
    assert_eq!(surface.controlled().unwrap().walker_shape().radius(), 1);
    assert!(
        surface
            .controlled()
            .unwrap()
            .walker_shape()
            .stands(surface.ground(), route[0])
    );

    let mut inside = setup(route[1], route[2]);
    let parts_before = inside.body().unwrap().parts.len();
    let hash_before = crate::snapshot::state_hash(&inside);
    assert_eq!(
        inside.apply(meal),
        Outcome::Rejected(Rejection::NoRoom),
        "the one-voxel interior accepted anatomy with a three-voxel cross-section"
    );
    assert_eq!(inside.body().unwrap().parts.len(), parts_before);
    assert_eq!(inside.controlled().unwrap().walker_shape().radius(), 0);
    assert!(inside.organisms.iter().any(|organism| organism.id == PREY));
    assert_ne!(
        crate::snapshot::state_hash(&inside),
        hash_before,
        "a rejected intent still advances the ordered world"
    );
}

#[test]
fn a_near_consumer_routes_to_a_recently_seen_target_around_a_turn() {
    // This is a player-carvable L bore, not a parallel collision fixture. The
    // prey is occluded at the turn, so the only target this tick is the
    // predator's replayed LastSeen state.
    let mut world = World::new(4_242, 0);
    for [x, z] in [[0, 0], [4, 0], [4, 4]] {
        let top = world
            .ground()
            .surface(x, z)
            .expect("the generated enclosure has surface terrain");
        world.ground.carve([x, top + 1, z], 1);
    }

    let mut stances = Vec::new();
    for z in -ENCLOSURE..=ENCLOSURE {
        for x in -ENCLOSURE..=ENCLOSURE {
            let Some(top) = world.ground().surface(x, z) else {
                continue;
            };
            let at = [x, top + 1, z];
            if world.ground().stands(at, WALKER_HEIGHT) {
                stances.push(at);
            }
        }
    }
    let encounter = stances.iter().find_map(|from| {
        stances.iter().find_map(|target| {
            let direct = step(world.ground(), *from, *target);
            let routed = route_step(world.ground(), *from, *target, 8)?;
            (routed != direct
                && !spot(world.ground(), *from, *target, 8)
                && step(world.ground(), routed, *target) != routed)
                .then_some((*from, *target, routed))
        })
    });
    let (from, last_seen_at, expected_next) =
        encounter.expect("the carved bore contains an occluded local turn");
    assert!(world.ground().stands(from, WALKER_HEIGHT));
    assert!(world.ground().stands(last_seen_at, WALKER_HEIGHT));

    let predator_id = world.controlled_id().expect("the fixture has a founder");
    let mut predator = Organism::founding(
        predator_id,
        SpeciesId(2),
        Kingdom::Consumer,
        VolumeRef::from_tag(16),
        [3, 1, 1],
        from,
        300,
    );
    predator.last_seen = Some(LastSeen {
        target: OrganismId(900),
        position: last_seen_at,
        ticks_left: 2,
    });
    let prey = Organism::founding(
        OrganismId(900),
        SpeciesId(3),
        Kingdom::Producer,
        VolumeRef::from_tag(18),
        [1, 1, 1],
        last_seen_at,
        300,
    );
    world.organisms = vec![predator, prey];
    let_go(&mut world);
    let mut twin = world.clone();

    assert!(matches!(world.apply(Intent::Idle), Outcome::Idled));
    twin.apply(Intent::Idle);
    let predator = world.controlled().expect("predator remains controlled");
    assert_eq!(
        predator.position, expected_next,
        "lost sight should take the bounded legal detour, not greedily stall"
    );
    assert_eq!(
        predator.last_seen,
        Some(LastSeen {
            target: OrganismId(900),
            position: last_seen_at,
            ticks_left: 1,
        })
    );
    assert!(world.ground().stands(predator.position, WALKER_HEIGHT));
    assert_eq!(
        crate::snapshot::state_hash(&world),
        crate::snapshot::state_hash(&twin)
    );
}

#[test]
fn a_moving_player_can_be_lost_and_reacquired_through_a_carved_turn() {
    // Keep the player and hunter in one replayed World. The test searches the
    // generated enclosure after a normal L carve, rather than giving either
    // actor a hand-authored navigation mesh or a scripted state transition.
    let mut terrain = World::new(4_242, 0);
    for [x, z] in [[0, 0], [4, 0], [4, 4]] {
        let top = terrain
            .ground()
            .surface(x, z)
            .expect("the generated enclosure has surface terrain");
        terrain.ground.carve([x, top + 1, z], 1);
    }
    let mut stances = Vec::new();
    for z in -ENCLOSURE..=ENCLOSURE {
        for x in -ENCLOSURE..=ENCLOSURE {
            let Some(top) = terrain.ground().surface(x, z) else {
                continue;
            };
            let at = [x, top + 1, z];
            if terrain.ground().stands(at, WALKER_HEIGHT) {
                stances.push(at);
            }
        }
    }

    let candidate = stances.iter().find_map(|from| {
        stances.iter().find_map(|seen| {
            if !spot(terrain.ground(), *from, *seen, 8) || seen == from {
                return None;
            }
            [[1, 0], [-1, 0], [0, 1], [0, -1]]
                .into_iter()
                .find_map(|[dx, dz]| {
                    let hidden = step(
                        terrain.ground(),
                        *seen,
                        [seen[0] + dx, seen[1], seen[2] + dz],
                    );
                    if hidden == *seen || !terrain.ground().stands(hidden, WALKER_HEIGHT) {
                        return None;
                    }

                    let mut initial = terrain.clone();
                    initial.organisms = vec![
                        Organism::founding(
                            OrganismId(0),
                            SpeciesId(3),
                            Kingdom::Producer,
                            VolumeRef::from_tag(18),
                            [1, 1, 1],
                            *seen,
                            300,
                        ),
                        Organism::founding(
                            OrganismId(900),
                            SpeciesId(2),
                            Kingdom::Consumer,
                            VolumeRef::from_tag(16),
                            [3, 1, 1],
                            *from,
                            300,
                        ),
                    ];
                    let mut probe = initial.clone();
                    let mut trace = vec![Intent::Idle];
                    probe.apply(Intent::Idle);
                    let hunter_after_sight = probe
                        .organisms
                        .iter()
                        .find(|organism| organism.id == OrganismId(900))
                        .expect("the short pursuit preserves its hunter")
                        .position;
                    let saw = probe
                        .organisms
                        .iter()
                        .find(|organism| organism.id == OrganismId(900))
                        .and_then(|organism| organism.last_seen)
                        .is_some_and(|memory| memory.position == *seen);
                    if !saw || spot(probe.ground(), hunter_after_sight, hidden, 8) {
                        return None;
                    }

                    let moved = probe.apply(Intent::Move {
                        delta: [
                            hidden[0] - seen[0],
                            hidden[1] - seen[1],
                            hidden[2] - seen[2],
                        ],
                    });
                    trace.push(Intent::Move {
                        delta: [
                            hidden[0] - seen[0],
                            hidden[1] - seen[1],
                            hidden[2] - seen[2],
                        ],
                    });
                    if !matches!(moved, Outcome::Moved) || probe.position() != Some(hidden) {
                        return None;
                    }
                    let lost = probe
                        .organisms
                        .iter()
                        .find(|organism| organism.id == OrganismId(900))
                        .and_then(|organism| organism.last_seen)
                        .is_some_and(|memory| memory.position == *seen && memory.ticks_left < 8);
                    if !lost {
                        return None;
                    }

                    for _ in 0..8 {
                        probe.apply(Intent::Idle);
                        trace.push(Intent::Idle);
                        let reacquired = probe
                            .organisms
                            .iter()
                            .find(|organism| organism.id == OrganismId(900))
                            .and_then(|organism| organism.last_seen)
                            .is_some_and(|memory| {
                                memory.position == hidden && memory.ticks_left == 8
                            });
                        if reacquired {
                            return Some((initial, trace, hidden));
                        }
                    }
                    None
                })
        })
    });
    let (mut world, trace, hidden) =
        candidate.expect("the carved turn admits a lost-sight reacquisition run");
    let mut twin = world.clone();
    for intent in trace {
        world.apply(intent.clone());
        twin.apply(intent);
        assert_grounded_near(&world);
    }
    let memory = world
        .organisms
        .iter()
        .find(|organism| organism.id == OrganismId(900))
        .and_then(|organism| organism.last_seen)
        .expect("the hunter reacquired the player");
    assert_eq!(memory.position, hidden);
    assert_eq!(memory.ticks_left, 8);
    assert_eq!(
        crate::snapshot::state_hash(&world),
        crate::snapshot::state_hash(&twin)
    );
}

#[test]
fn a_carve_opens_a_grounded_step_and_replays() {
    // Find a two-high face that owned locomotion cannot cross, then carve a
    // one-voxel doorway through it. The fixture searches real generated
    // ground rather than constructing a second terrain authority for a test.
    let mut world = World::new(4_242, 0);
    let directions = [[1, 0], [-1, 0], [0, 1], [0, -1]];
    let doorway = (-ENCLOSURE..=ENCLOSURE).find_map(|z| {
        (-ENCLOSURE..=ENCLOSURE).find_map(|x| {
            let top = world.ground().surface(x, z)?;
            let from = [x, top + 1, z];
            if !world.ground().stands(from, WALKER_HEIGHT) {
                return None;
            }
            directions.into_iter().find_map(|[dx, dz]| {
                let target = [from[0] + dx, from[1], from[2] + dz];
                let blocked = step(world.ground(), from, target) == from
                    && world.ground().solid(target)
                    && world.ground().solid([target[0], target[1] + 1, target[2]])
                    && world.ground().solid([target[0], target[1] - 1, target[2]]);
                blocked.then_some((from, target))
            })
        })
    });
    let (from, target) = doorway.expect("the seeded terrain contains a climb-blocking face");
    let me = world.controlled_id().expect("embodied");
    world
        .organisms
        .iter_mut()
        .find(|organism| organism.id == me)
        .expect("the controlled organism exists")
        .position = from;

    let mut twin = world.clone();
    let opening = [target[0], target[1] + 1, target[2]];
    let trace = [
        Intent::Carve {
            at: opening,
            radius: 1,
        },
        Intent::Move {
            delta: [target[0] - from[0], 0, target[2] - from[2]],
        },
    ];
    let outcomes = world.apply_all(&trace);
    twin.apply_all(&trace);

    assert!(matches!(outcomes[0], Outcome::Carved { removed, .. } if removed > 0));
    assert_eq!(
        world.position(),
        Some(target),
        "doorway did not admit the step"
    );
    assert!(world.ground().stands(target, WALKER_HEIGHT));
    assert_eq!(
        crate::snapshot::state_hash(&world),
        crate::snapshot::state_hash(&twin)
    );
}

#[test]
fn autonomous_near_bodies_need_sight_and_take_grounded_steps() {
    // Use the same generated wall as the player receipt, but put an
    // uncommanded predator behind it. Before the opening it cannot acquire
    // the producer; the carve makes the prey visible and lets ecology cross
    // one legal voxel into the doorway on its own tick.
    let mut world = World::new(4_242, 0);
    let directions = [[1, 0], [-1, 0], [0, 1], [0, -1]];
    let encounter = (-ENCLOSURE..=ENCLOSURE).find_map(|z| {
        (-ENCLOSURE..=ENCLOSURE).find_map(|x| {
            let top = world.ground().surface(x, z)?;
            let from = [x, top + 1, z];
            if !world.ground().stands(from, WALKER_HEIGHT) {
                return None;
            }
            directions.into_iter().find_map(|[dx, dz]| {
                let doorway = [from[0] + dx, from[1], from[2] + dz];
                let prey_top = world.ground().surface(doorway[0] + dx, doorway[2] + dz)?;
                let prey = [doorway[0] + dx, prey_top + 1, doorway[2] + dz];
                let blocked = step(world.ground(), from, doorway) == from
                    && world.ground().solid(doorway)
                    && world
                        .ground()
                        .solid([doorway[0], doorway[1] + 1, doorway[2]])
                    && world
                        .ground()
                        .solid([doorway[0], doorway[1] - 1, doorway[2]]);
                let close = (0..3).all(|axis| (prey[axis] - from[axis]).abs() <= 8);
                (blocked
                    && close
                    && world.ground().stands(prey, WALKER_HEIGHT)
                    && !spot(world.ground(), from, prey, 8))
                .then_some((from, doorway, prey))
            })
        })
    });
    let (from, doorway, prey_at) =
        encounter.expect("the seeded terrain contains a nearby occluded doorway");
    let predator_id = world.controlled_id().expect("the fixture has a founder");
    let predator = Organism::founding(
        predator_id,
        SpeciesId(2),
        Kingdom::Consumer,
        VolumeRef::from_tag(16),
        [3, 1, 1],
        from,
        300,
    );
    let prey = Organism::founding(
        OrganismId(900),
        SpeciesId(3),
        Kingdom::Producer,
        VolumeRef::from_tag(18),
        [1, 1, 1],
        prey_at,
        300,
    );
    world.organisms = vec![predator, prey];
    let_go(&mut world);

    assert!(matches!(world.apply(Intent::Idle), Outcome::Idled));
    assert_eq!(
        world.controlled().map(|organism| organism.position),
        Some(from),
        "an occluded prey must not become an abstract steering target"
    );

    let mut twin = world.clone();
    let opening = [doorway[0], doorway[1] + 1, doorway[2]];
    let outcome = world.apply(Intent::Carve {
        at: opening,
        radius: 1,
    });
    twin.apply(Intent::Carve {
        at: opening,
        radius: 1,
    });
    assert!(matches!(outcome, Outcome::Carved { removed, .. } if removed > 0));

    // The carve was the hand's, so that tick belongs to the hand and this body
    // holds still through it (TD4). Let go again and the ecology takes the
    // opening on the next tick, which is the claim under test.
    assert_eq!(
        world.controlled().map(|organism| organism.position),
        Some(from),
        "the tick a player acts on is the player's"
    );
    for world in [&mut world, &mut twin] {
        let_go(world);
        world.apply(Intent::Idle);
    }

    assert_eq!(
        world.controlled().map(|organism| organism.position),
        Some(doorway),
        "the embodied ecology did not enter the opened doorway"
    );
    assert!(world.ground().stands(doorway, WALKER_HEIGHT));
    assert_eq!(
        crate::snapshot::state_hash(&world),
        crate::snapshot::state_hash(&twin)
    );
}

#[test]
fn grounded_population_scale_is_deterministic_and_cohort_conserving() {
    // This is the G3 scale receipt. It deliberately uses the real founding
    // path, so both tiers, actual ground, body-derived drives, reproduction,
    // and the graph boundary are exercised together. Wall-clock cost belongs
    // to the release example; a test only asserts portable facts.
    const POPULATION: u32 = 300;
    // The first pass establishes the tier line and the second exercises the
    // resulting mix. Longer wall-clock sampling belongs in the release probe.
    const TICKS: u32 = 2;
    let run = || {
        let mut world = World::new(4_242, POPULATION - 1);
        assert_eq!(world.organisms.len(), POPULATION as usize);
        for _ in 0..TICKS {
            assert!(matches!(world.apply(Intent::Idle), Outcome::Idled));
            assert_grounded_near(&world);
        }
        world
    };

    let a = run();
    let b = run();
    assert_eq!(
        crate::snapshot::state_hash(&a),
        crate::snapshot::state_hash(&b)
    );

    let (actual_members, actual_biomass, actual_energy) = a
        .organisms
        .iter()
        .filter(|organism| organism.is_alive() && organism.tier == Tier::Far)
        .fold((0u64, 0u64, 0u64), |(count, biomass, energy), organism| {
            (
                count + 1,
                biomass + organism.biomass_mg(),
                energy + organism.energy_mg,
            )
        });
    assert!(
        actual_members > 0,
        "the population never exercised the far tier"
    );
    let cohorts = a.far_cohorts();
    assert_eq!(
        crate::cohort::conserved_totals(&cohorts),
        (actual_members, actual_biomass, actual_energy),
        "cohort formation lost a scalar from the population"
    );
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
    // It used to spawn a carcass; since TD6 it enriches the ground the
    // depositor is standing on, which is what "returns matter to the
    // enclosure" always meant. The column, not the roster, is what moves.
    let mut world = World::new(3, 2);
    let count = world.organisms.len();
    let before = world.soil().total_mg();

    let outcome = world.apply(Intent::Deposit { mass_mg: 200 });

    assert!(matches!(outcome, Outcome::Deposited { .. }));
    assert_eq!(world.organisms.len(), count, "no carcass is minted");
    // Loose lower bound: the tick that follows also pays rent into the ground
    // and lets any producer draw out of it, so the deposit is most of this
    // move but not all of it.
    assert!(
        world.soil().total_mg() >= before + 150,
        "the ground should be richer for it: {} -> {}",
        before,
        world.soil().total_mg()
    );
}
