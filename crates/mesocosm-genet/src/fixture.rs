// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The volumes and placement rules the host draws with.
//!
//! Placeholder content until authored parts exist. It lives in the host
//! because it is presentation-adjacent scaffolding, not world truth: the core
//! carries content addresses and knows nothing about what a volume looks like.

use mesocosm_core::{Intent, MorselId, PartId, VolumeRef, World, Yaw};
use mesocosm_mesh::{Volume, VolumeMap};

/// The founding body's extent, in voxels.
pub const CORE: i32 = 5;

pub fn volumes() -> VolumeMap {
    let mut map = VolumeMap::new();
    map.insert(VolumeRef::from_tag(1), Volume::solid([5, 5, 5], 1));
    for tag in 16..24u8 {
        map.insert(VolumeRef::from_tag(tag), Volume::solid(extent_of(tag), tag));
    }
    map.insert(VolumeRef::from_tag(64), Volume::solid([1, 1, 1], 5));
    map
}

fn extent_of(tag: u8) -> [u32; 3] {
    match tag % 4 {
        0 => [3, 2, 2],
        1 => [2, 4, 2],
        2 => [2, 2, 5],
        _ => [3, 3, 2],
    }
}

fn extent_i32(reference: VolumeRef) -> [i32; 3] {
    extent_of(reference.0[0]).map(|d| d as i32)
}

/// The nearest morsel the critter could eat, if any.
pub fn reachable(world: &World) -> Option<MorselId> {
    world
        .morsels
        .iter()
        .filter(|m| (0..3).all(|a| (m.position[a] - world.position[a]).abs() <= 8))
        .map(|m| m.id)
        .min()
}

/// Builds a metabolize intent that places the eaten part flush against a face
/// of the core, cycling by how many parts the body already has.
///
/// Offsets are derived from the eaten volume's size because **a part's local
/// origin is its lowest corner, not a pivot** (see the body pipeline plan's
/// findings). Yaw stays zero for the same reason: rotation turns a part about
/// its corner, so a rotated limb would swing off the joint it was flush to.
pub fn metabolize(world: &World, morsel: MorselId) -> Intent {
    let size = world
        .morsels
        .iter()
        .find(|m| m.id == morsel)
        .map(|m| extent_i32(m.volume))
        .unwrap_or([2, 2, 2]);

    let face = world.body.len() % 6;
    let offset = match face {
        0 => [CORE, 1, 1],
        1 => [-size[0], 1, 1],
        2 => [1, 1, CORE],
        3 => [1, 1, -size[2]],
        4 => [1, CORE, 1],
        _ => [1, -size[1], 1],
    };

    Intent::Metabolize { morsel, parent: PartId(0), offset, yaw: Yaw::Zero }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fixture_resolves_every_volume_a_world_mints() {
        let volumes = volumes();
        let world = World::new(7, 40);
        let mesh = mesocosm_mesh::mesh_body(&world.body, &volumes);
        assert!(mesh.is_ok(), "the founding body must resolve");
        for morsel in &world.morsels {
            assert!(
                mesocosm_mesh::VolumeSource::volume(&volumes, morsel.volume).is_some(),
                "morsel volume {:?} is unresolvable",
                morsel.volume
            );
        }
    }

    #[test]
    fn metabolize_places_parts_flush_and_cycles_faces() {
        let mut world = World::new(11, 40);
        let mut seen = Vec::new();
        for _ in 0..4 {
            let Some(target) = reachable(&world) else { break };
            let intent = metabolize(&world, target);
            if let Intent::Metabolize { offset, .. } = intent {
                seen.push(offset);
            }
            world.apply(intent);
        }
        assert!(seen.len() >= 2, "the fixture should place several parts");
        assert_ne!(seen[0], seen[1], "successive parts take different faces");
    }
}
