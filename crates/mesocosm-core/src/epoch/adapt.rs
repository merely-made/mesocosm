// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One lineage's turn: what it could become, what that would be worth, and
//! what it decides.
//!
//! The algorithm is Thrive's auto-evo, adopted deliberately: generate a handful
//! of candidate mutations, score them, keep the best — **or keep none**. It is
//! cheap enough that the roster can be large, and it is sufficient, which
//! matters more than it being clever.
//!
//! The divergence is legibility, not the algorithm. Thrive runs this in the
//! background and lets the player infer it from population numbers; here every
//! lineage takes a visible turn, so you watch the thing that eats your food
//! supply decide to eat it better rather than finding out afterwards.
//!
//! # Scoring punishes your worst answer, not your average one
//!
//! Fitness is the negated sum of **squared deficits** against every pressure,
//! minus upkeep. Squaring is the whole design: it means a lineage dies of the
//! thing it is worst at, so adaptation shores up weaknesses instead of piling
//! onto strengths. A linear score would let a lineage ignore a lethal pressure
//! as long as it was excellent somewhere else, which is not how anything lives.

use serde::{Deserialize, Serialize};

use super::lineage::{Lineage, Trait};
use super::standing::Standing;
use crate::rng::Rng;

/// How many candidates a lineage considers per turn. Thrive's number, kept.
pub const CANDIDATES: u32 = 5;

/// What a lineage might do with its bank.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mutation {
    /// Put a level into a trait.
    Gain { trait_: Trait },
    /// Move a level from one trait to another. Cheaper than gaining, because
    /// nothing new has to be fed — and it is the reason a lineage can change
    /// direction rather than only accumulate.
    Swap { from: Trait, to: Trait },
}

impl Mutation {
    /// What this costs out of the bank.
    ///
    /// Gaining charges the level being bought, so the tenth point of a trait
    /// costs ten and specialising has a natural ceiling. Swapping is flat and
    /// cheap: giving something up should be affordable, or a lineage that
    /// guessed wrong once is stuck with it forever.
    pub fn cost(self, lineage: &Lineage) -> i32 {
        match self {
            Mutation::Gain { trait_ } => lineage.level(trait_) + 1,
            Mutation::Swap { .. } => 2,
        }
    }

    /// The lineage this mutation would produce.
    pub fn applied(self, lineage: &Lineage) -> Lineage {
        let mut next = lineage.clone();
        match self {
            Mutation::Gain { trait_ } => {
                next.set_level(trait_, next.level(trait_) + 1);
            }
            Mutation::Swap { from, to } => {
                next.set_level(from, next.level(from) - 1);
                next.set_level(to, next.level(to) + 1);
            }
        }
        next.bank -= self.cost(lineage);
        next
    }

    /// Whether this is even coherent for a lineage: you cannot move a level out
    /// of a trait that has none.
    pub fn possible(self, lineage: &Lineage) -> bool {
        match self {
            Mutation::Gain { .. } => true,
            Mutation::Swap { from, to } => from != to && lineage.level(from) > 0,
        }
    }
}

/// What a lineage decided, and why. The record a player reads.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decision {
    pub lineage: u32,
    /// `None` means the status quo beat every candidate — a real outcome, and
    /// the one that keeps a lineage from mutating itself to death out of
    /// obligation to spend.
    pub chosen: Option<Mutation>,
    /// Fitness before and after. The visible part of the tradeoff.
    pub before: i32,
    pub after: i32,
    /// Everything it weighed, in the order it weighed them.
    pub considered: Vec<(Mutation, i32)>,
}

impl Decision {
    pub fn changed(&self) -> bool {
        self.chosen.is_some()
    }

    /// The improvement it bought. Zero when it stood pat.
    pub fn gain(&self) -> i32 {
        self.after - self.before
    }
}

/// How well a lineage is doing where it stands.
///
/// Negative throughout: it is a measure of what the world is doing to you that
/// you cannot answer, so zero would be a creature under no pressure at all.
pub fn fitness(lineage: &Lineage, standing: &Standing) -> i32 {
    let deficit: i32 = standing
        .pressures_on(lineage)
        .map(|(pressure, strength)| {
            let unmet = (strength - lineage.answer_to(pressure)).max(0);
            unmet * unmet
        })
        .sum();
    -(deficit + lineage.upkeep())
}

/// One lineage's turn.
///
/// Candidates are drawn against the standing *as it is now*, which is what
/// makes initiative order matter: a lineage acting later in the round is
/// scoring against a world the earlier ones have already changed.
pub fn take_turn(lineage: &Lineage, standing: &Standing, rng: &mut Rng) -> Decision {
    let before = fitness(lineage, standing);
    let mut considered = Vec::with_capacity(CANDIDATES as usize);

    for _ in 0..CANDIDATES {
        let candidate = propose(rng);
        if !candidate.possible(lineage) || candidate.cost(lineage) > lineage.bank {
            continue;
        }
        let score = fitness(&candidate.applied(lineage), standing);
        considered.push((candidate, score));
    }

    // Best strictly better than standing pat. Ties keep the status quo, because
    // paying for a lateral move is worse than not paying.
    let best = considered
        .iter()
        .filter(|(_, score)| *score > before)
        .max_by_key(|(_, score)| *score)
        .copied();

    match best {
        Some((mutation, after)) => Decision {
            lineage: lineage.id,
            chosen: Some(mutation),
            before,
            after,
            considered,
        },
        None => Decision {
            lineage: lineage.id,
            chosen: None,
            before,
            after: before,
            considered,
        },
    }
}

