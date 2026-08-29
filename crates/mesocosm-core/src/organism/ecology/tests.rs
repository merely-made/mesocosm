use super::*;
use crate::body::{SpeciesId, VolumeRef};
use crate::history::Event;
use crate::organism::{Kingdom, Signal};

/// A fixture body big enough to be the mass it is given.
///
/// The half-extent used to be `[1, 1, 1]` — twenty-seven voxels carrying three
/// hundred milligrams, which nothing noticed until TD6 gave a body plan an
/// adult mass derived from its own volume. `[5, 5, 5]` is 1,331 voxels, so
/// these fixtures sit well under their ceiling and can still be watched
/// growing. (2026-08-29 TD6)
fn organism(kingdom: Kingdom, mass: u64) -> Organism {
    Organism::founding(
        OrganismId(0),
        SpeciesId(2),
        kingdom,
        VolumeRef::from_tag(16),
        [5, 5, 5],
        [0, 0, 0],
        mass,
    )
}

/// A fixture soil with enough matter that these tests measure the rule they
/// name rather than the enclosure running out. Real worlds seed theirs from
/// `world::genesis`; the conservation invariant is proved there, over a
/// founded world, in `tests/matter.rs`.
fn soil() -> Soil {
    Soil::seeded(32, 100_000)
}

fn run(organisms: &mut Vec<Organism>, ticks: u32) -> Tally {
    let mut rng = Rng::from_seed(1);
    let mut next = 100;
    let mut total = Tally::default();
    let mut ground = soil();
    let lineages = registry(organisms);
    for _ in 0..ticks {
        let t = step(
            organisms,
            &mut next,
            &mut rng,
            &mut Vec::new(),
            &lineages,
            PartPalette::primitive(),
            &mut ground,
        );
        total.matured += t.matured;
        total.born += t.born;
        total.died += t.died;
        total.returned += t.returned;
    }
    total
}

/// Steps until `done`, up to a generous cap. Returns whether it happened.
///
/// Tick counts stopped being stable when upkeep became a function of body
/// mass and a banked budget, so tests state the outcome they are waiting
/// for rather than a number that has to be re-tuned.
fn until(world: &mut Vec<Organism>, done: impl Fn(&[Organism]) -> bool) -> bool {
    let mut next_id = 900;
    let mut rng = Rng::from_seed(7);
    let mut ground = soil();
    let lineages = registry(world);
    for _ in 0..4_000 {
        if done(world) {
            return true;
        }
        step(
            world,
            &mut next_id,
            &mut rng,
            &mut Vec::new(),
            &lineages,
            PartPalette::primitive(),
            &mut ground,
        );
    }
    done(world)
}

fn registry(organisms: &[Organism]) -> Lineages {
    let mut lineages = Lineages::new();
    for organism in organisms {
        lineages.found(organism.species);
    }
    lineages
}

#[test]
fn allometric_rates_cross_three_orders_without_flat_steps() {
    let masses = [100, 1_000, 10_000];
    let upkeep: Vec<_> = masses.iter().map(|mass| upkeep_for_mass(*mass)).collect();
    let feeding: Vec<_> = masses
        .iter()
        .map(|mass| feeding_rate_for_mass(*mass))
        .collect();
    let maturity: Vec<_> = masses.iter().map(|mass| maturity_for_mass(*mass)).collect();

    assert!(upkeep[0] < upkeep[1] && upkeep[1] < upkeep[2]);
    assert!(feeding[0] < feeding[1] && feeding[1] < feeding[2]);
    assert!(maturity[0] < maturity[1] && maturity[1] < maturity[2]);
    assert!(
        upkeep[2] < upkeep[0] * 80,
        "upkeep became linear: {upkeep:?}"
    );
}

#[test]
fn trophic_role_is_read_from_body_symmetry() {
    let producer = organism(Kingdom::Producer, 100);
    let consumer = organism(Kingdom::Consumer, 100);
    let decomposer = organism(Kingdom::Decomposer, 100);
    assert_eq!(producer.kingdom(), Kingdom::Producer);
    assert_eq!(consumer.kingdom(), Kingdom::Consumer);
    assert_eq!(decomposer.kingdom(), Kingdom::Decomposer);
}

