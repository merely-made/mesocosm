// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Resolving a body plan into an actual attachment.
//!
//! This is the default path: a player eats, and growth happens by the plan.
//! Explicit placement stays available for an editor, which is the ruling
//! Mark made — automatic and symmetric by default, total control possible but
//! never the resting state.

use crate::body::{Attachment, BodyDocument, PartId, Yaw};
use crate::plan::{Facing, Role, classify};

/// Where a part should go, and whether it grows a mirrored twin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Growth {
    pub parent: PartId,
    pub offset: [i32; 3],
    pub yaw: Yaw,
    pub facing: Facing,
    pub role: Role,
    /// The mirrored attachment, when the plan pairs this facing. One meal
    /// grows two parts and the mass is split between them.
    pub mirror: Option<(PartId, [i32; 3])>,
}

/// Finds where a part of the given size should attach, by the body's plan.
///
/// Candidate sites are generated from existing parts, tried in the plan's
/// facing order, and rejected if they would overlap anything already there.
/// Returns `None` when nothing fits, which the caller reports as a rejection
/// rather than forcing a part into occupied space.
pub fn resolve(body: &BodyDocument, half_extent: [i32; 3]) -> Option<Growth> {
    let role = classify(half_extent);
    let size = extent_of(half_extent);

    for facing in body.plan.candidates(role) {
        // Nearest the root first, so a body grows outward rather than
        // sprouting from whatever it ate last.
        for part in &body.parts {
            let anchor = body.world_offset(part.id)?;
            let host = extent_of(part.half_extent);
            let at = site(anchor, host, size, facing);

            if !free(body, at, size) {
                continue;
            }

            // A bilateral plan means what it says: if the mirror will not fit,
            // this is not a site for a pair, so keep looking rather than
            // growing a single lopsided limb. Best-effort mirroring drifts a
            // body sideways one failed pair at a time.
            let mirror = if body.plan.mirrors(facing) {
                let mirrored_at = site(anchor, host, size, facing.mirrored());
                if mirrored_at == at || !free(body, mirrored_at, size) {
                    continue;
                }
                Some((
                    part.id,
                    [
                        mirrored_at[0] - anchor[0],
                        mirrored_at[1] - anchor[1],
                        mirrored_at[2] - anchor[2],
                    ],
                ))
            } else {
                None
            };

            return Some(Growth {
                parent: part.id,
                offset: [at[0] - anchor[0], at[1] - anchor[1], at[2] - anchor[2]],
                yaw: Yaw::Zero,
                facing,
                role,
                mirror,
            });
        }
    }

    None
}

/// Full extent from a half-extent, never zero.
fn extent_of(half: [i32; 3]) -> [i32; 3] {
    [
        (half[0].abs() * 2).max(1),
        (half[1].abs() * 2).max(1),
        (half[2].abs() * 2).max(1),
    ]
}

/// Where a part of `size` sits when placed flush against a host's face.
///
/// The offset depends on the new part's size on the negative side, because a
/// part's local origin is its lowest corner rather than a pivot.
fn site(anchor: [i32; 3], host: [i32; 3], size: [i32; 3], facing: Facing) -> [i32; 3] {
    let (axis, sign) = facing.axis();
    let mut at = anchor;
    at[axis] += if sign > 0 { host[axis] } else { -size[axis] };
    at
}

/// Whether a box of `size` at `at` clears every existing part.
fn free(body: &BodyDocument, at: [i32; 3], size: [i32; 3]) -> bool {
    for part in &body.parts {
        let Some(anchor) = body.world_offset(part.id) else {
            continue;
        };
        let host = extent_of(part.half_extent);
        let overlaps = (0..3).all(|axis| {
            at[axis] < anchor[axis] + host[axis] && anchor[axis] < at[axis] + size[axis]
        });
        if overlaps {
            return false;
        }
    }
    true
}

/// Builds the attachment a [`Growth`] describes.
pub fn attachment(growth: &Growth) -> Attachment {
    Attachment { parent: growth.parent, offset: growth.offset, yaw: growth.yaw }
}

