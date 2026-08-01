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
use crate::organism::{Kingdom, Organism, OrganismId, Signal, Stage};
use crate::rng::Rng;

/// How an incorporated part finds its site.
///
/// A **policy**, not a destination. Placing a part explicitly is a different
/// way of growing, not a different thing to do with a meal, and folding it in
/// beside `Burn` conflated the two questions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Placement {
    /// The body plan decides. **The default**: growth is automatic and
    /// symmetric, and the player shapes the plan rather than the placement.
    Planned,
    /// An explicit site. The editor path: total control is possible, but it is
    /// never the resting state.
    Explicit { parent: PartId, offset: [i32; 3], yaw: Yaw },
}

/// Where a meal goes.
///
/// **This is the game's central question, and before it existed there was no
/// question.** Eating used to grant a part *and* half the mass as energy, so
/// the most important verb asked the player nothing. Splitting the destination
/// is what makes every meal ask: live now, or grow later?
///
/// Later destinations arrive when the systems that receive them do:
/// provisioning reproduction, depositing or building a niche, and cultivating
/// something outside the skin. Each is a different answer to *where does this
/// capability live: in me, in a relationship, or in the world?*
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Route {
    /// Burn it now. Full mass becomes usable energy and no part is kept.
    Burn,
    /// Commit it to growth. Yields **no** immediate energy, which is the whole
    /// tradeoff.
    Incorporate { placement: Placement },
}

/// What a host may ask the world to do. Hosts send intents; they never mutate
/// world state directly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Intent {
    /// Move the critter by a voxel delta.
    Move { delta: [i32; 3] },
    /// Eat something, and decide what it becomes. **The one verb.**
    ///
    /// Metabolize means routing matter through the organism rather than
    /// swallowing it: the meal is the same, the destination is the choice.
    Metabolize { organism: OrganismId, route: Route },
    /// Return mass to the enclosure as carrion.
    Deposit { mass_mg: u64 },
    /// Advance one tick without acting.
    Idle,
}

/// Why an intent could not be applied. Rejections are part of the recorded
/// outcome, so a replay that rejects the same intents is still identical.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rejection {
    NoSuchOrganism(OrganismId),
    NoSuchParent(PartId),
    OutOfReach,
    InsufficientMass,
    /// The body plan found nowhere for a part of this shape to go. Refusing is
    /// correct: forcing it would overlap existing parts, and a plan that
    /// cannot place something is telling you to change the plan.
    NoRoom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    Moved,
    /// A meal became energy and nothing else.
    Burned { organism: OrganismId, energy_mg: u64 },
    Incorporated { part: PartId },
    /// A bilateral plan grew a mirrored pair from one meal, splitting its mass.
    IncorporatedPair { part: PartId, mirror: PartId },
    Deposited { organism: OrganismId },
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
    pub organisms: Vec<Organism>,
    next_organism: u32,
    last_tally: crate::organism::Tally,
    /// Metabolic budget in milligrams. Spent by moving, gained by eating.
    pub energy_mg: u64,
}

/// The half-extent of an organism, by its volume tag.
///
/// Shapes vary so that [`crate::plan::classify`] finds real roles: a world of
/// identical cubes can only ever grow mass. A host's volumes must match, since
/// this is what placement and physics read.
pub fn organism_extent(tag: u8) -> [i32; 3] {
    match tag % 4 {
        0 => [3, 1, 1],
        1 => [1, 3, 1],
        2 => [3, 1, 3],
        _ => [2, 2, 2],
    }
}

/// How far the critter can reach to eat, in voxel units.
const REACH: i32 = 8;

/// Energy spent per unit of movement, in milligrams.
const MOVE_COST_MG: u64 = 1;

impl World {
    /// Builds the standard fixture: one critter and a deterministic scatter of
    /// organisms drawn from the seeded stream.
    pub fn new(seed: u64, organism_count: u32) -> Self {
        let mut rng = Rng::from_seed(seed);
        let body = BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 1_000, [2, 2, 2]);

        let mut organisms = Vec::with_capacity(organism_count as usize);
        for index in 0..organism_count {
            // Draws happen in a fixed order, so the scatter is reproducible.
            let x = rng.range_i32(-16, 16);
            let y = rng.range_i32(-2, 2);
            let z = rng.range_i32(-16, 16);
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
            organisms.push(Organism {
                id: OrganismId(index),
                species,
                kingdom,
                volume: VolumeRef::from_tag(16 + (index % 8) as u8),
                mass_mg: mass,
                half_extent: organism_extent(16 + (index % 8) as u8),
                position: [x, y, z],
                stage: Stage::Juvenile,
                age,
                since_offspring: rng.below(120) as u32,
                signal,
                venom_mg,
                guise,
            });
        }

