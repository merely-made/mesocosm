// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The body graph: parts, attachment frames, and per-part provenance.
//!
//! Coordinates are voxel units and masses are milligrams, both integers.
//! Rotations are quarter turns. Nothing here is a float, so a body's derived
//! quantities are bit-identical on every platform, and the float physics a
//! host runs sits outside this boundary.
//!
//! Coordinates are three-dimensional even when a host presents two: a 2.5D
//! projection constrains an axis rather than changing the document.

use serde::{Deserialize, Serialize};

use crate::plan::BodyPlan;

/// Stable index into [`BodyDocument::parts`]. Never reused within a body.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PartId(pub u32);

/// Identifies a lineage. Provenance records which one a part came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SpeciesId(pub u32);

/// Content address of the voxel volume a projection should draw. The core
/// never reads volume contents; it only carries the reference.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct VolumeRef(pub [u8; 32]);

impl VolumeRef {
    /// Test and fixture helper: a recognisable address from a small number.
    pub fn from_tag(tag: u8) -> Self {
        let mut bytes = [0u8; 32];
        bytes[0] = tag;
        Self(bytes)
    }
}

/// Quarter turns about the vertical axis. Enough to prove attachment while
/// keeping every transform exact in integers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Yaw {
    #[default]
    Zero,
    Quarter,
    Half,
    ThreeQuarter,
}

impl Yaw {
    fn rotate(self, v: [i32; 3]) -> [i32; 3] {
        let [x, y, z] = v;
        match self {
            Yaw::Zero => [x, y, z],
            Yaw::Quarter => [z, y, -x],
            Yaw::Half => [-x, y, -z],
            Yaw::ThreeQuarter => [-z, y, x],
        }
    }

    fn compose(self, inner: Yaw) -> Yaw {
        let steps = (self.steps() + inner.steps()) % 4;
        match steps {
            0 => Yaw::Zero,
            1 => Yaw::Quarter,
            2 => Yaw::Half,
            _ => Yaw::ThreeQuarter,
        }
    }

    fn steps(self) -> u8 {
        match self {
            Yaw::Zero => 0,
            Yaw::Quarter => 1,
            Yaw::Half => 2,
            Yaw::ThreeQuarter => 3,
        }
    }
}

/// Where a part used to be before it became part of this body. This is the
/// keystone record: every part carries the fact that it was once somebody.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Origin {
    /// Present when the lineage was founded.
    Founding,
    /// Taken from another organism by incorporation.
    Incorporated {
        from_species: SpeciesId,
        /// The part's identity in the body it came from.
        from_part: PartId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    pub origin: Origin,
    /// The epoch during which this part joined the body.
    pub epoch: u64,
}

impl Provenance {
    pub fn founding() -> Self {
        Self { origin: Origin::Founding, epoch: 0 }
    }
}

/// How a part is fixed to its parent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attachment {
    pub parent: PartId,
    /// Offset in the parent's frame, before the parent's own rotation.
    pub offset: [i32; 3],
    pub yaw: Yaw,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Part {
    pub id: PartId,
    pub volume: VolumeRef,
    pub mass_mg: u64,
    /// Half-extent in voxel units, so an extent can be derived without
    /// resolving the volume.
    pub half_extent: [i32; 3],
    /// `None` only for the root.
    pub attachment: Option<Attachment>,
    pub provenance: Provenance,
}

/// An axis-aligned box in body space.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Aabb {
    pub min: [i32; 3],
    pub max: [i32; 3],
}

impl Aabb {
    fn around(centre: [i32; 3], half: [i32; 3]) -> Self {
        Self {
            min: [centre[0] - half[0], centre[1] - half[1], centre[2] - half[2]],
            max: [centre[0] + half[0], centre[1] + half[1], centre[2] + half[2]],
        }
    }

    fn union(self, other: Aabb) -> Self {
        Self {
            min: [
                self.min[0].min(other.min[0]),
                self.min[1].min(other.min[1]),
                self.min[2].min(other.min[2]),
            ],
            max: [
                self.max[0].max(other.max[0]),
                self.max[1].max(other.max[1]),
                self.max[2].max(other.max[2]),
            ],
        }
    }

    pub fn extent(&self) -> [i32; 3] {
        [
            self.max[0] - self.min[0],
            self.max[1] - self.min[1],
            self.max[2] - self.min[2],
        ]
    }
}

/// The portable description of one critter's body.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BodyDocument {
    pub species: SpeciesId,
    pub root: PartId,
    /// The heritable rules that decide where growth goes. Parts fill in during
    /// an epoch; this changes between them.
    pub plan: BodyPlan,
    /// Ordered by `PartId`, so iteration is deterministic.
    pub parts: Vec<Part>,
}

