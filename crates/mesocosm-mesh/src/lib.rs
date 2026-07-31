// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Turns a body document into geometry.
//!
//! A projection, not a renderer. It emits quads in integer voxel space and a
//! host turns those into vertex buffers. Nothing here touches a GPU.
//!
//! **Two projections, not one.** A live renderer wants quads, meshed per part
//! and kept separate so limbs can move. A sprite baker wants the opposite: one
//! occupancy grid to project voxel by voxel, since a baked sprite has no moving
//! parts. [`flatten`] serves that lane. The shared organ is the body *document*,
//! not the quads.
//!
//! # The shape
//!
//! ```text
//! BodyDocument ──▶ placements ──▶ (PartMesh, offset, yaw) per part
//!      │                              ▲
//!      └── VolumeRef ──▶ VolumeSource ┘   meshed once, reused per placement
//! ```
//!
//! Parts are rigid and meshed individually, so a part's geometry depends only
//! on its volume. That is what makes attachment cheap: incorporating a part
//! adds a placement and, if the volume is new, one mesh. Nothing already on
//! the body is remeshed.
//!
//! # What this exists to test
//!
//! The body pipeline plan names runtime attachment as the wing's unproven
//! assumption: a part attaching during play, gaining collision and mass,
//! moving the centre of balance, and staying legible. The core proves the mass
//! and collision half. This crate proves the geometry half, and
//! [`MeshError::MissingVolume`] is deliberately loud rather than a silent skip,
//! so a part that cannot be drawn is a reported failure and not an invisible
//! critter.

pub mod flatten;
pub mod greedy;
pub mod volume;

use std::collections::BTreeMap;

use mesocosm_core::{BodyDocument, PartId, VolumeRef, Yaw};

pub use flatten::{Flattened, flatten};
pub use greedy::{PartMesh, Quad, mesh_volume, mesh_volume_naive};
pub use volume::{Volume, VolumeError, VolumeMap, VolumeSource};

/// One part's mesh, placed in body space.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placement {
    pub part: PartId,
    pub volume: VolumeRef,
    /// Offset in body space, from [`BodyDocument::world_offset`].
    pub offset: [i32; 3],
    /// Orientation in body space, from [`BodyDocument::world_yaw`].
    pub yaw: Yaw,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeshError {
    /// A part refers to a volume the source cannot resolve. Loud on purpose.
    MissingVolume { part: PartId, volume: VolumeRef },
    /// A part's attachment chain is malformed, so it has no body-space
    /// position. Constructors prevent this; a deserialized document might not.
    Unplaceable { part: PartId },
}

/// A body's geometry: one mesh per distinct volume, and one placement per part.
#[derive(Clone, Debug, Default)]
pub struct BodyMesh {
    /// Keyed by volume address, so a repeated part is meshed once.
    meshes: BTreeMap<[u8; 32], PartMesh>,
    pub placements: Vec<Placement>,
}

impl BodyMesh {
    pub fn mesh_for(&self, reference: VolumeRef) -> Option<&PartMesh> {
        self.meshes.get(&reference.0)
    }

    /// Distinct meshes built. Lower than [`Self::placement_count`] when parts
    /// share a volume.
    pub fn mesh_count(&self) -> usize {
        self.meshes.len()
    }

    pub fn placement_count(&self) -> usize {
        self.placements.len()
    }

    /// Total quads that would be drawn, counting each placement.
    pub fn drawn_quads(&self) -> usize {
        self.placements
            .iter()
            .filter_map(|p| self.mesh_for(p.volume))
            .map(|m| m.len())
            .sum()
    }

    /// The body's extent in body space, derived from placed geometry rather
    /// than from the document's declared half-extents.
    ///
    /// Returns `None` for a body whose parts are all empty volumes.
    pub fn bounds(&self) -> Option<([i32; 3], [i32; 3])> {
        let mut min = [i32::MAX; 3];
        let mut max = [i32::MIN; 3];
        let mut seen = false;

        for placement in &self.placements {
            let Some(mesh) = self.mesh_for(placement.volume) else {
                continue;
            };
            for quad in &mesh.quads {
                for corner in quad.corners() {
                    let placed = place_point(corner, placement.yaw, placement.offset);
                    for axis in 0..3 {
                        min[axis] = min[axis].min(placed[axis]);
                        max[axis] = max[axis].max(placed[axis]);
                    }
                    seen = true;
                }
            }
        }

        seen.then_some((min, max))
    }
}

/// Applies a placement's rotation and offset to a point in part-local space.
///
/// Integer-only, like everything the core hands over, so geometry derived on
/// two machines is identical.
pub fn place_point(point: [i32; 3], yaw: Yaw, offset: [i32; 3]) -> [i32; 3] {
    let rotated = rotate(point, yaw);
    [
        rotated[0] + offset[0],
        rotated[1] + offset[1],
        rotated[2] + offset[2],
    ]
}

fn rotate(v: [i32; 3], yaw: Yaw) -> [i32; 3] {
    let [x, y, z] = v;
    match yaw {
        Yaw::Zero => [x, y, z],
        Yaw::Quarter => [z, y, -x],
        Yaw::Half => [-x, y, -z],
        Yaw::ThreeQuarter => [-z, y, x],
    }
}