#[test]
fn a_contractile_consumer_can_take_live_consumer_prey() {
    // The half-extents stay long-and-thin, because that is what makes this
    // body a predator (a Limb-role part performs Contract). They are wider
    // than they were so a 300 mg body sits under its own TD6 adult mass and
    // still has room to take a bite. (2026-08-29 TD6)
    let predator = Organism::founding(
        OrganismId(0),
        SpeciesId(2),
        Kingdom::Consumer,
        VolumeRef::from_tag(16),
        [8, 2, 2],
        [0, 0, 0],
        300,
    );
    let prey = Organism::founding(
        OrganismId(1),
        SpeciesId(3),
        Kingdom::Consumer,
        VolumeRef::from_tag(17),
        [4, 4, 4],
        [2, 0, 0],
        300,
    );
    assert_eq!(predator.feeding_mode(), FeedingMode::Predator);
    assert_eq!(prey.kingdom(), Kingdom::Consumer);
    let mut world = vec![predator.clone(), prey];
    let mut events = Vec::new();
    let mut rng = Rng::from_seed(4);
    let mut next = 2;
    let lines = registry(&world);
    step(
        &mut world,
        &mut next,
        &mut rng,
        &mut events,
        &lines,
        PartPalette::primitive(),
        &mut soil(),
    );

    assert!(events.iter().any(|event| matches!(
        event,
        Event::Fed { eater, from, kind: crate::history::MealKind::Predation, .. }
            if *eater == predator.id && *from == OrganismId(1)
    )));
}

#[test]
fn an_exhausted_body_disperses_through_the_place_graph() {
    let mut place_rng = Rng::from_seed(99);
    let places = Places::scatter(&mut place_rng, 3, 16);
    let mut world = vec![organism(Kingdom::Consumer, 300)];
    world[0].energy_mg = 0;
    let before = world[0].position;
    let mut events = Vec::new();
    let mut rng = Rng::from_seed(7);
    let mut next = 10;
    let lines = registry(&world);
    let tally = step_with_places(
        &mut world,
        &mut next,
        &mut rng,
        &mut events,
        &lines,
        PartPalette::primitive(),
        &mut soil(),
        &places,
        Some([0, 0, 0]),
    );

    assert_ne!(
        world[0].position, before,
        "hunger must leave an exhausted place"
    );
    assert_eq!(tally.moved, 1);
    assert!(
        events.iter().any(
            |event| matches!(event, Event::Moved { organism, .. } if *organism == OrganismId(0))
        )
    );
}

#[test]
fn drive_selection_makes_fast_and_slow_bodies_different() {
    let mut place_rng = Rng::from_seed(121);
    let places = Places::scatter(&mut place_rng, 3, 32);
    let prey = Organism::founding(
        OrganismId(9),
        SpeciesId(3),
        Kingdom::Producer,
        VolumeRef::from_tag(18),
        [1, 1, 1],
        [25, 0, 0],
        300,
    );
    let fast = Organism::founding(
        OrganismId(0),
        SpeciesId(2),
        Kingdom::Consumer,
        VolumeRef::from_tag(19),
        [8, 1, 1],
        [0, 0, 0],
        300,
    );
    let slow = Organism::founding(
        OrganismId(0),
        SpeciesId(2),
        Kingdom::Consumer,
        VolumeRef::from_tag(20),
        [1, 1, 1],
        [0, 0, 0],
        300,
    );
    let mut fast_world = vec![fast, prey.clone()];
    let mut slow_world = vec![slow, prey];
    let mut fast_events = Vec::new();
    let mut slow_events = Vec::new();
    let mut fast_rng = Rng::from_seed(3);
    let mut slow_rng = Rng::from_seed(3);
    let mut fast_next = 20;
    let mut slow_next = 20;
    let fast_lines = registry(&fast_world);
    let slow_lines = registry(&slow_world);
    step_with_places(
        &mut fast_world,
        &mut fast_next,
        &mut fast_rng,
        &mut fast_events,
        &fast_lines,
        PartPalette::primitive(),
        &mut soil(),
        &places,
        Some([0, 0, 0]),
    );
    step_with_places(
        &mut slow_world,
        &mut slow_next,
        &mut slow_rng,
        &mut slow_events,
        &slow_lines,
        PartPalette::primitive(),
        &mut soil(),
        &places,
        Some([0, 0, 0]),
    );

    assert!(fast_world[0].position[0] > slow_world[0].position[0]);
    assert!(
        fast_events
            .iter()
            .any(|event| matches!(event, Event::Moved { .. }))
    );
    assert!(
        slow_events
            .iter()
            .any(|event| matches!(event, Event::Moved { .. }))
    );
}