/// Returned when an attachment names a part that does not exist or would
/// close a cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttachError {
    UnknownParent(PartId),
    CycleDetected,
}

impl BodyDocument {
    /// A body with a single root part.
    pub fn new(species: SpeciesId, volume: VolumeRef, mass_mg: u64, half_extent: [i32; 3]) -> Self {
        let root = PartId(0);
        Self {
            species,
            root,
            plan: BodyPlan::default(),
            parts: vec![Part {
                id: root,
                volume,
                mass_mg,
                half_extent,
                attachment: None,
                provenance: Provenance::founding(),
            }],
        }
    }

    pub fn part(&self, id: PartId) -> Option<&Part> {
        self.parts.get(id.0 as usize)
    }

    pub fn len(&self) -> usize {
        self.parts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    /// Adds a part fixed to `attachment.parent`, returning its new id.
    pub fn attach(
        &mut self,
        volume: VolumeRef,
        mass_mg: u64,
        half_extent: [i32; 3],
        attachment: Attachment,
        provenance: Provenance,
    ) -> Result<PartId, AttachError> {
        if self.part(attachment.parent).is_none() {
            return Err(AttachError::UnknownParent(attachment.parent));
        }
        let id = PartId(self.parts.len() as u32);
        self.parts.push(Part {
            id,
            volume,
            mass_mg,
            half_extent,
            attachment: Some(attachment),
            provenance,
        });
        Ok(id)
    }

    /// Position of a part in body space, walking up the parent chain.
    ///
    /// Returns `None` if the chain is malformed, which the constructors above
    /// prevent but a deserialized document could carry.
    pub fn world_offset(&self, id: PartId) -> Option<[i32; 3]> {
        let mut offset = [0i32; 3];
        let mut cursor = id;
        // Bounded by part count, so a cycle terminates rather than hanging.
        for _ in 0..=self.parts.len() {
            let part = self.part(cursor)?;
            match part.attachment {
                None => {
                    return Some(offset);
                }
                Some(a) => {
                    let rotated = a.yaw.rotate(offset);
                    offset = [
                        rotated[0] + a.offset[0],
                        rotated[1] + a.offset[1],
                        rotated[2] + a.offset[2],
                    ];
                    cursor = a.parent;
                }
            }
        }
        None
    }

    /// Orientation of a part in body space, composing every joint up the
    /// chain. A projection needs this alongside [`Self::world_offset`] to
    /// place a part's volume; position alone would draw every part unrotated.
    pub fn world_yaw(&self, id: PartId) -> Option<Yaw> {
        let mut yaw = Yaw::Zero;
        let mut cursor = id;
        for _ in 0..=self.parts.len() {
            let part = self.part(cursor)?;
            match part.attachment {
                None => return Some(yaw),
                Some(a) => {
                    yaw = a.yaw.compose(yaw);
                    cursor = a.parent;
                }
            }
        }
        None
    }

    pub fn total_mass_mg(&self) -> u64 {
        self.parts.iter().map(|p| p.mass_mg).sum()
    }

    /// Mass-weighted centre in voxel units, rounded toward zero.
    ///
    /// Uses each part's **centre**, not its origin. A part's origin is its
    /// lowest corner, so averaging origins biases the result by every part's
    /// size and reports a balance the body does not have. This is the same
    /// corner-versus-pivot confusion recorded in the body pipeline plan,
    /// surfacing a third time.
    ///
    /// Accumulated in `i128` so a large body cannot overflow into a different
    /// answer on a different platform.
    pub fn centre_of_mass(&self) -> [i32; 3] {
        let total = self.total_mass_mg();
        if total == 0 {
            return [0; 3];
        }
        let mut acc = [0i128; 3];
        for part in &self.parts {
            let Some(pos) = self.world_offset(part.id) else {
                continue;
            };
            for axis in 0..3 {
                let centre = pos[axis] + part.half_extent[axis];
                acc[axis] += centre as i128 * part.mass_mg as i128;
            }
        }
        let total = total as i128;
        [
            (acc[0] / total) as i32,
            (acc[1] / total) as i32,
            (acc[2] / total) as i32,
        ]
    }

    /// The body's collision extent: the union of every part's box.
    pub fn aabb(&self) -> Aabb {
        let mut result: Option<Aabb> = None;
        for part in &self.parts {
            let Some(pos) = self.world_offset(part.id) else {
                continue;
            };
            let box_ = Aabb::around(pos, part.half_extent);
            result = Some(match result {
                None => box_,
                Some(acc) => acc.union(box_),
            });
        }
        result.unwrap_or(Aabb { min: [0; 3], max: [0; 3] })
    }

    /// Every part that was taken from another organism, in id order.
    pub fn incorporated(&self) -> impl Iterator<Item = &Part> {
        self.parts
            .iter()
            .filter(|p| matches!(p.provenance.origin, Origin::Incorporated { .. }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seed_body() -> BodyDocument {
        BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 1_000, [2, 2, 2])
    }

    #[test]
    fn root_sits_at_origin() {
        let body = seed_body();
        assert_eq!(body.world_offset(body.root), Some([0, 0, 0]));
        assert_eq!(body.total_mass_mg(), 1_000);
    }

    #[test]
    fn attaching_extends_the_collision_box() {
        let mut body = seed_body();
        let before = body.aabb();
        body.attach(
            VolumeRef::from_tag(2),
            500,
            [1, 1, 1],
            Attachment { parent: body.root, offset: [6, 0, 0], yaw: Yaw::Zero },
            Provenance { origin: Origin::Incorporated { from_species: SpeciesId(9), from_part: PartId(0) }, epoch: 1 },
        )
        .expect("root exists");
        let after = body.aabb();
        assert!(after.extent()[0] > before.extent()[0]);
        assert_eq!(after.max[0], 7);
    }

    #[test]
    fn attaching_moves_the_centre_of_mass() {
        let mut body = seed_body();
        // The root spans 0..4 with half-extent 2, so its centre is [2, 2, 2].
        // A lone body's centre of mass is its own centre, not its corner.
        assert_eq!(body.centre_of_mass(), [2, 2, 2]);

        body.attach(
            VolumeRef::from_tag(2),
            1_000,
            [1, 1, 1],
            Attachment { parent: body.root, offset: [10, 0, 0], yaw: Yaw::Zero },
            Provenance::founding(),
        )
        .unwrap();

        // Equal masses at centres [2,2,2] and [11,1,1], so the mean is
        // [6.5, 1.5, 1.5], truncated toward zero.
        assert_eq!(body.centre_of_mass(), [6, 1, 1]);
    }

    #[test]
    fn yaw_rotates_child_offsets_exactly() {
        let mut body = seed_body();
        let arm = body
            .attach(
                VolumeRef::from_tag(2),
                100,
                [1, 1, 1],
                Attachment { parent: body.root, offset: [4, 0, 0], yaw: Yaw::Quarter },
                Provenance::founding(),
            )
            .unwrap();
        let hand = body
            .attach(
                VolumeRef::from_tag(3),
                100,
                [1, 1, 1],
                Attachment { parent: arm, offset: [4, 0, 0], yaw: Yaw::Zero },
                Provenance::founding(),
            )
            .unwrap();
        // The hand's own offset is rotated by the arm's quarter turn.
        assert_eq!(body.world_offset(hand), Some([4, 0, -4]));
    }

    #[test]
    fn nested_yaw_accumulates_up_the_chain() {
        let mut body = seed_body();
        let arm = body
            .attach(
                VolumeRef::from_tag(2),
                100,
                [1, 1, 1],
                Attachment { parent: body.root, offset: [4, 0, 0], yaw: Yaw::Quarter },
                Provenance::founding(),
            )
            .unwrap();
        let hand = body
            .attach(
                VolumeRef::from_tag(3),
                100,
                [1, 1, 1],
                Attachment { parent: arm, offset: [2, 0, 0], yaw: Yaw::Half },
                Provenance::founding(),
            )
            .unwrap();

        assert_eq!(body.world_yaw(body.root), Some(Yaw::Zero));
        assert_eq!(body.world_yaw(arm), Some(Yaw::Quarter));
        // Quarter turn at the shoulder plus a half turn at the wrist.
        assert_eq!(body.world_yaw(hand), Some(Yaw::ThreeQuarter));
    }

    #[test]
    fn unknown_parent_is_refused() {
        let mut body = seed_body();
        let err = body
            .attach(
                VolumeRef::from_tag(2),
                1,
                [1, 1, 1],
                Attachment { parent: PartId(99), offset: [0, 0, 0], yaw: Yaw::Zero },
                Provenance::founding(),
            )
            .unwrap_err();
        assert_eq!(err, AttachError::UnknownParent(PartId(99)));
    }

    #[test]
    fn incorporated_parts_are_listed_in_order() {
        let mut body = seed_body();
        for tag in 2..5u8 {
            body.attach(
                VolumeRef::from_tag(tag),
                10,
                [1, 1, 1],
                Attachment { parent: body.root, offset: [tag as i32, 0, 0], yaw: Yaw::Zero },
                Provenance {
                    origin: Origin::Incorporated {
                        from_species: SpeciesId(tag as u32),
                        from_part: PartId(0),
                    },
                    epoch: 1,
                },
            )
            .unwrap();
        }
        let ids: Vec<_> = body.incorporated().map(|p| p.id).collect();
        assert_eq!(ids, vec![PartId(1), PartId(2), PartId(3)]);
    }
}
