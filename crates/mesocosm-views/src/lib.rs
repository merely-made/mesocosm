// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's view layer, adapter-first.
//!
//! The stack's projection posture, adopted from birth rather than migrated to:
//! this crate **discloses** world facts as a [`sceno::Score`], `scenomise`
//! solves placement, and the paint leaves here realize the solved scene. The
//! layout debt isometry carried (hand-rolled positions the proofs plan
//! schedules for deletion) is never incurred.
//!
//! What a scene element *means* stays on this side of the contract, per the
//! per-vessel ruling: sceno carries geometry and source references; this crate
//! decides that a region's tint is the lineage that dominates it.

pub mod leaf;
pub mod minimap;

pub use leaf::MinimapLeaf;
pub use minimap::{
    MINIMAP_ADAPTER, dominant_lineages, lineage_tint, minimap_leaf, minimap_scene,
    minimap_score,
};
