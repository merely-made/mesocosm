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
mod records;

pub use genesis::Founding;

/// How far the enclosure reaches from its middle, in voxels.
///
/// **Sixty-four since S1** (scale plan, 2026-08-29): a 129-voxel span, 15.3x
/// the floor area of the 33-voxel world that shipped before it. Mark ruled
/// scale a feature in its own right, and the same day read the captures and
/// asked why the creatures looked so big — one number answers both. A palette
/// limb is 9 voxels long, which was **27%** of the old world's width and is
/// **7%** of this one's; the fix is the world rather than the bodies, because
/// `part_ceiling_mg` prices voxel volume and shrinking half-extents would drag
/// the economy TD6 and TD7 just tuned.
///
/// Terrain is O(area), never O(volume) — [`crate::places::SURFACE_BAND`] caps
/// height — so this costs bricks and soil columns quadratically and nothing
/// cubically.
pub const ENCLOSURE: i32 = 64;

/// Non-played founders, scaled to the enclosure's floor area.
///
/// **Density is what stays fixed.** The 33-voxel world founded 60 over its
/// 1,089 columns; a 129-voxel world founding the same 60 would be that
/// terrarium fifteen times emptier — big instead of diffuse is the failure the
/// scale plan names, and it names it at every rung. Derived rather than typed,
/// so widening the enclosure again carries the cohort with it. At
/// `ENCLOSURE = 64` this is 916, and the played critter makes 917.
pub const FOUNDERS: u32 = {
    let side = (2 * ENCLOSURE + 1) as u32;
    let reference_side: u32 = 33;
    side * side * 60 / (reference_side * reference_side)
};

/// Regions to a side. Nine is coarser than a crowding cell on purpose: a place
/// is somewhere you can be, not a bucket for counting neighbours.
///
/// **Deliberately unchanged by S1.** Growing it reorders every RNG draw and is
/// S3's whole subject (a spatial index, a distance-capped far tier, cohorts as
/// an execution path); S1 is constants and scaling only. The consequence is
/// measured and reported in the scale plan's S1 entry: nine regions over a
/// 129-voxel span makes each region 43 voxels across, so the near/far line —
/// tuned as `demote_hops = 2` against a diameter-2 graph — now falls tens of
/// voxels away instead of ten.
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

/// How long a hand may be still before its critter goes back to its instincts,
/// in ticks.
///
/// Three seconds at the canonical ten ticks a second — long enough that a
/// pause to look at something is not read as walking away, short enough that
/// walking away is answered while you are still watching. **The idle terrarium
/// is the feature**: an ant farm nobody is touching is the way its dynamics
/// are seen, so the resting state of a controlled critter is its own drives,
/// not a statue. (TD4, ruled 2026-08-29.)
pub const INSTINCT_IDLE_TICKS: u32 = 30;

/// Ticks of upkeep left in the budget below which a meal burns instead of
/// building — **for every organism in the enclosure**, not just the played one
/// (TD5, ruled 2026-08-29; the ecology's `earn` reads it too).
///
/// A hundred: ten seconds of standing still at the canonical tempo, and about
/// a third of a 1,000 mg starter's 333-tick budget. **Wide on purpose.** The
/// ecology's own hunger horizon is eight ticks (`rates::HUNGRY_UPKEEP_TICKS`),
/// which is the point a body starts eating itself; routing a meal there would
/// mean every meal grew you until the tick before you died, and the burn half
/// of the verb would never be seen. This is instead the width of a state you
/// can notice, play out of, and be caught by.
pub const STARVED_UPKEEP_TICKS: u64 = 100;

