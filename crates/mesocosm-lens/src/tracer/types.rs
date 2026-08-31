// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a caller hands the tracer and what it gets back: camera, frame
//! input, diagnostics, capture, errors.

use bytemuck::{Pod, Zeroable};
use modulus::BrickMap;

use super::LeasedAtlas;
use crate::{CritterPose, Flight, Grade};

const PERSPECTIVE: u32 = 0;
const ORTHOGRAPHIC: u32 = 1;
const LEGACY_ASPECT: f32 = 16.0 / 9.0;

/// The rays a camera contributes to the brick traversal.
///
/// Camera policy stays with the vessel. The tracer needs only an origin
/// plane, a forward direction, and the horizontal and vertical ray spans.
/// Perspective and orthographic projections therefore share the exact same
/// map bindings and DDA implementation.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, serde::Serialize, serde::Deserialize)]
pub struct TraceCamera {
    origin: [f32; 3],
    projection: u32,
    forward: [f32; 3],
    far: f32,
    right: [f32; 3],
    _right_pad: f32,
    up: [f32; 3],
    _up_pad: f32,
}

impl TraceCamera {
    /// A rectilinear perspective camera aimed at `target`.
    pub fn perspective(
        eye: [f32; 3],
        target: [f32; 3],
        up: [f32; 3],
        vertical_fov: f32,
        aspect: f32,
        far: f32,
    ) -> Option<Self> {
        if !(vertical_fov > 0.0 && vertical_fov < std::f32::consts::PI && aspect > 0.0 && far > 0.0)
        {
            return None;
        }
        let forward = normalize(sub(target, eye))?;
        let right_unit = normalize(cross(forward, up))?;
        let up_unit = normalize(cross(right_unit, forward))?;
        let half_height = (vertical_fov * 0.5).tan();
        Some(Self {
            origin: eye,
            projection: PERSPECTIVE,
            forward,
            far,
            right: scale(right_unit, half_height * aspect),
            _right_pad: 0.0,
            up: scale(up_unit, half_height),
            _up_pad: 0.0,
        })
    }

    /// An orthographic section whose ray interval is exactly `depth` voxels.
    ///
    /// `centre` is the middle of the retained slab rather than the eye. This
    /// keeps the near and far cut planes symmetric when the section is moved.
    pub fn orthographic_slab(
        centre: [f32; 3],
        forward: [f32; 3],
        up: [f32; 3],
        half_height: f32,
        aspect: f32,
        depth: f32,
    ) -> Option<Self> {
        if !(half_height > 0.0 && aspect > 0.0 && depth > 0.0) {
            return None;
        }
        let forward = normalize(forward)?;
        let right_unit = normalize(cross(forward, up))?;
        let up_unit = normalize(cross(right_unit, forward))?;
        Some(Self {
            origin: sub(centre, scale(forward, depth * 0.5)),
            projection: ORTHOGRAPHIC,
            forward,
            far: depth,
            right: scale(right_unit, half_height * aspect),
            _right_pad: 0.0,
            up: scale(up_unit, half_height),
            _up_pad: 0.0,
        })
    }

    fn from_flight(flight: &Flight) -> Self {
        let forward = [
            flight.yaw.sin() * flight.pitch.cos(),
            flight.pitch.sin(),
            flight.yaw.cos() * flight.pitch.cos(),
        ];
        let target = add(flight.eye, forward);
        let vertical_fov = 2.0 * ((flight.fov * 0.5).tan() / LEGACY_ASPECT).atan();
        Self::perspective(
            flight.eye,
            target,
            [0.0, 1.0, 0.0],
            vertical_fov,
            LEGACY_ASPECT,
            flight.far,
        )
        .expect("Flight carries a valid camera")
    }

