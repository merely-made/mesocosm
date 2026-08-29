// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! How a world begins.
//!
//! Split out of `world.rs` at the 600-line ceiling. Worldgen is a different
//! concern from running a world, and it is the one place where every seeded
//! decision about an enclosure is made in a fixed order.

use std::collections::BTreeMap;

use crate::body::SpeciesId;
use crate::development::{DevelopmentError, PartPalette};
use crate::organism::ecology;
use crate::organism::{Kingdom, Organism, OrganismId, Signal, Stage};
use crate::rng::Rng;
use crate::species::Lineages;

use super::{DEVELOPMENT_SALT, ENCLOSURE, PLACE_SALT, PLACE_SIDE, RECIPE_SALT, World};

/// Matter in one voxel column when the enclosure is founded.
///
/// **The world's entire matter budget is this times the column count**, and
/// nothing ever adds to it: light is the open input, matter is not. (TD6)
///
/// A hundred milligrams is the ecology's own reference body mass, so the rule
/// reads plainly: *the enclosure opens holding one reference body's worth of
/// substance under every voxel column it has*. At `ENCLOSURE = 16` that is a
/// 33x33 grid and 108,900 mg — about three times what a 61-founder cohort
/// carries in bodies and reserves, so the terrarium opens with real room to
/// grow into and a fixed ceiling to grow within. Sized from the constant, so
/// widening the enclosure widens the budget with it.
const SOIL_SEED_MG_PER_COLUMN: u64 = 100;

struct Founder {
    id: OrganismId,
    species: SpeciesId,
    kingdom: Kingdom,
    mass_mg: u64,
    position: [i32; 3],
    stage: Stage,
    age: u32,
    since_offspring: u32,
    signal: Signal,
    venom_mg: u64,
    guise: Kingdom,
    development_seed: u64,
}

impl World {
    /// Builds the standard fixture: one critter and a deterministic scatter of
    /// organisms drawn from the seeded stream.
    pub fn new(seed: u64, organism_count: u32) -> Self {
        Self::with_development_palette(seed, organism_count, PartPalette::primitive())
            .expect("the baseline developmental palette is valid")
    }

