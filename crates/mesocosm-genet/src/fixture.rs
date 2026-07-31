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
use mesocosm_mesh::{Flattened, Volume, VolumeMap, VolumeSource, flatten};

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

/// Builds a metabolize intent that places the eaten part in the first free
/// spot flush against an existing part.
///
/// **This replaced a six-face cycle that stacked parts on top of each other.**
/// The old policy placed by `body.len() % 6`, so the seventh part landed
/// exactly on the first, and a well-fed critter collapsed into a z-fighting
/// pile. Growth has to know what space is already occupied.
///
/// Candidates are tried in part order, then face order, so growth branches
/// outward from the core and stays deterministic. Offsets are derived from the
/// eaten volume's size because a part's local origin is its lowest corner, not
/// a pivot (see the body pipeline plan's findings). Yaw stays zero for the same
/// reason: rotation turns a part about its corner, so a rotated limb would
/// swing off the joint it was flush to.
///
/// Placement policy lives here rather than in the core because the core takes
/// an explicit `(parent, offset, yaw)` and only validates it. A real growth
/// system belongs in a designed layer; this is scaffolding that has to not lie.
pub fn metabolize(world: &World, morsel: MorselId, volumes: &VolumeMap) -> Intent {
    let size = world
        .morsels
        .iter()
        .find(|m| m.id == morsel)
        .and_then(|m| volumes.volume(m.volume))
        .map(|v| [v.size[0] as i32, v.size[1] as i32, v.size[2] as i32])
        .unwrap_or([2, 2, 2]);

    let occupied = flatten(&world.body, volumes).ok();

    for part in &world.body.parts {
        let Some(anchor) = world.body.world_offset(part.id) else {
            continue;
        };
        let Some(host) = volumes.volume(part.volume) else {
            continue;
        };
        let host_size = [
            host.size[0] as i32,
            host.size[1] as i32,
            host.size[2] as i32,
        ];

        for face in 0..6 {
            let at = match face {
                0 => [anchor[0] + host_size[0], anchor[1], anchor[2]],
                1 => [anchor[0] - size[0], anchor[1], anchor[2]],
                2 => [anchor[0], anchor[1], anchor[2] + host_size[2]],
                3 => [anchor[0], anchor[1], anchor[2] - size[2]],
                4 => [anchor[0], anchor[1] + host_size[1], anchor[2]],
                _ => [anchor[0], anchor[1] - size[1], anchor[2]],
            };

            if fits(occupied.as_ref(), at, size) {
                // Yaws are all zero, so a child's world position is its
                // parent's plus the attachment offset.
                let offset = [
                    at[0] - anchor[0],
                    at[1] - anchor[1],
                    at[2] - anchor[2],
                ];
                return Intent::Metabolize {
                    morsel,
                    parent: part.id,
                    offset,
                    yaw: Yaw::Zero,
                };
            }
        }
    }

    // Nothing flush was free. Park it clear of the body rather than inside it.
    let reach = occupied
        .as_ref()
        .map(|f| f.origin[0] + f.volume.size[0] as i32 + 1)
        .unwrap_or(6);
    Intent::Metabolize {
        morsel,
        parent: PartId(0),
        offset: [reach, 0, 0],
        yaw: Yaw::Zero,
    }
}

/// Whether a box of `size` at `at` would land in empty space.
fn fits(occupied: Option<&Flattened>, at: [i32; 3], size: [i32; 3]) -> bool {
    let Some(flat) = occupied else { return true };
    for z in 0..size[2] {
        for y in 0..size[1] {
            for x in 0..size[0] {
                if flat.at([at[0] + x, at[1] + y, at[2] + z]) != 0 {
                    return false;
                }
            }
        }
    }
    true
}

pub fn reachable(world: &World) -> Option<MorselId> {
    world
        .morsels
        .iter()
        .filter(|m| (0..3).all(|a| (m.position[a] - world.position[a]).abs() <= 8))
        .map(|m| m.id)
        .min()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fixture_resolves_every_volume_a_world_mints() {
        let volumes = volumes();
        let world = World::new(7, 40);
        assert!(mesocosm_mesh::mesh_body(&world.body, &volumes).is_ok());
        for morsel in &world.morsels {
            assert!(volumes.volume(morsel.volume).is_some());
        }
    }

    /// The regression this policy exists for: a well-fed critter used to
    /// collapse into a pile because placement cycled six faces and the seventh
    /// part landed on the first.
    #[test]
    fn parts_never_land_on_top_of_each_other() {
        let volumes = volumes();
        let mut world = World::new(2024, 80);

        for _ in 0..14 {
            let Some(target) = reachable(&world) else { break };
            world.apply(metabolize(&world, target, &volumes));
        }

        assert!(world.body.len() > 6, "the fixture must out-eat one face cycle");

        // Every solid voxel is accounted for exactly once: nothing overlaps.
        let flat = flatten(&world.body, &volumes).unwrap();
        let expected: usize = world
            .body
            .parts
            .iter()
            .filter_map(|p| volumes.volume(p.volume))
            .map(|v| v.solid_count())
            .sum();
        assert_eq!(
            flat.volume.solid_count(),
            expected,
            "overlapping parts would lose voxels to overwriting"
        );
    }

    #[test]
    fn growth_is_deterministic() {
        let volumes = volumes();
        let grow = || {
            let mut world = World::new(99, 60);
            for _ in 0..8 {
                let Some(target) = reachable(&world) else { break };
                world.apply(metabolize(&world, target, &volumes));
            }
            world.body
        };
        assert_eq!(grow(), grow());
    }
}
