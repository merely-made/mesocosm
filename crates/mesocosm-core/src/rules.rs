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

use serde::{Deserialize, Serialize};

use crate::process::Registry;

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
/// One component today. Material and field definitions, environmental
/// schedules and the generator version join it as their own gates land
/// (playable ecology plan §2); each arrives as another digest here rather than
/// as a second place a rule can live.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct WorldRules {
    /// The process ruleset this world admitted.
    pub processes: RulesetDigest,
}

impl WorldRules {
    /// The rules a world founded on this build's own definitions realizes.
    pub fn native() -> Self {
        Self::of(Registry::native())
    }

    /// The rules an admitted registry amounts to.
    pub fn of(registry: &Registry) -> Self {
        Self {
            processes: registry.digest(),
        }
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

    #[test]
    fn a_repeated_qualified_id_is_refused() {
        let mut defs: Vec<_> = Registry::native().all().cloned().collect();
        let clash = defs[0].clone();
        defs.push(clash.clone());
        assert_eq!(Registry::admit(defs), Err(clash.id));
    }
}
