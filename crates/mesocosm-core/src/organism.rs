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

use crate::body::BodyDocument;

pub mod ecology;

pub use ecology::step;
use ecology::{GESTATION, OFFSPRING_COST, STARVATION_MG, UPKEEP_MG, UPKEEP_SHARE};

use crate::body::{SpeciesId, VolumeRef};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct OrganismId(pub u32);

/// Trophic role. Not a character class: these are the three ways of making a
/// living, and a lineage may combine them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kingdom {
    /// Fixes energy from the world itself. The base of every chain.
    Producer,
    /// Must eat. Pays upkeep and starves without a meal.
    Consumer,
    /// Lives on the dead, returning locked matter to circulation.
    Decomposer,
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
    pub kingdom: Kingdom,
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
    pub position: [i32; 3],
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
    pub guise: Kingdom,
}

impl Organism {
    /// A minimal organism: one root part, and the scalars the ecology moves.
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
        Self {
            id,
            species,
            kingdom,
            body: BodyDocument::new(species, volume, mass_mg, half_extent),
            development_seed,
            position,
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

    /// Adds substance, to the root part.
    ///
    /// Growth by feeding thickens what is already there. Gaining a *part* is
    /// incorporation, which is a different act with a different cost.
    pub fn gain_mass(&mut self, mg: u64) {
        let root = self.body.root;
        if let Some(part) = self.body.parts.get_mut(root.0 as usize) {
            part.mass_mg = part.mass_mg.saturating_add(mg);
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

    /// What this organism spends per tick simply existing.
    ///
    /// **Scales with what it is carrying**, which is the whole point of
    /// reconciling the ledgers: before this, upkeep was a flat milligram and
    /// a body could grow without limit for free. Growing is now a standing
    /// cost, which is what gives *burn or grow* a downside to weigh.
    pub fn upkeep_mg(&self) -> u64 {
        UPKEEP_MG + self.biomass_mg() / UPKEEP_SHARE
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
        self.guise != self.kingdom || self.signals_falsely()
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
    pub fn betrays_itself(&self) -> bool {
        self.guise == Kingdom::Producer && self.kingdom != Kingdom::Producer
    }

    /// Whether this organism is ready to produce an offspring.
    pub fn can_reproduce(&self) -> bool {
        self.stage == Stage::Mature
            && self.since_offspring >= GESTATION
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
}