/// One random candidate. Swaps are proposed a third of the time, so a lineage
/// can change direction without waiting for a bank it may never accumulate.
///
/// Draws blind to the lineage on purpose: this is hill-climbing, and the
/// selection pressure lives entirely in the scoring. A proposer that already
/// knew which traits were good would be doing the search twice, and would quietly
/// stop the world from surprising anybody.
fn propose(rng: &mut Rng) -> Mutation {
    let pick = |rng: &mut Rng| Trait::ALL[rng.below(Trait::ALL.len() as u64) as usize];
    if rng.below(3) == 0 {
        let from = pick(rng);
        let to = pick(rng);
        // A degenerate swap is left to `possible` rather than resampled, so the
        // candidate count stays honest: a wasted draw is a wasted draw.
        Mutation::Swap { from, to }
    } else {
        Mutation::Gain { trait_: pick(rng) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::epoch::lineage::Role;
    use crate::epoch::worlds::{HEAVY_DEEP, TIDAL_SHELF};

    fn lineage(bank: i32) -> Lineage {
        let mut it = Lineage::new(0, "test", Role::Consumer, [1, 1, 1, 1, 1, 1, 1]);
        it.bank = bank;
        it
    }

    #[test]
    fn gaining_costs_more_the_more_you_have() {
        let mut it = lineage(100);
        assert_eq!(
            Mutation::Gain {
                trait_: Trait::Shell
            }
            .cost(&it),
            2
        );
        it.set_level(Trait::Shell, 9);
        assert_eq!(
            Mutation::Gain {
                trait_: Trait::Shell
            }
            .cost(&it),
            10
        );
    }

    #[test]
    fn swapping_is_cheap_so_a_wrong_guess_is_survivable() {
        let mut it = lineage(100);
        it.set_level(Trait::Jaws, 8);
        let swap = Mutation::Swap {
            from: Trait::Jaws,
            to: Trait::Shell,
        };
        assert!(
            swap.cost(&it)
                < Mutation::Gain {
                    trait_: Trait::Shell
                }
                .cost(&it)
                    + 8
        );

        let after = swap.applied(&it);
        assert_eq!(after.level(Trait::Jaws), 7);
        assert_eq!(after.level(Trait::Shell), 2);
        assert_eq!(
            after.complexity(),
            it.complexity(),
            "a swap moves, it does not add"
        );
    }

    #[test]
    fn you_cannot_swap_out_of_nothing() {
        let mut it = lineage(100);
        it.set_level(Trait::Jaws, 0);
        assert!(
            !Mutation::Swap {
                from: Trait::Jaws,
                to: Trait::Shell
            }
            .possible(&it)
        );
        assert!(
            !Mutation::Swap {
                from: Trait::Shell,
                to: Trait::Shell
            }
            .possible(&it)
        );
    }

    #[test]
    fn fitness_punishes_the_worst_answer_hardest() {
        // The squared-deficit design. A lineage that is excellent at one thing
        // and helpless at another must score worse than an even one, or it
        // would rationally ignore a lethal pressure.
        let roster = [];
        let standing = Standing::new(&HEAVY_DEEP, &roster);

        let even = Lineage::new(0, "even", Role::Consumer, [3, 3, 3, 3, 3, 3, 0]);
        let mut lopsided = Lineage::new(1, "lopsided", Role::Consumer, [0; 7]);
        lopsided.set_level(Trait::Frame, 18);

        assert_eq!(
            even.complexity(),
            lopsided.complexity(),
            "same budget spent"
        );
        assert!(
            fitness(&even, &standing) > fitness(&lopsided, &standing),
            "spreading beats specialising when the world pushes on several axes"
        );
    }

    #[test]
    fn a_lineage_with_no_bank_can_do_nothing() {
        let roster = [];
        let standing = Standing::new(&TIDAL_SHELF, &roster);
        let decision = take_turn(&lineage(0), &standing, &mut Rng::from_seed(1));

        assert!(!decision.changed(), "nothing is affordable");
        assert_eq!(decision.gain(), 0);
        assert!(
            decision.considered.is_empty(),
            "and nothing was even weighable"
        );
    }

    #[test]
    fn standing_pat_is_a_real_outcome() {
        // Keeping none when none beats the status quo. A lineage that always
        // spent would mutate itself to death out of obligation, and upkeep
        // would make that fatal.
        let roster = [];
        let standing = Standing::new(&TIDAL_SHELF, &roster);

        // Already answering this world well; almost anything added is upkeep.
        let mut settled = Lineage::new(0, "settled", Role::Producer, [0, 3, 0, 4, 0, 6, 0]);
        settled.bank = 3;

        let mut stood_pat = 0;
        for seed in 0..40 {
            let decision = take_turn(&settled, &standing, &mut Rng::from_seed(seed));
            if !decision.changed() {
                stood_pat += 1;
            }
        }
        assert!(
            stood_pat > 0,
            "a well-adapted lineage sometimes declines to spend"
        );
    }

    #[test]
    fn a_chosen_mutation_always_improves_things() {
        let roster = [];
        let standing = Standing::new(&HEAVY_DEEP, &roster);
        let mut poor = Lineage::new(0, "poor", Role::Consumer, [0; 7]);
        poor.bank = 20;

        for seed in 0..40 {
            let decision = take_turn(&poor, &standing, &mut Rng::from_seed(seed));
            if decision.changed() {
                assert!(decision.gain() > 0, "never pays for a lateral move");
            }
        }
    }

    #[test]
    fn turns_are_deterministic() {
        let roster = [];
        let standing = Standing::new(&HEAVY_DEEP, &roster);
        let it = lineage(30);
        let a = take_turn(&it, &standing, &mut Rng::from_seed(7));
        let b = take_turn(&it, &standing, &mut Rng::from_seed(7));
        assert_eq!(a, b);
    }
}
