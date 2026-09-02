// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What rules a world actually realized. (PD3)
//!
//! The playable ecology plan §2 asks a durable world to record its admitted
//! definitions, schedules, generator version and digests, because a seed alone
//! is insufficient once the code that reads it can change. `WorldRules` is that
//! record's working label, and this is its **first proven component**: the
//! digest of the [`Registry`](crate::process::Registry) a world was founded
//! under.
//!
//! # Identity, not a copy
//!
//! A world carries the digest, not the definitions. That is the same
//! arrangement `ProcessRef` already uses one scale down — a content address
//! rather than an inline copy — and it keeps a snapshot the size of a world
//! rather than the size of a world plus its biology. What the digest buys is
//! the refusal: a save restored against a ruleset that is not the one it ran
//! under is refused by name (`SnapshotError::Ruleset`) instead of quietly
//! simulating something else.
//!
//! # What is rule-bearing
//!
//! Everything [`Registry::digest`](crate::process::Registry::digest) folds,
//! and nothing else. Each definition contributes its identity, its site
//! requirement and its seeding; the set is folded in sorted order, so neither
//! the order a pack declared its files in nor a definition's plain label,
//! explanation text or native binding can move this number.
//!
//! # The second component: what ends an epoch (PE3)
//!
//! [`EpochRule`] is the first rule here that is not a digest, and it belongs
//! beside the ruleset for the same reason the ruleset does: a world that ended
//! its epochs on a different budget is a different game, and a save restored
//! against the wrong one would quietly re-time the whole run.

use serde::{Deserialize, Serialize};

use crate::process::Registry;

/// Ticks an epoch runs for by default.
///
/// **A hundred seconds at the canonical ten ticks a second, and a third of a
/// 1,000 mg starter's 3,000-tick lifespan** — so a played line reviews itself
/// about three times in the life of one body, and a recorded demo of a few
/// thousand steps crosses several boundaries rather than none. Shorter, and
/// the review interrupts a life that has not had time to become anything;
/// longer, and a body lives and dies without its line ever taking a turn.
pub const DEFAULT_EPOCH_TICKS: u64 = 1_000;

/// How long a candidate is grown before its flow record is read. (P4b)
///
/// **One brood interval at the ecology's reference body** —
/// `rates::GESTATION_BASE`, the same number `gestation_for_mass` is quoted
/// against. It is that reading and not a shorter one because a revision only
/// ever shows up in *descendants*: a window with no birth of the line in it
/// scores the candidate and the status quo identically, and a window with a
/// birth but no life after it prices only what the newborn paid to grow the
/// organ. Measured, not guessed — at one judgement window
/// ([`WARN_AFTER_TICKS`](crate::flow::WARN_AFTER_TICKS), 60 ticks) every line
/// in the probe scored the gland *worse* by about a tenth of a percent, which
/// is its development cost and nothing else; the sign turns between 240 and
/// 600, which is where the organ has been carried long enough to be worth
/// carrying. One brood interval is the shortest span with a name that reaches
/// past that, and it costs a boundary two runs of this length per candidate
/// per line.
pub const DEFAULT_SCORE_TICKS: u64 = crate::organism::ecology::GESTATION_BASE as u64;

/// What deterministic condition ends an epoch.
///
/// **Three separate rules, not a composition** (playable ecology plan §6,
/// ruled by Mark 2026-09-01). Only [`Timed`](Self::Timed) is built; the other
/// two are named here as data so that the vocabulary, the serialized shape and
/// the digest are settled before either arrives, and a world holding one
/// simply never ends an epoch on its own — which [`Self::built`] says out loud
/// rather than leaving a caller to infer from silence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EpochRule {
    /// The epoch ends when a fixed tick budget is spent. **Built first.**
    Timed { ticks: u64 },
    /// The epoch ends when named world conditions are all met. **Not built**;
    /// it comes second, and it needs the conditions named first.
    Gated,
    /// The epoch ends on demand. **Not built, and never play**: it is the dev
    /// tools plan's DT3, and it is listed here so nothing later mistakes it
    /// for a rule a shipped world could hold.
    PlayerTriggered,
}

impl Default for EpochRule {
    fn default() -> Self {
        Self::Timed {
            ticks: DEFAULT_EPOCH_TICKS,
        }
    }
}

impl EpochRule {
    /// Whether this rule is implemented. `false` for the two named-only ones.
    pub fn built(self) -> bool {
        matches!(self, Self::Timed { .. })
    }

    /// Whether `elapsed` ticks of the current epoch spend its budget.
    ///
    /// An unbuilt rule answers `false` at every tick, which is the honest
    /// behaviour for a rule with no condition behind it: the epoch does not
    /// end rather than ending on a guess.
    pub fn spent(self, elapsed: u64) -> bool {
        match self {
            Self::Timed { ticks } => ticks > 0 && elapsed >= ticks,
            Self::Gated | Self::PlayerTriggered => false,
        }
    }

    fn bytes(self) -> Vec<u8> {
        match self {
            Self::Timed { ticks } => {
                let mut bytes = vec![0u8];
                bytes.extend_from_slice(&ticks.to_le_bytes());
                bytes
            }
            Self::Gated => vec![1],
            Self::PlayerTriggered => vec![2],
        }
    }
}

/// A content address for one complete admitted ruleset.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct RulesetDigest(pub u64);

