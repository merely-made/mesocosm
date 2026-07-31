// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Composing a whole body into one occupancy grid.
//!
//! The live renderer wants quads, one mesh per part, kept separate so limbs
//! can move. A **sprite baker wants the opposite**: a single grid it can
//! project voxel by voxel with depth sorting, because a baked sprite has no
//! moving parts.
//!
//! Both derive from the same body document, which is the actual shared organ.
//! An earlier note in the plan claimed Isometry's baker wanted the mesher's
//! quads; it does not. `isometry-voxel::bake_facing` takes an occupancy grid,
//! so this is the adapter that lane needs, and Isometry's baker requires no
//! change.

use mesocosm_core::{BodyDocument, Yaw};

use crate::{MeshError, Volume, VolumeSource, place_point};

/// A body composed into one grid, plus where that grid sits in body space.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flattened {
    pub volume: Volume,
    /// Body-space coordinate of the grid's `(0, 0, 0)` cell. Body space has
    /// negative coordinates; a grid does not.
    pub origin: [i32; 3],
}

impl Flattened {
    /// Reads a cell by body-space coordinate.
    pub fn at(&self, body_space: [i32; 3]) -> u8 {
        let local = [
            body_space[0] - self.origin[0],
            body_space[1] - self.origin[1],
            body_space[2] - self.origin[2],
        ];
        if local.iter().any(|c| *c < 0) {
            return 0;
        }
        self.volume
            .get(local[0] as u32, local[1] as u32, local[2] as u32)
    }
}

/// Composes every part of a body into a single occupancy grid.
///
/// Parts are written in document order, so a later part overwrites an earlier
/// one where they overlap. That is the right behaviour for a sprite: the grid
/// records what is solid, and the baker decides what is visible.
pub fn flatten(
    body: &BodyDocument,
    source: &impl VolumeSource,
) -> Result<Flattened, MeshError> {
    // Two passes: find the extent, then fill. A body's parts can sit anywhere,
    // and a grid sized to the union avoids both clipping and waste.
    let mut min = [i32::MAX; 3];
    let mut max = [i32::MIN; 3];
    let mut seen = false;

    for part in &body.parts {
        let (offset, yaw, volume) = resolve(body, source, part.id)?;
        for corner in volume_corners(volume) {
            let placed = place_point(corner, yaw, offset);
            for axis in 0..3 {
                min[axis] = min[axis].min(placed[axis]);
                max[axis] = max[axis].max(placed[axis]);
            }
            seen = true;
        }
    }

    if !seen {
        return Ok(Flattened {
            volume: Volume::empty([1, 1, 1]),
            origin: [0, 0, 0],
        });
    }

    let size = [
        (max[0] - min[0] + 1) as u32,
        (max[1] - min[1] + 1) as u32,
        (max[2] - min[2] + 1) as u32,
    ];
    let mut grid = Volume::empty(size);

    for part in &body.parts {
        let (offset, yaw, volume) = resolve(body, source, part.id)?;
        for z in 0..volume.size[2] {
            for y in 0..volume.size[1] {
                for x in 0..volume.size[0] {
                    let material = volume.get(x, y, z);
                    if material == 0 {
                        continue;
                    }
                    let placed =
                        place_point([x as i32, y as i32, z as i32], yaw, offset);
                    grid.set(
                        (placed[0] - min[0]) as u32,
                        (placed[1] - min[1]) as u32,
                        (placed[2] - min[2]) as u32,
                        material,
                    );
                }
            }
        }
    }

    Ok(Flattened { volume: grid, origin: min })
}

fn resolve<'a>(
    body: &BodyDocument,
    source: &'a impl VolumeSource,
    part: mesocosm_core::PartId,
) -> Result<([i32; 3], Yaw, &'a Volume), MeshError> {
    let offset = body
        .world_offset(part)
        .ok_or(MeshError::Unplaceable { part })?;
    let yaw = body.world_yaw(part).ok_or(MeshError::Unplaceable { part })?;
    let reference = body
        .part(part)
        .ok_or(MeshError::Unplaceable { part })?
        .volume;
    let volume = source
        .volume(reference)
        .ok_or(MeshError::MissingVolume { part, volume: reference })?;
    Ok((offset, yaw, volume))
}

