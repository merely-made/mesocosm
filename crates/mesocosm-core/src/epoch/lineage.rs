// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Lineages: what takes a turn in the adaptation phase.
//!
//! A lineage is a species over time — the thing that persists when individuals
//! do not, and the unit Mesocosm's care granularity is set to. The player is
//! **one lineage among many at this table**, which is the decision that makes
//! the world feel like it is playing too.
//!
//! # Complexity is the initiative key, and it is derived
//!
//! Initiative is descending metabolic complexity: the most complex lineages
//! commit first, and simpler ones act afterwards *knowing what they did*. That
//! compresses generation tempo into one legible round — a fruit fly passes
//! through many generations inside one cicada lifecycle, so its lineage gets
//! the informational advantage instead of the player watching hundreds of
//! turns.
//!
//! So complexity must not be a field somebody sets. It is **derived from what
//! the lineage is made of**, which means a lineage cannot buy initiative
//! without paying for it in traits it then has to feed.

use serde::{Deserialize, Serialize};

use super::worlds::Pressure;

/// What a lineage can be good at. Each answers at least one [`Pressure`], and
/// each costs upkeep, so no trait is free and none is universally right.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Trait {
    /// Structure that holds itself up.
    Frame,
    /// Keeping heat that wants to leave.
    Insulation,
    /// Finding, holding, and spending solvent carefully.
    Water,
    /// Perceiving without light.
    Sense,
    /// Not being dissolved, and not being bitten.
    Shell,
    /// Making more of yourself, faster.
    Fecundity,
    /// Taking what somebody else grew.
    Jaws,
}

impl Trait {
    pub const ALL: [Trait; 7] = [
        Trait::Frame,
        Trait::Insulation,
        Trait::Water,
        Trait::Sense,
        Trait::Shell,
        Trait::Fecundity,
        Trait::Jaws,
    ];

    /// The pressures this trait answers, and how well.
    ///
    /// Overlapping on purpose. A trait that answered exactly one pressure would
    /// make adaptation a lookup table; overlap is what makes a choice a
    /// tradeoff rather than a puzzle with one solution.
    pub fn answers(self) -> &'static [(Pressure, i32)] {
        match self {
            Trait::Frame => &[(Pressure::Gravity, 3), (Pressure::Predation, 1)],
            Trait::Insulation => &[(Pressure::Cold, 3), (Pressure::Drought, 1)],
            Trait::Water => &[(Pressure::Drought, 3), (Pressure::Corrosive, 1)],
            Trait::Sense => &[(Pressure::Dark, 3), (Pressure::Predation, 1)],
            Trait::Shell => &[(Pressure::Corrosive, 3), (Pressure::Predation, 2)],
            Trait::Fecundity => &[(Pressure::Crowding, 3), (Pressure::Predation, 1)],
            Trait::Jaws => &[(Pressure::Crowding, 2), (Pressure::Predation, -1)],
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Trait::Frame => "frame",
            Trait::Insulation => "insulation",
            Trait::Water => "water",
            Trait::Sense => "sense",
            Trait::Shell => "shell",
            Trait::Fecundity => "fecundity",
            Trait::Jaws => "jaws",
        }
    }
}

/// Which side of the trophic cycle a lineage works.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Producer,
    Consumer,
    Decomposer,
}

/// A species over time.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lineage {
    pub id: u32,
    pub name: String,
    pub role: Role,
    /// Trait levels, indexed by [`Trait::ALL`]. Integer, so a run replays.
    pub traits: [i32; 7],
    /// What the epoch banked, and what this phase has to spend.
    pub bank: i32,
    /// Whether the player has ever inhabited this line. **Carried for the
    /// player's sake, never read by the adaptation rule** — an unplayed line
    /// adapts by exactly the same code, which is Law C holding at the level of
    /// the simulation rather than the file format.
    pub played: bool,
    /// Extinct lineages stay in the roster as a record. Removing them would
    /// erase the trophic cascade that killed them.
    pub extinct: bool,
}

impl Lineage {
    pub fn new(id: u32, name: impl Into<String>, role: Role, traits: [i32; 7]) -> Self {
        Self {
            id,
            name: name.into(),
            role,
            traits,
            bank: 0,
            played: false,
            extinct: false,
        }
    }

