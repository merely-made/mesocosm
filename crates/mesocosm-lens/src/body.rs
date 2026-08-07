// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Projects the authoritative parts graph into the Lens capsule vocabulary.
//!
//! The projection deliberately simplifies each living voxel part to one
//! capsule. It retains the part address, content address, and provenance beside
//! that capsule, so the simplification never becomes a second body format.

use std::collections::BTreeMap;

use mesocosm_core::{
    BodyDocument, Part, PartId, Provenance, VolumeRef,
    snapshot::{encode, hash_bytes},
};
use serde::{Deserialize, Serialize};

use crate::{CritterPose, MAX_CAPSULES, critter::Capsule};

/// Content identity of the authoritative body used for one projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BodyRevision(pub u64);

/// Where body space is realized in the world presentation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BodyPlacement {
    /// World-space point beneath body-space `[0, min_y, 0]`.
    pub ground: [f32; 3],
    /// World units per body voxel.
    pub scale: f32,
    pub tint: [f32; 3],
}

/// One capsule's exact ancestry back into the body document.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LensPart {
    pub part: PartId,
    pub volume: VolumeRef,
    pub provenance: Provenance,
    pub capsule: u16,
    /// Hash of every fact this part's Lens realization reads, including its
    /// resolved placement. A changed parent therefore changes descendants.
    pub dependency: u64,
}

/// A Lens pose plus the identity it intentionally omits from the GPU uniform.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BodyLensProjection {
    pub revision: BodyRevision,
    pub pose: CritterPose,
    pub parts: Vec<LensPart>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BodyProjectionError {
    Encode,
    InvalidScale,
    Unplaceable(PartId),
    TooManyCapsules { actual: usize, maximum: usize },
}

impl BodyLensProjection {
    /// Simplifies every living part to one placed capsule.
    pub fn project(
        body: &BodyDocument,
        placement: BodyPlacement,
    ) -> Result<Self, BodyProjectionError> {
        if !placement.scale.is_finite() || placement.scale <= 0.0 {
            return Err(BodyProjectionError::InvalidScale);
        }
        let living: Vec<_> = body.living().collect();
        if living.len() > MAX_CAPSULES {
            return Err(BodyProjectionError::TooManyCapsules {
                actual: living.len(),
                maximum: MAX_CAPSULES,
            });
        }

        let body_bytes = encode(body).map_err(|_| BodyProjectionError::Encode)?;
        let revision = BodyRevision(hash_bytes(&body_bytes));
        let floor = body.aabb().min[1];
        let realize = |point: [i32; 3]| {
            [
                placement.ground[0] + point[0] as f32 * placement.scale,
                placement.ground[1] + (point[1] - floor) as f32 * placement.scale,
                placement.ground[2] + point[2] as f32 * placement.scale,
            ]
        };

        let mut capsules = Vec::with_capacity(living.len());
        let mut parts = Vec::with_capacity(living.len());
        for part in living {
            let (a, b, radius) = capsule_for(body, part, placement.scale)?;
            let pivot = body
                .world_pivot(part.id)
                .ok_or(BodyProjectionError::Unplaceable(part.id))?;
            let yaw = body
                .world_yaw(part.id)
                .ok_or(BodyProjectionError::Unplaceable(part.id))?;
            let placement_bits = [
                placement.ground[0].to_bits(),
                placement.ground[1].to_bits(),
                placement.ground[2].to_bits(),
                placement.scale.to_bits(),
                placement.tint[0].to_bits(),
                placement.tint[1].to_bits(),
                placement.tint[2].to_bits(),
            ];
            // Hash exactly what this projection reads. In particular, body
            // mass changes under upkeep but cannot alter a capsule, so mass
            // must not churn presentation dependencies.
            let dependency_bytes = encode(&(
                part.id,
                part.volume,
                part.half_extent,
                part.pivot,
                part.attachment,
                &part.provenance,
                pivot,
                yaw,
                floor,
                placement_bits,
            ))
            .map_err(|_| BodyProjectionError::Encode)?;
            let capsule = u16::try_from(capsules.len()).expect("admission checked above");
            capsules.push(Capsule {
                a: realize(a),
                ra: radius,
                b: realize(b),
                rb: radius,
            });
            parts.push(LensPart {
                part: part.id,
                volume: part.volume,
                provenance: part.provenance.clone(),
                capsule,
                dependency: hash_bytes(&dependency_bytes),
            });
        }

        let eyes = eyes_for(body, placement.scale, realize)?;
        Ok(Self {
            revision,
            pose: CritterPose::from_capsules(capsules, eyes, placement.tint),
            parts,
        })
    }

    /// Parts whose Lens dependencies differ between two body projections.
    pub fn changed_parts(&self, previous: &Self) -> Vec<PartId> {
        let before: BTreeMap<_, _> = previous
            .parts
            .iter()
            .map(|part| (part.part, part.dependency))
            .collect();
        let after: BTreeMap<_, _> = self
            .parts
            .iter()
            .map(|part| (part.part, part.dependency))
            .collect();
        before
            .keys()
            .chain(after.keys())
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .filter(|part| before.get(part) != after.get(part))
            .collect()
    }
}