    /// Builds a world under an explicitly admitted developmental palette.
    ///
    /// The palette is snapshotted with the world. A host can therefore replace
    /// the baseline fixture references without smuggling asset choices into a
    /// lineage recipe or making replay depend on ambient configuration.
    pub fn with_development_palette(
        seed: u64,
        organism_count: u32,
        development_palette: PartPalette,
    ) -> Result<Self, DevelopmentError> {
        let mut rng = Rng::from_seed(seed);

        // Draft identities and ecology first. Bodies cannot be developed until
        // every founding lineage has its recipe, and constructing root-only
        // placeholder organisms here would preserve the split authority this
        // migration is removing.
        let mut founders = Vec::with_capacity(organism_count as usize + 1);
        // Kingdom floor: guarantee the non-played species cover all three
        // kingdoms before any founder draws a role, so a seed can no longer
        // found with a missing rung (2 of 10 seeds drew zero producers under
        // the old free draw -- guaranteed collapse). The 3 non-played species
        // ids are fixed by the `rng.below(3)` draw below; a Fisher-Yates
        // shuffle of the 3 kingdoms assigns one each, deterministic from the
        // seeded stream. A species beyond the floor (none exist today) draws
        // freely in the `or_insert_with` below, so variety survives if the
        // roster ever grows past 3. (2026-08-29 TD2b)
        let mut floor_kingdoms = [Kingdom::Producer, Kingdom::Consumer, Kingdom::Decomposer];
        for i in (1..floor_kingdoms.len()).rev() {
            floor_kingdoms.swap(i, rng.below(i as u64 + 1) as usize);
        }
        let mut species_of: BTreeMap<Kingdom, SpeciesId> = BTreeMap::new();
        for (offset, kingdom) in floor_kingdoms.into_iter().enumerate() {
            species_of.insert(kingdom, SpeciesId(2 + offset as u32));
        }
        // **Counts make the pyramid** (2026-08-29 TD7). A uniform species draw
        // founded equal thirds, which is an ecology standing on its point: the
        // 20-odd consumers it put on 20-odd producers over-grazed the stand
        // within 200 ticks in every seed of TD6's receipt. The tiers are
        // therefore drawn as a composition rather than per founder — exactly
        // `PRODUCER_SHARE` producers and `CONSUMER_SHARE` consumers of the
        // non-played founders, the rest decomposers — and shuffled into
        // arrival order from the same seeded stream, so the pyramid is the
        // world's shape rather than a distribution it usually lands near. At
        // the shipping 60 that is 40 / 15 / 5. Individual sizes stay what the
        // bodies honestly say.
        let mut kingdoms = pyramid(organism_count as usize);
        for i in (1..kingdoms.len()).rev() {
            kingdoms.swap(i, rng.below(i as u64 + 1) as usize);
        }
        founders.push(Founder {
            id: OrganismId(0),
            species: SpeciesId(1),
            kingdom: Kingdom::Consumer,
            mass_mg: 1_000,
            position: [0, 0, 0],
            stage: Stage::Juvenile,
            // Kept newborn, unlike the mid-life stagger below: the player's
            // life should start near its beginning, not drawn from the same
            // whole-life distribution as the ecology around it. (TD5b)
            age: 0,
            since_offspring: 0,
            signal: Signal::Plain,
            venom_mg: 0,
            guise: Kingdom::Consumer,
            development_seed: founder_seed(seed, OrganismId(0)),
        });

        for index in 1..=organism_count {
            // Draws happen in a fixed order, so the scatter is reproducible.
            let x = rng.range_i32(-ENCLOSURE, ENCLOSURE);
            let y = rng.range_i32(-2, 2);
            let z = rng.range_i32(-ENCLOSURE, ENCLOSURE);
            let mass = 100 + rng.below(400);
            // A species has one inherited silhouette, so the tier this founder
            // drew names the species rather than the other way round.
            let kingdom = kingdoms[(index - 1) as usize];
            let species = species_of[&kingdom];
            // Staggered ages, so the enclosure is mid-life rather than all
            // hatching on the same tick. Proportional to the founder's own
            // lifespan_for_mass rather than a flat 200: a flat stagger left
            // every founder a newborn against a 2,000-3,000-tick life, so
            // nothing died of old age until ~1,800 and the enclosure held no
            // real carrion until then (TD5 finding). Uniform over the whole
            // life puts the founding cohort's mean age at its own midpoint —
            // mid-life — with a near-death tail that seeds carrion from the
            // first ticks, the shape a throwaway rng.below(2000) diagnostic
            // proved: 42 decomposers alive at the 10,000-tick horizon in seed
            // 7. (2026-08-29 TD5b)
            let age = rng.below(u64::from(ecology::lifespan_for_mass(mass).max(1))) as u32;
            // Staggered the same way: an un-staggered founder pool all reads
            // since_offspring 0, gating the world's whole first brood behind
            // one full gestation. (2026-08-29 TD2b)
            let since_offspring =
                rng.below(u64::from(ecology::gestation_for_mass(mass).max(1))) as u32;

            // Most things are honest. A minority lie, in both directions: a
            // harmless thing wearing a warning, and a dangerous thing wearing
            // none. Both are rare, because a world of liars teaches nothing.
            let (signal, venom_mg, guise) = match rng.below(10) {
                0 => (Signal::Warning, 0, kingdom),
                1 => (Signal::Plain, 90 + rng.below(60), Kingdom::Producer),
                2..=3 => (Signal::Warning, 60 + rng.below(60), kingdom),
                _ => (Signal::Plain, 0, kingdom),
            };
            founders.push(Founder {
                id: OrganismId(index),
                species,
                kingdom,
                mass_mg: mass,
                position: [x, y, z],
                stage: Stage::Mature,
                age,
                since_offspring,
                signal,
                venom_mg,
                guise,
                development_seed: founder_seed(seed, OrganismId(index)),
            });
        }

        // Everything the world began with is a founding lineage: no parent,
        // no name, because nobody was there to give it one.
        let mut lineages = Lineages::new();
        for founder in &founders {
            lineages.found(founder.species);
        }
        // Each line draws its recipe from its own stream, so body generation
        // never advances the ecology stream. Producers stay simple; anything
        // that moves gets stretches and limbs.
        for founder in &founders {
            if lineages
                .get(founder.species)
                .is_some_and(|species| species.recipe.appendages() > 1)
            {
                continue;
            }
            let mut stream = Rng::from_seed(seed ^ RECIPE_SALT ^ u64::from(founder.species.0));
            let limbed = founder.kingdom != Kingdom::Producer;
            lineages.set_recipe(founder.species, crate::axis::seed(&mut stream, limbed));
            lineages.set_symmetry(founder.species, founder.kingdom.symmetry());
        }

        // A founder's selected mass is a lower bound. Genesis has no parent
        // ledger to debit, so when a rare recipe needs more than the draw to
        // keep every part positive-mass, the world starts it at that exact
        // structural floor. Births below enforce the stricter filial rule and
        // wait for provisioning instead.
        let mut organisms = Vec::with_capacity(founders.len());
        for founder in founders {
            let lineage = lineages
                .get(founder.species)
                .expect("every founder registered a lineage");
            let mut body = match lineage.realize(
                founder.development_seed,
                founder.mass_mg,
                development_palette,
            ) {
                Ok(body) => body,
                Err(DevelopmentError::InsufficientMass { parts, .. }) => lineage.realize(
                    founder.development_seed,
                    u64::from(parts),
                    development_palette,
                )?,
                Err(error) => return Err(error),
            };
            let mass_mg = body.total_mass_mg();
            body.plan.symmetry = founder.kingdom.symmetry();
            organisms.push(Organism {
                id: founder.id,
                species: founder.species,
                body,
                development_seed: founder.development_seed,
                life_history_mass_mg: mass_mg,
                position: founder.position,
                tier: crate::places::Tier::Near,
                last_seen: None,
                fauna_policy: crate::organism::FaunaPolicy::default(),
                last_fauna_decision: None,
                energy_mg: mass_mg,
                stage: founder.stage,
                age: founder.age,
                since_offspring: founder.since_offspring,
                signal: founder.signal,
                venom_mg: founder.venom_mg,
                guise: founder.guise,
            });
        }

        // The founding population enters the record. Without this a seeded
        // creature's first event is whatever happened *to* it, so its origin
        // is invisible and its causal line begins in the middle.
        let pending = organisms
            .iter()
            .map(|o| crate::history::Event::Born {
                organism: o.id,
                species: o.species,
                parent: None,
            })
            .collect();

        let grown = crate::places::Places::grown(seed ^ PLACE_SALT, PLACE_SIDE, ENCLOSURE);
        let ground = crate::places::Ground::grow(&grown, ENCLOSURE);

        // The old enclosure was an abstract field, so founders carried an
        // arbitrary y draw. Brick truth makes that invalid: an embodied
        // creature must begin on footing, with enough headroom for the
        // near-tier walker. Keep the draw above in the seeded sequence so the
        // landscape transition does not rearrange every later founder choice.
        for organism in &mut organisms {
            let shape = organism.walker_shape();
            organism.position =
                crate::places::surface_stance_for(&ground, shape, organism.position)
                    .expect("the grown enclosure covers every founding body");
            debug_assert!(shape.stands(&ground, organism.position));
        }

        let mut world = Self {
            tick: 0,
            epoch: 0,
            rng,
            controlled: Some(OrganismId(0)),
            control_lost: None,
            // A world opens under the hand. Nobody has idled yet, so the
            // first tick's instincts leave the played critter alone.
            idle_run: 0,
            unlocked: std::collections::BTreeSet::from([SpeciesId(1)]),
            // The starting body already counts: the player is holding it, so
            // the frontier begins where they begin rather than at nothing.
            // Filled after the registry exists, since intricacy reads it.
            frontier: 0,
            lineages,
            development_palette,
            // Places take their own stream, so dividing an enclosure does
            // not rearrange the creatures scattered across it. Grown, not
            // scattered (G1 adoption 2026-08-08): same site draws as the old
            // lattice, so the partition is bit-identical, but links derive
            // from the landscape and the ground below is real.
            places: grown.places.clone(),
            ground,
            soil: crate::places::Soil::seeded(ENCLOSURE, SOIL_SEED_MG_PER_COLUMN),
            ranges: std::collections::BTreeMap::new(),
            record: crate::record::WorldRecord::new(),
            organisms,
            next_organism: organism_count + 1,
            last_tally: crate::organism::Tally::default(),
            pending,
        };

        // The starting body already counts, and intricacy needs the registry,
        // so the high-water mark is set once the world exists rather than in
        // the initialiser.
        world.frontier = world
            .organisms
            .first()
            .map(|o| world.intricacy(o))
            .unwrap_or(0);
        Ok(world)
    }
}