    #[cfg(test)]
    fn ray_at(self, ndc: [f32; 2]) -> ([f32; 3], [f32; 3]) {
        if self.projection == ORTHOGRAPHIC {
            (
                add(
                    self.origin,
                    add(scale(self.right, ndc[0]), scale(self.up, ndc[1])),
                ),
                self.forward,
            )
        } else {
            (
                self.origin,
                normalize(add(
                    self.forward,
                    add(scale(self.right, ndc[0]), scale(self.up, ndc[1])),
                ))
                .expect("camera rays are nonzero"),
            )
        }
    }
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(value: [f32; 3], amount: f32) -> [f32; 3] {
    [value[0] * amount, value[1] * amount, value[2] * amount]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(value: [f32; 3]) -> Option<[f32; 3]> {
    let length = (value[0] * value[0] + value[1] * value[1] + value[2] * value[2]).sqrt();
    (length > 1e-6).then(|| scale(value, 1.0 / length))
}

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
    pub camera: TraceCamera,
    pub grade: &'a Grade,
    /// Presentation-only SDF bodies. Their source remains the caller's
    /// projection, never the brick map or world state.
    pub pose: Option<&'a CritterPose>,
    /// Every other body in frame, at the roster's reduced capsule budget.
    ///
    /// Additive: a frame that names none traces exactly what it always did.
    /// The caller culls to what its camera shows; the tracer keeps the first
    /// [`crate::MAX_ROSTER`] and drops the rest.
    pub roster: &'a [CritterPose],
    /// Where the atlas texture's voxels come from this frame. `None`
    /// keeps the CPU upload from [`BrickMap`], which is also the
    /// downlevel path. See [`LeasedAtlas`].
    pub leased_atlas: Option<LeasedAtlas<'a>>,
    /// Column-major world-to-clip matrix for the depth join. Required by
    /// [`super::BrickTracer::encode_with_depth`], which writes
    /// `@builtin(frag_depth)` from it so a raster tenant sharing the same
    /// matrix occludes and is occluded exactly. Ignored by plain `encode`.
    pub clip_from_world: Option<[[f32; 4]; 4]>,
    /// The host schedule epoch this frame expects its leased atlas bytes
    /// to have been made safe at. `Some` turns the lease's `read_epoch`
    /// from a producer promise into a tracer-validated identity: a lease
    /// at any other epoch is refused and the CPU path fills the frame.
    pub expected_read_epoch: Option<u64>,
}

impl<'a> BrickFrameInput<'a> {
    pub fn new(
        map: &'a BrickMap,
        revision: BrickRevision,
        flight: &'a Flight,
        grade: &'a Grade,
    ) -> Self {
        Self::for_camera(map, revision, TraceCamera::from_flight(flight), grade)
    }

    /// Construct a frame from a vessel-owned camera policy.
    pub fn for_camera(
        map: &'a BrickMap,
        revision: BrickRevision,
        camera: TraceCamera,
        grade: &'a Grade,
    ) -> Self {
        Self {
            map,
            revision,
            change: BrickChange::Full,
            camera,
            grade,
            pose: None,
            roster: &[],
            leased_atlas: None,
            clip_from_world: None,
            expected_read_epoch: None,
        }
    }

    pub fn changed(mut self, change: BrickChange<'a>) -> Self {
        self.change = change;
        self
    }

    /// Fill the atlas from a GPU-resident producer this frame instead of
    /// uploading it from the CPU.
    pub fn with_leased_atlas(mut self, leased: LeasedAtlas<'a>) -> Self {
        self.leased_atlas = Some(leased);
        self
    }

    pub fn with_pose(mut self, pose: &'a CritterPose) -> Self {
        self.pose = Some(pose);
        self
    }

    /// Draw every other body in frame beside [`Self::with_pose`]'s.
    pub fn with_roster(mut self, roster: &'a [CritterPose]) -> Self {
        self.roster = roster;
        self
    }

    /// Supply the raster tenant's column-major world-to-clip matrix so the
    /// depth join can write comparable fragment depth.
    pub fn with_clip_from_world(mut self, clip_from_world: [[f32; 4]; 4]) -> Self {
        self.clip_from_world = Some(clip_from_world);
        self
    }