    pub fn level(&self, of: Trait) -> i32 {
        self.traits[Trait::ALL
            .iter()
            .position(|t| *t == of)
            .expect("a known trait")]
    }

    pub fn set_level(&mut self, of: Trait, level: i32) {
        let index = Trait::ALL
            .iter()
            .position(|t| *t == of)
            .expect("a known trait");
        self.traits[index] = level.max(0);
    }

    /// Metabolic complexity: everything this lineage is carrying.
    ///
    /// Derived rather than stored, so initiative cannot be bought. A lineage
    /// that wants to act first has to be genuinely expensive, and being
    /// expensive is what upkeep charges it for.
    pub fn complexity(&self) -> i32 {
        self.traits.iter().sum()
    }

    /// What this lineage spends every round simply existing.
    ///
    /// Superlinear on purpose: complexity has to *cost* something, or every
    /// lineage climbs forever and the simple ones never get their turn to
    /// matter. This is the pressure that keeps a fruit fly viable next to a
    /// cicada.
    pub fn upkeep(&self) -> i32 {
        let complexity = self.complexity();
        complexity + complexity * complexity / 12
    }

    /// How well this lineage answers one pressure.
    pub fn answer_to(&self, pressure: Pressure) -> i32 {
        Trait::ALL
            .iter()
            .map(|trait_| {
                let level = self.level(*trait_);
                trait_
                    .answers()
                    .iter()
                    .filter(|(against, _)| *against == pressure)
                    .map(|(_, weight)| level * weight)
                    .sum::<i32>()
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain(id: u32) -> Lineage {
        Lineage::new(id, "plain", Role::Consumer, [1, 1, 1, 1, 1, 1, 1])
    }

    #[test]
    fn complexity_is_what_the_lineage_carries() {
        assert_eq!(plain(0).complexity(), 7);

        let mut heavy = plain(1);
        heavy.set_level(Trait::Shell, 6);
        assert_eq!(
            heavy.complexity(),
            12,
            "adding armour makes you more complex"
        );
    }

    #[test]
    fn complexity_cannot_be_set_directly() {
        // The property that keeps initiative honest: there is no field to
        // write, so acting first always costs traits you then have to feed.
        let mut lineage = plain(0);
        let before = lineage.complexity();
        lineage.bank = 10_000;
        lineage.played = true;
        assert_eq!(lineage.complexity(), before, "nothing but traits moves it");
    }

    #[test]
    fn upkeep_outruns_complexity() {
        // Superlinear, or nothing ever stops climbing.
        let simple = Lineage::new(0, "simple", Role::Producer, [1, 0, 0, 0, 0, 1, 0]);
        let complex = Lineage::new(1, "complex", Role::Consumer, [6, 6, 6, 6, 6, 6, 6]);

        let simple_ratio = simple.upkeep() as f64 / simple.complexity() as f64;
        let complex_ratio = complex.upkeep() as f64 / complex.complexity() as f64;
        assert!(
            complex_ratio > simple_ratio * 2.0,
            "the elaborate lineage pays disproportionately ({complex_ratio} vs {simple_ratio})"
        );
    }

    #[test]
    fn traits_answer_the_pressures_they_claim_to() {
        let mut cold = plain(0);
        cold.set_level(Trait::Insulation, 5);
        let mut armoured = plain(1);
        armoured.set_level(Trait::Shell, 5);

        assert!(
            cold.answer_to(Pressure::Cold) > armoured.answer_to(Pressure::Cold),
            "insulation is the cold answer"
        );
        assert!(
            armoured.answer_to(Pressure::Corrosive) > cold.answer_to(Pressure::Corrosive),
            "shell is the corrosive one"
        );
    }

    #[test]
    fn jaws_make_you_worth_eating() {
        // The one negative weight, and it is the interesting one: a lineage
        // that specialises in taking makes itself a target. Without a cost like
        // this, every lineage converges on predation.
        let mut hunter = Lineage::new(0, "hunter", Role::Consumer, [0; 7]);
        hunter.set_level(Trait::Jaws, 4);
        assert!(hunter.answer_to(Pressure::Predation) < 0);
    }

    #[test]
    fn levels_never_go_negative() {
        let mut lineage = plain(0);
        lineage.set_level(Trait::Frame, -5);
        assert_eq!(lineage.level(Trait::Frame), 0);
    }
}
