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
        let mut species_roles = BTreeMap::from([(SpeciesId(1), Kingdom::Consumer)]);
        founders.push(Founder {
            id: OrganismId(0),
            species: SpeciesId(1),
            kingdom: Kingdom::Consumer,
            mass_mg: 1_000,
            position: [0, 0, 0],
            stage: Stage::Juvenile,
            age: 0,
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
            // A species has one inherited silhouette. Keep the original
            // seeded role draw for the first founder, then inherit that role
            // for later founders of the same species.
            let sampled_kingdom = match rng.below(6) {
                0 => Kingdom::Consumer,
                1 => Kingdom::Decomposer,
                _ => Kingdom::Producer,
            };
            let kingdom = *species_roles.entry(species).or_insert(sampled_kingdom);
            // Staggered ages, so the enclosure is mid-life rather than all
            // hatching on the same tick.
            let age = rng.below(200) as u32;

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
                energy_mg: mass_mg,
                stage: founder.stage,
                age: founder.age,
                since_offspring: 0,
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

        let mut world = Self {
            tick: 0,
            epoch: 0,
            rng,
            controlled: Some(OrganismId(0)),
            control_lost: None,
            unlocked: std::collections::BTreeSet::from([SpeciesId(1)]),
            // The starting body already counts: the player is holding it, so
            // the frontier begins where they begin rather than at nothing.
            // Filled after the registry exists, since intricacy reads it.
            frontier: 0,
            lineages,
            development_palette,
            // Places take their own stream, so dividing an enclosure does not
            // rearrange the creatures scattered across it.
            places: crate::places::Places::scatter(
                &mut Rng::from_seed(seed ^ PLACE_SALT),
                PLACE_SIDE,
                ENCLOSURE,
            ),
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
