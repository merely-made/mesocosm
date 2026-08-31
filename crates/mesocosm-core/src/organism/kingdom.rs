// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a body makes its living by, read off the parts it feeds with.
//!
//! Split out of `organism.rs` at the 600-line ceiling when the reading stopped
//! being one line.
//!
//! # The unbinding (DC1.5)
//!
//! A kingdom used to be `body.plan.symmetry`, a bijection: radial was a
//! producer, bilateral a consumer, networked a decomposer. That made a body's
//! whole trophic life a consequence of the field that decides where a limb's
//! twin goes, and it meant no anatomy could disagree with it — you could not
//! grow a leaf and become a producer, or lose your mouth and stop being a
//! consumer. Symmetry is now geometry only and a kingdom is read from the
//! organs a body actually feeds with:
//!
//! | reading | anatomy |
//! | --- | --- |
//! | [`Kingdom::Producer`] | a living part performs [`Process::Fix`] — a plate, a frond, a leaf |
//! | [`Kingdom::Consumer`] | no fixing part, and the head bears a mouth |
//! | [`Kingdom::Decomposer`] | neither: it absorbs across its surface |
//!
//! and the consumer's mouth is read the same way, by shape:
//!
//! | reading | anatomy |
//! | --- | --- |
//! | [`FeedingMode::Predator`] | the mouth is `Limb`-classified — a jaw, which swings |
//! | [`FeedingMode::Grazer`] | the mouth is bulk — a crop, which does not |
//!
//! # Three things this reading deliberately does
//!
//! **It orders fixing ahead of the mouth.** A body carrying both organs is a
//! mixotroph, which the rulings register defers (§13): archetypes are
//! single-mode, so the precedence is stated rather than discovered, and the
//! founding draws never produce one.
//!
//! **It makes the decomposer the residual.** That is not a shrug — it is the
//! anatomy. A saprotroph has no ingesting organ: it digests outside itself and
//! takes the result in across its whole surface, which is exactly
//! [`Process::Intake`]'s own third clause ("a mouth, a gut, *an absorbing
//! surface*"), and every body's bulk segments already perform it. A decomposer
//! is the body that has nothing more specialised than that.
//!
//! **It lets anatomy change a life.** Sever a grazer's mouth and it can no
//! longer take a living meal; graft a plate onto a consumer and it fixes. That
//! is the point of a reading rather than a field, and it is new behaviour —
//! see the plan's DC1.5 findings.

use serde::{Deserialize, Serialize};

use crate::body::BodyDocument;
use crate::plan::{Role, Symmetry, classify};
use crate::process::{FeedingMode, Process};

/// Trophic role. Not a character class: these are the three ways of making a
/// living, and a lineage may combine them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Kingdom {
    /// Fixes energy from the world itself. The base of every chain.
    Producer,
    /// Must eat. Pays upkeep and starves without a meal.
    Consumer,
    /// Lives on the dead, returning locked matter to circulation.
    Decomposer,
}

impl Kingdom {
    /// The silhouette a founding body of this tier is given.
    ///
    /// **A founding default, not a reading.** Until DC1.5 this was one half of
    /// a bijection and `Kingdom::from_symmetry` was the other; now nothing
    /// derives a kingdom from symmetry and this only says which shape a
    /// worldgen tier opens with, so a stand still reads radial at a glance.
    pub fn symmetry(self) -> Symmetry {
        match self {
            Self::Producer => Symmetry::Radial,
            Self::Consumer => Symmetry::Bilateral,
            Self::Decomposer => Symmetry::None,
        }
    }

    /// Reads a body's kingdom off the organs it feeds with.
    ///
    /// Fixing first, because a body that carries both organs is the deferred
    /// mixotroph and the tie has to be broken somewhere stated.
    pub fn of_body(body: &BodyDocument) -> Self {
        if body.performs(Process::Fix) {
            Self::Producer
        } else if body.mouth().is_some() {
            Self::Consumer
        } else {
            Self::Decomposer
        }
    }
}

impl FeedingMode {
    /// What a body does with matter, read off the same anatomy.
    pub fn of_body(body: &BodyDocument) -> Self {
        match Kingdom::of_body(body) {
            Kingdom::Producer => Self::Producer,
            Kingdom::Decomposer => Self::Scavenger,
            // A jaw swings, so it takes something that runs; a crop does not,
            // so it takes something that stands still.
            Kingdom::Consumer => match body.mouth() {
                Some(Role::Limb) => Self::Predator,
                _ => Self::Grazer,
            },
        }
    }
}