#[test]
fn far_bodies_form_conserved_cohorts_and_promote_at_the_focus() {
    let mut place_rng = Rng::from_seed(55);
    let places = Places::scatter(&mut place_rng, 3, 32);
    let mut far_a = organism(Kingdom::Producer, 300);
    let mut far_b = organism(Kingdom::Producer, 300);
    far_a.id = OrganismId(1);
    far_b.id = OrganismId(2);
    far_a.position = [24, 0, 24];
    far_b.position = [24, 0, 24];
    far_a.tier = Tier::Far;
    far_b.tier = Tier::Far;
    let mut world = vec![far_a, far_b];
    let mut events = Vec::new();
    let mut rng = Rng::from_seed(8);
    let mut next = 10;
    let lines = registry(&world);
    let tally = step_with_places(
        &mut world,
        &mut next,
        &mut rng,
        &mut events,
        &lines,
        PartPalette::primitive(),
        &mut soil(),
        &places,
        Some([0, 0, 0]),
    );
    assert_eq!(tally.far_cohorts, 1);
    assert_eq!(tally.far_members, 2);
    assert_eq!(tally.far_biomass_mg, 600);

    world[0].position = [0, 0, 0];
    let tally = step_with_places(
        &mut world,
        &mut next,
        &mut rng,
        &mut events,
        &lines,
        PartPalette::primitive(),
        &mut soil(),
        &places,
        Some([0, 0, 0]),
    );
    assert_eq!(tally.promoted, 1);
}

#[test]
fn a_producer_grows_while_left_alone() {
    let mut world = vec![organism(Kingdom::Producer, 100)];
    run(&mut world, 50);
    assert!(
        world[0].biomass_mg() > 100,
        "waiting should be worth something: {} mg",
        world[0].biomass_mg()
    );
}

#[test]
fn a_consumer_starves_without_a_meal() {
    // Upkeep takes the budget first and the body second, so a creature
    // with something in reserve outlives one without. It still dies.
    let mut world = vec![organism(Kingdom::Consumer, 60)];
    let died = until(&mut world, |w| {
        w.first().is_some_and(|o| o.stage == Stage::Carrion)
    });
    assert!(died, "upkeep must actually kill");
    assert!(
        until(&mut world, |w| w.is_empty()),
        "carrion eventually returns to the world"
    );
}

#[test]
fn a_reserve_buys_time_and_then_the_body_pays() {
    let mut fed = organism(Kingdom::Consumer, 200);
    fed.energy_mg = 500;
    let mut empty = organism(Kingdom::Consumer, 200);
    empty.energy_mg = 0;
    let body_before = empty.biomass_mg();

    for _ in 0..10 {
        fed.pay_upkeep();
        empty.pay_upkeep();
    }

    assert_eq!(
        fed.biomass_mg(),
        200,
        "a stocked creature spends its budget"
    );
    assert!(fed.energy_mg < 500, "and it does spend it");
    assert!(empty.biomass_mg() < body_before, "an empty one eats itself");
}

#[test]
fn organisms_mature_then_reproduce() {
    let mut world = vec![organism(Kingdom::Producer, 400)];
    let tally = run(&mut world, gestation_for_mass(400) + 10);
    assert_eq!(tally.matured, 1, "only the parent has come of age yet");
    assert!(tally.born >= 1, "a mature producer should have offspring");
    assert!(world.len() > 1);
}

#[test]
fn an_offspring_costs_its_parent_mass() {
    let mut world = vec![organism(Kingdom::Producer, 400)];
    let ticks = gestation_for_mass(400) + 10;
    run(&mut world, ticks);

    let child = world.iter().find(|o| o.id.0 >= 100).expect("an offspring");
    assert!(child.biomass_mg() > 0, "an offspring starts with real mass");
    assert_eq!(child.stage, Stage::Juvenile, "and starts young");
    assert_eq!(child.species, world[0].species, "lineage carries forward");
    let mut unbred = vec![organism(Kingdom::Producer, 400)];
    unbred[0].since_offspring = 0;
    unbred[0].life_history_mass_mg = 10_000;
    run(&mut unbred, ticks);
    assert!(
        world[0].biomass_mg() < unbred[0].biomass_mg(),
        "the parent paid for birth"
    );
}

