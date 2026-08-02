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

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::body::{PartId, SpeciesId, Yaw};
use crate::organism::{Organism, OrganismId};
use crate::places::{PlaceId, Places};
use crate::process::Unmet;
use crate::record::WorldRecord;
use crate::rng::Rng;
use crate::score::Reading;

mod act;
mod genesis;
mod read;

/// How far the enclosure reaches from its middle, in voxels.
pub const ENCLOSURE: i32 = 16;

/// Regions to a side. Nine is coarser than a crowding cell on purpose: a place
/// is somewhere you can be, not a bucket for counting neighbours.
pub const PLACE_SIDE: u16 = 3;

/// Places draw from their own stream, derived from the world's seed.
///
/// Not from `World`'s, which the ecology spends every tick: dividing the
/// enclosure would shift every draw after it, so adding regions to a world would
/// silently rearrange the creatures in it.
pub const PLACE_SALT: u64 = 0x504C_4143_4553_0001;

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
///
/// `Clone` rather than `Copy`, because naming a lineage carries a name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Split the line you are in, and name it.
    ///
    /// **The act that makes a species.** Splitting is not a threshold anybody
    /// crosses; it is something a player does, and the name is the doing.
    /// Takes the creature you are holding and nothing else, so a new line
    /// begins with one founder and its former kin keep the old one.
    Speciate { name: String },
    /// Inhabit another critter.
    ///
    /// **A recorded intent, not a side door.** Lineage switching is gameplay,
    /// and ordered intents are the only way world state changes. A control
    /// change made outside this path would replay every fact about a run
    /// except who was living it.
    TakeControl { organism: OrganismId },
    /// Advance one tick without acting.
    Idle,
}

/// Why an intent could not be applied. Rejections are part of the recorded
/// outcome, so a replay that rejects the same intents is still identical.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rejection {
    /// Nobody may inhabit that, and this says why.
    Ineligible(Ineligible),
    /// A critter cannot eat itself.
    ///
    /// Only expressible since the played critter joined the organism
    /// vector: before that it was not a thing anyone could target.
    Itself,
    /// Nobody is being played, so there is nothing to act with. A world
    /// running with no one in it is a legitimate state.
    Disembodied,
    NoSuchOrganism(OrganismId),
    NoSuchParent(PartId),
    /// The played critter could not touch it, and this says why: no actuator
    /// at all, or one that does not extend far enough.
    OutOfReach(Unmet),
    InsufficientMass,
    /// The body plan found nowhere for a part of this shape to go. Refusing is
    /// correct: forcing it would overlap existing parts, and a plan that
    /// cannot place something is telling you to change the plan.
    NoRoom,
}

/// Why an organism cannot be inhabited.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ineligible {
    NoSuchOrganism,
    /// A carcass is not a critter you can play.
    NotAlive,
    /// More elaborate than anything you have earned.
    ///
    /// Stepping *down* into a newly viable niche is the point of switching;
    /// stepping across into an unearned peer is what this refuses.
    AboveTheFrontier { frontier: i32, target: i32 },
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
    /// Control moved to another critter.
    Inhabited { organism: OrganismId },
    /// A line split, and was named.
    Speciated { species: SpeciesId, from: SpeciesId, founder: OrganismId },
    Idled,
    Rejected(Rejection),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct World {
    pub tick: u64,
    pub epoch: u64,
    rng: Rng,
    /// Which organism the player is, if any.
    ///
    /// **Control is a pointer, not a shape.** Switching lineage moves this and
    /// reconstructs nothing; a played critter and an unplayed one serialize
    /// identically; and no rule can branch on who is being played, because
    /// there is no flag to branch on.
    ///
    /// `None` is a first-class state, not a stale pointer. A world does not
    /// require anyone to be in it, and the difference between *nobody is
    /// embodied* and *this id no longer names anything* is one a caller has to
    /// be able to see.
    controlled: Option<OrganismId>,
    /// Who the world stopped being able to play on the most recent tick.
    control_lost: Option<OrganismId>,
    /// Lineages the player has inhabited.
    ///
    /// What "unlocked" means for the complexity frontier. Ordered, so the
    /// frontier is deterministic.
    #[serde(default)]
    unlocked: std::collections::BTreeSet<SpeciesId>,
    /// The most elaborate thing the player has ever held.
    ///
    /// **A high-water mark, not a current reading.** An earlier cut computed
    /// this from living organisms of unlocked species, which meant a lineage
    /// dying out collapsed the frontier to zero and left the world permanently
    /// uninhabitable. That contradicts the ruling that disembodiment is a seam
    /// rather than a dead end: losing a body must not unearn what reaching it
    /// cost.
    #[serde(default)]
    frontier: i32,
    /// Which lineages exist, what they are called, and what they came from.
    #[serde(default)]
    lineages: crate::species::Lineages,
    /// The enclosure, divided. Derived from the seed and fixed for a world's
    /// life: regions do not move, or nothing could be said to have happened in
    /// one.
    #[serde(default)]
    places: Places,
    /// Everywhere each lineage has been.
    ///
    /// **A high-water set**, for the same reason the frontier is a high-water
    /// mark: where a creature has been is not readable from where it is, and a
    /// lineage that withdrew from half the enclosure still reached it. Union is
    /// commutative, associative, and idempotent, so two worlds' ranges join the
    /// way their records do, without a protocol.
    #[serde(default)]
    ranges: BTreeMap<SpeciesId, BTreeSet<PlaceId>>,
    /// What this world has seen anyone do.
    #[serde(default)]
    record: WorldRecord,
    /// Ordered by id, so iteration never depends on hashing.
    pub organisms: Vec<Organism>,
    next_organism: u32,
    last_tally: crate::organism::Tally,
    /// What happened on the most recent tick, waiting to be drained.
    ///
    /// **One tick, never more.** History is derivable from a seed and ordered
    /// intents, so keeping it in the world would grow every snapshot without
    /// bound and cost the cheap whole-state capture the wing's rollback
    /// thinking rests on. A caller drains this into a [`History`], which lives
    /// beside the world and persists in its own slot.
    /// Not skipped when empty: postcard is positional, so a field written
    /// conditionally cannot be read back. That trap already cost one decode
    /// failure here.
    #[serde(default)]
    pending: Vec<crate::history::Event>,
}

