// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The adaptation phase: every species takes a turn, in initiative order.
//!
//! The second half of the epoch loop, and the half that makes the world feel
//! like it is playing too. The player is **one lineage among many at this
//! table** — the single decision this module is built around.
//!
//! # Initiative is descending metabolic complexity
//!
//! The most complex lineages commit first; simpler ones act afterwards, knowing
//! what those expensive forms just became. That compresses generation tempo
//! into one legible round: a fruit fly passes through many generations inside
//! one cicada lifecycle, so its lineage receives the *informational* advantage
//! rather than the player watching hundreds of repeated turns.
//!
//! The ordering only means something because [`Standing`] recomputes what a
//! lineage faces from the roster's current state. Commits land immediately, so
//! a later actor scores its candidates against a world the earlier ones have
//! already changed. Freeze the world for the duration of a round and the order
//! becomes decoration.
//!
//! # Nothing here knows which lineage is the player's
//!
//! [`Lineage::played`] exists for the player's sake and is never read by this
//! module. Unplayed lines adapt by exactly the same code — they grow, decline,
//! and go extinct while the player is elsewhere — which is the wing's third law
//! holding at the level of the simulation rather than the file format. A line
//! you left is not frozen; returning means entering its descendants.

pub mod adapt;
pub mod lineage;
pub mod standing;
pub mod worlds;

pub use adapt::{Decision, Mutation, fitness, take_turn};
pub use lineage::{Lineage, Role, Trait};
pub use standing::Standing;
pub use worlds::{AUTHORED, Force, HEAVY_DEEP, LONG_YEAR, Pressure, TIDAL_SHELF, WorldProfile};

use serde::{Deserialize, Serialize};

use crate::rng::Rng;

/// Fitness below which a lineage does not survive the round.
///
/// Extinction, not death, is the failure state: an individual dying is a bad
/// afternoon, losing the species is the end. The threshold is deliberately
/// generous — a lineage should die of a cascade it could not answer, not of one
/// unlucky round.
pub const EXTINCTION_FLOOR: i32 = -400;

/// One adaptation phase: who acted, in what order, and what happened.
///
/// The record is the feature. Thrive runs this in the background and lets the
/// player infer it from population numbers; here it is a transcript, because a
/// trophic cascade nobody watched is indistinguishable from procedural noise.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Round {
    /// Decisions in the order they were taken, which is initiative order.
    pub decisions: Vec<Decision>,
    /// Lineages that did not survive this round.
    pub extinctions: Vec<u32>,
}

impl Round {
    pub fn changes(&self) -> impl Iterator<Item = &Decision> {
        self.decisions.iter().filter(|decision| decision.changed())
    }

    /// What one lineage decided this round.
    pub fn decision(&self, lineage: u32) -> Option<&Decision> {
        self.decisions.iter().find(|decision| decision.lineage == lineage)
    }

    /// The order lineages acted in.
    pub fn order(&self) -> Vec<u32> {
        self.decisions.iter().map(|decision| decision.lineage).collect()
    }
}

/// Initiative: descending metabolic complexity, ties broken by id.
///
/// Ties break by id purely so a round replays identically; nothing in the
/// design says a lower id deserves to act first.
pub fn initiative(roster: &[Lineage]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..roster.len()).filter(|i| !roster[*i].extinct).collect();
    order.sort_by_key(|i| (-roster[*i].complexity(), roster[*i].id));
    order
}

/// Runs one adaptation phase over `roster`, in place.
///
/// Each lineage banks what the epoch gave it, takes its turn, and **commits
/// immediately** so the lineages after it are answering the world as it now
/// stands.
pub fn adapt_round(world: &WorldProfile, roster: &mut [Lineage], rng: &mut Rng) -> Round {
    let mut round = Round::default();

    for index in initiative(roster) {
        let decision = {
            let standing = Standing::new(world, roster);
            take_turn(&roster[index], &standing, rng)
        };

        if let Some(mutation) = decision.chosen {
            let next = mutation.applied(&roster[index]);
            roster[index] = next;
        }
        round.decisions.push(decision);
    }

    // Extinction is judged after everyone has acted, so a lineage is never
    // killed by a neighbour that had not taken its turn yet.
    let standing_snapshot = roster.to_vec();
    let standing = Standing::new(world, &standing_snapshot);
    for lineage in roster.iter_mut() {
        if lineage.extinct {
            continue;
        }
        if fitness(lineage, &standing) < EXTINCTION_FLOOR {
            lineage.extinct = true;
            round.extinctions.push(lineage.id);
        }
    }

    round
}

