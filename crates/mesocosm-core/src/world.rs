// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The world: an enclosure holding one critter and the loose matter it can
//! metabolize.
//!
//! A world is a pure function of its seed and the ordered intents applied to
//! it. There are no clock reads, no unordered iteration that reaches the
//! simulation, and no randomness outside the seeded stream, so replaying the
//! same trace against the same seed reproduces the same world exactly.

use serde::{Deserialize, Serialize};

use crate::body::{
    Aabb, Attachment, BodyDocument, Origin, PartId, Provenance, SpeciesId, VolumeRef, Yaw,
};
use crate::rng::Rng;

/// A piece of loose matter in the enclosure, available to be eaten.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Morsel {
    pub id: MorselId,
    /// Which lineage this used to belong to. Incorporation carries it forward.
    pub species: SpeciesId,
    pub volume: VolumeRef,
    pub mass_mg: u64,
    pub half_extent: [i32; 3],
    pub position: [i32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MorselId(pub u32);

/// What a host may ask the world to do. Hosts send intents; they never mutate
/// world state directly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Intent {
    /// Move the critter by a voxel delta.
    Move { delta: [i32; 3] },
    /// Eat a morsel and incorporate it at the given attachment.
    Metabolize {
        morsel: MorselId,
        parent: PartId,
        offset: [i32; 3],
        yaw: Yaw,
    },
    /// Return mass to the enclosure as a new morsel.
    Deposit { mass_mg: u64 },
    /// Advance one tick without acting.
    Idle,
}

/// Why an intent could not be applied. Rejections are part of the recorded
/// outcome, so a replay that rejects the same intents is still identical.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rejection {
    NoSuchMorsel(MorselId),
    NoSuchParent(PartId),
    OutOfReach,
    InsufficientMass,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    Moved,
    Incorporated { part: PartId },
    Deposited { morsel: MorselId },
    Idled,
    Rejected(Rejection),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct World {
    pub tick: u64,
    pub epoch: u64,
    rng: Rng,
    pub position: [i32; 3],
    pub body: BodyDocument,
    /// Ordered by id, so iteration never depends on hashing.
    pub morsels: Vec<Morsel>,
    next_morsel: u32,
    /// Metabolic budget in milligrams. Spent by moving, gained by eating.
    pub energy_mg: u64,
}

/// How far the critter can reach to eat, in voxel units.
const REACH: i32 = 8;

/// Energy spent per unit of movement, in milligrams.
const MOVE_COST_MG: u64 = 1;

impl World {
    /// Builds the standard fixture: one critter and a deterministic scatter of
    /// morsels drawn from the seeded stream.
    pub fn new(seed: u64, morsel_count: u32) -> Self {
        let mut rng = Rng::from_seed(seed);
        let body = BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 1_000, [2, 2, 2]);

        let mut morsels = Vec::with_capacity(morsel_count as usize);
        for index in 0..morsel_count {
            // Draws happen in a fixed order, so the scatter is reproducible.
            let x = rng.range_i32(-16, 16);
            let y = rng.range_i32(-2, 2);
            let z = rng.range_i32(-16, 16);
            let mass = 100 + rng.below(400);
            let species = SpeciesId(2 + (rng.below(3) as u32));
            morsels.push(Morsel {
                id: MorselId(index),
                species,
                volume: VolumeRef::from_tag(16 + (index % 8) as u8),
                mass_mg: mass,
                half_extent: [1, 1, 1],
                position: [x, y, z],
            });
        }