/// Builds a body's geometry, meshing each distinct volume once.
pub fn mesh_body(body: &BodyDocument, source: &impl VolumeSource) -> Result<BodyMesh, MeshError> {
    let mut result = BodyMesh::default();

    for part in &body.parts {
        let Some(offset) = body.world_offset(part.id) else {
            return Err(MeshError::Unplaceable { part: part.id });
        };
        let Some(yaw) = body.world_yaw(part.id) else {
            return Err(MeshError::Unplaceable { part: part.id });
        };
        let Some(volume) = source.volume(part.volume) else {
            return Err(MeshError::MissingVolume { part: part.id, volume: part.volume });
        };

        result
            .meshes
            .entry(part.volume.0)
            .or_insert_with(|| mesh_volume(volume));

        result.placements.push(Placement {
            part: part.id,
            volume: part.volume,
            offset,
            yaw,
        });
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mesocosm_core::{Attachment, Provenance, SpeciesId};

    fn source() -> VolumeMap {
        let mut map = VolumeMap::new();
        map.insert(VolumeRef::from_tag(1), Volume::solid([3, 3, 3], 1));
        map.insert(VolumeRef::from_tag(2), Volume::solid([2, 1, 1], 2));
        map
    }

    fn seed_body() -> BodyDocument {
        BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 1_000, [1, 1, 1])
    }

    #[test]
    fn a_lone_body_meshes_to_one_placement() {
        let mesh = mesh_body(&seed_body(), &source()).unwrap();
        assert_eq!(mesh.placement_count(), 1);
        assert_eq!(mesh.mesh_count(), 1);
        assert_eq!(mesh.drawn_quads(), 6);
    }

    #[test]
    fn attaching_a_part_adds_geometry_and_grows_the_bounds() {
        let mut body = seed_body();
        let before = mesh_body(&body, &source()).unwrap();
        let (_, max_before) = before.bounds().unwrap();

        body.attach(
            VolumeRef::from_tag(2),
            500,
            [1, 1, 1],
            Attachment { parent: body.root, offset: [8, 0, 0], yaw: Yaw::Zero },
            Provenance::founding(),
        )
        .unwrap();

        let after = mesh_body(&body, &source()).unwrap();
        let (_, max_after) = after.bounds().unwrap();

        assert_eq!(after.placement_count(), 2);
        assert!(after.drawn_quads() > before.drawn_quads());
        assert!(
            max_after[0] > max_before[0],
            "the new part must extend the drawn body"
        );
    }

    #[test]
    fn repeated_volumes_are_meshed_once() {
        let mut body = seed_body();
        for i in 0..4 {
            body.attach(
                VolumeRef::from_tag(2),
                100,
                [1, 1, 1],
                Attachment {
                    parent: body.root,
                    offset: [5 + i * 3, 0, 0],
                    yaw: Yaw::Zero,
                },
                Provenance::founding(),
            )
            .unwrap();
        }
        let mesh = mesh_body(&body, &source()).unwrap();
        assert_eq!(mesh.placement_count(), 5);
        assert_eq!(mesh.mesh_count(), 2, "two distinct volumes, five placements");
    }

    #[test]
    fn a_parts_mesh_does_not_depend_on_where_it_is_attached() {
        let build = |offset: [i32; 3], yaw: Yaw| {
            let mut body = seed_body();
            body.attach(
                VolumeRef::from_tag(2),
                100,
                [1, 1, 1],
                Attachment { parent: body.root, offset, yaw },
                Provenance::founding(),
            )
            .unwrap();
            mesh_body(&body, &source()).unwrap()
        };

        let near = build([4, 0, 0], Yaw::Zero);
        let far = build([40, 0, 12], Yaw::Half);

        assert_eq!(
            near.mesh_for(VolumeRef::from_tag(2)),
            far.mesh_for(VolumeRef::from_tag(2)),
            "geometry is cacheable by volume, independent of placement"
        );
    }

    #[test]
    fn rotation_moves_placed_geometry() {
        let mut body = seed_body();
        body.attach(
            VolumeRef::from_tag(2),
            100,
            [1, 1, 1],
            Attachment { parent: body.root, offset: [6, 0, 0], yaw: Yaw::Quarter },
            Provenance::founding(),
        )
        .unwrap();
        let mesh = mesh_body(&body, &source()).unwrap();
        let placement = &mesh.placements[1];
        assert_eq!(placement.yaw, Yaw::Quarter);

        // The 2x1x1 arm points along +x locally; a quarter turn sends it to -z.
        let tip = place_point([2, 0, 0], placement.yaw, placement.offset);
        assert_eq!(tip, [6, 0, -2]);
    }

    #[test]
    fn a_missing_volume_is_reported_not_skipped() {
        let mut body = seed_body();
        body.attach(
            VolumeRef::from_tag(99),
            100,
            [1, 1, 1],
            Attachment { parent: body.root, offset: [4, 0, 0], yaw: Yaw::Zero },
            Provenance::founding(),
        )
        .unwrap();
        let err = mesh_body(&body, &source()).unwrap_err();
        assert_eq!(
            err,
            MeshError::MissingVolume { part: PartId(1), volume: VolumeRef::from_tag(99) }
        );
    }

    #[test]
    fn meshing_a_body_is_deterministic() {
        let mut body = seed_body();
        body.attach(
            VolumeRef::from_tag(2),
            100,
            [1, 1, 1],
            Attachment { parent: body.root, offset: [7, 1, -2], yaw: Yaw::ThreeQuarter },
            Provenance::founding(),
        )
        .unwrap();
        let a = mesh_body(&body, &source()).unwrap();
        let b = mesh_body(&body, &source()).unwrap();
        assert_eq!(a.placements, b.placements);
        assert_eq!(
            a.mesh_for(VolumeRef::from_tag(2)),
            b.mesh_for(VolumeRef::from_tag(2))
        );
    }
}