#[test]
fn an_underprovisioned_body_waits_without_spending_or_drawing() {
    let mut parent = organism(Kingdom::Consumer, 200);
    parent.stage = Stage::Mature;
    parent.since_offspring = gestation_for_mass(parent.biomass_mg());
    parent.energy_mg = 1_000;
    let mut world = vec![parent];
    let mut lineages = Lineages::new();
    lineages.found(SpeciesId(2));
    lineages.set_recipe(SpeciesId(2), crate::axis::catalogue::centipede(20));
    let mut next = 100;
    let mut rng = Rng::from_seed(9);
    let before_rng = rng;
    let before_mass = world[0].biomass_mg();

    let tally = step(
        &mut world,
        &mut next,
        &mut rng,
        &mut Vec::new(),
        &lineages,
        PartPalette::primitive(),
        &mut soil(),
    );

    assert_eq!(tally.born, 0);
    assert_eq!(world.len(), 1);
    assert_eq!(
        world[0].biomass_mg(),
        before_mass,
        "no filial mass was spent"
    );
    assert_eq!(next, 100, "no child identity was consumed");
    assert_eq!(
        rng, before_rng,
        "a refused birth did not move ecology entropy"
    );
}

#[test]
fn the_dead_become_carrion_then_return() {
    let mut world = vec![organism(Kingdom::Consumer, 30)];
    assert!(
        until(&mut world, |w| w
            .first()
            .is_some_and(|o| o.stage == Stage::Carrion)),
        "starving leaves a body"
    );
    assert!(
        until(&mut world, |w| w.is_empty()),
        "carrion returns to the world"
    );
}

#[test]
fn a_decomposer_feeds_on_the_dead_beside_it() {
    let mut world = vec![
        organism(Kingdom::Decomposer, 200),
        Organism {
            id: OrganismId(1),
            stage: Stage::Carrion,
            ..organism(Kingdom::Consumer, 300)
        },
    ];
    let before = world[0].biomass_mg();
    run(&mut world, 10);
    assert!(
        world[0].biomass_mg() > before,
        "a decomposer earns beside a corpse"
    );
}

#[test]
fn a_decomposer_alone_declines() {
    let mut world = vec![organism(Kingdom::Decomposer, 200)];
    let (body, budget) = (world[0].biomass_mg(), world[0].energy_mg);
    run(&mut world, 10);
    assert!(
        world[0].energy_mg < budget || world[0].biomass_mg() < body,
        "no dead, no living: something has to be draining"
    );
    assert!(
        until(&mut world, |w| w.first().is_none_or(|o| !o.is_alive())),
        "an unfed decomposer does not last forever"
    );
}

#[test]
fn producers_alone_spread_until_something_eats_them() {
    let mut world: Vec<Organism> = (0..40)
        .map(|i| Organism {
            id: OrganismId(i),
            position: [(i as i32) % 4, 0, (i as i32) / 4 % 4],
            ..organism(Kingdom::Producer, 300)
        })
        .collect();
    let start = world.len();
    run(&mut world, 800);
    assert!(
        world.iter().filter(|o| o.is_alive()).count() > start,
        "an ungrazed pasture spreads"
    );
}

#[test]
fn a_mixed_world_holds_its_population() {
    let mut world: Vec<Organism> = (0..60)
        .map(|i| {
            let kingdom = match i % 6 {
                0 => Kingdom::Consumer,
                1 => Kingdom::Decomposer,
                _ => Kingdom::Producer,
            };
            Organism {
                id: OrganismId(i),
                position: [(i as i32 * 3) % 24 - 12, 0, (i as i32 * 5) % 24 - 12],
                age: (i * 7) % 200,
                ..organism(kingdom, 300)
            }
        })
        .collect();

    let start = world.iter().filter(|o| o.is_alive()).count();
    run(&mut world, 800);
    let end = world.iter().filter(|o| o.is_alive()).count();
    assert!(end > 0, "the world must not go extinct: {start} -> {end}");
    assert!(end < start * 4, "nor run away: {start} -> {end}");
}

