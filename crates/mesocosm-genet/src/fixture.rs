// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The volumes and placement rules the host draws with.
//!
//! Placeholder content until authored parts exist. It lives in the host
//! because it is presentation-adjacent scaffolding, not world truth: the core
//! carries content addresses and knows nothing about what a volume looks like.

use mesocosm_core::{
    Intent, OrganismId, PartPalette, Role, Route, VolumeRef, World, world::organism_extent,
};
use mesocosm_mesh::{Volume, VolumeMap};

pub fn volumes() -> VolumeMap {
    let mut map = VolumeMap::new();

    // Every template the world can develop a part from, sized from the
    // palette itself. Enumerating roles rather than hardcoding tags is what
    // keeps this table from silently falling behind the core: when the
    // palette grew a sensor, a literal list quietly stopped resolving and
    // `mesh_body` failed with `MissingVolume`.
    let palette = PartPalette::primitive();
    for role in [Role::Mass, Role::Limb, Role::Plate, Role::Sensor] {
        let template = palette.template(role);
        let size = template.half_extent.map(|half| (half * 2).max(1) as u32);
        // `from_tag` puts the tag in byte zero; the material is only a
        // palette index for the placeholder look.
        map.insert(template.volume, Volume::solid(size, template.volume.0[0]));
    }

    for tag in 16..24u8 {
        // A volume is exactly the extent the core placed with, or the picture
        // and the physics would disagree about the same part.
        let half = organism_extent(tag);
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
/// Explicit placement still exists as `Route::Place` for an editor, but
/// automatic and symmetric is the resting state.
pub fn metabolize(
    _world: &World,
    organism: OrganismId,
    _volumes: &VolumeMap,
    route: Route,
) -> Intent {
    Intent::Metabolize { organism, route }
}

/// One step toward the nearest thing worth eating, or `None` when there is
/// nothing left or it is already in reach.
pub fn toward_prey(world: &World) -> Option<[i32; 3]> {
    let here = world.position()?;
    let at = world
        .organisms
        .iter()
        .filter(|m| Some(m.id) != world.controlled_id() && m.is_alive())
        .map(|m| m.position)
        .min_by_key(|at: &[i32; 3]| (0..3).map(|a| (at[a] - here[a]).abs()).max().unwrap_or(0))?;

    let step = [0, 1, 2].map(|a| (at[a] - here[a]).signum());
    if step == [0, 0, 0] { None } else { Some(step) }
}

pub fn reachable(world: &World) -> Option<OrganismId> {
    world
        .organisms
        .iter()
        // Never offer the critter itself. Since P1 the played organism is in
        // this vector like everything else, so it is a candidate unless it is
        // filtered out.
        .filter(|m| Some(m.id) != world.controlled_id() && m.is_alive())
        // Anatomy decides how far this critter can touch, not a constant. A
        // stubby one reaches about three voxels; a limbed one reaches further.
        .filter(|m| world.in_reach(m.position))
        .map(|m| m.id)
        .min()
}

#[cfg(test)]
mod tests {
    use super::*;
    use mesocosm_core::Placement;
    use mesocosm_mesh::{VolumeSource, flatten};

    #[test]
    fn the_fixture_resolves_every_volume_a_world_mints() {
        let volumes = volumes();
        let world = World::new(7, 40);
        assert!(mesocosm_mesh::mesh_body(world.body().unwrap(), &volumes).is_ok());
        for organism in &world.organisms {
            assert!(volumes.volume(organism.volume()).is_some());
        }
    }

    /// The regression this policy exists for: a well-fed critter used to
    /// collapse into a pile because placement cycled six faces and the seventh
    /// part landed on the first.
    #[test]
    fn parts_never_land_on_top_of_each_other() {
        let volumes = volumes();
        let mut world = World::new(2024, 80);

        // Walk to prey rather than assuming it is adjacent: reach is anatomy
        // now, and a starting critter touches very little.
        let mut meals = 0;
        for _ in 0..800 {
            if meals >= 14 {
                break;
            }
            if let Some(target) = reachable(&world) {
                world.apply(metabolize(
                    &world,
                    target,
                    &volumes,
                    Route::Incorporate { placement: Placement::Planned },
                ));
                meals += 1;
                continue;
            }
            let Some(here) = world.position() else { break };
            let Some(at) = world
                .organisms
                .iter()
                .filter(|m| Some(m.id) != world.controlled_id() && m.is_alive())
                .map(|m| m.position)
                .min_by_key(|at: &[i32; 3]| {
                    (0..3).map(|a| (at[a] - here[a]).abs()).max().unwrap_or(0)
                })
            else {
                break;
            };
            world.apply(Intent::Move { delta: [0, 1, 2].map(|a| (at[a] - here[a]).signum()) });
        }

        assert!(world.body().unwrap().len() > 6, "the fixture must out-eat one face cycle");

        // Every solid voxel is accounted for exactly once: nothing overlaps.
        let flat = flatten(world.body().unwrap(), &volumes).unwrap();
        let expected: usize = world
            .body()
            .unwrap()
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
                world.apply(metabolize(&world, target, &volumes, Route::Incorporate { placement: Placement::Planned }));
            }
            world.body().unwrap().clone()
        };
        assert_eq!(grow(), grow());
    }
}
