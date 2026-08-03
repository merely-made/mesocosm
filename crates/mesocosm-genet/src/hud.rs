// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The HUD lane: leaves rasterized by netrender on the game's own device.
//!
//! Route A of the staged host ruling (2026-08-02). The leaf paints
//! `PaintCmd`s, `paint_list_render` lowers them, netrender's vello path
//! rasterizes into a transparent texture, and the overlay pass blends it over
//! the frame. Textless by decree: the moment chrome wants a word, that is the
//! consumer pull for the cambium lane, not a reason to teach this one
//! lettering.

use mesocosm_core::World;
use mesocosm_render::composite::Composite;
use mesocosm_render::{Camera, Renderer, SceneItem};
use mesocosm_views::MinimapLeaf;
use netrender::{ColorLoad, Renderer as NetRenderer, WgpuHandles, create_netrender_instance};
use sprigging::{Leaf, PaintCx, Size};

/// The minimap's square side, in pixels. Also the rasterized texture's size,
/// so the leaf paints at the resolution it is shown.
const SIDE: u32 = 160;

/// Distance from the frame's corner.
const MARGIN: f32 = 12.0;

/// Steps between backdrop re-renders. The ruling asks for dynamically
/// generated, not per-frame: the enclosure drifts on ecology time, and a
/// cadence keeps the world's self-portrait current without paying a second
/// scene render every frame.
///
/// **An ambient background**: the non-interactive backdrop subtype, mere's
/// Game of Life tier. Backdrop names where a layer sits, not whether it
/// acts (ruled 2026-08-03); this one does not act. Nothing in it has a
/// hull, exerts a field, or is navigable, because the world it portrays
/// has no such thing yet. When the world grows props and fields, their
/// projection enters as an interactive backdrop through the scene lane,
/// not by enriching this one.
const BACKDROP_CADENCE: u64 = 10;

pub struct Hud {
    net: NetRenderer,
    composite: Composite,
    leaf: MinimapLeaf,
    view: wgpu::TextureView,
    /// The same texels in an sRGB-tagged twin, for compositing.
    ///
    /// Vello writes display-encoded values into a linear-tagged texture (its
    /// target must be plain Unorm because it writes through a storage
    /// binding, which sRGB formats cannot be). Sampling those bytes raw and
    /// writing to an sRGB target encodes twice and brightens everything, so
    /// each raster is byte-copied into this copy-compatible sRGB texture,
    /// whose decode-on-sample cancels the target's encode.
    sample_view: wgpu::TextureView,
    sample_texture: wgpu::Texture,
    /// Keeps the vello target alive for its view.
    texture: wgpu::Texture,
    /// The backdrop's own small renderer, sized to the minimap.
    shot: Renderer,
    backdrop_view: wgpu::TextureView,
    _backdrop: wgpu::Texture,
    /// The step count the backdrop was last rendered at.
    rendered_at: Option<u64>,
}

/// Straight down at the whole enclosure, aligned with the minimap's mapping:
/// world +x reads right and +z reads down, so the cells sit over the terrain
/// they govern. Pitch stops a degree short of vertical because a look-at with
/// view parallel to up is singular.
fn overhead() -> Camera {
    Camera {
        target: [0.0, 0.0, 0.0],
        extent: mesocosm_core::world::ENCLOSURE as f32 + 2.0,
        yaw: std::f32::consts::FRAC_PI_2,
        pitch: 1.553_343_f32,
        aspect: 1.0,
    }
}

