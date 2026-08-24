// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Ephemeral target views for one ecology tick.
//!
//! Positions and live body shapes are copied before any organism changes, so
//! every decision sees the same enclosure. None of this is stored world state.

use std::collections::BTreeMap;

use crate::organism::{Kingdom, Organism, OrganismId, Signal};
use crate::places::{Ground, Tier, WalkerShape, spot_for};

/// An embodied mind's local visual horizon. Anatomy changes the sight ray,
/// while this bound keeps terrain sight local.
const NEAR_SIGHT_RANGE: i32 = 8;
/// Target-query cells are finer than crowding cells: they bound perception
/// work, while crowding deliberately groups a wider ecological neighbourhood.
const SENSORY_CELL: i32 = 4;

#[derive(Clone, Copy, Debug)]
pub(in crate::organism::ecology) struct LivingTarget {
    pub(in crate::organism::ecology) id: OrganismId,
    pub(in crate::organism::ecology) position: [i32; 3],
    pub(in crate::organism::ecology) organism_index: usize,
    pub(in crate::organism::ecology) kingdom: Kingdom,
    pub(in crate::organism::ecology) mass_mg: u64,
    pub(in crate::organism::ecology) signal: Signal,
    pub(in crate::organism::ecology) shape: WalkerShape,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::organism::ecology) struct CarrionTarget {
    pub(in crate::organism::ecology) position: [i32; 3],
    pub(in crate::organism::ecology) organism_index: usize,
    pub(in crate::organism::ecology) shape: WalkerShape,
}

pub(super) type Cells = BTreeMap<(i32, i32), Vec<usize>>;

fn sensory_cell(position: [i32; 3]) -> (i32, i32) {
    (
        position[0].div_euclid(SENSORY_CELL),
        position[2].div_euclid(SENSORY_CELL),
    )
}

pub(in crate::organism::ecology) fn living_cells(living: &[LivingTarget]) -> Cells {
    cells(living.iter().map(|target| target.position))
}

pub(in crate::organism::ecology) fn carrion_cells(carrion: &[CarrionTarget]) -> Cells {
    cells(carrion.iter().map(|target| target.position))
}

fn cells(positions: impl Iterator<Item = [i32; 3]>) -> Cells {
    let mut cells = Cells::new();
    for (index, position) in positions.enumerate() {
        cells.entry(sensory_cell(position)).or_default().push(index);
    }
    cells
}

/// Candidate indexes in the horizontal cells intersecting a local range.
/// Buckets are BTree-ordered; caller tie-breaks retain original vector order.
pub(super) fn nearby_indexes(
    cells: &Cells,
    position: [i32; 3],
    range: i32,
) -> impl Iterator<Item = usize> + '_ {
    let (min_x, min_z) = sensory_cell([position[0] - range, position[1], position[2] - range]);
    let (max_x, max_z) = sensory_cell([position[0] + range, position[1], position[2] + range]);
    cells
        .range((min_x, i32::MIN)..=(max_x, i32::MAX))
        .filter(move |((_, z), _)| *z >= min_z && *z <= max_z)
        .flat_map(|(_, indexes)| indexes.iter().copied())
}

pub(super) fn sight_range(organism: &Organism, reach: i32, ground: Option<&Ground>) -> i32 {
    if organism.tier == Tier::Near && ground.is_some() {
        reach.min(NEAR_SIGHT_RANGE)
    } else {
        reach
    }
}

pub(super) fn can_perceive(
    organism: &Organism,
    observer_shape: WalkerShape,
    target: &LivingTarget,
    range: i32,
    ground: Option<&Ground>,
) -> bool {
    can_perceive_position(
        organism,
        observer_shape,
        target.position,
        target.shape,
        range,
        ground,
    )
}

pub(super) fn can_perceive_position(
    organism: &Organism,
    observer_shape: WalkerShape,
    target: [i32; 3],
    target_shape: WalkerShape,
    range: i32,
    ground: Option<&Ground>,
) -> bool {
    match (organism.tier, ground) {
        (Tier::Near, Some(ground)) => spot_for(
            ground,
            observer_shape,
            organism.position,
            target_shape,
            target,
            range,
        ),
        _ => true,
    }
}