        Self {
            tick: 0,
            epoch: 0,
            rng,
            position: [0, 0, 0],
            body,
            organisms,
            next_organism: organism_count,
            last_tally: crate::organism::Tally::default(),
            energy_mg: 1_000,
        }
    }

    /// Applies one intent and advances the tick. This is the only way world
    /// state changes.
    pub fn apply(&mut self, intent: Intent) -> Outcome {
        let outcome = self.resolve(intent);
        // The enclosure lives whether or not the player acted. This is what
        // separates an ecology from a field of pickups: things grow, breed,
        // starve, and rot on their own schedule.
        self.last_tally = crate::organism::step(
            &mut self.organisms,
            &mut self.next_organism,
            &mut self.rng,
        );
        self.tick += 1;
        outcome
    }

    /// What the most recent tick did to the enclosure.
    pub fn last_tally(&self) -> crate::organism::Tally {
        self.last_tally
    }

    /// Living organisms, in id order.
    pub fn living(&self) -> impl Iterator<Item = &Organism> {
        self.organisms.iter().filter(|o| o.is_alive())
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

            Intent::Metabolize { organism, route } => self.metabolize(organism, route),

            Intent::Deposit { mass_mg } => {
                if mass_mg == 0 || mass_mg > self.energy_mg {
                    return Outcome::Rejected(Rejection::InsufficientMass);
                }
                self.energy_mg -= mass_mg;
                let id = OrganismId(self.next_organism);
                self.next_organism += 1;
                self.organisms.push(Organism {
                    id,
                    species: self.body.species,
                    // Deposited matter is dead matter: it feeds decomposers
                    // and returns to the world rather than growing.
                    kingdom: Kingdom::Decomposer,
                    volume: VolumeRef::from_tag(64),
                    mass_mg,
                    half_extent: [1, 1, 1],
                    position: self.position,
                    stage: Stage::Carrion,
                    age: 0,
                    since_offspring: 0,
                    signal: Signal::Plain,
                    venom_mg: 0,
                    guise: Kingdom::Decomposer,
                });
                Outcome::Deposited { organism: id }
            }
        }
    }

    /// Eats a organism and grows it where the plan says.
    /// Eat something and route it. The one verb, in one place, so every meal
    /// pays the same costs whatever it becomes.
    ///
    /// **One transaction.** Everything that can refuse is checked before the
    /// organism leaves the roster, a failed attachment puts it back, and the
    /// energy ledger moves only once the meal has actually landed. An earlier
    /// cut consumed the meal and charged its venom before the attachment was
    /// known to succeed.
    fn metabolize(&mut self, organism: OrganismId, route: Route) -> Outcome {
        let Some(index) = self.organisms.iter().position(|m| m.id == organism) else {
            return Outcome::Rejected(Rejection::NoSuchOrganism(organism));
        };
        if let Route::Incorporate { placement: Placement::Explicit { parent, .. } } = route
            && self.body.part(parent).is_none()
        {
            return Outcome::Rejected(Rejection::NoSuchParent(parent));
        }
        if !self.within_reach(self.organisms[index].position) {
            return Outcome::Rejected(Rejection::OutOfReach);
        }

        // Resolve planned placement before the meal is consumed, so a body
        // with nowhere to put a part refuses without eating anything.
        let growth = match route {
            Route::Incorporate { placement: Placement::Planned } => {
                match crate::growth::resolve(&self.body, self.organisms[index].half_extent) {
                    Some(growth) => Some(growth),
                    None => return Outcome::Rejected(Rejection::NoRoom),
                }
            }
            _ => None,
        };

        let eaten = self.organisms.remove(index);
        let (outcome, gain_mg) = self.land(&eaten, route, growth);

        if matches!(outcome, Outcome::Rejected(_)) {
            // Nothing landed, so nothing was eaten and nothing is owed.
            self.organisms.insert(index, eaten);
            return outcome;
        }

        // **Gains before costs.** Subtracting venom first let a nearly starved
        // critter lose part of a toxin to the zero floor and then collect the
        // full meal, so being close to death made a venomous thing *safer*.
        //
        // The floor itself remains: energy is unsigned, so venom beyond what a
        // critter has is forgiven rather than owed. A debt or damage model is a
        // later decision, recorded in the phenotype plan.
        self.energy_mg = (self.energy_mg + gain_mg).saturating_sub(eaten.venom_mg);
        outcome
    }

    /// Attempts the routed outcome, returning it and the energy it yields.
    /// Mutates the body but never the roster or the ledger.
    fn land(
        &mut self,
        eaten: &Organism,
        route: Route,
        growth: Option<crate::growth::Growth>,
    ) -> (Outcome, u64) {
        match route {
            Route::Burn => (
                Outcome::Burned { organism: eaten.id, energy_mg: eaten.mass_mg },
                eaten.mass_mg,
            ),
            Route::Incorporate { placement: Placement::Explicit { parent, offset, yaw } } => {
                let provenance = self.taken_from(eaten);
                let attached = self.body.attach(
                    eaten.volume,
                    eaten.mass_mg,
                    eaten.half_extent,
                    Attachment { parent, offset, yaw },
                    provenance,
                );
                match attached {
                    Ok(part) => (Outcome::Incorporated { part }, 0),
                    Err(_) => (Outcome::Rejected(Rejection::NoSuchParent(parent)), 0),
                }
            }
            Route::Incorporate { placement: Placement::Planned } => {
                let growth = growth.expect("resolved above for this route");
                let provenance = self.taken_from(eaten);

                // A mirrored pair splits the mass it came from, so the budget
                // stays honest however symmetric the body becomes.
                let parts = if growth.mirror.is_some() { 2 } else { 1 };
                let each = eaten.mass_mg / parts;

                let Ok(part) = self.body.attach(
                    eaten.volume,
                    each,
                    eaten.half_extent,
                    crate::growth::attachment(&growth),
                    provenance.clone(),
                ) else {
                    return (Outcome::Rejected(Rejection::NoRoom), 0);
                };

                // No energy. Growing is the slow answer, and a meal cannot be
                // both meals.
                let outcome = match crate::growth::mirror_attachment(&growth) {
                    Some(mirrored) => match self.body.attach(
                        eaten.volume,
                        each,
                        eaten.half_extent,
                        mirrored,
                        provenance,
                    ) {
                        Ok(mirror) => Outcome::IncorporatedPair { part, mirror },
                        Err(_) => Outcome::Incorporated { part },
                    },
                    None => Outcome::Incorporated { part },
                };
                (outcome, 0)
            }
        }
    }

    /// Provenance for a part taken off `eaten`.
    fn taken_from(&self, eaten: &Organism) -> Provenance {
        Provenance {
            origin: Origin::Incorporated { from_species: eaten.species, from_part: PartId(0) },
            epoch: self.epoch,
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

    fn near_organism(world: &World) -> OrganismId {
        world
            .organisms
            .iter()
            .find(|m| world.within_reach(m.position))
            .expect("fixture places at least one organism in reach")
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
        assert_ne!(a.organisms, b.organisms);
    }

    #[test]
    fn metabolize_grows_mass_and_collision() {
        let mut world = World::new(99, 24);
        let target = near_organism(&world);
        let mass_before = world.total_mass_mg();
        let box_before = world.collision();

        let outcome = world.apply(Intent::Metabolize { organism: target, route: Route::Incorporate { placement: Placement::Explicit { parent: world.body.root, offset: [5, 0, 0], yaw: Yaw::Zero } } });

        assert!(matches!(outcome, Outcome::Incorporated { .. }));
        assert!(world.total_mass_mg() > mass_before);
        assert!(world.collision().extent()[0] > box_before.extent()[0]);
    }

    #[test]
    fn metabolize_records_where_the_part_came_from() {
        let mut world = World::new(7, 24);
        let target = near_organism(&world);
        let eaten_species = world
            .organisms
            .iter()
            .find(|m| m.id == target)
            .map(|m| m.species)
            .unwrap();

        let Outcome::Incorporated { part } = world.apply(Intent::Metabolize { organism: target, route: Route::Incorporate { placement: Placement::Explicit { parent: world.body.root, offset: [4, 0, 0], yaw: Yaw::Zero } } }) else {
            panic!("expected incorporation");
        };

        let provenance = &world.body.part(part).unwrap().provenance;
        assert_eq!(
            provenance.origin,
            Origin::Incorporated { from_species: eaten_species, from_part: PartId(0) }
        );
    }

    #[test]
    fn out_of_reach_organisms_are_refused() {
        let mut world = World::new(5, 4);
        world.organisms.push(Organism {
            id: OrganismId(900),
            species: SpeciesId(3),
            kingdom: Kingdom::Producer,
            volume: VolumeRef::from_tag(2),
            mass_mg: 100,
            half_extent: [1, 1, 1],
            position: [500, 0, 0],
            stage: Stage::Mature,
            age: 0,
            since_offspring: 0,
            signal: Signal::Plain,
            venom_mg: 0,
            guise: Kingdom::Producer,
        });
        let outcome = world.apply(Intent::Metabolize { organism: OrganismId(900), route: Route::Incorporate { placement: Placement::Explicit { parent: world.body.root, offset: [1, 0, 0], yaw: Yaw::Zero } } });
        assert_eq!(outcome, Outcome::Rejected(Rejection::OutOfReach));
    }

    #[test]
    fn rejected_intents_still_advance_the_tick() {
        let mut world = World::new(11, 2);
        let before = world.tick;
        let outcome = world.apply(Intent::Metabolize { organism: OrganismId(4242), route: Route::Incorporate { placement: Placement::Explicit { parent: world.body.root, offset: [0, 0, 0], yaw: Yaw::Zero } } });
        assert_eq!(outcome, Outcome::Rejected(Rejection::NoSuchOrganism(OrganismId(4242))));
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
        let count = world.organisms.len();
        let outcome = world.apply(Intent::Deposit { mass_mg: 200 });
        assert!(matches!(outcome, Outcome::Deposited { .. }));
        assert_eq!(world.organisms.len(), count + 1);
    }
}
