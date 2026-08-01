// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The other things living here.
//!
//! These used to be `Morsel`s: inert matter with a mass, waiting to be
//! collected. That name was the design flaw written down — a morsel is *a
//! small piece of food*, which is what you call something that exists to be
//! eaten. Mark's diagnosis: "we're just kinda munchin'. A free meal, talk
//! about the opposite of a game."
//!
//! An organism runs the same loop the player runs. It grows, matures,
//! reproduces, ages, dies, and rots, whether or not anyone is watching. Eating
//! one is a decision with a cost and a moment, because the thing in front of
//! you is going somewhere on its own.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::body::BodyDocument;

use crate::body::{SpeciesId, VolumeRef};
use crate::rng::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct OrganismId(pub u32);

/// Trophic role. Not a character class: these are the three ways of making a
/// living, and a lineage may combine them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kingdom {
    /// Fixes energy from the world itself. The base of every chain.
    Producer,
    /// Must eat. Pays upkeep and starves without a meal.
    Consumer,
    /// Lives on the dead, returning locked matter to circulation.
    Decomposer,
}

/// What an organism advertises about itself.
///
/// Signalling and counter-signalling: an advertisement is a claim, and a claim
/// can be false. This is what makes choosing a meal a decision rather than a
/// collection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Signal {
    /// Claims nothing. Ordinary, unremarkable, probably safe.
    Plain,
    /// Claims to be dangerous. Bright, loud, conspicuous.
    Warning,
}

/// Where an organism is in its life.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stage {
    /// Growing. Worth less now than it will be shortly.
    Juvenile,
    /// Grown, and able to reproduce.
    Mature,
    /// Dead, and not yet returned. Food for decomposers, poor food for others.
    Carrion,
    /// Fully returned to the world. Removed at the end of the tick.
    Spent,
}

impl Stage {
    pub fn is_alive(self) -> bool {
        matches!(self, Stage::Juvenile | Stage::Mature)
    }
}

/// Ticks of growth before an organism can reproduce.
const MATURITY: u32 = 90;
/// Ticks of life before an organism dies of age.
const LIFESPAN: u32 = 600;
/// Ticks between one offspring and the next.
const GESTATION: u32 = 120;
/// Milligrams a producer fixes per tick in open ground.
const FIXES_MG: u64 = 3;
/// Milligrams **anything alive** burns per tick simply existing.
///
/// Everything pays rent. Without this a producer's income is free and the
/// population grows without bound, which is what the first run of this did:
/// 75 organisms became 1530 in 600 ticks. Biomass share is meaningless when
/// everyone's share grows.
const UPKEEP_MG: u64 = 1;
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
const OFFSPRING_COST: u64 = 4;
/// Mass below which an organism cannot sustain itself.
const STARVATION_MG: u64 = 20;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Organism {
    pub id: OrganismId,
    /// Which lineage this belongs to. Incorporation carries it forward, so a
    /// part you take always knows whose it was.
    pub species: SpeciesId,
    pub kingdom: Kingdom,
    /// This organism's anatomy.
    ///
    /// **Every organism has one**, played or not. Before P1 only the critter
    /// the player inhabited had a body and everything else was a `VolumeRef`
    /// and a `half_extent`, which meant anatomy could never constrain an
    /// unplayed creature and prey had no parts to lose. The pair it replaces
    /// are now readings of the root part, so there is one place a shape is
    /// written down.
    ///
    /// Most organisms carry a single root part. That is a body, not a special
    /// case, and it grows by the same rules as any other.
    pub body: BodyDocument,
    pub position: [i32; 3],
    /// What this organism can spend. Distinct from [`Self::mass_mg`], which is
    /// what it weighs.
    ///
    /// Before P1 only the played critter had a budget, so nothing else could
    /// run out of one.
    pub energy_mg: u64,
    pub mass_mg: u64,
    pub stage: Stage,
    pub age: u32,
    /// Ticks since this organism last reproduced.
    pub since_offspring: u32,
    /// What this organism *advertises*.
    pub signal: Signal,
    /// What it actually does to something that eats it, in milligrams.
    ///
    /// The gap between this and [`Self::signal`] is the whole mechanic. An
    /// honest organism's claim matches its bite. A **Batesian** mimic warns
    /// without a bite: safe, and eaten only by something that learned better.
    /// An **aggressive** mimic looks plain and bites hard: the trap.
    pub venom_mg: u64,
    /// The kingdom this organism *appears* to belong to.
    ///
    /// Usually its own. A mimic's differs, which is what breaks the shape
    /// contract on purpose: roles are read from geometry, so the game teaches
    /// that form tells you function, and a simulacrum violates exactly that
    /// lesson.
    pub guise: Kingdom,
}