/// Whether the player may step into `target` from the lineages they hold.
///
/// The gate is the world's **complexity frontier**: an unlocked lineage must be
/// *more* complex than the target. Stepping downward into a newly viable niche
/// is the point; minting an unearned peer at the frontier is not.
pub fn can_switch_to(roster: &[Lineage], target: u32) -> bool {
    let Some(target) = roster.iter().find(|lineage| lineage.id == target) else {
        return false;
    };
    if target.extinct {
        return false;
    }
    roster
        .iter()
        .filter(|lineage| lineage.played && !lineage.extinct)
        .any(|held| held.complexity() > target.complexity())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roster() -> Vec<Lineage> {
        let mut big = Lineage::new(1, "cicada", Role::Consumer, [3, 3, 2, 2, 3, 1, 3]);
        big.bank = 40;
        let mut small = Lineage::new(2, "fly", Role::Consumer, [1, 0, 0, 1, 0, 2, 0]);
        small.bank = 40;
        let mut moss = Lineage::new(3, "moss", Role::Producer, [1, 1, 1, 0, 0, 2, 0]);
        moss.bank = 40;
        vec![big, small, moss]
    }

    #[test]
    fn the_most_complex_acts_first() {
        let roster = roster();
        let order = initiative(&roster);
        let complexities: Vec<i32> = order.iter().map(|i| roster[*i].complexity()).collect();

        let mut descending = complexities.clone();
        descending.sort_by(|a, b| b.cmp(a));
        assert_eq!(complexities, descending, "initiative is descending complexity");
        assert_eq!(roster[order[0]].name, "cicada");
    }

    #[test]
    fn the_extinct_do_not_take_turns() {
        let mut roster = roster();
        roster[0].extinct = true;
        assert_eq!(initiative(&roster).len(), 2);
    }

    #[test]
    fn a_simpler_lineage_answers_what_the_complex_one_just_did() {
        // The reason the order exists. The fly acts after the cicada, so the
        // cicada's commit is already part of the world the fly is scoring
        // against. Compare the fly's turn against a roster where the cicada
        // never got to act.
        let world = &TIDAL_SHELF;

        let mut committed = roster();
        let round = adapt_round(world, &mut committed, &mut Rng::from_seed(11));
        assert_eq!(round.order()[0], 1, "the cicada went first");

        let fly_position = round.order().iter().position(|id| *id == 2).unwrap();
        assert!(fly_position > 0, "and the fly went after it");

        // The fly weighed its options against a world containing the cicada's
        // *new* form, not its old one.
        let original = roster();
        let before = Standing::new(world, &original);
        let after = Standing::new(world, &committed);
        let fly = &original[1];
        assert!(
            after.on(fly, Pressure::Predation) >= before.on(fly, Pressure::Predation),
            "acting later means facing whatever the earlier lineages became"
        );
    }

    #[test]
    fn commits_land_before_the_next_lineage_scores() {
        // Directly: a hunter that arms itself must be visible to the lineage
        // acting after it, or the initiative rule is decoration.
        let world = &TIDAL_SHELF;
        let mut hunter = Lineage::new(1, "hunter", Role::Consumer, [4, 4, 4, 4, 4, 4, 4]);
        hunter.bank = 200;
        let mut prey = Lineage::new(2, "prey", Role::Consumer, [1, 0, 0, 0, 0, 1, 0]);
        prey.bank = 200;

        let mut roster = vec![hunter, prey];
        let before_jaws = roster[0].level(Trait::Jaws);
        let round = adapt_round(world, &mut roster, &mut Rng::from_seed(3));

        assert_eq!(round.order(), vec![1, 2], "complex first");
        if roster[0].level(Trait::Jaws) > before_jaws {
            let standing = Standing::new(world, &roster);
            assert!(
                standing.on(&roster[1], Pressure::Predation) > world.strength(Pressure::Predation),
                "the prey now lives somewhere more dangerous"
            );
        }
    }

    #[test]
    fn unplayed_lineages_adapt_by_the_same_code() {
        // Autonomous inactive lineages. Nothing in the round reads `played`, so
        // a line the player left keeps changing while they are elsewhere.
        let world = &LONG_YEAR;
        let mut played = roster();
        played[0].played = true;
        let mut unplayed = roster();

        let a = adapt_round(world, &mut played, &mut Rng::from_seed(5));
        let b = adapt_round(world, &mut unplayed, &mut Rng::from_seed(5));

        assert_eq!(a, b, "the round is identical whether or not anyone played it");
        assert_ne!(unplayed, roster(), "and the unplayed roster still changed");
    }

    #[test]
    fn a_round_is_a_transcript() {
        let mut roster = roster();
        let round = adapt_round(&HEAVY_DEEP, &mut roster, &mut Rng::from_seed(9));

        assert_eq!(round.decisions.len(), 3, "everybody took a visible turn");
        for decision in &round.decisions {
            assert!(!decision.considered.is_empty(), "and showed its work");
        }
        assert!(round.changes().count() > 0, "at least one of them did something");
    }

    #[test]
    fn rounds_are_deterministic() {
        let mut a = roster();
        let mut b = roster();
        let ra = adapt_round(&HEAVY_DEEP, &mut a, &mut Rng::from_seed(4));
        let rb = adapt_round(&HEAVY_DEEP, &mut b, &mut Rng::from_seed(4));
        assert_eq!(ra, rb);
        assert_eq!(a, b);
    }

    #[test]
    fn you_may_step_down_but_not_across() {
        // The complexity frontier. Holding something elaborate earns you a
        // simpler niche, never an unearned peer at the top.
        let mut roster = roster();
        roster[0].played = true; // the cicada, the most complex

        assert!(can_switch_to(&roster, 2), "into the simpler fly, yes");
        assert!(!can_switch_to(&roster, 1), "into an equal, no");

        roster[0].played = false;
        roster[1].played = true; // now holding only the fly
        assert!(!can_switch_to(&roster, 1), "the fly does not earn the cicada");
    }

    #[test]
    fn you_cannot_enter_the_dead_or_the_absent() {
        let mut roster = roster();
        roster[0].played = true;
        roster[1].extinct = true;
        assert!(!can_switch_to(&roster, 2), "an extinct line is not a destination");
        assert!(!can_switch_to(&roster, 99), "nor is one that does not exist");
    }
}
