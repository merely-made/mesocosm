// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! How a world begins.
//!
//! Split out of `world.rs` at the 600-line ceiling. Worldgen is a different
//! concern from running a world, and it is the one place where every seeded
//! decision about an enclosure is made in a fixed order.

use crate::body::{BodyDocument, SpeciesId, VolumeRef};
use crate::organism::{Kingdom, Organism, OrganismId, Signal, Stage};
use crate::rng::Rng;

use super::{ENCLOSURE, PLACE_SALT, PLACE_SIDE, World, organism_extent};

impl World {
    /// Builds the standard fixture: one critter and a deterministic scatter of
    /// organisms drawn from the seeded stream.
    pub fn new(seed: u64, organism_count: u32) -> Self {
        let mut rng = Rng::from_seed(seed);

        // The played critter is organism zero, built by the same constructor
        // as everything else. It is distinguished only by being pointed at.
        let mut organisms = Vec::with_capacity(organism_count as usize + 1);
        organisms.push(Organism::founding(
            OrganismId(0),
            SpeciesId(1),
            Kingdom::Consumer,
            VolumeRef::from_tag(1),
            [2, 2, 2],
            [0, 0, 0],
            1_000,
        ));

        for index in 1..=organism_count {
            // Draws happen in a fixed order, so the scatter is reproducible.
            let x = rng.range_i32(-ENCLOSURE, ENCLOSURE);
            let y = rng.range_i32(-2, 2);
            let z = rng.range_i32(-ENCLOSURE, ENCLOSURE);
            let mass = 100 + rng.below(400);
            let species = SpeciesId(2 + (rng.below(3) as u32));
            // A mix that can actually sustain itself: mostly producers,
            // because a world without income runs down.
            let kingdom = match rng.below(6) {
                0 => Kingdom::Consumer,
                1 => Kingdom::Decomposer,
                _ => Kingdom::Producer,
            };
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
            let volume = VolumeRef::from_tag(16 + (index % 8) as u8);
            let half_extent = organism_extent(16 + (index % 8) as u8);
            organisms.push(Organism {
                body: BodyDocument::new(species, volume, mass, half_extent),
                energy_mg: mass,
                position: [x, y, z],
                stage: Stage::Mature,
                age,
                since_offspring: 0,
                signal,
                venom_mg,
                guise,
                ..Organism::founding(
                    OrganismId(index),
                    species,
                    kingdom,
                    volume,
                    half_extent,
                    [x, y, z],
                    mass,
                )
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

        Self {
            tick: 0,
            epoch: 0,
            rng,
            controlled: Some(OrganismId(0)),
            control_lost: None,
            unlocked: std::collections::BTreeSet::from([SpeciesId(1)]),
            // The starting body already counts: the player is holding it, so
            // the frontier begins where they begin rather than at nothing.
            frontier: organisms.first().map(Organism::complexity).unwrap_or(0),
            lineages: {
                // Everything the world began with is a founding lineage: no
                // parent, no name, because nobody was there to give it one.
                let mut lineages = crate::species::Lineages::new();
                for organism in &organisms {
                    lineages.found(organism.species);
                }
                lineages
            },
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
        }
    }
}
