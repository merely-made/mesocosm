// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The vocabulary a host speaks: what it may ask, and what it is told back.
//!
//! Split out of `world.rs` at the 600-line ceiling on 2026-09-01, when PD2
//! added the developmental verb. Nothing here mutates anything — `world::act`
//! is still the only room behind the one door — so this file is exactly the
//! contract between a host and the world, in one place.

use serde::{Deserialize, Serialize};

use crate::body::SpeciesId;
use crate::body::{PartId, Yaw};
use crate::organism::OrganismId;
use crate::phenotype::{CellId, Refusal};
use crate::process::{ProcessRef, Unmet};

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
    /// Take **one organ** off a carcass. (PE2)
    ///
    /// A different act from [`Metabolize`], not a parameter on it. Eating a
    /// whole body is calories and the body routes them; this names an organ on
    /// something that has stopped being alive, settles exactly what that organ
    /// weighs, and writes down which part of which line it came off — the first
    /// time `Origin::Incorporated`'s `from_part` has been anything but the
    /// donor's root.
    ///
    /// It attaches one part and never a branch: the subtree under the organ
    /// stays on the corpse, because live subtree transfer is phenotype P3's
    /// gate and this proof deliberately stops short of it.
    ///
    /// [`Metabolize`]: Intent::Metabolize
    Consume { organism: OrganismId, part: PartId },
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
    /// Answer a checkpoint by continuing as you are.
    ///
    /// **The other half of [`TakeControl`], and the reason the pair is a
    /// choice.** At a reproduction checkpoint the two answers are *take the
    /// offspring* and *stay in the parent*; at a control loss they are *continue
    /// through a descendant* and *let the line go*. Staying is a decision, and a
    /// decision has to be sayable, or the record cannot tell a hand that chose
    /// to carry on from a hand that was not there.
    ///
    /// To the world it is [`Idle`] that admits to being a hand: nothing moves,
    /// but the idle run resets, because somebody answered. Everything a
    /// checkpoint *is* — when one opens, what it says, how long it holds — lives
    /// in the driver, since a bounded pause is a question about when to step and
    /// the world only ever knows what is. (PE1.)
    ///
    /// [`TakeControl`]: Intent::TakeControl
    /// [`Idle`]: Intent::Idle
    Resume,
    /// Carve a pocket of air around a nearby point. Recorded like every
    /// mutation, so a burrow is part of the world's replayable history.
    /// The energetics of digging await the metabolize-earth ruling; for
    /// now legality is embodiment plus reach.
    Carve { at: [i32; 3], radius: i32 },
    /// Rearrange one part's tissue: **PD2's editor operation**, and the only
    /// thing in the game that moves allocation.
    ///
    /// It carries the *complete* desired allocation for one part, because
    /// that is what [`BodyPhenotype::develop`] validates — a complete desired
    /// state is order-independent and a stale one is refusable, where a series
    /// of drags is neither. One part, so the milligram it costs is priced in
    /// that part's own tissue and a receipt can say where the cost came from.
    ///
    /// **A temporary authoring path, deleted at PD3.** The processdef plan
    /// permits exactly this for PD2 — "a native developmental fixture or an
    /// explicit editor operation" — and requires it to go when packs and the
    /// developmental bridge arrive. What survives is the validator underneath
    /// it, which is already shared with automatic arrangement.
    ///
    /// [`BodyPhenotype::develop`]: crate::phenotype::BodyPhenotype::develop
    Rearrange { part: PartId, sites: Vec<Allocate> },
}

/// One site an [`Intent::Rearrange`] wants to exist on the part it names.
///
/// The definition travels as a content address rather than a friendly name,
/// for PD1b's reason: a world that does not hold that exact definition must
/// refuse rather than substitute the nearest thing it does hold.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Allocate {
    pub process: ProcessRef,
    /// Existing cell ids, sorted and deduplicated. The validator says so.
    pub cells: Vec<CellId>,
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
    /// That body has no such part. (PE2)
    NoSuchPart(PartId),
    /// Its organs are not on offer, because it is still using them. Taking one
    /// would be live dismemberment, which phenotype P3 owns and PE2's bounded
    /// part-meal proof does not open.
    StillLiving(OrganismId),
    /// The part is severed or already taken, so there is nothing there to
    /// settle. A severed part's milligrams have already left the conservation
    /// account; eating one would create matter.
    NothingLeft(PartId),
    /// The played critter could not touch it, and this says why: no actuator
    /// at all, or one that does not extend far enough.
    OutOfReach(Unmet),
    InsufficientMass,
    /// The body plan found nowhere for a part of this shape to go, or the
    /// resulting live body would not fit its current Ground stance. Refusing
    /// keeps both body topology and terrain occupancy honest.
    NoRoom,
    /// The development would not validate, and this is the boundary that
    /// failed. (PD2)
    ///
    /// Carried whole rather than flattened into a handful of world-level
    /// words: PD1b made the refusal order part of the contract precisely so
    /// two callers submitting the same invalid candidate get the same answer,
    /// and re-encoding fifteen named boundaries as three would throw that away
    /// at the one door a player actually knocks on.
    Refused(Refusal),
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
    /// One organ came off a carcass and onto this body. (PE2)
    ///
    /// Says both ends of the provenance, because that is the whole point of
    /// the verb: which part it became here, and which part of which body it
    /// was. `mass_mg` is exactly what that part weighed — no split, no spill.
    Consumed {
        part: PartId,
        from: OrganismId,
        from_part: PartId,
        mass_mg: u64,
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
    /// A checkpoint was answered by carrying on unchanged.
    Resumed,
    /// A part's tissue was reallocated, and paid for. (PD2)
    ///
    /// `cost_mg` is what the development cost: the cells whose expression
    /// changed, priced at what a cell of that part's tissue is worth. It left
    /// the body's reserve and went into the ground under it, because nothing
    /// evaporates.
    Rearranged {
        part: PartId,
        cost_mg: u64,
        /// The phenotype revision this development created, so a receipt can
        /// point at the ordering rather than only at the fact.
        revision: u32,
    },
    Rejected(Rejection),
}
