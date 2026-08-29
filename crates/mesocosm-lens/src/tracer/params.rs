// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The two uniform blocks the trace pass reads: the frame's own parameters
//! (camera, space, grade, single pose) and the pose roster beside them.

use bytemuck::{Pod, Zeroable};
use modulus::BrickTraceSpace;

use super::types::{BrickChange, BrickFrameInput, BrickTraceError};
use crate::{CritterPose, MAX_CAPSULES, MAX_ROSTER, MAX_ROSTER_CAPSULES};

pub(super) const IDENTITY: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(super) struct TraceParams {
    pub camera: super::TraceCamera,
    pub space: BrickTraceSpace,
    pub fog: [f32; 4],
    pub look: [f32; 4],
    /// Column-major, identity when the frame carries no depth join.
    pub clip_from_world: [[f32; 4]; 4],
    pub critter: CritterParams,
}

/// The frame's single full-fidelity pose. Unchanged since the tracer's first
/// cut, and left alone by the roster: `MAX_CAPSULES` capsules, one tint, one
/// bounds sphere, two eyes.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(super) struct CritterParams {
    pub bounds: [f32; 4],
    pub tint_count: [f32; 4],
    pub eyes: [[f32; 4]; 2],
    pub pairs: [[f32; 4]; MAX_CAPSULES * 2],
}

/// One roster member: a background silhouette, so no eyes and the reduced
/// capsule budget the cap arithmetic in [`crate::MAX_ROSTER`] buys.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(super) struct RosterPose {
    pub bounds: [f32; 4],
    pub tint_count: [f32; 4],
    pub pairs: [[f32; 4]; MAX_ROSTER_CAPSULES * 2],
}

/// Bytes of roster header before the first member. `count.x` members follow.
pub(super) const ROSTER_HEADER_BYTES: u64 = 16;

/// The whole roster binding, headroom included.
pub(super) const ROSTER_BUFFER_BYTES: u64 =
    ROSTER_HEADER_BYTES + (MAX_ROSTER * size_of::<RosterPose>()) as u64;

impl TraceParams {
    pub(super) fn from_input(input: BrickFrameInput<'_>) -> Self {
        Self {
            camera: input.camera,
            space: BrickTraceSpace::from_map(input.map),
            fog: [
                input.grade.fog[0],
                input.grade.fog[1],
                input.grade.fog[2],
                input.grade.fog_start,
            ],
            look: [
                input.grade.dither,
                input.grade.fog_bands,
                input.grade.palette_len as f32,
                0.0,
            ],
            clip_from_world: input.clip_from_world.unwrap_or(IDENTITY),
            critter: CritterParams::from_pose(input.pose),
        }
    }
}

impl CritterParams {
    fn from_pose(pose: Option<&CritterPose>) -> Self {
        let Some(pose) = pose else {
            return Self::zeroed();
        };
        let mut pairs = [[0.0; 4]; MAX_CAPSULES * 2];
        for (index, capsule) in pose.capsules.iter().enumerate() {
            pairs[index * 2] = [capsule.a[0], capsule.a[1], capsule.a[2], capsule.ra];
            pairs[index * 2 + 1] = [capsule.b[0], capsule.b[1], capsule.b[2], capsule.rb];
        }
        Self {
            bounds: [
                pose.bounds_centre[0],
                pose.bounds_centre[1],
                pose.bounds_centre[2],
                pose.bounds_radius,
            ],
            tint_count: [
                pose.tint[0],
                pose.tint[1],
                pose.tint[2],
                pose.capsules.len() as f32,
            ],
            eyes: pose.eyes,
            pairs,
        }
    }
}

impl RosterPose {
    /// Capsules past [`crate::MAX_ROSTER_CAPSULES`] are dropped rather than
    /// refused.
    pub(super) fn from_pose(pose: &CritterPose) -> Self {
        let mut pairs = [[0.0; 4]; MAX_ROSTER_CAPSULES * 2];
        let count = pose.capsules.len().min(MAX_ROSTER_CAPSULES);
        for (index, capsule) in pose.capsules.iter().take(count).enumerate() {
            pairs[index * 2] = [capsule.a[0], capsule.a[1], capsule.a[2], capsule.ra];
            pairs[index * 2 + 1] = [capsule.b[0], capsule.b[1], capsule.b[2], capsule.rb];
        }
        Self {
            bounds: [
                pose.bounds_centre[0],
                pose.bounds_centre[1],
                pose.bounds_centre[2],
                pose.bounds_radius,
            ],
            tint_count: [pose.tint[0], pose.tint[1], pose.tint[2], count as f32],
            pairs,
        }
    }
}

/// The roster the frame actually carries, capped and laid out for upload.
pub(super) fn roster_of(input: BrickFrameInput<'_>) -> Vec<RosterPose> {
    input
        .roster
        .iter()
        .take(MAX_ROSTER)
        .map(RosterPose::from_pose)
        .collect()
}

pub(super) fn validates_pose(input: BrickFrameInput<'_>) -> Result<(), BrickTraceError> {
    let actual = input.pose.map_or(0, |pose| pose.capsules.len());
    if actual > MAX_CAPSULES {
        return Err(BrickTraceError::TooManyCapsules {
            actual,
            maximum: MAX_CAPSULES,
        });
    }
    Ok(())
}

pub(super) fn validates_change(input: BrickFrameInput<'_>) -> Result<(), BrickTraceError> {
    if let BrickChange::Slots(slots) = input.change {
        for slot in slots {
            if input.map.pointer_coord(*slot).is_none() {
                return Err(BrickTraceError::UnknownBrickSlot(*slot));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The cap arithmetic documented on [`crate::MAX_ROSTER`], asserted
    /// against the limit that actually binds — the downlevel WebGL2 one.
    #[test]
    fn the_roster_binding_fits_the_downlevel_uniform_limit() {
        let limit = wgpu::Limits::downlevel_webgl2_defaults().max_uniform_buffer_binding_size;
        assert_eq!(limit, 16_384);
        assert_eq!(size_of::<CritterParams>(), 3136);
        assert_eq!(size_of::<RosterPose>(), 352);
        assert_eq!(ROSTER_BUFFER_BYTES, 14_096);
        assert!(ROSTER_BUFFER_BYTES < limit);
        assert!((size_of::<TraceParams>() as u64) < limit);
        // What the reduced budget buys off: at `MAX_CAPSULES` per member the
        // same binding would hold five bodies.
        assert_eq!(
            (limit - ROSTER_HEADER_BYTES) / size_of::<CritterParams>() as u64,
            5
        );
    }
}
