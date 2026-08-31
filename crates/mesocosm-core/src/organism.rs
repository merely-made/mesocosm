// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The other things living here.
//!
//! These used to be `Morsel`s: inert matter with a mass, waiting to be
//! collected. That name was the design flaw written down — a morsel is *a
//! small piece of food*, which is what you call something that exists to be
//! eaten. Mark's diagnosis: "we're just kinda munchin'. A free meal, talk
//! about the opposite of a game."
//!
//! An organism runs the same loop the player runs. It grows, matures,
//! reproduces, ages, dies, and rots, whether or not anyone is watching. Eating
//! one is a decision with a cost and a moment, because the thing in front of
//! you is going somewhere on its own.

use serde::{Deserialize, Serialize};

use crate::body::{Attachment, BodyDocument, Provenance, Yaw};
use crate::places::{Tier, WalkerShape};
use crate::plan::{Role, classify};
use crate::process::{FeedingMode, Process};

mod behavior;
pub mod ecology;
mod kingdom;

pub use behavior::{
    FaunaDecisionTrace, FaunaDrive, FaunaDriveScores, FaunaPolicy, FaunaSenses, FaunaTraits,
};
pub use ecology::step;
use ecology::{OFFSPRING_COST, STARVATION_MG};
pub use kingdom::Kingdom;

use crate::body::{SpeciesId, VolumeRef};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct OrganismId(pub u32);

/// A recent direct observation. It is simulation state: losing sight changes
/// what an embodied creature does next, so it must replay with the organism
/// rather than live in a host-side perception cache.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LastSeen {
    pub target: OrganismId,
    pub position: [i32; 3],
    pub ticks_left: u8,
}

/// What an organism advertises about itself.
///
/// Signalling and counter-signalling: an advertisement is a claim, and a claim
/// can be false. This is what makes choosing a meal a decision rather than a
/// collection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Signal {
    /// Claims nothing. Ordinary, unremarkable, probably safe.
    Plain,
    /// Claims to be dangerous. Bright, loud, conspicuous.
    Warning,
}

/// Where an organism is in its life.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stage {
    /// Growing. Worth less now than it will be shortly.
    Juvenile,
    /// Grown, and able to reproduce.
    Mature,
    /// Dead, and not yet returned. Food for decomposers, poor food for others.
    Carrion,
    /// Fully returned to the world. Removed at the end of the tick.
    Spent,
}