/// Share of the non-played founders that are producers. Two thirds: the base
/// of the chain has to out-number what grazes it, and TD6 measured what
/// happens when it does not.
const PRODUCER_SHARE: (usize, usize) = (2, 3);
/// Share that are consumers. A quarter — fewer mouths than plants, and still
/// enough of them to be an ecology rather than a stand with visitors.
const CONSUMER_SHARE: (usize, usize) = (1, 4);

/// The founding composition: many producers, fewer consumers, few
/// decomposers, in that order.
///
/// Exact rather than drawn, so the pyramid is a guarantee. **Every kingdom is
/// still founded** — the TD2b floor, kept: a tier that rounds to nothing takes
/// one founder from the widest rather than leaving a rung out of the chain.
fn pyramid(count: usize) -> Vec<Kingdom> {
    let producers = count * PRODUCER_SHARE.0 / PRODUCER_SHARE.1;
    let consumers = count * CONSUMER_SHARE.0 / CONSUMER_SHARE.1;
    let mut tiers = [
        producers,
        consumers,
        count.saturating_sub(producers + consumers),
    ];
    for tier in 0..tiers.len() {
        if tiers[tier] > 0 {
            continue;
        }
        let widest = (0..tiers.len())
            .max_by_key(|&other| tiers[other])
            .expect("three tiers");
        if tiers[widest] > 1 {
            tiers[widest] -= 1;
            tiers[tier] += 1;
        }
    }
    [Kingdom::Producer, Kingdom::Consumer, Kingdom::Decomposer]
        .into_iter()
        .zip(tiers)
        .flat_map(|(kingdom, many)| std::iter::repeat_n(kingdom, many))
        .collect()
}

