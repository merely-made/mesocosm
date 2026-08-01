// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The enclosure living on its own schedule.
//!
//! Split out of `organism.rs` on 2026-08-01, because that file had grown past
//! this repo's six-hundred-line ceiling and the rule is to split before adding
//! rather than after. The types live next door; this is what happens to them
//! every tick.
//!
//! Nothing here knows which organism is being played. That is not an oversight;
//! it is the wing's third law holding at the level of the simulation.

use std::collections::BTreeMap;

use crate::body::BodyDocument;
use crate::rng::Rng;

use super::{Kingdom, Organism, OrganismId, Stage, Tally};

/// Ticks of growth before an organism can reproduce.
pub(crate) const MATURITY: u32 = 90;
/// Ticks of life before an organism dies of age.
pub(crate) const LIFESPAN: u32 = 600;
/// Ticks between one offspring and the next.
pub(crate) const GESTATION: u32 = 120;
/// Milligrams a producer fixes per tick in open ground.
const FIXES_MG: u64 = 3;
/// Milligrams of upkeep every living thing owes before its size is counted.
///
/// The floor. What a body actually pays is this plus a share of what it
/// weighs, so being large is a standing cost rather than a free upgrade.
///
/// Everything pays rent. Without this a producer's income is free and the
/// population grows without bound, which is what the first run of this did:
/// 75 organisms became 1530 in 600 ticks. Biomass share is meaningless when
/// everyone's share grows.
pub(crate) const UPKEEP_MG: u64 = 1;
/// Milligrams of body a creature can carry per extra milligram of upkeep.
///
/// Tuned so a starting critter pays a few milligrams a tick and a
/// well-fed one pays enough to notice. This is the number that decides
/// whether growing is worth it.
pub(crate) const UPKEEP_SHARE: u64 = 250;
/// Edge of a crowding cell, in voxel units.
const CROWD_CELL: i32 = 8;
/// Neighbours a cell supports before its occupants start shading each other
/// out. Beyond this a producer's income falls away and self-thinning begins.
const CROWD_COMFORT: u32 = 4;
/// Milligrams a decomposer draws from nearby carrion per tick.
const DECAYS_MG: u64 = 3;
/// How far a decomposer reaches for the dead, in voxel units.
const DECOMPOSE_RANGE: i32 = 6;
/// Milligrams a consumer takes from living prey per tick.
const GRAZES_MG: u64 = 4;
/// How far a consumer reaches for a meal, in voxel units.
const GRAZE_RANGE: i32 = 5;
/// Fraction of a parent's mass an offspring costs, as a divisor.
pub(crate) const OFFSPRING_COST: u64 = 4;
/// Mass below which an organism cannot sustain itself.
pub(crate) const STARVATION_MG: u64 = 20;

/// Which crowding cell a position falls in.
fn cell_of(position: [i32; 3]) -> (i32, i32) {
    (
        position[0].div_euclid(CROWD_CELL),
        position[2].div_euclid(CROWD_CELL),
    )
}

