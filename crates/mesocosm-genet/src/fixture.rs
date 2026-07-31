// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The volumes and placement rules the host draws with.
//!
//! Placeholder content until authored parts exist. It lives in the host
//! because it is presentation-adjacent scaffolding, not world truth: the core
//! carries content addresses and knows nothing about what a volume looks like.

use mesocosm_core::{Intent, MorselId, VolumeRef, World, world::morsel_extent};
use mesocosm_mesh::{Volume, VolumeMap};

pub fn volumes() -> VolumeMap {
    let mut map = VolumeMap::new();
    map.insert(VolumeRef::from_tag(1), Volume::solid([4, 4, 4], 1));
    for tag in 16..24u8 {
        // A volume is exactly the extent the core placed with, or the picture
        // and the physics would disagree about the same part.
        let half = morsel_extent(tag);
        let size = [
            (half[0] * 2).max(1) as u32,
            (half[1] * 2).max(1) as u32,
            (half[2] * 2).max(1) as u32,
        ];
        map.insert(VolumeRef::from_tag(tag), Volume::solid(size, tag));
    }
    map.insert(VolumeRef::from_tag(64), Volume::solid([1, 1, 1], 5));
    map
}

/// Eating, the default way: the body plan decides where the part goes.
///
/// Explicit placement still exists on `Intent::Metabolize` for an editor, but
/// automatic and symmetric is the resting state.
pub fn metabolize(_world: &World, morsel: MorselId, _volumes: &VolumeMap) -> Intent {
    Intent::Incorporate { morsel }
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
    use mesocosm_mesh::{VolumeSource, flatten};

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