/// Builds the mirrored attachment, when there is one.
pub fn mirror_attachment(growth: &Growth) -> Option<Attachment> {
    growth
        .mirror
        .map(|(parent, offset)| Attachment { parent, offset, yaw: growth.yaw })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::{Provenance, SpeciesId, VolumeRef};
    use crate::plan::Symmetry;

    fn body() -> BodyDocument {
        BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 1_000, [3, 3, 3])
    }

    fn grow(body: &mut BodyDocument, half: [i32; 3], mass: u64) -> Option<Growth> {
        let growth = resolve(body, half)?;
        let parts = if growth.mirror.is_some() { 2 } else { 1 };
        let each = mass / parts;
        body.attach(
            VolumeRef::from_tag(9),
            each,
            half,
            attachment(&growth),
            Provenance::founding(),
        )
        .ok()?;
        if let Some(mirrored) = mirror_attachment(&growth) {
            body.attach(
                VolumeRef::from_tag(9),
                each,
                half,
                mirrored,
                Provenance::founding(),
            )
            .ok()?;
        }
        Some(growth)
    }

    #[test]
    fn a_limb_grows_to_the_side_and_brings_its_pair() {
        let mut body = body();
        let growth = grow(&mut body, [4, 1, 1], 400).expect("a limb fits");

        assert_eq!(growth.role, Role::Limb);
        assert!(growth.facing.is_lateral(), "limbs go out to the sides");
        assert!(growth.mirror.is_some(), "bilateral plans pair lateral growth");
        assert_eq!(body.len(), 3, "one meal, two limbs, plus the core");
    }

    #[test]
    fn a_mirrored_pair_splits_the_mass_it_came_from() {
        let mut body = body();
        let before = body.total_mass_mg();
        grow(&mut body, [4, 1, 1], 400).unwrap();
        assert_eq!(
            body.total_mass_mg(),
            before + 400,
            "the pair together weigh what was eaten"
        );
    }

    #[test]
    fn a_mirrored_body_is_balanced_across_its_lateral_axis() {
        let mut body = body();
        // The midline is the root's centre, not the origin, because a part's
        // position is its lowest corner. The root has half-extent 3, so it
        // spans 0..6 and its midline is x = 3.
        let midline = body.centre_of_mass()[0];
        assert_eq!(midline, 3);

        grow(&mut body, [4, 1, 1], 400).unwrap();
        assert_eq!(
            body.centre_of_mass()[0],
            midline,
            "a paired limb leaves the centre of mass on the midline"
        );
    }

    /// The regression for best-effort mirroring: a bilateral plan that grows a
    /// single limb whenever the mirror will not fit drifts the whole body
    /// sideways, one failed pair at a time.
    #[test]
    fn a_bilateral_body_never_drifts_off_its_midline() {
        let mut body = body();
        let midline = body.centre_of_mass()[0];

        for _ in 0..10 {
            if grow(&mut body, [4, 1, 1], 300).is_none() {
                break;
            }
        }

        assert!(body.len() >= 5, "the body must actually grow limbs");
        assert_eq!(
            body.centre_of_mass()[0],
            midline,
            "every lateral part is paired, so the body stays balanced"
        );
    }

    #[test]
    fn parts_never_overlap_however_much_is_eaten() {
        let mut body = body();
        for i in 0..12 {
            let half = if i % 3 == 0 { [4, 1, 1] } else { [2, 2, 2] };
            if grow(&mut body, half, 200).is_none() {
                break;
            }
        }
        assert!(body.len() > 6, "the fixture must actually grow");

        // Every pair of parts is disjoint.
        for a in &body.parts {
            for b in &body.parts {
                if a.id >= b.id {
                    continue;
                }
                let (pa, pb) = (
                    body.world_offset(a.id).unwrap(),
                    body.world_offset(b.id).unwrap(),
                );
                let (ea, eb) = (extent_of(a.half_extent), extent_of(b.half_extent));
                let overlaps = (0..3)
                    .all(|ax| pa[ax] < pb[ax] + eb[ax] && pb[ax] < pa[ax] + ea[ax]);
                assert!(!overlaps, "{:?} overlaps {:?}", a.id, b.id);
            }
        }
    }

    #[test]
    fn an_unmirrored_plan_grows_one_at_a_time() {
        let mut body = body();
        body.plan.symmetry = Symmetry::None;
        let growth = grow(&mut body, [4, 1, 1], 400).unwrap();
        assert!(growth.mirror.is_none());
        assert_eq!(body.len(), 2);
    }

    #[test]
    fn the_plan_decides_where_a_role_goes() {
        let mut body = body();
        body.plan.set_preference(Role::Limb, Facing::Above);
        body.plan.tolerance = 0;
        let growth = grow(&mut body, [4, 1, 1], 100).unwrap();
        assert_eq!(growth.facing, Facing::Above);
        assert!(growth.mirror.is_none(), "vertical growth does not pair");
    }

    #[test]
    fn growth_is_deterministic() {
        let run = || {
            let mut b = body();
            for _ in 0..6 {
                if grow(&mut b, [4, 1, 1], 200).is_none() {
                    break;
                }
            }
            b
        };
        assert_eq!(run(), run());
    }
}
