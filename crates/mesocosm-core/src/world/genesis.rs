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
        let mut species_roles: BTreeMap<SpeciesId, Kingdom> =
            BTreeMap::from([(SpeciesId(1), Kingdom::Consumer)]);
        for (offset, kingdom) in floor_kingdoms.into_iter().enumerate() {
            species_roles.insert(SpeciesId(2 + offset as u32), kingdom);
        }
        founders.push(Founder {
            id: OrganismId(0),
            species: SpeciesId(1),
            kingdom: Kingdom::Consumer,
            mass_mg: 1_000,
            position: [0, 0, 0],
            stage: Stage::Juvenile,
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
            let species = SpeciesId(2 + (rng.below(3) as u32));
            // A species has one inherited silhouette: the kingdom floor above
            // already set it for every species that exists today, so this
            // only fires for a species the floor did not reach.
            let kingdom = *species_roles
                .entry(species)
                .or_insert_with(|| match rng.below(6) {
                    0 => Kingdom::Consumer,
                    1 => Kingdom::Decomposer,
                    _ => Kingdom::Producer,
                });
            // Staggered ages, so the enclosure is mid-life rather than all
            // hatching on the same tick.
            let age = rng.below(200) as u32;
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
}