/// Advances every organism one tick.
///
/// Deterministic: organisms are visited in id order, offspring are appended in
/// the order their parents produced them, and every random value comes from
/// the seeded stream.
pub fn step(organisms: &mut Vec<Organism>, next_id: &mut u32, rng: &mut Rng) -> Tally {
    let mut tally = Tally::default();

    // Crowding, counted once per tick on a coarse grid. A BTreeMap keeps
    // iteration ordered, and bucketing keeps this O(n) rather than O(n^2),
    // which matters at the population sizes this produces.
    let mut density: BTreeMap<(i32, i32), u32> = BTreeMap::new();
    for organism in organisms.iter().filter(|o| o.is_alive()) {
        *density.entry(cell_of(organism.position)).or_default() += 1;
    }

    // Carrion positions are read before anything changes, so decomposers all
    // see the same world within a tick rather than racing each other.
    let carrion: Vec<([i32; 3], usize)> = organisms
        .iter()
        .enumerate()
        .filter(|(_, o)| o.stage == Stage::Carrion)
        .map(|(index, o)| (o.position, index))
        .collect();

    // Living producers, read before anything changes, so grazers all see the
    // same pasture within a tick rather than racing each other.
    let grazeable: Vec<([i32; 3], usize)> = organisms
        .iter()
        .enumerate()
        .filter(|(_, o)| o.is_alive() && o.kingdom == Kingdom::Producer)
        .map(|(index, o)| (o.position, index))
        .collect();

    let mut drained: Vec<(usize, u64)> = Vec::new();
    let mut newborns: Vec<Organism> = Vec::new();

    for organism in organisms.iter_mut() {
        organism.age = organism.age.saturating_add(1);

        match organism.stage {
            Stage::Juvenile | Stage::Mature => {
                // Everything alive pays rent, every tick, **and the rent
                // scales with the body**. Budget first, then the body itself:
                // a creature with nothing left to spend eats itself.
                organism.pay_upkeep();

                match organism.kingdom {
                    // Producers make biomass from the world, but they shade
                    // each other out. Income falls with crowding, so a stand
                    // thins itself instead of growing without bound.
                    Kingdom::Producer => {
                        let crowd = density
                            .get(&cell_of(organism.position))
                            .copied()
                            .unwrap_or(1);
                        let share = FIXES_MG
                            .saturating_mul(CROWD_COMFORT as u64)
                            / crowd.max(1) as u64;
                        // Floored at rent. A shaded-out producer stagnates
                        // rather than starving, because otherwise an entire
                        // stand of identical plants crosses the starvation
                        // line on the same tick and the patch goes extinct
                        // instead of thinning.
                        let share = share.clamp(UPKEEP_MG, FIXES_MG);
                        organism.gain_mass(share);
                    }
                    // Consumers eat. Without this they were guaranteed to
                    // starve, so every world converged to producers only and
                    // the trophic cycle had a missing rung.
                    Kingdom::Consumer => {
                        if let Some((_, prey)) = grazeable.iter().copied().find(|(at, _)| {
                            (0..3).all(|a| (at[a] - organism.position[a]).abs() <= GRAZE_RANGE)
                        }) {
                            organism.gain_mass(GRAZES_MG);
                            drained.push((prey, GRAZES_MG));
                        }
                    }
                    // Decomposers only earn where something has died.
                    Kingdom::Decomposer => {
                        if let Some((_, source)) = carrion.iter().copied().find(|(at, _)| {
                            (0..3).all(|a| (at[a] - organism.position[a]).abs() <= DECOMPOSE_RANGE)
                        }) {
                            organism.gain_mass(DECAYS_MG);
                            drained.push((source, DECAYS_MG));
                        }
                    }
                }

                if organism.stage == Stage::Juvenile && organism.age >= MATURITY {
                    organism.stage = Stage::Mature;
                    tally.matured += 1;
                }

                organism.since_offspring = organism.since_offspring.saturating_add(1);

                let starved = organism.biomass_mg() <= STARVATION_MG;
                let aged = organism.age >= LIFESPAN;
                if starved || aged {
                    organism.stage = Stage::Carrion;
                    organism.since_offspring = 0;
                    tally.died += 1;
                }
            }

            Stage::Carrion => {
                // The dead return whether or not a decomposer is present, just
                // far more slowly. Locked matter is a real failure mode.
                organism.spend_mass(1);
                if organism.biomass_mg() == 0 {
                    organism.stage = Stage::Spent;
                    tally.returned += 1;
                }
            }

            Stage::Spent => {}
        }
    }

    // What was fed on pays for it. Grazed prey can be killed outright, which
    // is how a consumer turns a producer into carrion for a decomposer.
    for (index, amount) in drained {
        let eaten = &mut organisms[index];
        eaten.spend_mass(amount);
        if eaten.biomass_mg() == 0 {
            eaten.stage = Stage::Spent;
            tally.returned += 1;
        } else if eaten.is_alive() && eaten.biomass_mg() <= STARVATION_MG {
            eaten.stage = Stage::Carrion;
            eaten.since_offspring = 0;
            tally.died += 1;
        }
    }

    // Reproduction, after everything has been fed, so a tick's births do not
    // depend on where in the list a parent happened to sit.
    let ready: Vec<usize> = organisms
        .iter()
        .enumerate()
        .filter(|(_, o)| o.can_reproduce())
        .map(|(index, _)| index)
        .collect();
    for index in ready {
        let parent = &organisms[index];
        let cost = parent.biomass_mg() / OFFSPRING_COST;
        // Wide enough to leave a crowded cell. Dispersal is how a stand
        // escapes its own shade, so a short throw would trap every offspring
        // in the same competition its parent is already losing.
        let scatter = [
            rng.range_i32(-12, 12),
            0,
            rng.range_i32(-12, 12),
        ];
        let child = Organism {
            id: OrganismId(*next_id),
            species: parent.species,
            kingdom: parent.kingdom,
            // **A child starts small.** Cloning the parent's anatomy while
            // charging a quarter of its scalar mass manufactured structural
            // mass: a forty-part parent produced a forty-part child and paid
            // for a fraction of one. So an offspring gets a fresh root sized
            // to exactly what was paid, wearing its parent's shape.
            //
            // Inheriting a developmental program and regrowing a phenotype
            // from it is P4. This is the honest placeholder until then, and it
            // conserves mass, which the clone did not.
            body: BodyDocument::new(
                parent.species,
                parent.volume(),
                cost,
                parent.half_extent(),
            ),
            energy_mg: cost,
            position: [
                parent.position[0] + scatter[0],
                parent.position[1],
                parent.position[2] + scatter[2],
            ],
            stage: Stage::Juvenile,
            age: 0,
            since_offspring: 0,
            // A lie is heritable. An offspring wears its parent's colours and
            // carries its parent's bite, which is what makes a mimic lineage a
            // thing you can learn rather than a coin flip per organism.
            signal: parent.signal,
            venom_mg: parent.venom_mg,
            guise: parent.guise,
        };
        *next_id += 1;
        newborns.push(child);

        let parent = &mut organisms[index];
        parent.spend_mass(cost);
        parent.since_offspring = 0;
        tally.born += 1;
    }

    organisms.extend(newborns);
    organisms.retain(|o| o.stage != Stage::Spent);
    tally
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::{SpeciesId, VolumeRef};
    use crate::organism::Signal;

    fn organism(kingdom: Kingdom, mass: u64) -> Organism {
        Organism::founding(
            OrganismId(0),
            SpeciesId(2),
            kingdom,
            VolumeRef::from_tag(16),
            [1, 1, 1],
            [0, 0, 0],
            mass,
        )
    }

    fn run(organisms: &mut Vec<Organism>, ticks: u32) -> Tally {
        let mut rng = Rng::from_seed(1);
        let mut next = 100;
        let mut total = Tally::default();
        for _ in 0..ticks {
            let t = step(organisms, &mut next, &mut rng);
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
        for _ in 0..4_000 {
            if done(world) {
                return true;
            }
            step(world, &mut next_id, &mut rng);
        }
        done(world)
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
        //
        // Run until it does rather than to a tuned tick count: how long a
        // creature lasts is now a function of what it weighs and what it has
        // banked, and a hardcoded number would be re-tuned every time either
        // rule moved.
        let mut world = vec![organism(Kingdom::Consumer, 60)];
        let died = until(&mut world, |w| w.first().is_some_and(|o| o.stage == Stage::Carrion));
        assert!(died, "upkeep must actually kill");

        // And then it returns to the world rather than lying there forever.
        let gone = until(&mut world, |w| w.is_empty());
        assert!(gone, "carrion eventually returns to the world");
    }

    #[test]
    fn a_reserve_buys_time_and_then_the_body_pays() {
        // The order that makes burning a meal worth something: budget first,
        // then autophagy. A creature with nothing banked starts shrinking
        // immediately; one with a reserve does not.
        let mut fed = organism(Kingdom::Consumer, 200);
        fed.energy_mg = 500;
        let mut empty = organism(Kingdom::Consumer, 200);
        empty.energy_mg = 0;

        let body_before = empty.biomass_mg();
        for _ in 0..10 {
            fed.pay_upkeep();
            empty.pay_upkeep();
        }

        assert_eq!(fed.biomass_mg(), 200, "a stocked creature spends its budget");
        assert!(fed.energy_mg < 500, "and it does spend it");
        assert!(empty.biomass_mg() < body_before, "an empty one eats itself");
    }

    #[test]
    fn organisms_mature_then_reproduce() {
        // Long enough for the parent to mature and breed once, short enough
        // that its offspring has not yet matured in turn.
        let mut world = vec![organism(Kingdom::Producer, 400)];
        let tally = run(&mut world, GESTATION + 10);
        assert_eq!(tally.matured, 1, "only the parent has come of age yet");
        assert!(tally.born >= 1, "a mature producer should have offspring");
        assert!(world.len() > 1);
    }

    #[test]
    fn an_offspring_costs_its_parent_mass() {
        let mut world = vec![organism(Kingdom::Producer, 400)];
        run(&mut world, GESTATION + 10);

        let child = world.iter().find(|o| o.id.0 >= 100).expect("an offspring");
        assert!(child.biomass_mg() > 0, "an offspring starts with real mass");
        assert_eq!(child.stage, Stage::Juvenile, "and starts young");
        assert_eq!(child.species, world[0].species, "lineage carries forward");

        // Breeding is not free: the parent paid for it out of its own body.
        let parent = &world[0];
        assert!(
            parent.biomass_mg() < 400 + FIXES_MG * (GESTATION as u64 + 10),
            "the parent is lighter than an un-bred one would be"
        );
    }

    #[test]
    fn the_dead_become_carrion_then_return() {
        let mut world = vec![organism(Kingdom::Consumer, 30)];
        assert!(
            until(&mut world, |w| w.first().is_some_and(|o| o.stage == Stage::Carrion)),
            "starving leaves a body"
        );
        assert!(until(&mut world, |w| w.is_empty()), "carrion returns to the world");
    }

    #[test]
    fn a_decomposer_feeds_on_the_dead_beside_it() {
        let mut world = vec![
            organism(Kingdom::Decomposer, 200),
            Organism { id: OrganismId(1), stage: Stage::Carrion,  ..organism(Kingdom::Consumer, 300) },
        ];
        let before = world[0].biomass_mg();
        run(&mut world, 10);
        assert!(world[0].biomass_mg() > before, "a decomposer earns beside a corpse");
    }

    #[test]
    fn a_decomposer_alone_declines() {
        // It pays rent and earns nothing. Since upkeep now takes the budget
        // before the body, the decline shows in the budget first and in the
        // flesh once that is gone.
        let mut world = vec![organism(Kingdom::Decomposer, 200)];
        let (body, budget) = (world[0].biomass_mg(), world[0].energy_mg);
        run(&mut world, 10);

        assert!(
            world[0].energy_mg < budget || world[0].biomass_mg() < body,
            "no dead, no living: something has to be draining"
        );

        // And left alone long enough, it dies of it.
        assert!(
            until(&mut world, |w| w.first().is_none_or(|o| !o.is_alive())),
            "an unfed decomposer does not last forever"
        );
    }

    /// Producers alone are unbounded, and that is correct rather than a bug.
    /// Crowding limits what a *patch* supports, but dispersal escapes the
    /// patch, so a pasture with nothing grazing it fills the world.
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

    /// The runaway this whole trophic loop exists to answer. Before consumers
    /// could eat, every world converged to producers only and the population
    /// ran away: 75 organisms became 1530 in 600 ticks, which makes biomass
    /// share meaningless because everyone's share grows.
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
        assert!(
            end < start * 4,
            "nor run away: {start} -> {end}"
        );
    }

    /// All three rungs have to be able to make a living, or the cycle has a
    /// hole in it and the world converges to whichever rung can.
    #[test]
    fn every_kingdom_can_earn() {
        let mut world = vec![
            Organism { id: OrganismId(0), ..organism(Kingdom::Producer, 300) },
            Organism { id: OrganismId(1), position: [2, 0, 0], ..organism(Kingdom::Consumer, 300) },
            Organism {
                id: OrganismId(2),
                position: [3, 0, 0],
                stage: Stage::Carrion,
                ..organism(Kingdom::Decomposer, 400)
            },
            Organism { id: OrganismId(3), position: [4, 0, 0], ..organism(Kingdom::Decomposer, 300) },
        ];

        let consumer_before = world[1].biomass_mg();
        let decomposer_before = world[3].biomass_mg();
        run(&mut world, 20);

        let consumer = world.iter().find(|o| o.id == OrganismId(1)).unwrap();
        let decomposer = world.iter().find(|o| o.id == OrganismId(3)).unwrap();
        assert!(consumer.biomass_mg() > consumer_before, "a grazer beside a plant eats");
        assert!(decomposer.biomass_mg() > decomposer_before, "a decomposer beside a corpse eats");
    }

    #[test]
    fn an_uncrowded_producer_still_prospers() {
        // Alone in its cell, so it earns full income and comes out ahead.
        let mut world = vec![organism(Kingdom::Producer, 200)];
        let before = world[0].biomass_mg();
        run(&mut world, 40);
        assert!(world[0].biomass_mg() > before, "open ground is worth having");
    }

    /// The ecology lesson as a mechanic: a world with no producers has no
    /// income, so its consumers spend down to nothing.
    #[test]
    fn a_world_without_producers_runs_down() {
        let mut world: Vec<Organism> = (0..5)
            .map(|i| Organism { id: OrganismId(i), ..organism(Kingdom::Consumer, 200) })
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
        assert!(!honestly_armed.signals_falsely(), "a real warning is not a lie");
    }

    /// Batesian: harmless, wearing a warning. Safe to eat, and only something
    /// that learned better will risk it.
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

    /// Aggressive: looks like an ordinary plant, is not. The trap, and the one
    /// that makes reading the world worth doing.
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

    /// The tell is diegetic rather than a marker: a thing wearing a producer's
    /// look but living a consumer's life does not gain mass in open ground.
    ///
    /// Since upkeep takes a banked budget before it takes flesh, a well-fed
    /// mimic holds its weight for a while rather than visibly wasting. The
    /// tell is therefore the *absence of growth* beside a real plant, which
    /// takes patience to read. That is a better tell than a shrinking one.
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
        run(&mut world, GESTATION + 10);
        let child = world.iter().find(|o| o.id.0 >= 100).expect("an offspring");
        assert_eq!(child.signal, Signal::Warning);
        assert_eq!(child.venom_mg, 0);
        assert!(child.is_mimic(), "a mimic lineage is learnable, not a coin flip");
    }

    #[test]
    fn stepping_is_deterministic() {
        let build = || {
            vec![
                organism(Kingdom::Producer, 400),
                Organism { id: OrganismId(1), ..organism(Kingdom::Consumer, 300) },
            ]
        };
        let mut a = build();
        let mut b = build();
        run(&mut a, 250);
        run(&mut b, 250);
        assert_eq!(a, b);
    }
}