/// The eight corners of a volume's occupied range, in local coordinates.
fn volume_corners(volume: &Volume) -> [[i32; 3]; 8] {
    let hi = [
        volume.size[0].saturating_sub(1) as i32,
        volume.size[1].saturating_sub(1) as i32,
        volume.size[2].saturating_sub(1) as i32,
    ];
    [
        [0, 0, 0],
        [hi[0], 0, 0],
        [0, hi[1], 0],
        [0, 0, hi[2]],
        [hi[0], hi[1], 0],
        [hi[0], 0, hi[2]],
        [0, hi[1], hi[2]],
        [hi[0], hi[1], hi[2]],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VolumeMap;
    use mesocosm_core::{Attachment, PartId, Provenance, SpeciesId, VolumeRef};

    fn source() -> VolumeMap {
        let mut map = VolumeMap::new();
        map.insert(VolumeRef::from_tag(1), Volume::solid([3, 3, 3], 1));
        map.insert(VolumeRef::from_tag(2), Volume::solid([2, 1, 1], 7));
        map
    }

    fn body() -> BodyDocument {
        BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 100, [1, 1, 1])
    }

    #[test]
    fn a_lone_body_flattens_to_its_own_volume() {
        let flat = flatten(&body(), &source()).unwrap();
        assert_eq!(flat.volume.size, [3, 3, 3]);
        assert_eq!(flat.origin, [0, 0, 0]);
        assert_eq!(flat.volume.solid_count(), 27);
    }

    #[test]
    fn an_attached_part_lands_at_its_offset() {
        let mut body = body();
        body.attach(
            VolumeRef::from_tag(2),
            10,
            [1, 1, 1],
            Attachment { parent: PartId(0), offset: [3, 0, 0], yaw: Yaw::Zero },
            Provenance::founding(),
        )
        .unwrap();

        let flat = flatten(&body, &source()).unwrap();
        // The arm occupies x = 3..5 in body space.
        assert_eq!(flat.at([3, 0, 0]), 7);
        assert_eq!(flat.at([4, 0, 0]), 7);
        assert_eq!(flat.at([2, 0, 0]), 1, "the core is still there");
        assert_eq!(flat.volume.size, [5, 3, 3]);
    }

    #[test]
    fn negative_placements_shift_the_origin_rather_than_clipping() {
        let mut body = body();
        body.attach(
            VolumeRef::from_tag(2),
            10,
            [1, 1, 1],
            Attachment { parent: PartId(0), offset: [-2, 0, 0], yaw: Yaw::Zero },
            Provenance::founding(),
        )
        .unwrap();

        let flat = flatten(&body, &source()).unwrap();
        assert_eq!(flat.origin, [-2, 0, 0]);
        assert_eq!(flat.at([-2, 0, 0]), 7, "nothing was clipped off the low side");
        assert_eq!(flat.at([0, 0, 0]), 1);
    }

    #[test]
    fn yaw_rotates_what_gets_written() {
        let mut body = body();
        body.attach(
            VolumeRef::from_tag(2),
            10,
            [1, 1, 1],
            Attachment { parent: PartId(0), offset: [0, 0, 0], yaw: Yaw::Quarter },
            Provenance::founding(),
        )
        .unwrap();

        let flat = flatten(&body, &source()).unwrap();
        // The 2x1x1 arm points along +x locally; a quarter turn sends it to -z.
        assert_eq!(flat.at([0, 0, -1]), 7);
    }

    #[test]
    fn every_solid_voxel_survives_when_parts_do_not_overlap() {
        let mut body = body();
        body.attach(
            VolumeRef::from_tag(2),
            10,
            [1, 1, 1],
            Attachment { parent: PartId(0), offset: [3, 0, 0], yaw: Yaw::Zero },
            Provenance::founding(),
        )
        .unwrap();
        let flat = flatten(&body, &source()).unwrap();
        assert_eq!(flat.volume.solid_count(), 27 + 2);
    }

    #[test]
    fn flattening_is_deterministic() {
        let mut body = body();
        body.attach(
            VolumeRef::from_tag(2),
            10,
            [1, 1, 1],
            Attachment { parent: PartId(0), offset: [4, 1, -2], yaw: Yaw::Half },
            Provenance::founding(),
        )
        .unwrap();
        assert_eq!(
            flatten(&body, &source()).unwrap(),
            flatten(&body, &source()).unwrap()
        );
    }

    #[test]
    fn a_missing_volume_is_reported() {
        let mut body = body();
        body.attach(
            VolumeRef::from_tag(99),
            10,
            [1, 1, 1],
            Attachment { parent: PartId(0), offset: [3, 0, 0], yaw: Yaw::Zero },
            Provenance::founding(),
        )
        .unwrap();
        assert!(matches!(
            flatten(&body, &source()),
            Err(MeshError::MissingVolume { .. })
        ));
    }
}