/// Where a meal goes.
///
/// **This is the game's central question, and before it existed there was no
/// question.** Eating used to grant a part *and* half the mass as energy, so
/// the most important verb asked the player nothing. Splitting the destination
/// is what makes every meal ask: live now, or grow later?
///
/// **The question is no longer put to the player's fingers.** Mark rejected
/// the hotkey pair (2026-08-28: "not a workable ui") and ruled the answer
/// diegetic on 2026-08-29: a starved body burns its meal, a provisioned one
/// builds with it, and the state that decides is already on the vitals panel.
/// So this survives as what the body *concluded*, resolved inside
/// [`World::apply`] — never as something an intent carries, which is why
/// replays cannot disagree about it.
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
    /// Steer the critter toward a local voxel offset.
    ///
    /// One intent resolves to one legal near-tier step over the ground. The
    /// requested offset supplies the heading and vertical preference; it is
    /// never a licence to cross solid voxels or teleport across a slope.
    Move { delta: [i32; 3] },
    /// Eat something. **The one verb.**
    ///
    /// Metabolize means routing matter through the organism rather than
    /// swallowing it. What the meal *becomes* is not carried here: the body
    /// decides, burning when its budget is starved and building when it is
    /// provisioned (see [`Route`] and [`STARVED_UPKEEP_TICKS`]). What is
    /// carried is the other question entirely — *where a kept part goes* —
    /// which is a growth policy rather than a destination, and stays with the
    /// intent because an editor has to be able to say it.
    Metabolize {
        organism: OrganismId,
        placement: Placement,
    },
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
    /// The body plan found nowhere for a part of this shape to go, or the
    /// resulting live body would not fit its current Ground stance. Refusing
    /// keeps both body topology and terrain occupancy honest.
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
    Carved {
        at: [i32; 3],
        removed: u32,
    },
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
    /// Consecutive [`Intent::Idle`] applications. Any other intent resets it.
    ///
    /// **World state, and hashed like the rest of it.** It is a pure function
    /// of the trace — count the idles at the end — so putting it here costs a
    /// replay nothing and buys the rule that reads it (`INSTINCT_IDLE_TICKS`)
    /// the one thing a host-side timer could never have: the same answer in
    /// every host, at every frame rate, on every replay. A wall clock in the
    /// host would have made the ecology's behaviour depend on how fast the
    /// machine drew.
    #[serde(default)]
    idle_run: u32,
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
    /// The enclosure's matter, per voxel column.
    ///
    /// **The closed cycle's other half** (TD6). Producers draw out of it,
    /// bodies return to it, and the player's deposit enriches it, so the sum
    /// of soil, living substance, carrion and reserves is a constant of the
    /// run. World state like everything else here: serialized, hashed, and a
    /// pure function of the seed and the trace.
    #[serde(default)]
    soil: crate::places::Soil,
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
    pending: Vec<crate::flow::RecordedEvent>,
    /// What moved on the most recent tick, waiting to be reduced.
    ///
    /// **Beside the world rather than in it** (PE0). Dense per-tick flow in a
    /// snapshot is exactly what the plan's stop rules forbid, so this is
    /// `serde(skip)` and transparent to equality — the `drain_ground_dirty`
    /// arrangement, for the same reason: a host that drains readings every frame
    /// and a headless replay that never drains still hash identically.
    ///
    /// Opened at the top of every tick rather than accumulating like `pending`,
    /// because a stream this dense must be bounded whether or not anyone is
    /// listening. `pending` keeps its old contract: sparse, and a caller's to
    /// drain.
    #[serde(skip)]
    flows: crate::flow::Ledger,
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
        // Counted before the act, so a hand that is acting right now is
        // already holding at zero when the ecology below reads it.
        self.idle_run = if matches!(intent, Intent::Idle) {
            self.idle_run.saturating_add(1)
        } else {
            0
        };
        // One tick's flows, and only one. Opened before anything can move
        // matter, so the act and the ecology write into the same stamped
        // stream.
        self.flows.open(self.tick);
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
        if let Some(event) = records::event_for(&outcome, actor) {
            let place = self.acted_at(actor);
            self.pending.insert(
                boundary,
                crate::flow::Envelope::new(self.tick, place, event),
            );
        }

        // The enclosure lives whether or not the player acted. This is what
        // separates an ecology from a field of pickups: things grow, breed,
        // starve, and rot on their own schedule.
        let focus = self.position();
        let held = self.held();
        let tick = self.tick;
        let mut records =
            crate::flow::Records::new(tick, Some(&self.places), &mut self.pending, &mut self.flows);
        self.last_tally = crate::organism::ecology::step_with_ground(
            &mut self.organisms,
            &mut self.next_organism,
            &mut self.rng,
            &mut records,
            &self.lineages,
            self.development_palette,
            &mut self.soil,
            &self.places,
            &self.ground,
            focus,
            held,
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
mod behavior_tests;
#[cfg(test)]
mod tests;
