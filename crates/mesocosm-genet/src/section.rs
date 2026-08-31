// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The main view: the ruled side-on terrarium section, brick-traced.
//!
//! `mesocosm-lens` owns the tracer; this module owns the vessel's policy over
//! it — which Ground the map binds, where the slab sits, which body is posed,
//! and how the traced texture reaches the surface the HUD then composites on.
//! No world state lives here and no rule is decided here.

use mesocosm_core::places::Ground;
use mesocosm_core::{BodyDocument, Organism, World};
use mesocosm_lens::{
    BodyLensProjection, BodyPlacement, BrickChange, BrickFrameInput, BrickMap, BrickRevision,
    BrickTracer, CritterPose, FRAME_FORMAT, Grade, MAX_ROSTER, TraceCamera,
};
use mesocosm_render::composite::Composite;

/// The G2 slab: section depth and the palette depth of the retro grade, plus
/// the half-height Mark ruled on 2026-08-29. Kept as defaults, not as constants
/// a camera policy may not vary.
///
/// **28, framing the content's height and letting the width follow.** The world
/// is 5.2:1 (129 voxels against a 25-voxel band) inside a 1.78:1 window, so no
/// half-height both frames the width and fills the height; 28 shows 77% of the
/// world and reads a 9-voxel limb at 9% of frame. A host may still carry
/// another ([`HostConfig::slab_half_height`](crate::HostConfig)), and every
/// framing replays to one hash. See the scale plan, "The framing".
pub const SLAB_HALF_HEIGHT: f32 = 28.0;
/// **Slice thickness, and deliberately not scaled with the enclosure.** A
/// section shows a cut of fixed depth; widening it to keep the same fraction of
/// a bigger world would only stack more bodies into the same pixels, which is
/// the pile the S1 finding started from.
const SLAB_DEPTH: f32 = 16.0;
const PALETTE: u32 = 3;

/// Voxels one arrow-key press shifts the section by. Presentation only: this
/// number never reaches an intent.
pub const PAN_STEP: f32 = 2.0;

/// The section looks along -z, so the slab cuts the enclosure across x and y.
const FORWARD: [f32; 3] = [0.0, 0.0, -1.0];
const UP: [f32; 3] = [0.0, 1.0, 0.0];

/// Captures are written in this format regardless of the surface's, so the
/// PNG encoder never has to swizzle a BGRA frame.
const CAPTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

/// A presentation-only offset added to the follow centre.
#[derive(Clone, Copy, Debug, Default)]
pub struct Pan {
    pub x: f32,
    pub y: f32,
}

/// The host's presentation reads for one frame, taken off the stepped world
/// before the device is borrowed. None of it is world state.
#[derive(Clone, Copy)]
pub struct SectionFrame<'a> {
    pub ground: &'a Ground,
    /// The host's drain of the world's changed bricks. The slots they map to
    /// are the only region the tracer re-uploads, so a carve costs its own
    /// bricks and not the enclosure.
    pub dirty: &'a [[i16; 3]],
    pub centre: [f32; 3],
    /// The controlled critter, at full capsule fidelity.
    pub pose: Option<&'a CritterPose>,
    /// Everything else alive in the slab.
    pub roster: &'a [CritterPose],
}

pub struct Section {
    device: wgpu::Device,
    queue: wgpu::Queue,
    tracer: BrickTracer,
    map: BrickMap,
    grade: Grade,
    width: u32,
    height: u32,
    /// How much world this section frames. Presentation, so it lives beside the
    /// device rather than in the world.
    half_height: f32,
    /// What the tracer writes: display-encoded values in a linear-tagged
    /// format, exactly as the lens's own captures read them back.
    traced: wgpu::Texture,
    traced_view: wgpu::TextureView,
    /// The same texels in an sRGB-tagged twin. Sampling the raw bytes into an
    /// sRGB target would encode a second time and wash the section out; the
    /// twin's decode-on-sample cancels the target's encode. Same remedy, same
    /// reason, as the HUD's vello raster.
    display: wgpu::Texture,
    display_view: wgpu::TextureView,
    composite: Composite,
}