impl Stage {
    pub fn is_alive(self) -> bool {
        matches!(self, Stage::Juvenile | Stage::Mature)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Organism {
    pub id: OrganismId,
    /// Which lineage this belongs to. Incorporation carries it forward, so a
    /// part you take always knows whose it was.
    pub species: SpeciesId,
    /// This organism's anatomy.
    ///
    /// **Every organism has one**, played or not. Before P1 only the critter
    /// the player inhabited had a body and everything else was a `VolumeRef`
    /// and a `half_extent`, which meant anatomy could never constrain an
    /// unplayed creature and prey had no parts to lose. The pair it replaces
    /// are now readings of the root part, so there is one place a shape is
    /// written down.
    ///
    /// Most organisms carry a single root part. That is a body, not a special
    /// case, and it grows by the same rules as any other.
    pub body: BodyDocument,
    /// Entropy that realized this individual from its lineage recipe.
    ///
    /// Stored because the body is causal state, not a renderer accident. A
    /// founder preview using this seed and the same declared inputs must grow
    /// the same body, while descendants derive distinct seeds from it.
    #[serde(default)]
    pub development_seed: u64,
    /// Mass at the life-history transition that founded this individual.
    ///
    /// Rates are allometric, but maturity and senescence must not move away
    /// from an organism as it grows. This is the individual's life-history
    /// reference mass, not a second live biomass account.
    #[serde(default)]
    pub life_history_mass_mg: u64,
    pub position: [i32; 3],
    /// Which simulation tier currently owns this body's next decision.
    ///
    /// It is state, not a cache: hysteresis means the same position can have
    /// different ownership depending on where the organism came from.
    #[serde(default)]
    pub tier: Tier,
    /// The last directly perceived living target, while its short pursuit
    /// window remains. This is generic perception, not a Hunter FSM state.
    #[serde(default)]
    pub last_seen: Option<LastSeen>,
    /// The inherited, quantized proposal policy used only by near fauna.
    /// Movement legality and ecology resolution remain outside it.
    #[serde(default)]
    pub fauna_policy: FaunaPolicy,
    /// Inspectable evidence for the policy's most recent proposal.
    #[serde(default)]
    pub last_fauna_decision: Option<FaunaDecisionTrace>,
    /// What this organism can spend: its **budget**.
    ///
    /// Distinct from what it weighs, which is its body's business. Before P1
    /// only the played critter had a budget, so nothing else could run out of
    /// one.
    pub energy_mg: u64,
    pub stage: Stage,
    pub age: u32,
    /// Ticks since this organism last reproduced.
    pub since_offspring: u32,
    /// What this organism *advertises*.
    pub signal: Signal,
    /// What it actually does to something that eats it, in milligrams.
    ///
    /// The gap between this and [`Self::signal`] is the whole mechanic. An
    /// honest organism's claim matches its bite. A **Batesian** mimic warns
    /// without a bite: safe, and eaten only by something that learned better.
    /// An **aggressive** mimic looks plain and bites hard: the trap.
    pub venom_mg: u64,
    /// The kingdom this organism *appears* to belong to.
    ///
    /// Usually its own. A mimic's differs, which is what breaks the shape
    /// contract on purpose: roles are read from geometry, so the game teaches
    /// that form tells you function, and a simulacrum violates exactly that
    /// lesson.
    ///
    /// **Since DC1.5 the claim is about anatomy.** It used to be a claim about
    /// silhouette, because that is what a kingdom was; it is now a claim to
    /// carry the organs of a way of making a living — "there is a leaf on me" —
    /// told by a body that does not. What tells the lie is unchanged and no
    /// reader of it moved; what the lie is *about* did.
    pub guise: Kingdom,
}

/// The smallest part of each role a founding body can be given a feeding organ
/// out of: a frond, a crop, a jaw. Smallest because a fixture's organ should
/// make the reading and nothing else — a larger one would move the body's mass
/// ceiling, and with it every rate derived from it.
const FROND: [i32; 3] = [3, 2, 1];
const CROP: [i32; 3] = [2, 1, 1];
const JAW: [i32; 3] = [3, 1, 1];

impl Organism {
    /// A minimal organism: a root part, the feeding organ that makes it the
    /// kingdom it is asked for, and the scalars the ecology moves.
    ///
    /// **The organ is not decoration.** Since DC1.5 a kingdom is a reading of
    /// feeding anatomy, so a body that was handed a kingdom has to carry the
    /// anatomy that reads as one; a bare root reads `Decomposer` whatever it
    /// was asked for. A consumer built out of long thin bulk gets a jaw and one
    /// built out of any other shape gets a crop, which keeps a fixture the
    /// feeding mode its caller's half-extent has always asked for.
    ///
    /// The organ's milligram comes out of the root, so the body still weighs
    /// exactly what it was given.
    #[allow(clippy::too_many_arguments)]
    pub fn founding(
        id: OrganismId,
        species: SpeciesId,
        kingdom: Kingdom,
        volume: VolumeRef,
        half_extent: [i32; 3],
        position: [i32; 3],
        mass_mg: u64,
    ) -> Self {
        let development_seed = u64::from(id.0) << 32 | u64::from(species.0);
        let organ = match kingdom {
            Kingdom::Producer => Some((FROND, 1)),
            Kingdom::Consumer if classify(half_extent) == Role::Limb => Some((JAW, -1)),
            Kingdom::Consumer => Some((CROP, -1)),
            Kingdom::Decomposer => None,
        };
        let organ_mg = u64::from(organ.is_some()).min(mass_mg);
        let mut body = BodyDocument::new(species, volume, mass_mg - organ_mg, half_extent);
        body.plan.symmetry = kingdom.symmetry();
        if let Some((shape, side)) = organ {
            let root = body.root;
            let offset = [0, side * (half_extent[1].abs() + shape[1].abs()), 0];
            body.attach(
                volume,
                organ_mg,
                shape,
                Attachment {
                    parent: root,
                    offset,
                    yaw: Yaw::Zero,
                },
                Provenance::founding(),
            )
            .expect("the organ attaches to a root that was just built");
        }
        Self {
            id,
            species,
            body,
            development_seed,
            life_history_mass_mg: mass_mg,
            position,
            tier: Tier::Near,
            last_seen: None,
            fauna_policy: FaunaPolicy::default(),
            last_fauna_decision: None,
            energy_mg: mass_mg,
            stage: Stage::Juvenile,
            age: 0,
            since_offspring: 0,
            signal: Signal::Plain,
            venom_mg: 0,
            guise: kingdom,
        }
    }

    /// The volume a projection should draw for this organism: its root part's.
    pub fn volume(&self) -> VolumeRef {
        self.body
            .part(self.body.root)
            .map(|p| p.volume)
            .unwrap_or(VolumeRef([0; 32]))
    }

    /// This organism's overall half-extent, read off its root part.
    ///
    /// A reading rather than a field, so a body and the shape the world sees
    /// cannot disagree.
    pub fn half_extent(&self) -> [i32; 3] {
        self.body
            .part(self.body.root)
            .map(|p| p.half_extent)
            .unwrap_or([1, 1, 1])
    }

    /// The cross-section this body's live anatomy presents to Ground.
    ///
    /// This is recomputed from the collision box so incorporation, growth, or
    /// injury can change where the organism fits without synchronizing a
    /// second locomotion size field.
    pub fn walker_shape(&self) -> WalkerShape {
        WalkerShape::from_aabb(self.body.aabb())
    }

    /// What this organism weighs: the sum of its surviving parts.
    ///
    /// **The only body mass there is.** Until 2026-08-01 an `Organism` also
    /// carried a scalar `mass_mg` that the ecology moved, so anatomy and the
    /// ecology kept separate accounts of the same quantity and neither could
    /// see the other. That is why a forty-part critter cost exactly what a
    /// single cell cost.
    pub fn biomass_mg(&self) -> u64 {
        self.body.total_mass_mg()
    }

    pub(crate) fn life_history_mass_mg(&self) -> u64 {
        if self.life_history_mass_mg == 0 {
            self.biomass_mg()
        } else {
            self.life_history_mass_mg
        }
    }

    /// Trophic role read from the organs this body feeds with, not retained as
    /// a genesis decree. A reshaped body therefore changes the role the ecology
    /// sees. See [`kingdom`] for the rules and why they replaced symmetry.
    pub fn kingdom(&self) -> Kingdom {
        Kingdom::of_body(&self.body)
    }

    /// What this body does with matter: the same anatomy, read one level
    /// finer. A jaw at the head makes a predator; a crop makes a grazer.
    pub fn feeding_mode(&self) -> FeedingMode {
        FeedingMode::of_body(&self.body)
    }

    /// How far this body's actuators swing, in voxels: each living
    /// contractile part's longest half-extent, summed.
    ///
    /// **Zero for a body with no actuator**, which is what a sessile plan
    /// honestly reads. [`Self::locomotion`] floors the same number at one for
    /// the drive selector's arithmetic; rent prices this one, so a plant pays
    /// nothing for a machinery it does not carry. (TD7)
    pub fn actuator_span(&self) -> u32 {
        self.body
            .living()
            .filter(|part| self.body.processes(part.id).contains(&Process::Contract))
            .map(|part| {
                part.half_extent
                    .iter()
                    .map(|v| v.unsigned_abs())
                    .max()
                    .unwrap_or(0)
            })
            .sum::<u32>()
    }

    /// How far this body's sense organs extend, in voxels: each living
    /// sensing part's longest half-extent, summed. The exact shape of
    /// [`Self::actuator_span`], read off [`Process::Sense`] instead of
    /// [`Process::Contract`]. (TD11)
    ///
    /// **Zero for a body with no sense organ**, which is what a blind plan
    /// honestly reads — see [`ecology::sight_for_body`], where zero is the
    /// near horizon a body has always had rather than no horizon at all.
    ///
    /// Span rather than [`FaunaTraits::sensory_parts`]'s count, for the reason
    /// rent reads span: a count cannot say that a bigger organ senses more, and
    /// the two coincide anyway on the primitive palette, whose sensor template
    /// is `[1, 1, 1]`.
    pub fn sensor_span(&self) -> u32 {
        self.body
            .living()
            .filter(|part| self.body.processes(part.id).contains(&Process::Sense))
            .map(|part| {
                part.half_extent
                    .iter()
                    .map(|v| v.unsigned_abs())
                    .max()
                    .unwrap_or(0)
            })
            .sum::<u32>()
    }

    /// A compact locomotion reading used by the drive selector. It is based
    /// on the same contractile geometry that makes a body a predator.
    pub fn locomotion(&self) -> u32 {
        self.actuator_span().max(1)
    }

    /// The adult mass this body plan describes.
    ///
    /// **Determinate growth** (TD6, ruled 2026-08-29). Income, upkeep and the
    /// reproduction tax all scale as `m^0.75`, so the sign of net growth never
    /// depended on size and no body ever arrived at an adult mass. The ceiling
    /// is derived per part from what the plan already knows — the part's own
    /// voxel volume — and summed, so a longer or bulkier recipe is a larger
    /// adult and gigantism is a lineage strategy rather than an authored
    /// number. It is no longer the stability mechanism (conservation is), so
    /// it can be exactly this simple.
    ///
    /// The way past it is the game's own verb: eating adds *parts*, and every
    /// part brings its own ceiling with it.
    pub fn mass_ceiling_mg(&self) -> u64 {
        self.body
            .living()
            .map(|part| ecology::part_ceiling_mg(part.half_extent))
            .sum()
    }

    /// How much more matter this body could take in, as substance or reserve.
    ///
    /// What a feeding body should ask for: drawing beyond it would only have
    /// to be handed straight back to the world.
    pub fn intake_room_mg(&self) -> u64 {
        let ceiling = self.mass_ceiling_mg();
        ceiling.saturating_sub(self.biomass_mg()) + ceiling.saturating_sub(self.energy_mg)
    }

    /// Adds substance, to the root part, up to [`Self::mass_ceiling_mg`].
    ///
    /// Growth by feeding thickens what is already there. Gaining a *part* is
    /// incorporation, which is a different act with a different cost.
    ///
    /// Returns what would not fit. Matter is conserved, so a caller has to put
    /// that somewhere real rather than letting it evaporate.
    pub fn gain_mass(&mut self, mg: u64) -> u64 {
        let room = self.mass_ceiling_mg().saturating_sub(self.biomass_mg());
        let kept = mg.min(room);
        let root = self.body.root;
        match self.body.parts.get_mut(root.0 as usize) {
            Some(part) => {
                part.mass_mg = part.mass_mg.saturating_add(kept);
                mg - kept
            }
            None => mg,
        }
    }

    /// Removes substance across the living body in stable part order.
    ///
    /// Returns what could not be paid. A body that cannot cover a cost is
    /// starving, and the caller decides what that means.
    pub fn spend_mass(&mut self, mg: u64) -> u64 {
        let mut unpaid = mg;
        for part in self.body.parts.iter_mut().filter(|part| !part.severed) {
            let paid = part.mass_mg.min(unpaid);
            part.mass_mg -= paid;
            unpaid -= paid;
            if unpaid == 0 {
                break;
            }
        }
        unpaid
    }

    /// How elaborate this organism is.
    ///
    /// **Derived from anatomy**, like everything else a body decides. Parts
    /// carry more weight than substance, because a creature is complex by
    /// having many things rather than by being large, and each part is
    /// something that has to be fed and connected.
    ///
    /// `epoch::Lineage` has its own complexity over a trait array. That one is
    /// the provisional scaffolding the phenotype plan schedules for deletion;
    /// this is the one the world uses.
    pub fn complexity(&self) -> i32 {
        let parts = self.body.living().count() as i32;
        parts * 4 + (self.biomass_mg() / 500) as i32
    }

    /// What this organism spends per tick simply existing — **and moving**.
    ///
    /// **Scales with what it is carrying**, which is the whole point of
    /// reconciling the ledgers: before this, upkeep was a flat milligram and
    /// a body could grow without limit for free. Growing is now a standing
    /// cost, which is what gives *burn or grow* a downside to weigh.
    ///
    /// TD7 adds the second half: rent prices what a body *does*, not only what
    /// it weighs. The three numbers here are all the body plan's own —
    /// [`Self::biomass_mg`], [`Self::actuator_span`], and
    /// [`Self::mass_ceiling_mg`] — so the trophic asymmetry between a plant and
    /// an animal is anatomy rather than an authored constant. See
    /// [`ecology::upkeep_for_body`].
    pub fn upkeep_mg(&self) -> u64 {
        ecology::upkeep_for_body(
            self.biomass_mg(),
            self.actuator_span(),
            self.mass_ceiling_mg(),
        )
    }

    /// Whether the budget holds fewer than `ticks` ticks of upkeep.
    ///
    /// **The one shape every hunger question in the game takes.** Hunger is
    /// never a milligram count — a large body burns through the same number
    /// faster — so it is asked in the only unit that means the same thing to
    /// every body: how long this one could go on doing nothing. The callers
    /// pick their own horizon and say why: the ecology wanders at
    /// `HUNGRY_UPKEEP_TICKS` (nearly out), and a meal routes at
    /// `STARVED_UPKEEP_TICKS` (out soon enough to matter).
    pub fn budget_below(&self, ticks: u64) -> bool {
        self.energy_mg < self.upkeep_mg().saturating_mul(ticks)
    }

    /// Pays one tick of upkeep: from the budget first, then from the body.
    ///
    /// Eating to burn buys survival directly; when there is nothing left to
    /// spend, a creature consumes itself. Returns what it still could not pay,
    /// which is starvation.
    pub fn pay_upkeep(&mut self) -> u64 {
        let owed = self.upkeep_mg();
        let from_budget = self.energy_mg.min(owed);
        self.energy_mg -= from_budget;
        self.spend_mass(owed - from_budget)
    }

    pub fn is_alive(&self) -> bool {
        self.stage.is_alive()
    }

    /// Whether this organism is pretending to be something it is not.
    pub fn is_mimic(&self) -> bool {
        self.guise != self.kingdom() || self.signals_falsely()
    }

    /// Whether its advertisement is a lie, in either direction.
    pub fn signals_falsely(&self) -> bool {
        match self.signal {
            Signal::Warning => self.venom_mg == 0,
            Signal::Plain => self.venom_mg > 0,
        }
    }

    /// The tell.
    ///
    /// A thing wearing a producer's look but living a consumer's life does not
    /// gain mass in open ground, because it is not fixing anything. Watch it
    /// for a while and the lie shows. Unfair is fine here; unknowable is not,
    /// so every mimic leaves something a second encounter can find.
    ///
    /// **The claim got sharper at DC1.5**, and stayed true. It used to mean "a
    /// producer's silhouette over a body the symmetry field says is not one";
    /// it now means "a producer's claim over a body that carries no part
    /// performing [`Process::Fix`]" — which is precisely why the tell exists,
    /// rather than being a coincidence of two enums lining up.
    pub fn betrays_itself(&self) -> bool {
        self.guise == Kingdom::Producer && self.kingdom() != Kingdom::Producer
    }

    /// Whether this organism is ready to produce an offspring.
    ///
    /// **Adult mass, not an absolute floor** (TD8, and the missing half of
    /// TD6). The mass clause used to be a flat `STARVATION_MG * OFFSPRING_COST`
    /// — 80 mg, a number that knows nothing about the body asking. TD6 derived
    /// an adult mass from the body plan and made growth determinate; this makes
    /// breeding ask about it, so a big-plan body has to grow up first and a body
    /// stalled at a fifth of its ceiling stops shedding broods it cannot afford.
    ///
    /// The gestation clock is unchanged, and the 80 mg clause stays underneath
    /// as what it always really was: the guarantee that a brood costing a
    /// quarter of the parent is born above [`ecology::STARVATION_MG`] rather
    /// than already starving. The two do not conflict — a small plan's share of
    /// its ceiling can fall under 80 mg, and then the floor is the binding one.
    pub fn can_reproduce(&self) -> bool {
        self.stage == Stage::Mature
            && self.since_offspring >= ecology::gestation_for_mass(self.life_history_mass_mg())
            && self.biomass_mg() >= ecology::breeding_mass_mg(self.mass_ceiling_mg())
            && self.biomass_mg() > STARVATION_MG * OFFSPRING_COST
    }
}

/// What a tick did to the world's organisms, so a host can show it and a test
/// can assert on it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tally {
    pub matured: u32,
    pub born: u32,
    pub died: u32,
    pub returned: u32,
    pub moved: u32,
    pub far_cohorts: u32,
    pub far_members: u32,
    pub far_biomass_mg: u64,
    pub promoted: u32,
    pub demoted: u32,
}