impl Organism {
    /// A minimal organism: one root part, and the scalars the ecology moves.
    #[allow(clippy::too_many_arguments)]
    pub fn founding(
        id: OrganismId,
        species: SpeciesId,
        kingdom: Kingdom,
        volume: VolumeRef,
        half_extent: [i32; 3],
        position: [i32; 3],
        mass_mg: u64,
    ) -> Self {
        Self {
            id,
            species,
            kingdom,
            body: BodyDocument::new(species, volume, mass_mg, half_extent),
            position,
            energy_mg: mass_mg,
            mass_mg,
            stage: Stage::Juvenile,
            age: 0,
            since_offspring: 0,
            signal: Signal::Plain,
            venom_mg: 0,
            guise: kingdom,
        }
    }

    /// The volume a projection should draw for this organism: its root part's.
    pub fn volume(&self) -> VolumeRef {
        self.body.part(self.body.root).map(|p| p.volume).unwrap_or(VolumeRef([0; 32]))
    }

    /// This organism's overall half-extent, read off its root part.
    ///
    /// A reading rather than a field, so a body and the shape the world sees
    /// cannot disagree.
    pub fn half_extent(&self) -> [i32; 3] {
        self.body.part(self.body.root).map(|p| p.half_extent).unwrap_or([1, 1, 1])
    }

    pub fn is_alive(&self) -> bool {
        self.stage.is_alive()
    }

    /// Whether this organism is pretending to be something it is not.
    pub fn is_mimic(&self) -> bool {
        self.guise != self.kingdom || self.signals_falsely()
    }

    /// Whether its advertisement is a lie, in either direction.
    pub fn signals_falsely(&self) -> bool {
        match self.signal {
            Signal::Warning => self.venom_mg == 0,
            Signal::Plain => self.venom_mg > 0,
        }
    }

    /// The tell.
    ///
    /// A thing wearing a producer's look but living a consumer's life does not
    /// gain mass in open ground, because it is not fixing anything. Watch it
    /// for a while and the lie shows. Unfair is fine here; unknowable is not,
    /// so every mimic leaves something a second encounter can find.
    pub fn betrays_itself(&self) -> bool {
        self.guise == Kingdom::Producer && self.kingdom != Kingdom::Producer
    }

    /// Whether this organism is ready to produce an offspring.
    pub fn can_reproduce(&self) -> bool {
        self.stage == Stage::Mature
            && self.since_offspring >= GESTATION
            && self.mass_mg > STARVATION_MG * OFFSPRING_COST
    }
}

