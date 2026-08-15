// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use bytemuck::{Pod, Zeroable};

use crate::{BrickMap, CritterPose, Flight, Grade, MAX_CAPSULES};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct BrickRevision(pub u64);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BrickChange<'a> {
    #[default]
    Full,
    Slots(&'a [u32]),
}

#[derive(Clone, Copy)]
pub struct BrickFrameInput<'a> {
    pub map: &'a BrickMap,
    pub revision: BrickRevision,
    pub change: BrickChange<'a>,
    pub flight: &'a Flight,
    pub grade: &'a Grade,
    /// Presentation-only SDF bodies. Their source remains the caller's
    /// projection, never the brick map or world state.
    pub pose: Option<&'a CritterPose>,
}

impl<'a> BrickFrameInput<'a> {
    pub fn new(
        map: &'a BrickMap,
        revision: BrickRevision,
        flight: &'a Flight,
        grade: &'a Grade,
    ) -> Self {
        Self {
            map,
            revision,
            change: BrickChange::Full,
            flight,
            grade,
            pose: None,
        }
    }

    pub fn changed(mut self, change: BrickChange<'a>) -> Self {
        self.change = change;
        self
    }

    pub fn with_pose(mut self, pose: &'a CritterPose) -> Self {
        self.pose = Some(pose);
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BrickDiagnostics {
    pub cpu_prepare_us: u64,
    pub brick_upload_bytes: u64,
    pub uniform_upload_bytes: u64,
    pub resource_creations: u32,
    pub bind_group_rebuilds: u32,
    pub map_recreated: bool,
    pub trace_passes: u32,
    pub readback_bytes: u64,
}

#[derive(Clone, Debug)]
pub struct BrickCapture {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub diagnostics: BrickDiagnostics,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BrickTraceError {
    CaptureFormat(wgpu::TextureFormat),
    TooManyCapsules { actual: usize, maximum: usize },
    DevicePoll(String),
    Readback(String),
}

impl std::fmt::Display for BrickTraceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CaptureFormat(format) => {
                write!(
                    f,
                    "brick capture requires Rgba8Unorm output, not {format:?}"
                )
            }
            Self::TooManyCapsules { actual, maximum } => {
                write!(f, "brick trace has {actual} capsules; maximum is {maximum}")
            }
            Self::DevicePoll(message) => write!(f, "device poll failed: {message}"),
            Self::Readback(message) => write!(f, "readback failed: {message}"),
        }
    }
}

impl std::error::Error for BrickTraceError {}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(super) struct TraceParams {
    pub eye: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    pub far: f32,
    pub _pad: f32,
    pub world_min: [f32; 4],
    pub pointer_extent: [u32; 4],
    pub atlas_slots: [u32; 4],
    pub fog: [f32; 4],
    pub look: [f32; 4],
    pub critter: CritterParams,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(super) struct CritterParams {
    pub bounds: [f32; 4],
    pub tint_count: [f32; 4],
    pub eyes: [[f32; 4]; 2],
    pub pairs: [[f32; 4]; 192],
}

impl TraceParams {
    pub(super) fn from_input(input: BrickFrameInput<'_>) -> Self {
        let map = input.map;
        Self {
            eye: input.flight.eye,
            yaw: input.flight.yaw,
            pitch: input.flight.pitch,
            fov: input.flight.fov,
            far: input.flight.far,
            _pad: 0.0,
            world_min: [
                map.origin()[0] as f32 * 8.0,
                map.origin()[1] as f32 * 8.0,
                map.origin()[2] as f32 * 8.0,
                0.0,
            ],
            pointer_extent: [
                map.pointer_extent()[0],
                map.pointer_extent()[1],
                map.pointer_extent()[2],
                0,
            ],
            atlas_slots: [map.slots()[0], map.slots()[1], map.slots()[2], 0],
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
            critter: CritterParams::from_pose(input.pose),
        }
    }
}

impl CritterParams {
    fn from_pose(pose: Option<&CritterPose>) -> Self {
        let Some(pose) = pose else {
            return Self::zeroed();
        };
        let mut pairs = [[0.0; 4]; 192];
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