/// The event an outcome amounts to, if any.
///
/// Refusals produce nothing: a history records what happened, and a rejected
/// intent is a thing that did not.
fn event_for(outcome: &Outcome, actor: Option<OrganismId>) -> Option<crate::history::Event> {
    use crate::history::Event;
    match *outcome {
        // Burning names the meal, growing names the grower. Both need the
        // actor, because a history is keyed by who a thing happened to and an
        // event citing nobody would fork that creature's line.
        Outcome::Burned { energy_mg, .. } => {
            Some(Event::Burned { organism: actor?, energy_mg })
        }
        Outcome::Incorporated { part } | Outcome::IncorporatedPair { part, .. } => {
            Some(Event::Grew { organism: actor?, part })
        }
        Outcome::Inhabited { organism } => Some(Event::Inhabited { organism }),
        Outcome::Speciated { species, from, founder } => {
            Some(Event::Speciated { species, from, founder })
        }
        Outcome::Moved | Outcome::Deposited { .. } | Outcome::Idled | Outcome::Rejected(_) => None,
    }
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

impl World {
    /// Applies one intent and advances the tick. This is the only way world
    /// state changes.
    pub fn apply(&mut self, intent: Intent) -> Outcome {
        let actor = self.controlled;
        let outcome = self.resolve(intent);

        // Recorded before the ecology steps, because that is the order these
        // happened in and a history that reversed them would let a creature's
        // death precede its last act. It also means the actor is still whoever
        // acted, even if this tick kills them.
        if let Some(event) = event_for(&outcome, actor) {
            self.pending.push(event);
        }

        // The enclosure lives whether or not the player acted. This is what
        // separates an ecology from a field of pickups: things grow, breed,
        // starve, and rot on their own schedule.
        self.last_tally = crate::organism::step(
            &mut self.organisms,
            &mut self.next_organism,
            &mut self.rng,
            &mut self.pending,
        );

        // What you reach, you keep. The frontier rises with the body you are
        // holding and never falls, so a lineage dying out costs you that body
        // rather than everything it earned.
        if let Some(held) = self.controlled().map(Organism::complexity) {
            self.frontier = self.frontier.max(held);
        }

        // Control ends with the life it was attached to. Nothing exempts the
        // played critter from the ecology, so this is where a run can lose its
        // body without the world ending.
        self.control_lost = match self.controlled {
            Some(id) if !self.is_eligible(id) => {
                self.controlled = None;
                Some(id)
            }
            _ => None,
        };

        // Where each lineage has been, unioned in after the ecology has moved
        // everything. Reading it before the step would record last tick's
        // positions against this tick's roster.
        for organism in self.organisms.iter().filter(|o| o.is_alive()) {
            if let Some(place) = self.places.at(organism.position) {
                self.ranges.entry(organism.species).or_default().insert(place);
            }
        }

        self.tick += 1;
        outcome
    }

    /// Takes the events of the most recent tick, leaving the world empty of
    /// them. Recording them is a caller's business; the world only reports.
    pub fn drain_events(&mut self) -> Vec<crate::history::Event> {
        std::mem::take(&mut self.pending)
    }

    /// The events of the most recent tick, without taking them.
    pub fn events(&self) -> &[crate::history::Event] {
        &self.pending
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
        trace.iter().map(|i| self.apply(i.clone())).collect()
    }

    /// Ends the current epoch, and reckons what it came to.
    ///
    /// Bodies change between epochs, not during them, and this is also where a
    /// world finally writes down what its lineages did. It takes the past
    /// because history lives beside a world rather than inside it: a world can
    /// say what is, never what happened.
    ///
    /// Returns every reading, each carrying whether it took the record, which is
    /// what an epoch-boundary screen is made of.
    pub fn end_epoch(&mut self, history: &crate::history::History) -> Vec<Reading> {
        let mut readings = crate::score::readings(self, history);
        for reading in &mut readings {
            reading.took =
                self.record.note(reading.feat, reading.scale, reading.value, reading.species);
        }
        self.epoch += 1;
        readings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::{Origin, VolumeRef};
    use crate::organism::{Kingdom, Stage};

    /// Walks the critter to its nearest neighbour and returns it.
    ///
    /// Since reach became anatomy rather than a constant, a starting critter
    /// touches about three voxels, so a fixture has to travel like a player
    /// does instead of assuming a meal is adjacent.
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
        let target = near_organism(&mut world);
        let mass_before = world.total_mass_mg();
        let box_before = world.collision().unwrap();

        let outcome = world.apply(Intent::Metabolize { organism: target, route: Route::Incorporate { placement: Placement::Explicit { parent: world.body().unwrap().root, offset: [5, 0, 0], yaw: Yaw::Zero } } });

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

        let Outcome::Incorporated { part } = world.apply(Intent::Metabolize { organism: target, route: Route::Incorporate { placement: Placement::Explicit { parent: world.body().unwrap().root, offset: [4, 0, 0], yaw: Yaw::Zero } } }) else {
            panic!("expected incorporation");
        };

        let provenance = &world.body().unwrap().part(part).unwrap().provenance;
        assert_eq!(
            provenance.origin,
            Origin::Incorporated { from_species: eaten_species, from_part: PartId(0) }
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
        let outcome = world.apply(Intent::Metabolize { organism: OrganismId(900), route: Route::Incorporate { placement: Placement::Explicit { parent: world.body().unwrap().root, offset: [1, 0, 0], yaw: Yaw::Zero } } });
        // Five hundred voxels away, and the body says why rather than only
        // that it failed.
        assert!(matches!(
            outcome,
            Outcome::Rejected(Rejection::OutOfReach(crate::process::Unmet::NoProcess { .. }))
        ), "got {outcome:?}");
    }

    #[test]
    fn rejected_intents_still_advance_the_tick() {
        let mut world = World::new(11, 2);
        let before = world.tick;
        let outcome = world.apply(Intent::Metabolize { organism: OrganismId(4242), route: Route::Incorporate { placement: Placement::Explicit { parent: world.body().unwrap().root, offset: [0, 0, 0], yaw: Yaw::Zero } } });
        assert_eq!(outcome, Outcome::Rejected(Rejection::NoSuchOrganism(OrganismId(4242))));
        assert_eq!(world.tick, before + 1);
    }

    #[test]
    fn movement_spends_the_budget() {
        // Movement is five, and upkeep takes its share of the same budget in
        // the same tick. Before the ledgers were reconciled, upkeep came out
        // of a scalar the player's budget could not see.
        let mut world = World::new(3, 2);
        let before = world.energy_mg().unwrap();
        let upkeep = world.controlled().unwrap().upkeep_mg();

        world.apply(Intent::Move { delta: [3, 0, -2] });

        assert_eq!(world.energy_mg().unwrap(), before - 5 - upkeep);
    }

    #[test]
    fn a_bigger_body_costs_more_to_carry() {
        // The reconciliation, in one assertion: upkeep is a function of what
        // a critter is made of. Flat upkeep is why growing used to be free.
        let world = World::new(3, 2);
        let small = world.controlled().unwrap().upkeep_mg();

        let mut grown = world.clone();
        let me = grown.controlled_id().unwrap();
        {
            let organism = grown.organisms.iter_mut().find(|o| o.id == me).unwrap();
            organism.gain_mass(5_000);
        }

        assert!(
            grown.controlled().unwrap().upkeep_mg() > small,
            "a heavier body pays more rent: {} vs {}",
            grown.controlled().unwrap().upkeep_mg(),
            small
        );
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