/// What a tick did to the world's organisms, so a host can show it and a test
/// can assert on it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tally {
    pub matured: u32,
    pub born: u32,
    pub died: u32,
    pub returned: u32,
}

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
                // Everything alive pays rent, every tick.
                organism.mass_mg = organism.mass_mg.saturating_sub(UPKEEP_MG);

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
                        organism.mass_mg = organism.mass_mg.saturating_add(share);
                    }
                    // Consumers eat. Without this they were guaranteed to
                    // starve, so every world converged to producers only and
                    // the trophic cycle had a missing rung.
                    Kingdom::Consumer => {
                        if let Some((_, prey)) = grazeable.iter().copied().find(|(at, _)| {
                            (0..3).all(|a| (at[a] - organism.position[a]).abs() <= GRAZE_RANGE)
                        }) {
                            organism.mass_mg = organism.mass_mg.saturating_add(GRAZES_MG);
                            drained.push((prey, GRAZES_MG));
                        }
                    }
                    // Decomposers only earn where something has died.
                    Kingdom::Decomposer => {
                        if let Some((_, source)) = carrion.iter().copied().find(|(at, _)| {
                            (0..3).all(|a| (at[a] - organism.position[a]).abs() <= DECOMPOSE_RANGE)
                        }) {
                            organism.mass_mg = organism.mass_mg.saturating_add(DECAYS_MG);
                            drained.push((source, DECAYS_MG));
                        }
                    }
                }

                if organism.stage == Stage::Juvenile && organism.age >= MATURITY {
                    organism.stage = Stage::Mature;
                    tally.matured += 1;
                }

                organism.since_offspring = organism.since_offspring.saturating_add(1);

                let starved = organism.mass_mg <= STARVATION_MG;
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
                organism.mass_mg = organism.mass_mg.saturating_sub(1);
                if organism.mass_mg == 0 {
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
        eaten.mass_mg = eaten.mass_mg.saturating_sub(amount);
        if eaten.mass_mg == 0 {
            eaten.stage = Stage::Spent;
            tally.returned += 1;
        } else if eaten.is_alive() && eaten.mass_mg <= STARVATION_MG {
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
        let cost = parent.mass_mg / OFFSPRING_COST;
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
            mass_mg: cost,
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
        parent.mass_mg = parent.mass_mg.saturating_sub(cost);
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

    #[test]
    fn a_producer_grows_while_left_alone() {
        let mut world = vec![organism(Kingdom::Producer, 100)];
        run(&mut world, 50);
        assert!(
            world[0].mass_mg > 100,
            "waiting should be worth something: {} mg",
            world[0].mass_mg
        );
    }

    #[test]
    fn a_consumer_starves_without_a_meal() {
        // 60 mg, 1 mg/tick upkeep, starving at 20: dead around tick 40.
        let mut world = vec![organism(Kingdom::Consumer, 60)];
        run(&mut world, 45);
        assert_eq!(world[0].stage, Stage::Carrion, "upkeep must actually kill");

        // And then it returns to the world rather than lying there forever.
        run(&mut world, 40);
        assert!(world.is_empty());
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
        assert!(child.mass_mg > 0, "an offspring starts with real mass");
        assert_eq!(child.stage, Stage::Juvenile, "and starts young");
        assert_eq!(child.species, world[0].species, "lineage carries forward");

        // Breeding is not free: the parent paid for it out of its own body.
        let parent = &world[0];
        assert!(
            parent.mass_mg < 400 + FIXES_MG * (GESTATION as u64 + 10),
            "the parent is lighter than an un-bred one would be"
        );
    }

    #[test]
    fn the_dead_become_carrion_then_return() {
        let mut world = vec![organism(Kingdom::Consumer, 30)];
        run(&mut world, 20);
        assert_eq!(world[0].stage, Stage::Carrion, "starving leaves a body");
        run(&mut world, 100);
        assert!(world.is_empty(), "carrion eventually returns to the world");
    }

    #[test]
    fn a_decomposer_feeds_on_the_dead_beside_it() {
        let mut world = vec![
            organism(Kingdom::Decomposer, 200),
            Organism { id: OrganismId(1), stage: Stage::Carrion, mass_mg: 300, ..organism(Kingdom::Consumer, 300) },
        ];
        let before = world[0].mass_mg;
        run(&mut world, 10);
        assert!(world[0].mass_mg > before, "a decomposer earns beside a corpse");
    }

    #[test]
    fn a_decomposer_alone_declines() {
        let mut world = vec![organism(Kingdom::Decomposer, 200)];
        let before = world[0].mass_mg;
        run(&mut world, 10);
        assert!(
            world[0].mass_mg < before,
            "no dead, no living: it pays rent and earns nothing"
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

        let consumer_before = world[1].mass_mg;
        let decomposer_before = world[3].mass_mg;
        run(&mut world, 20);

        let consumer = world.iter().find(|o| o.id == OrganismId(1)).unwrap();
        let decomposer = world.iter().find(|o| o.id == OrganismId(3)).unwrap();
        assert!(consumer.mass_mg > consumer_before, "a grazer beside a plant eats");
        assert!(decomposer.mass_mg > decomposer_before, "a decomposer beside a corpse eats");
    }

    #[test]
    fn an_uncrowded_producer_still_prospers() {
        // Alone in its cell, so it earns full income and comes out ahead.
        let mut world = vec![organism(Kingdom::Producer, 200)];
        let before = world[0].mass_mg;
        run(&mut world, 40);
        assert!(world[0].mass_mg > before, "open ground is worth having");
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

        assert!(honest[0].mass_mg > 200, "a real plant fixes energy");
        assert!(
            trap[0].mass_mg < 200,
            "a plant that does not photosynthesise is not a plant"
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