impl RulesetDigest {
    /// The form a receipt, a panel or a diagnostic prints.
    pub fn hex(self) -> String {
        format!("{:016x}", self.0)
    }
}

/// The immutable record of the rules a world realized.
///
/// Material and field definitions, environmental schedules and the generator
/// version join these as their own gates land (playable ecology plan §2); each
/// arrives here rather than as a second place a rule can live.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct WorldRules {
    /// The process ruleset this world admitted.
    pub processes: RulesetDigest,
    /// What ends an epoch here. (PE3)
    #[serde(default)]
    pub epoch: EpochRule,
    /// Ticks a candidate is grown for before its flow record is read. (P4b)
    ///
    /// Rule-bearing, and that is why it is here rather than a constant in the
    /// scorer: it decides what an unplayed line commits, and what an unplayed
    /// line commits is the world.
    #[serde(default = "default_score_ticks")]
    pub score_ticks: u64,
}

fn default_score_ticks() -> u64 {
    DEFAULT_SCORE_TICKS
}

/// **Written out rather than derived**, because a derived one would give this
/// build's defaults for the epoch rule and a zero-length scoring window — and a
/// zero-length window scores every candidate identically, which is a rule
/// nobody chose arriving through a `..Default::default()`.
impl Default for WorldRules {
    fn default() -> Self {
        Self {
            processes: RulesetDigest::default(),
            epoch: EpochRule::default(),
            score_ticks: DEFAULT_SCORE_TICKS,
        }
    }
}

impl WorldRules {
    /// The rules a world founded on this build's own definitions realizes.
    pub fn native() -> Self {
        Self::of(Registry::native())
    }

    /// The rules an admitted registry amounts to, under this build's defaults
    /// for everything a registry does not decide.
    pub fn of(registry: &Registry) -> Self {
        Self {
            processes: registry.digest(),
            epoch: EpochRule::default(),
            score_ticks: DEFAULT_SCORE_TICKS,
        }
    }

    /// The same rules under a different epoch rule.
    pub fn ending(self, epoch: EpochRule) -> Self {
        Self { epoch, ..self }
    }

    /// The same rules under a different scoring window.
    pub fn scoring_over(self, score_ticks: u64) -> Self {
        Self {
            score_ticks,
            ..self
        }
    }

    /// This whole record's identity: every component folded, in a fixed order.
    ///
    /// What a refusal names when two worlds disagree about their rules and it
    /// is not the ruleset they disagree about. It is not stored — a world
    /// carries the components and this reads them — so nothing can hold a
    /// digest that has fallen behind what it is a digest of.
    pub fn digest(self) -> u64 {
        let mut bytes = self.processes.0.to_le_bytes().to_vec();
        bytes.extend_from_slice(&self.epoch.bytes());
        bytes.extend_from_slice(&self.score_ticks.to_le_bytes());
        crate::snapshot::hash_bytes(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ruleset_digest_is_stable_across_calls() {
        assert_eq!(WorldRules::native(), WorldRules::native());
        assert_eq!(
            WorldRules::native().processes,
            Registry::native().digest(),
            "a world's rules are the registry's digest and nothing else"
        );
    }

    #[test]
    fn declaration_order_is_not_rule_bearing() {
        // The property a pack depends on: a manifest may list its files in any
        // order, and the ruleset it lowers to is the same ruleset.
        let mut reversed: Vec<_> = Registry::native().all().cloned().collect();
        reversed.reverse();
        let admitted = Registry::admit(reversed).expect("no collision");
        assert_eq!(admitted, *Registry::native(), "canonical order is restored");
        assert_eq!(admitted.digest(), Registry::native().digest());
    }

    /// The epoch rule is folded into the record's identity, so two worlds on
    /// different budgets cannot agree about their rules. (PE3)
    #[test]
    fn the_epoch_rule_is_digested() {
        let native = WorldRules::native();
        assert_eq!(native.epoch, EpochRule::Timed { ticks: 1_000 });
        assert_eq!(native.digest(), WorldRules::native().digest());

        let brisk = native.ending(EpochRule::Timed { ticks: 500 });
        assert_ne!(brisk.digest(), native.digest(), "a budget is rule-bearing");
        assert_eq!(
            brisk.processes, native.processes,
            "and it moved nothing about the biology"
        );

        let quick = native.scoring_over(10);
        assert_ne!(quick.digest(), native.digest(), "so is the score window");
    }

    /// Timed is built; the other two are named data and end nothing.
    #[test]
    fn only_the_timed_rule_ends_an_epoch() {
        let timed = EpochRule::Timed { ticks: 3 };
        assert!(timed.built());
        assert!(!timed.spent(2), "not before the budget is spent");
        assert!(timed.spent(3), "and exactly when it is");

        for unbuilt in [EpochRule::Gated, EpochRule::PlayerTriggered] {
            assert!(!unbuilt.built());
            assert!(
                !unbuilt.spent(u64::MAX),
                "a rule with no condition behind it does not end an epoch on a guess"
            );
        }
    }

    #[test]
    fn a_repeated_qualified_id_is_refused() {
        let mut defs: Vec<_> = Registry::native().all().cloned().collect();
        let clash = defs[0].clone();
        defs.push(clash.clone());
        assert_eq!(Registry::admit(defs), Err(clash.id));
    }
}