impl Section {
    /// Binds the live Ground at genesis and builds the tracer on the host's
    /// own device. `format` is the surface's, for the composite that lands
    /// the traced frame under the HUD.
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        ground: &Ground,
        half_height: f32,
    ) -> Result<Self, String> {
        let map = BrickMap::from_ground(ground).map_err(|error| error.to_string())?;
        let tracer =
            BrickTracer::with_format(device.clone(), queue.clone(), width, height, FRAME_FORMAT);
        let composite = Composite::new(&device, format);
        let (traced, traced_view) = target(&device, width, height, FRAME_FORMAT, "traced section");
        let (display, display_view) = target(
            &device,
            width,
            height,
            CAPTURE_FORMAT,
            "traced section srgb",
        );
        Ok(Self {
            device,
            queue,
            tracer,
            map,
            grade: Grade::retro(PALETTE),
            width,
            height,
            half_height: half_height_or_default(half_height),
            traced,
            traced_view,
            display,
            display_view,
            composite,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width.max(1);
        self.height = height.max(1);
        self.tracer.resize(self.width, self.height);
        (self.traced, self.traced_view) = target(
            &self.device,
            self.width,
            self.height,
            FRAME_FORMAT,
            "traced section",
        );
        (self.display, self.display_view) = target(
            &self.device,
            self.width,
            self.height,
            CAPTURE_FORMAT,
            "traced section srgb",
        );
    }

    /// Aspect comes from the window rather than a fixed 16:9, so a resized
    /// section stretches nothing.
    fn aspect(&self) -> f32 {
        self.width as f32 / self.height.max(1) as f32
    }

    /// Depth stays the G2 number; the half-height is the host's.
    fn camera(&self, centre: [f32; 3]) -> Option<TraceCamera> {
        TraceCamera::orthographic_slab(
            centre,
            FORWARD,
            UP,
            self.half_height,
            self.aspect(),
            SLAB_DEPTH,
        )
    }

    /// How much world the section frames, in voxels of half-height.
    pub fn half_height(&self) -> f32 {
        self.half_height
    }

    /// The world box this camera actually shows, from the camera's own
    /// numbers. What falls outside cannot reach a pixel, so it is what the
    /// roster culls against.
    pub fn slab_window(&self, centre: [f32; 3]) -> SlabWindow {
        SlabWindow {
            centre,
            half: [
                self.half_height * self.aspect(),
                self.half_height,
                SLAB_DEPTH * 0.5,
            ],
        }
    }

    /// Roster members the last traced frame drew. The receipt's evidence that
    /// the section shows the ecology rather than one body.
    pub fn last_roster_members(&self) -> u32 {
        self.tracer
            .last_diagnostics()
            .map_or(0, |diagnostics| diagnostics.roster_members)
    }

    /// Capsules the last frame's roster members carried past their budget. The
    /// widest are kept, so this is detail spent rather than bodies lost.
    pub fn last_roster_capsules_dropped(&self) -> u32 {
        self.tracer
            .last_diagnostics()
            .map_or(0, |diagnostics| diagnostics.roster_capsules_dropped)
    }

    /// Traces one frame and composites it into `surface`.
    pub fn draw(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        surface: &wgpu::TextureView,
        frame: SectionFrame<'_>,
    ) -> Result<(), String> {
        let slots = if frame.dirty.is_empty() {
            Vec::new()
        } else {
            self.map
                .refresh(frame.ground, frame.dirty.iter().copied())
                .map_err(|error| error.to_string())?
        };
        let camera = self
            .camera(frame.centre)
            .ok_or("invalid terrarium camera")?;
        let mut input = BrickFrameInput::for_camera(
            &self.map,
            BrickRevision(frame.ground.revision()),
            camera,
            &self.grade,
        )
        .changed(BrickChange::Slots(&slots))
        .with_roster(frame.roster);
        if let Some(pose) = frame.pose {
            input = input.with_pose(pose);
        }
        self.tracer
            .encode(encoder, &self.traced_view, input)
            .map_err(|error| error.to_string())?;
        self.copy_to_display(encoder);
        self.composite.draw(
            &self.device,
            &self.queue,
            encoder,
            surface,
            &self.display_view,
            (0.0, 0.0, self.width as f32, self.height as f32),
            (self.width, self.height),
        );
        Ok(())
    }

    fn copy_to_display(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.copy_texture_to_texture(
            self.traced.as_image_copy(),
            self.display.as_image_copy(),
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Reads the most recently traced frame back as RGBA8, with `overlay`
    /// given the chance to composite chrome over it first.
    ///
    /// The last frame rather than a fresh trace: the capture is meant to be
    /// the picture the player was looking at, and re-tracing would show a
    /// world one frame further on than the receipt's own hash.
    pub fn capture(
        &self,
        overlay: impl FnOnce(&mut wgpu::CommandEncoder, &wgpu::TextureView, wgpu::TextureFormat),
    ) -> Option<(u32, u32, Vec<u8>)> {
        let (shot, view) = target(
            &self.device,
            self.width,
            self.height,
            CAPTURE_FORMAT,
            "section capture",
        );
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("section capture"),
            });
        Composite::new(&self.device, CAPTURE_FORMAT).draw(
            &self.device,
            &self.queue,
            &mut encoder,
            &view,
            &self.display_view,
            (0.0, 0.0, self.width as f32, self.height as f32),
            (self.width, self.height),
        );
        overlay(&mut encoder, &view, CAPTURE_FORMAT);
        self.read_back(encoder, &shot)
            .map(|pixels| (self.width, self.height, pixels))
    }

    fn read_back(
        &self,
        mut encoder: wgpu::CommandEncoder,
        colour: &wgpu::Texture,
    ) -> Option<Vec<u8>> {
        // Copy rows must be aligned, so the staging buffer is padded and the
        // padding is stripped after mapping.
        let unpadded = self.width * 4;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded = unpadded.div_ceil(align) * align;
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("section readback"),
            size: (padded * self.height) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: colour,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &staging,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit(Some(encoder.finish()));

        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
        let mapped = slice.get_mapped_range().ok()?;
        let mut pixels = Vec::with_capacity((unpadded * self.height) as usize);
        for row in 0..self.height {
            let start = (row * padded) as usize;
            pixels.extend_from_slice(&mapped[start..start + unpadded as usize]);
        }
        drop(mapped);
        staging.unmap();
        Some(pixels)
    }
}

fn target(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    label: &str,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    (texture, view)
}

/// The half-height a host actually frames with: its own, or the default when
/// it named none. One reading, so the camera and the follow centre's clamp
/// cannot disagree about how tall the frame is.
pub fn half_height_or_default(configured: f32) -> f32 {
    if configured > 0.0 {
        configured
    } else {
        SLAB_HALF_HEIGHT
    }
}

/// Where the slab sits: on the controlled critter, plus the pan, **clamped so
/// the frame never dips below bedrock**.
///
/// The z component takes no pan, because the section's whole claim is that it
/// cuts through whoever is being played.
///
/// The clamp is the companion ruled with the half-height (2026-08-29): the
/// world's lowest brick is y = 0, so a frame centred below `half_height` shows
/// void along the bottom, which reads as a slab floating rather than a
/// terrarium. Past roughly 24 the shipped follow centre does exactly that.
/// It is presentation, like the framing — it moves no intent, so it cannot
/// reach the trace.
pub fn centre_on(at: [i32; 3], pan: Pan, half_height: f32) -> [f32; 3] {
    [
        at[0] as f32 + pan.x,
        (at[1] as f32 + pan.y).max(half_height),
        at[2] as f32,
    ]
}

/// The world box the section's slab shows, in voxels around its centre.
#[derive(Clone, Copy, Debug)]
pub struct SlabWindow {
    pub centre: [f32; 3],
    /// Half extents in x, y and z.
    pub half: [f32; 3],
}

impl SlabWindow {
    /// Whether a voxel position falls inside the window. Position alone, not
    /// the body's extent: a body straddling the cut plane is drawn whole and
    /// the tracer's own ray interval does the trimming.
    pub fn holds(&self, at: [i32; 3]) -> bool {
        (0..3).all(|axis| (at[axis] as f32 - self.centre[axis]).abs() <= self.half[axis])
    }
}

/// The controlled critter's pose, through the landed V2 projection.
///
/// It stays the tracer's single pose rather than a roster member, because a
/// member's capsule budget is smaller than the played body's: see
/// [`mesocosm_lens::MAX_ROSTER`].
/// The pose comes back with the count of parts the capsule budget dropped, so
/// a truncated player is reported rather than merely smaller.
pub fn pose_of(world: &World, tint: [f32; 3]) -> Option<(CritterPose, u32)> {
    pose_at(world.body()?, world.position()?, tint)
}

/// Every other living organism the window holds, posed and tinted.
///
/// The scan is a bounds test per organism and a projection only for those
/// inside, so the frame's cost tracks organisms in the slab rather than
/// organisms in the world. The lens truncates whatever exceeds its own cap;
/// the take here just stops projecting once the cap is met.
pub fn roster_of(
    world: &World,
    window: SlabWindow,
    tint: impl Fn(&Organism) -> [f32; 3],
) -> Vec<CritterPose> {
    let controlled = world.controlled_id();
    world
        .organisms
        .iter()
        .filter(|organism| {
            organism.is_alive()
                && Some(organism.id) != controlled
                && window.holds(organism.position)
        })
        .filter_map(|organism| {
            pose_at(&organism.body, organism.position, tint(organism)).map(|(pose, _)| pose)
        })
        .take(MAX_ROSTER)
        .collect()
}

/// One body placed where it stands, and the parts its capsule budget could
/// not carry.
///
/// Body space is world voxels — the raster lane draws a part's voxels at
/// `position + v` — so the scale is 1 and the projection's floor subtraction
/// is undone, or the two views would disagree about where the same voxel is.
///
/// **A body past the lens's capsule limit is drawn truncated, never dropped.**
/// Until DC3 this swallowed the refusal with `.ok()` and the frame simply had
/// no body in it, which is how a played critter could disappear while alive.
/// The overflow is now a number the host puts in its receipt.
fn pose_at(body: &BodyDocument, at: [i32; 3], tint: [f32; 3]) -> Option<(CritterPose, u32)> {
    let floor = body.aabb().min[1] as f32;
    let placement = BodyPlacement {
        ground: [at[0] as f32, at[1] as f32 + floor, at[2] as f32],
        scale: 1.0,
        tint,
    };
    BodyLensProjection::project_truncated(body, placement)
        .ok()
        .map(|(projected, dropped)| (projected.pose, dropped as u32))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The companion rule ruled with the half-height: the frame's floor is the
    /// world's floor, whatever the body it is following is standing on.
    #[test]
    fn the_follow_centre_never_frames_below_bedrock() {
        let pan = Pan::default();
        for half in [20.0, SLAB_HALF_HEIGHT, 36.3, 48.0] {
            for y in [0, 3, 12, 24, 40] {
                let centre = centre_on([7, y, -5], pan, half);
                assert!(
                    centre[1] - half >= 0.0,
                    "half {half} at y {y} framed {} below bedrock",
                    half - centre[1]
                );
                assert_eq!(
                    [centre[0], centre[2]],
                    [7.0, -5.0],
                    "the clamp moved x or z"
                );
            }
        }
    }

    /// It is a floor, not a lock: a body standing above it is still followed,
    /// and the pan still pans.
    #[test]
    fn a_body_above_the_floor_is_followed_and_panned() {
        let high = centre_on([0, 60, 0], Pan { x: 2.0, y: 3.0 }, SLAB_HALF_HEIGHT);
        assert_eq!(high, [2.0, 63.0, 0.0]);
        // Panning down into the void stops at the floor rather than showing it.
        let low = centre_on([0, 30, 0], Pan { x: 0.0, y: -20.0 }, SLAB_HALF_HEIGHT);
        assert_eq!(low[1], SLAB_HALF_HEIGHT);
    }

    /// A host that names no half-height frames with the ruled default, and the
    /// clamp reads the same number the camera does.
    #[test]
    fn an_unset_half_height_falls_back_to_the_ruled_default() {
        assert_eq!(half_height_or_default(0.0), SLAB_HALF_HEIGHT);
        assert_eq!(half_height_or_default(-1.0), SLAB_HALF_HEIGHT);
        assert_eq!(half_height_or_default(36.3), 36.3);
    }

    /// The vanish, retired at the boundary that used to swallow it. Mark's
    /// second playtest reached 304 living parts; the host now poses that body
    /// and says what it could not carry.
    #[test]
    fn a_body_of_the_playtest_size_is_still_posed() {
        use mesocosm_core::{Attachment, Provenance, SpeciesId, VolumeRef, Yaw};

        let mut body = BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 100, [2, 2, 2]);
        for index in 0..303 {
            body.attach(
                VolumeRef::from_tag(2),
                10,
                [2, 1, 1],
                Attachment {
                    parent: body.root,
                    offset: [index + 4, 0, 0],
                    yaw: Yaw::Zero,
                },
                Provenance::founding(),
            )
            .unwrap();
        }
        assert_eq!(body.living().count(), 304);
        let (pose, dropped) = pose_at(&body, [0, 8, 0], [0.4, 0.6, 0.4]).expect("a posed body");
        assert_eq!(pose.capsules.len(), mesocosm_lens::MAX_CAPSULES);
        assert_eq!(dropped, 48);
    }
}