    /// State the host schedule epoch a leased atlas must carry this frame.
    pub fn with_expected_read_epoch(mut self, epoch: u64) -> Self {
        self.expected_read_epoch = Some(epoch);
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BrickDiagnostics {
    pub cpu_prepare_us: u64,
    pub brick_upload_bytes: u64,
    /// Voxel bytes that reached the atlas from a GPU-resident producer
    /// rather than the CPU. These are not counted in
    /// `brick_upload_bytes`, which stays the CPU-upload measure.
    pub leased_atlas_bytes: u64,
    /// Producer schedule epoch observed by the accepted atlas lease.
    pub observed_read_epoch: Option<u64>,
    /// Leases refused because their Ground revision did not match the frame's.
    pub stale_lease_rejections: u32,
    /// Leases refused because they materialized another selected projection.
    pub projection_lease_rejections: u32,
    /// Leases refused because their read epoch was not the one the frame
    /// expected.
    pub epoch_lease_rejections: u32,
    /// Leases refused because their extent did not fit the leased range.
    pub misfit_lease_rejections: u32,
    /// Leases refused because they did not cover every atlas region named by
    /// the frame's change declaration.
    pub incomplete_lease_rejections: u32,
    pub uniform_upload_bytes: u64,
    /// Roster members this frame drew, beside the single pose.
    pub roster_members: u32,
    /// Roster members the frame named past [`crate::MAX_ROSTER`].
    pub roster_dropped: u32,
    /// Capsules the drawn roster members carried past
    /// [`crate::MAX_ROSTER_CAPSULES`]. The widest are kept, so this is how much
    /// of the ecology's detail the budget spent rather than how many bodies
    /// went missing.
    pub roster_capsules_dropped: u32,
    pub resource_creations: u32,
    pub bind_group_rebuilds: u32,
    pub map_recreated: bool,
    /// Existing equal-sized textures were retained and fully republished for a
    /// new selected projection.
    pub projection_replaced: bool,
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
    TooManyCapsules {
        actual: usize,
        maximum: usize,
    },
    UnknownBrickSlot(u32),
    /// The depth join cannot write comparable fragment depth without the
    /// raster tenant's world-to-clip matrix.
    MissingClipFromWorld,
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
            Self::UnknownBrickSlot(slot) => {
                write!(f, "brick change names unknown atlas slot {slot}")
            }
            Self::MissingClipFromWorld => {
                write!(
                    f,
                    "the depth join needs a clip_from_world matrix on its frame input"
                )
            }
            Self::DevicePoll(message) => write!(f, "device poll failed: {message}"),
            Self::Readback(message) => write!(f, "readback failed: {message}"),
        }
    }
}

impl std::error::Error for BrickTraceError {}

#[cfg(test)]
mod camera_tests {
    use super::*;

    #[test]
    fn orthographic_rays_share_direction_but_move_across_the_slab() {
        let camera = TraceCamera::orthographic_slab(
            [0.0, 4.0, 8.0],
            [0.0, 0.0, -1.0],
            [0.0, 1.0, 0.0],
            5.0,
            2.0,
            6.0,
        )
        .unwrap();
        let left = camera.ray_at([-1.0, 0.0]);
        let right = camera.ray_at([1.0, 0.0]);
        assert_eq!(left.1, right.1);
        assert_eq!(left.0[0], -10.0);
        assert_eq!(right.0[0], 10.0);
        assert_eq!(left.0[2], 11.0);
    }

    #[test]
    fn perspective_rays_share_an_eye_and_diverge() {
        let camera = TraceCamera::perspective(
            [0.0, 0.0, 0.0],
            [0.0, 0.0, -1.0],
            [0.0, 1.0, 0.0],
            std::f32::consts::FRAC_PI_2,
            1.0,
            20.0,
        )
        .unwrap();
        let left = camera.ray_at([-1.0, 0.0]);
        let right = camera.ray_at([1.0, 0.0]);
        assert_eq!(left.0, right.0);
        assert!(left.1[0] < 0.0);
        assert!(right.1[0] > 0.0);
    }
}