#[test]
fn every_kingdom_can_earn() {
    let mut world = vec![
        Organism {
            id: OrganismId(0),
            ..organism(Kingdom::Producer, 300)
        },
        Organism {
            id: OrganismId(1),
            position: [2, 0, 0],
            ..organism(Kingdom::Consumer, 300)
        },
        Organism {
            id: OrganismId(2),
            position: [3, 0, 0],
            stage: Stage::Carrion,
            ..organism(Kingdom::Decomposer, 400)
        },
        Organism {
            id: OrganismId(3),
            position: [4, 0, 0],
            ..organism(Kingdom::Decomposer, 300)
        },
    ];

    let consumer_before = world[1].biomass_mg();
    let decomposer_before = world[3].biomass_mg();
    run(&mut world, 20);
    let consumer = world.iter().find(|o| o.id == OrganismId(1)).unwrap();
    let decomposer = world.iter().find(|o| o.id == OrganismId(3)).unwrap();
    assert!(
        consumer.biomass_mg() > consumer_before,
        "a grazer beside a plant eats"
    );
    assert!(
        decomposer.biomass_mg() > decomposer_before,
        "a decomposer beside a corpse eats"
    );
}

#[test]
fn an_uncrowded_producer_still_prospers() {
    let mut world = vec![organism(Kingdom::Producer, 200)];
    let before = world[0].biomass_mg();
    run(&mut world, 40);
    assert!(
        world[0].biomass_mg() > before,
        "open ground is worth having"
    );
}

#[test]
fn a_world_without_producers_runs_down() {
    let mut world: Vec<Organism> = (0..5)
        .map(|i| Organism {
            id: OrganismId(i),
            ..organism(Kingdom::Consumer, 200)
        })
        .collect();
    run(&mut world, 400);
    assert!(
        world.iter().all(|o| !o.is_alive()),
        "consumers alone cannot sustain a world"
    );
}

#[test]
fn an_honest_organism_does_not_lie() {
    let plain = organism(Kingdom::Producer, 100);
    assert!(!plain.is_mimic());
    assert!(!plain.signals_falsely());

    let honestly_armed = Organism {
        signal: Signal::Warning,
        venom_mg: 80,
        ..organism(Kingdom::Producer, 100)
    };
    assert!(
        !honestly_armed.signals_falsely(),
        "a real warning is not a lie"
    );
}

#[test]
fn a_bluffer_warns_without_a_bite() {
    let bluffer = Organism {
        signal: Signal::Warning,
        venom_mg: 0,
        ..organism(Kingdom::Producer, 100)
    };
    assert!(bluffer.signals_falsely());
    assert!(bluffer.is_mimic());
}

#[test]
fn a_trap_looks_plain_and_bites() {
    let trap = Organism {
        signal: Signal::Plain,
        venom_mg: 120,
        guise: Kingdom::Producer,
        ..organism(Kingdom::Consumer, 100)
    };
    assert!(trap.is_mimic());
    assert!(trap.signals_falsely());
    assert!(
        trap.betrays_itself(),
        "unfair is fine, unknowable is not: a trap must leave a tell"
    );
}

#[test]
fn the_tell_is_that_a_trap_does_not_grow() {
    let mut honest = vec![organism(Kingdom::Producer, 200)];
    let mut trap = vec![Organism {
        guise: Kingdom::Producer,
        signal: Signal::Plain,
        venom_mg: 120,
        ..organism(Kingdom::Consumer, 200)
    }];

    run(&mut honest, 30);
    run(&mut trap, 30);
    assert!(honest[0].biomass_mg() > 200, "a real plant fixes energy");
    assert!(
        trap[0].biomass_mg() <= 200,
        "a plant that does not photosynthesise does not put on weight"
    );
    assert!(
        honest[0].biomass_mg() > trap[0].biomass_mg(),
        "and the gap between them is the tell"
    );
}

#[test]
fn a_lie_is_heritable() {
    let mut world = vec![Organism {
        signal: Signal::Warning,
        venom_mg: 0,
        ..organism(Kingdom::Producer, 400)
    }];
    run(&mut world, gestation_for_mass(400) + 10);
    let child = world.iter().find(|o| o.id.0 >= 100).expect("an offspring");
    assert_eq!(child.signal, Signal::Warning);
    assert_eq!(child.venom_mg, 0);
    assert!(
        child.is_mimic(),
        "a mimic lineage is learnable, not a coin flip"
    );
}

#[test]
fn stepping_is_deterministic() {
    let build = || {
        vec![
            organism(Kingdom::Producer, 400),
            Organism {
                id: OrganismId(1),
                ..organism(Kingdom::Consumer, 300)
            },
        ]
    };
    let mut a = build();
    let mut b = build();
    run(&mut a, 250);
    run(&mut b, 250);
    assert_eq!(a, b);
}