impl BodyDocument {
    /// The feeding organ the head carries, as the shape class it reads as.
    ///
    /// **A mouth is a living part borne under the head**: attached to the root
    /// — which development always builds as the axis' front-most segment — and
    /// hung below it, which is where development puts a feeding structure and
    /// where the body plan puts bulk. Both halves are load-bearing. Without
    /// "under", the next segment along the axis would read as a mouth, since a
    /// spine is Mass-classified parts attached to Mass-classified parts;
    /// without "the head", a body would be a consumer for any bulk hanging
    /// anywhere off it.
    ///
    /// `Limb` wins a body that grew more than one, because a jaw is the organ
    /// that decides what the body can take.
    pub fn mouth(&self) -> Option<Role> {
        let mut found = None;
        for part in self.living() {
            let borne_under_the_head = part
                .attachment
                .is_some_and(|at| at.parent == self.root && at.offset[1] < 0);
            if !borne_under_the_head {
                continue;
            }
            match classify(part.half_extent) {
                Role::Limb => return Some(Role::Limb),
                Role::Mass => found = Some(Role::Mass),
                // A sensor under the head is a feeler and a plate is a frond;
                // neither takes a meal in.
                Role::Plate | Role::Sensor => {}
            }
        }
        found
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::{Attachment, Provenance, SpeciesId, VolumeRef, Yaw};

    /// A bulk root, and whatever organs a test hangs off it.
    fn body() -> BodyDocument {
        BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 1_000, [2, 2, 2])
    }

    fn bear(
        body: &mut BodyDocument,
        half_extent: [i32; 3],
        offset: [i32; 3],
    ) -> crate::body::PartId {
        let parent = body.root;
        body.attach(
            VolumeRef::from_tag(2),
            100,
            half_extent,
            Attachment {
                parent,
                offset,
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .expect("attaches")
    }

    #[test]
    fn a_bare_body_absorbs_and_so_decomposes() {
        // The residual, stated as a positive: bulk performs Intake, and a body
        // whose only feeding surface is its own bulk lives on the dead.
        let bare = body();
        assert!(bare.performs(Process::Intake), "bulk absorbs");
        assert_eq!(Kingdom::of_body(&bare), Kingdom::Decomposer);
        assert_eq!(FeedingMode::of_body(&bare), FeedingMode::Scavenger);
    }

    #[test]
    fn a_fixing_part_makes_a_producer() {
        let mut plant = body();
        bear(&mut plant, [4, 4, 1], [0, 5, 0]);
        assert_eq!(Kingdom::of_body(&plant), Kingdom::Producer);
        assert_eq!(FeedingMode::of_body(&plant), FeedingMode::Producer);
    }

    #[test]
    fn mouth_geometry_splits_the_consumer() {
        let mut grazer = body();
        bear(&mut grazer, [2, 1, 1], [0, -3, 0]);
        assert_eq!(Kingdom::of_body(&grazer), Kingdom::Consumer);
        assert_eq!(FeedingMode::of_body(&grazer), FeedingMode::Grazer);

        let mut predator = body();
        bear(&mut predator, [3, 1, 1], [0, -3, 0]);
        assert_eq!(Kingdom::of_body(&predator), Kingdom::Consumer);
        assert_eq!(FeedingMode::of_body(&predator), FeedingMode::Predator);
    }

    #[test]
    fn a_leg_is_not_a_jaw() {
        // The blocker this slice was ruled to close: a limbed grazer stays a
        // grazer, because what makes a predator is the mouth and not the legs.
        let mut grazer = body();
        bear(&mut grazer, [2, 1, 1], [0, -3, 0]);
        bear(&mut grazer, [4, 1, 1], [5, 0, 0]);
        bear(&mut grazer, [4, 1, 1], [-5, 0, 0]);
        assert!(grazer.performs(Process::Contract), "it walks");
        assert_eq!(FeedingMode::of_body(&grazer), FeedingMode::Grazer);
    }

    #[test]
    fn a_spine_segment_is_not_a_mouth() {
        // Segments chain along the axis and are Mass-classified, exactly like a
        // cropping mouth. Only the offset tells them apart.
        let mut worm = body();
        bear(&mut worm, [2, 2, 2], [0, 0, 4]);
        assert_eq!(worm.mouth(), None);
        assert_eq!(Kingdom::of_body(&worm), Kingdom::Decomposer);
    }

    #[test]
    fn losing_the_mouth_loses_the_kingdom() {
        // A reading follows the body. Under the symmetry bijection an injury
        // could never do this, which is the thing the unbinding buys.
        let mut grazer = body();
        let mouth = bear(&mut grazer, [2, 1, 1], [0, -3, 0]);
        assert_eq!(Kingdom::of_body(&grazer), Kingdom::Consumer);
        grazer.sever(mouth);
        assert_eq!(Kingdom::of_body(&grazer), Kingdom::Decomposer);
    }

    #[test]
    fn fixing_is_ordered_ahead_of_the_mouth() {
        // The deferred mixotroph, pinned rather than discovered: a body with
        // both organs reads Producer, and the founding draws never make one.
        let mut both = body();
        bear(&mut both, [4, 4, 1], [0, 5, 0]);
        bear(&mut both, [3, 1, 1], [0, -3, 0]);
        assert_eq!(Kingdom::of_body(&both), Kingdom::Producer);
    }

    #[test]
    fn symmetry_no_longer_says_anything_about_a_kingdom() {
        let mut plant = body();
        bear(&mut plant, [4, 4, 1], [0, 5, 0]);
        for symmetry in [Symmetry::Bilateral, Symmetry::Radial, Symmetry::None] {
            plant.plan.symmetry = symmetry;
            assert_eq!(
                Kingdom::of_body(&plant),
                Kingdom::Producer,
                "{symmetry:?} moved the reading"
            );
        }
    }
}