fn capsule_for(
    body: &BodyDocument,
    part: &Part,
    scale: f32,
) -> Result<([i32; 3], [i32; 3], f32), BodyProjectionError> {
    let half = part.half_extent.map(i32::abs);
    let axis = (0..3).max_by_key(|axis| half[*axis]).unwrap_or(0);
    let mut cross = (0..3)
        .filter(|other| *other != axis)
        .map(|other| half[other]);
    let radius_voxels = cross
        .next()
        .unwrap_or(1)
        .min(cross.next().unwrap_or(1))
        .max(1);
    let run = (half[axis] - radius_voxels).max(0);
    let mut local_a = part.pivot;
    let mut local_b = part.pivot;
    local_a[axis] -= run;
    local_b[axis] += run;
    let a = body
        .place(part.id, local_a)
        .ok_or(BodyProjectionError::Unplaceable(part.id))?;
    let b = body
        .place(part.id, local_b)
        .ok_or(BodyProjectionError::Unplaceable(part.id))?;
    Ok((a, b, radius_voxels as f32 * scale))
}

fn eyes_for(
    body: &BodyDocument,
    scale: f32,
    realize: impl Fn([i32; 3]) -> [f32; 3],
) -> Result<[[f32; 4]; 2], BodyProjectionError> {
    let root = body
        .part(body.root)
        .ok_or(BodyProjectionError::Unplaceable(body.root))?;
    let side = (root.half_extent[0].abs() / 2).max(1);
    let up = (root.half_extent[1].abs() / 2).max(1);
    let front = root.half_extent[2].abs().max(1);
    let eye = |sign: i32| {
        let local = [
            root.pivot[0] + side * sign,
            root.pivot[1] + up,
            root.pivot[2] + front,
        ];
        body.place(root.id, local)
            .map(|point| {
                let at = realize(point);
                [at[0], at[1], at[2], (scale * 0.28).max(0.08)]
            })
            .ok_or(BodyProjectionError::Unplaceable(root.id))
    };
    Ok([eye(-1)?, eye(1)?])
}

#[cfg(test)]
mod tests {
    use super::*;
    use mesocosm_core::{Attachment, Origin, SpeciesId, Yaw};

    fn body() -> BodyDocument {
        let mut body = BodyDocument::new(SpeciesId(7), VolumeRef::from_tag(1), 100, [2, 2, 2]);
        body.attach(
            VolumeRef::from_tag(2),
            40,
            [1, 1, 3],
            Attachment {
                parent: body.root,
                offset: [3, 0, 0],
                yaw: Yaw::Quarter,
            },
            Provenance {
                origin: Origin::Incorporated {
                    from_species: SpeciesId(42),
                    from_part: PartId(3),
                },
                epoch: 2,
            },
        )
        .unwrap();
        body
    }

    fn placement() -> BodyPlacement {
        BodyPlacement {
            ground: [20.0, 4.0, 30.0],
            scale: 0.5,
            tint: [0.3, 0.7, 0.4],
        }
    }

    #[test]
    fn every_capsule_points_back_to_one_living_part() {
        let body = body();
        let projected = BodyLensProjection::project(&body, placement()).unwrap();
        assert_eq!(projected.pose.capsules.len(), 2);
        assert_eq!(
            projected
                .parts
                .iter()
                .map(|part| part.part)
                .collect::<Vec<_>>(),
            vec![PartId(0), PartId(1)]
        );
        assert_eq!(
            projected.parts[1].provenance,
            body.part(PartId(1)).unwrap().provenance
        );
    }

    #[test]
    fn adding_one_part_invalidates_only_that_part() {
        let before_body = body();
        let mut after_body = before_body.clone();
        let added = after_body
            .attach(
                VolumeRef::from_tag(3),
                20,
                [1, 1, 1],
                Attachment {
                    parent: after_body.root,
                    offset: [-3, 0, 0],
                    yaw: Yaw::Zero,
                },
                Provenance::founding(),
            )
            .unwrap();
        let before = BodyLensProjection::project(&before_body, placement()).unwrap();
        let after = BodyLensProjection::project(&after_body, placement()).unwrap();
        assert_eq!(after.changed_parts(&before), vec![added]);
        assert_ne!(after.revision, before.revision);
    }

    #[test]
    fn moving_a_parent_invalidates_its_descendants() {
        let mut before_body = body();
        before_body
            .attach(
                VolumeRef::from_tag(3),
                10,
                [1, 1, 1],
                Attachment {
                    parent: PartId(1),
                    offset: [0, 0, 4],
                    yaw: Yaw::Zero,
                },
                Provenance::founding(),
            )
            .unwrap();
        let mut after_body = before_body.clone();
        after_body.parts[1].attachment.as_mut().unwrap().offset[0] += 1;
        let before = BodyLensProjection::project(&before_body, placement()).unwrap();
        let after = BodyLensProjection::project(&after_body, placement()).unwrap();
        assert_eq!(after.changed_parts(&before), vec![PartId(1), PartId(2)]);
    }

    #[test]
    fn severed_parts_do_not_project() {
        let mut body = body();
        body.sever(PartId(1));
        let projected = BodyLensProjection::project(&body, placement()).unwrap();
        assert_eq!(projected.parts.len(), 1);
        assert_eq!(projected.parts[0].part, body.root);
    }

    #[test]
    fn a_global_tint_change_invalidates_every_capsule() {
        let body = body();
        let before = BodyLensProjection::project(&body, placement()).unwrap();
        let mut recoloured = placement();
        recoloured.tint = [0.8, 0.2, 0.3];
        let after = BodyLensProjection::project(&body, recoloured).unwrap();
        assert_eq!(after.changed_parts(&before), vec![PartId(0), PartId(1)]);
    }
}