        Self {
            tick: 0,
            epoch: 0,
            rng,
            position: [0, 0, 0],
            body,
            morsels,
            next_morsel: morsel_count,
            energy_mg: 1_000,
        }
    }

    /// Applies one intent and advances the tick. This is the only way world
    /// state changes.
    pub fn apply(&mut self, intent: Intent) -> Outcome {
        let outcome = self.resolve(intent);
        self.tick += 1;
        outcome
    }

    /// Applies an ordered trace, returning every outcome in order.
    pub fn apply_all(&mut self, trace: &[Intent]) -> Vec<Outcome> {
        trace.iter().map(|i| self.apply(*i)).collect()
    }

    fn resolve(&mut self, intent: Intent) -> Outcome {
        match intent {
            Intent::Idle => Outcome::Idled,

            Intent::Move { delta } => {
                let distance = delta.iter().map(|d| d.unsigned_abs() as u64).sum::<u64>();
                let cost = distance * MOVE_COST_MG;
                if cost > self.energy_mg {
                    return Outcome::Rejected(Rejection::InsufficientMass);
                }
                self.energy_mg -= cost;
                for (axis, step) in self.position.iter_mut().zip(delta) {
                    *axis += step;
                }
                Outcome::Moved
            }

            Intent::Metabolize { morsel, parent, offset, yaw } => {
                let Some(index) = self.morsels.iter().position(|m| m.id == morsel) else {
                    return Outcome::Rejected(Rejection::NoSuchMorsel(morsel));
                };
                if self.body.part(parent).is_none() {
                    return Outcome::Rejected(Rejection::NoSuchParent(parent));
                }
                if !self.within_reach(self.morsels[index].position) {
                    return Outcome::Rejected(Rejection::OutOfReach);
                }

                let eaten = self.morsels.remove(index);
                let provenance = Provenance {
                    origin: Origin::Incorporated {
                        from_species: eaten.species,
                        from_part: PartId(0),
                    },
                    epoch: self.epoch,
                };
                match self.body.attach(
                    eaten.volume,
                    eaten.mass_mg,
                    eaten.half_extent,
                    Attachment { parent, offset, yaw },
                    provenance,
                ) {
                    Ok(part) => {
                        // Half the mass becomes usable budget, the rest becomes body.
                        self.energy_mg += eaten.mass_mg / 2;
                        Outcome::Incorporated { part }
                    }
                    Err(_) => {
                        self.morsels.insert(index, eaten);
                        Outcome::Rejected(Rejection::NoSuchParent(parent))
                    }
                }
            }

            Intent::Deposit { mass_mg } => {
                if mass_mg == 0 || mass_mg > self.energy_mg {
                    return Outcome::Rejected(Rejection::InsufficientMass);
                }
                self.energy_mg -= mass_mg;
                let id = MorselId(self.next_morsel);
                self.next_morsel += 1;
                self.morsels.push(Morsel {
                    id,
                    species: self.body.species,
                    volume: VolumeRef::from_tag(64),
                    mass_mg,
                    half_extent: [1, 1, 1],
                    position: self.position,
                });
                Outcome::Deposited { morsel: id }
            }
        }
    }

    fn within_reach(&self, target: [i32; 3]) -> bool {
        (0..3).all(|axis| (target[axis] - self.position[axis]).abs() <= REACH)
    }

    /// The body's collision extent in body space.
    pub fn collision(&self) -> Aabb {
        self.body.aabb()
    }

    pub fn total_mass_mg(&self) -> u64 {
        self.body.total_mass_mg()
    }

    /// Ends the current epoch. Bodies change between epochs, not during them.
    pub fn end_epoch(&mut self) {
        self.epoch += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near_morsel(world: &World) -> MorselId {
        world
            .morsels
            .iter()
            .find(|m| world.within_reach(m.position))
            .expect("fixture places at least one morsel in reach")
            .id
    }

    #[test]
    fn same_seed_builds_the_same_world() {
        let a = World::new(1234, 12);
        let b = World::new(1234, 12);
        assert_eq!(a, b);
    }

    #[test]
    fn different_seeds_build_different_worlds() {
        let a = World::new(1, 12);
        let b = World::new(2, 12);
        assert_ne!(a.morsels, b.morsels);
    }

    #[test]
    fn metabolize_grows_mass_and_collision() {
        let mut world = World::new(99, 24);
        let target = near_morsel(&world);
        let mass_before = world.total_mass_mg();
        let box_before = world.collision();

        let outcome = world.apply(Intent::Metabolize {
            morsel: target,
            parent: world.body.root,
            offset: [5, 0, 0],
            yaw: Yaw::Zero,
        });

        assert!(matches!(outcome, Outcome::Incorporated { .. }));
        assert!(world.total_mass_mg() > mass_before);
        assert!(world.collision().extent()[0] > box_before.extent()[0]);
    }

    #[test]
    fn metabolize_records_where_the_part_came_from() {
        let mut world = World::new(7, 24);
        let target = near_morsel(&world);
        let eaten_species = world
            .morsels
            .iter()
            .find(|m| m.id == target)
            .map(|m| m.species)
            .unwrap();

        let Outcome::Incorporated { part } = world.apply(Intent::Metabolize {
            morsel: target,
            parent: world.body.root,
            offset: [4, 0, 0],
            yaw: Yaw::Zero,
        }) else {
            panic!("expected incorporation");
        };

        let provenance = &world.body.part(part).unwrap().provenance;
        assert_eq!(
            provenance.origin,
            Origin::Incorporated { from_species: eaten_species, from_part: PartId(0) }
        );
    }

    #[test]
    fn out_of_reach_morsels_are_refused() {
        let mut world = World::new(5, 4);
        world.morsels.push(Morsel {
            id: MorselId(900),
            species: SpeciesId(3),
            volume: VolumeRef::from_tag(2),
            mass_mg: 100,
            half_extent: [1, 1, 1],
            position: [500, 0, 0],
        });
        let outcome = world.apply(Intent::Metabolize {
            morsel: MorselId(900),
            parent: world.body.root,
            offset: [1, 0, 0],
            yaw: Yaw::Zero,
        });
        assert_eq!(outcome, Outcome::Rejected(Rejection::OutOfReach));
    }

    #[test]
    fn rejected_intents_still_advance_the_tick() {
        let mut world = World::new(11, 2);
        let before = world.tick;
        let outcome = world.apply(Intent::Metabolize {
            morsel: MorselId(4242),
            parent: world.body.root,
            offset: [0, 0, 0],
            yaw: Yaw::Zero,
        });
        assert_eq!(outcome, Outcome::Rejected(Rejection::NoSuchMorsel(MorselId(4242))));
        assert_eq!(world.tick, before + 1);
    }

    #[test]
    fn movement_spends_the_budget() {
        let mut world = World::new(3, 2);
        let before = world.energy_mg;
        world.apply(Intent::Move { delta: [3, 0, -2] });
        assert_eq!(world.energy_mg, before - 5);
    }

    #[test]
    fn deposit_returns_matter_to_the_enclosure() {
        let mut world = World::new(3, 2);
        let count = world.morsels.len();
        let outcome = world.apply(Intent::Deposit { mass_mg: 200 });
        assert!(matches!(outcome, Outcome::Deposited { .. }));
        assert_eq!(world.morsels.len(), count + 1);
    }
}
