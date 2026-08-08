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

/// Body recipes draw from their own stream too, per lineage.
pub const RECIPE_SALT: u64 = 0x5245_4349_5045_0001;

/// Individual developmental variation has its own worldgen stream.
///
/// It must not consume the ecology stream: adding a body detail must not move
/// every organism or change the timing of every later birth.
pub const DEVELOPMENT_SALT: u64 = 0x4445_5645_4C4F_5001;

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
    Explicit {
        parent: PartId,
        offset: [i32; 3],
        yaw: Yaw,
    },
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
    /// Carve a pocket of air around a nearby point. Recorded like every
    /// mutation, so a burrow is part of the world's replayable history.
    /// The energetics of digging await the metabolize-earth ruling; for
    /// now legality is embodiment plus reach.
    Carve { at: [i32; 3], radius: i32 },
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
    AboveTheFrontier {
        frontier: i32,
        target: i32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    Moved,
    /// Ground removed around a point.
    Carved { at: [i32; 3], removed: u32 },
    /// A meal became energy and nothing else.
    Burned {
        organism: OrganismId,
        energy_mg: u64,
    },
    Incorporated {
        part: PartId,
    },
    /// A bilateral plan grew a mirrored pair from one meal, splitting its mass.
    IncorporatedPair {
        part: PartId,
        mirror: PartId,
    },
    Deposited {
        organism: OrganismId,
    },
    /// Control moved to another critter.
    Inhabited {
        organism: OrganismId,
    },
    /// A line split, and was named.
    Speciated {
        species: SpeciesId,
        from: SpeciesId,
        founder: OrganismId,
    },
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
    /// Part vocabulary admitted by this world for developmental realization.
    /// Recipes travel; this palette is the local interpretation they grow in.
    #[serde(default)]
    development_palette: crate::development::PartPalette,
    /// The enclosure, divided. Derived from the seed and fixed for a world's
    /// life: regions do not move, or nothing could be said to have happened in
    /// one.
    #[serde(default)]
    places: Places,
    /// The ground: brick truth raised from the same seed's landscape.
    /// Serialized whole, so carves live inside the replay hash (G1).
    #[serde(default)]
    ground: crate::places::Ground,
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
        Outcome::Burned { energy_mg, .. } => Some(Event::Burned {
            organism: actor?,
            energy_mg,
        }),
        Outcome::Incorporated { part } | Outcome::IncorporatedPair { part, .. } => {
            Some(Event::Grew {
                organism: actor?,
                part,
            })
        }
        Outcome::Inhabited { organism } => Some(Event::Inhabited { organism }),
        // Carving air did not happen to anyone; only removed matter is
        // biographical.
        Outcome::Carved { at, removed } if removed > 0 => Some(Event::Carved {
            organism: actor?,
            at,
            removed,
        }),
        Outcome::Carved { .. } => None,
        Outcome::Speciated {
            species,
            from,
            founder,
        } => Some(Event::Speciated {
            species,
            from,
            founder,
        }),
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
        // Where the act's own consequences begin. Resolving can record
        // events of its own (learning a word from a meal), and those follow
        // from the act rather than preceding it, so the act is inserted at
        // this boundary rather than pushed after them.
        let boundary = self.pending.len();
        let outcome = self.resolve(intent);

        // Recorded before the ecology steps, because that is the order these
        // happened in and a history that reversed them would let a creature's
        // death precede its last act. It also means the actor is still whoever
        // acted, even if this tick kills them.
        if let Some(event) = event_for(&outcome, actor) {
            self.pending.insert(boundary, event);
        }

        // The enclosure lives whether or not the player acted. This is what
        // separates an ecology from a field of pickups: things grow, breed,
        // starve, and rot on their own schedule.
        let focus = self.position();
        self.last_tally = crate::organism::ecology::step_with_places(
            &mut self.organisms,
            &mut self.next_organism,
            &mut self.rng,
            &mut self.pending,
            &self.lineages,
            self.development_palette,
            &self.places,
            focus,
        );

        // What you reach, you keep. The frontier rises with the body you are
        // holding and never falls, so a lineage dying out costs you that body
        // rather than everything it earned.
        if let Some(held) = self.controlled().map(|o| self.intricacy(o)) {
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
                self.ranges
                    .entry(organism.species)
                    .or_default()
                    .insert(place);
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
                self.record
                    .note(reading.feat, reading.scale, reading.value, reading.species);
        }
        self.epoch += 1;
        readings
    }
}

#[cfg(test)]
mod tests;
