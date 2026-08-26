// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use bytemuck::{Pod, Zeroable};
use conatus_brick::{BrickMap, BrickProjectionRevision, BrickTraceSpace};

use crate::{CritterPose, Flight, Grade, MAX_CAPSULES};

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
    /// Where the atlas texture's voxels come from this frame. `None`
    /// keeps the CPU upload from [`BrickMap`], which is also the
    /// downlevel path. See [`LeasedAtlas`].
    pub leased_atlas: Option<LeasedAtlas<'a>>,
    /// Column-major world-to-clip matrix for the depth join. Required by
    /// [`super::BrickTracer::encode_with_depth`], which writes
    /// `@builtin(frag_depth)` from it so a raster tenant sharing the same
    /// matrix occludes and is occluded exactly. Ignored by plain `encode`.
    pub clip_from_world: Option<[[f32; 4]; 4]>,
}

/// A GPU-resident atlas the tracer may fill its texture from without the
/// CPU seeing a voxel.
///
/// The tracer samples `texture_3d` and holds no storage buffer, because
/// it is fragment-only for downlevel reach (WebGL2 has neither compute
/// nor storage buffers). So a resident producer does not change the
/// tracer's bindings; it changes where the atlas texture's bytes come
/// from. This is deliberately a plain wgpu triple rather than a
/// producer's own view type: the buffer contract is the meeting point,
/// so the lens depends on no compute stack.
///
/// The producer owns the allocation and its lifetime. `revision` must
/// be the world revision those bytes were materialized at, so a stale
/// lease cannot be presented as current.
#[derive(Clone, Copy, Debug)]
pub struct LeasedAtlas<'a> {
    pub buffer: &'a wgpu::Buffer,
    /// Start and length of the producer allocation. Source coordinates below
    /// are relative to this range rather than forged into the buffer offset.
    pub offset: u64,
    pub size: u64,
    /// Source voxel coordinate and row/image strides inside the producer's
    /// R8 allocation. These permit one brick to be leased from a larger atlas.
    pub source_origin: [u32; 3],
    pub source_bytes_per_row: u32,
    pub source_rows_per_image: u32,
    /// Where in the atlas these voxels belong, and how many.
    pub slot_origin: [u32; 3],
    pub extent: [u32; 3],
    pub revision: BrickRevision,
    /// Selected brick projection these resident bytes materialize.
    pub projection_revision: BrickProjectionRevision,
    /// Host-issued schedule epoch at which the producer made these bytes
    /// safe for reader tenants.
    pub read_epoch: u64,
}

impl LeasedAtlas<'_> {
    /// The bytes `extent` describes, at one byte per voxel (the atlas is
    /// `R8Uint`).
    pub fn byte_len(&self) -> u64 {
        self.extent
            .into_iter()
            .try_fold(1u64, |total, axis| total.checked_mul(u64::from(axis)))
            .unwrap_or(u64::MAX)
    }

    /// Whether the strided source extent fits inside the leased range.
    ///
    /// Load-bearing rather than defensive: a producer's allocator pools
    /// many planes into one buffer, so copying an extent larger than the
    /// lease does not fault, it reads whatever plane happens to sit
    /// next in the pool and paints it into the world. Silent corruption
    /// is worse than a refusal, so an ill-fitting lease is refused.
    pub fn fits(&self) -> bool {
        if self.extent.contains(&0)
            || self.source_bytes_per_row == 0
            || self.source_rows_per_image == 0
        {
            return false;
        }
        let Some(source_x_end) = self.source_origin[0].checked_add(self.extent[0]) else {
            return false;
        };
        let Some(source_y_end) = self.source_origin[1].checked_add(self.extent[1]) else {
            return false;
        };
        if source_x_end > self.source_bytes_per_row || source_y_end > self.source_rows_per_image {
            return false;
        }
        self.source_end().is_some_and(|end| end <= self.size)
    }

    /// Whether wgpu can copy this source into the destination atlas without
    /// crossing either range or violating buffer-copy alignment.
    pub fn copyable_into(&self, atlas_extent: [u32; 3]) -> bool {
        let destination_fits = (0..3).all(|axis| {
            self.slot_origin[axis]
                .checked_add(self.extent[axis])
                .is_some_and(|end| end <= atlas_extent[axis])
        });
        let alignment = wgpu::COPY_BUFFER_ALIGNMENT;
        self.fits()
            && destination_fits
            && self.extent[0].is_multiple_of(alignment as u32)
            && self.source_bytes_per_row.is_multiple_of(alignment as u32)
            && self
                .source_start()
                .and_then(|start| self.offset.checked_add(start))
                .is_some_and(|start| start.is_multiple_of(alignment))
    }

    /// Whether this destination lease contains another atlas box completely.
    pub fn covers(&self, origin: [u32; 3], extent: [u32; 3]) -> bool {
        (0..3).all(|axis| {
            let Some(required_end) = origin[axis].checked_add(extent[axis]) else {
                return false;
            };
            let Some(leased_end) = self.slot_origin[axis].checked_add(self.extent[axis]) else {
                return false;
            };
            origin[axis] >= self.slot_origin[axis] && required_end <= leased_end
        })
    }

    fn source_start(&self) -> Option<u64> {
        let bytes_per_row = u64::from(self.source_bytes_per_row);
        let rows_per_image = u64::from(self.source_rows_per_image);
        let image_stride = bytes_per_row.checked_mul(rows_per_image)?;
        u64::from(self.source_origin[2])
            .checked_mul(image_stride)?
            .checked_add(u64::from(self.source_origin[1]).checked_mul(bytes_per_row)?)?
            .checked_add(u64::from(self.source_origin[0]))
    }

    fn source_end(&self) -> Option<u64> {
        let bytes_per_row = u64::from(self.source_bytes_per_row);
        let rows_per_image = u64::from(self.source_rows_per_image);
        let image_stride = bytes_per_row.checked_mul(rows_per_image)?;
        self.source_start()?
            .checked_add(u64::from(self.extent[2] - 1).checked_mul(image_stride)?)?
            .checked_add(u64::from(self.extent[1] - 1).checked_mul(bytes_per_row)?)?
            .checked_add(u64::from(self.extent[0]))
    }
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
            camera: TraceCamera::from_flight(flight),
            grade,
            pose: None,
            leased_atlas: None,
            clip_from_world: None,
        }
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
            leased_atlas: None,
            clip_from_world: None,
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

    /// Supply the raster tenant's column-major world-to-clip matrix so the
    /// depth join can write comparable fragment depth.
    pub fn with_clip_from_world(mut self, clip_from_world: [[f32; 4]; 4]) -> Self {
        self.clip_from_world = Some(clip_from_world);
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
    /// Leases refused because their extent did not fit the leased range.
    pub misfit_lease_rejections: u32,
    /// Leases refused because they did not cover every atlas region named by
    /// the frame's change declaration.
    pub incomplete_lease_rejections: u32,
    pub uniform_upload_bytes: u64,
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
    TooManyCapsules { actual: usize, maximum: usize },
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

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(super) struct TraceParams {
    pub camera: TraceCamera,
    pub space: BrickTraceSpace,
    pub fog: [f32; 4],
    pub look: [f32; 4],
    /// Column-major, identity when the frame carries no depth join.
    pub clip_from_world: [[f32; 4]; 4],
    pub critter: CritterParams,
}

pub(super) const IDENTITY: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

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
            camera: input.camera,
            space: BrickTraceSpace::from_map(map),
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