fn founder_seed(world_seed: u64, organism: OrganismId) -> u64 {
    let mut stream = Rng::from_seed(world_seed ^ DEVELOPMENT_SALT ^ u64::from(organism.0));
    stream.next_u64()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Before the kingdom floor (2026-08-29, TD2b), 2 of these 10 seeds
    // founded zero producer species -- guaranteed collapse under any
    // constants. Every seed must now found all three kingdoms among the
    // non-played species.
    #[test]
    fn every_seed_founds_all_three_kingdoms() {
        for seed in 1u64..=10 {
            let world = World::new(seed, 60);
            let mut kingdoms: BTreeMap<Kingdom, u32> = BTreeMap::new();
            for organism in &world.organisms {
                if organism.species != SpeciesId(1) {
                    *kingdoms.entry(organism.kingdom()).or_default() += 1;
                }
            }
            for kingdom in [Kingdom::Producer, Kingdom::Consumer, Kingdom::Decomposer] {
                assert!(
                    kingdoms.get(&kingdom).is_some_and(|&count| count > 0),
                    "seed {seed} founded no {kingdom:?} among the non-played species: {kingdoms:?}"
                );
            }
        }
    }

    // The founding composition is a pyramid, not equal thirds (2026-08-29,
    // TD7): many producers, fewer consumers, few decomposers, exactly and in
    // every seed rather than on average.
    #[test]
    fn founding_is_a_pyramid_in_every_seed() {
        for seed in 1u64..=10 {
            let world = World::new(seed, 60);
            let mut kingdoms: BTreeMap<Kingdom, u32> = BTreeMap::new();
            for organism in &world.organisms {
                if organism.species != SpeciesId(1) {
                    *kingdoms.entry(organism.kingdom()).or_default() += 1;
                }
            }
            assert_eq!(
                (
                    kingdoms.get(&Kingdom::Producer).copied(),
                    kingdoms.get(&Kingdom::Consumer).copied(),
                    kingdoms.get(&Kingdom::Decomposer).copied(),
                ),
                (Some(40), Some(15), Some(5)),
                "seed {seed} founded {kingdoms:?} rather than the 2/3 : 1/4 : rest pyramid"
            );
        }
    }

    // The pyramid never costs the TD2b floor: a founding too small to give a
    // tier its share still gives it a founder.
    #[test]
    fn a_small_pyramid_still_founds_every_kingdom() {
        for count in 3..=12 {
            let tiers = pyramid(count);
            assert_eq!(tiers.len(), count);
            for kingdom in [Kingdom::Producer, Kingdom::Consumer, Kingdom::Decomposer] {
                assert!(
                    tiers.contains(&kingdom),
                    "a founding of {count} left out {kingdom:?}: {tiers:?}"
                );
            }
        }
    }

    // Before the stagger (2026-08-29, TD2b), every founder read
    // since_offspring 0, gating a world's whole first brood behind one full
    // gestation. Founders beyond the played critter must now spread across
    // the range, the way `age` already does.
    #[test]
    fn since_offspring_is_staggered_like_age() {
        let world = World::new(4_242, 60);
        let distinct: std::collections::BTreeSet<u32> = world
            .organisms
            .iter()
            .filter(|o| o.species != SpeciesId(1))
            .map(|o| o.since_offspring)
            .collect();
        assert!(
            distinct.len() > 1,
            "every non-played founder read the same since_offspring: {distinct:?}"
        );
        assert!(
            distinct.iter().any(|&v| v > 0),
            "no founder started with a head start on gestation"
        );
    }

    // Before the mid-life stagger (2026-08-29, TD5b), every founder's age was
    // rng.below(200) against a lifespan in the thousands, so nothing died of
    // old age until deep into the run and the enclosure held no real carrion
    // until then (TD5's corpse-drought finding). Age must now range well
    // past the old flat cap, and the played critter must stay a newborn --
    // the player's life should start near its beginning, not drawn from the
    // same distribution as the ecology around it.
    #[test]
    fn ages_are_staggered_across_the_founders_own_lifespan() {
        let world = World::new(4_242, 60);
        let max_age = world
            .organisms
            .iter()
            .filter(|o| o.species != SpeciesId(1))
            .map(|o| o.age)
            .max()
            .expect("60 non-played founders");
        assert!(
            max_age > 200,
            "no founder aged past the old flat rng.below(200) cap: {max_age}"
        );
        let played = world
            .organisms
            .iter()
            .find(|o| o.id == OrganismId(0))
            .expect("the played critter founds as organism 0");
        assert_eq!(played.age, 0, "the played critter did not start a newborn");
    }
}