impl Hud {
    /// Builds the HUD on the game's device. `None` if netrender declines the
    /// device, in which case the game simply runs chromeless.
    pub fn new(handles: WgpuHandles, format: wgpu::TextureFormat, world: &World) -> Option<Self> {
        let device = handles.device.clone();
        let queue = handles.queue.clone();
        let net = create_netrender_instance(
            handles,
            netrender::NetrenderOptions {
                tile_cache_size: Some(SIDE),
                enable_vello: true,
                ..Default::default()
            },
        )
        .ok()?;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("minimap"),
            size: wgpu::Extent3d { width: SIDE, height: SIDE, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        let sample_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("minimap srgb"),
            size: wgpu::Extent3d { width: SIDE, height: SIDE, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let sample_view = sample_texture.create_view(&Default::default());

        let shot = Renderer::with_device(device.clone(), queue, SIDE, SIDE);
        let backdrop = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("minimap backdrop"),
            size: wgpu::Extent3d { width: SIDE, height: SIDE, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: shot.format(),
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let backdrop_view = backdrop.create_view(&Default::default());

        Some(Self {
            net,
            composite: Composite::new(&device, format),
            leaf: mesocosm_views::minimap_leaf(world),
            view,
            sample_view,
            sample_texture,
            texture,
            shot,
            backdrop_view,
            _backdrop: backdrop,
            rendered_at: None,
        })
    }

    /// Re-renders the enclosure's self-portrait if the cadence has elapsed.
    ///
    /// The same scene items the frame draws, seen from straight above: the
    /// backdrop is the world's own image, generated, never an asset.
    pub fn render_backdrop(&mut self, items: &[SceneItem], steps: u64) {
        if self
            .rendered_at
            .is_some_and(|then| steps.saturating_sub(then) < BACKDROP_CADENCE)
        {
            return;
        }
        self.rendered_at = Some(steps);

        let mut encoder = self
            .shot
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("backdrop") });
        self.shot.draw_scene(&mut encoder, &self.backdrop_view, items, &overhead());
        self.shot.queue().submit(Some(encoder.finish()));
    }

    /// Reprojects the world and rasterizes the minimap if anything changed.
    ///
    /// The leaf dedups identical projections, so an idle world costs a scene
    /// build and no raster.
    pub fn refresh(&mut self, world: &World) {
        self.leaf.refresh_from(mesocosm_views::minimap_leaf(world));

        if !self.leaf.paint_dirty() {
            return;
        }
        let mut cmds = Vec::new();
        let mut cx = PaintCx::new(&mut cmds, Size { width: SIDE as f32, height: SIDE as f32 });
        self.leaf.paint(&mut cx);

        let translated = paint_list_render::translate_paint_cmd_stream(
            paint_list_api::DeviceIntSize::new(SIDE as i32, SIDE as i32),
            &cmds,
            &[],
            &[],
        );
        self.net.render_vello(
            &translated.scene,
            &self.view,
            ColorLoad::Clear(wgpu::Color::TRANSPARENT),
        );

        // Into the sRGB twin the composite samples. Byte-identical; only the
        // format tag differs, which is the entire point.
        let mut encoder = self
            .shot
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("srgb twin") });
        encoder.copy_texture_to_texture(
            self.texture.as_image_copy(),
            self.sample_texture.as_image_copy(),
            wgpu::Extent3d { width: SIDE, height: SIDE, depth_or_array_layers: 1 },
        );
        self.shot.queue().submit(Some(encoder.finish()));
    }

    /// Composites into a capture frame, which uses the offscreen format
    /// rather than the surface's. Builds a one-shot composite pipeline; capture
    /// is rare and evidence beats economy there.
    pub fn capture_composite(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        frame: (u32, u32),
    ) {
        let composite = Composite::new(device, format);
        let x = frame.0 as f32 - SIDE as f32 - MARGIN;
        let dest = (x.max(0.0), MARGIN, SIDE as f32, SIDE as f32);
        composite.draw(device, queue, encoder, target, &self.backdrop_view, dest, frame);
        composite.draw(device, queue, encoder, target, &self.sample_view, dest, frame);
    }

    /// Blends the minimap into the frame's top-right corner.
    pub fn composite(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        frame: (u32, u32),
    ) {
        let x = frame.0 as f32 - SIDE as f32 - MARGIN;
        let dest = (x.max(0.0), MARGIN, SIDE as f32, SIDE as f32);
        // Terrain under territory: the world's own image first, the cells
        // whose translucency exists for exactly this on top.
        self.composite.draw(device, queue, encoder, target, &self.backdrop_view, dest, frame);
        self.composite.draw(device, queue, encoder, target, &self.sample_view, dest, frame);
    }
}
