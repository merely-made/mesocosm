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
use mesocosm_render::overlay::Overlay;
use mesocosm_views::MinimapLeaf;
use netrender::{ColorLoad, Renderer as NetRenderer, WgpuHandles, create_netrender_instance};
use sprigging::{Leaf, PaintCx, Size};

/// The minimap's square side, in pixels. Also the rasterized texture's size,
/// so the leaf paints at the resolution it is shown.
const SIDE: u32 = 160;

/// Distance from the frame's corner.
const MARGIN: f32 = 12.0;

pub struct Hud {
    net: NetRenderer,
    overlay: Overlay,
    leaf: MinimapLeaf,
    view: wgpu::TextureView,
    /// Keeps the texture alive for its view.
    _texture: wgpu::Texture,
}

impl Hud {
    /// Builds the HUD on the game's device. `None` if netrender declines the
    /// device, in which case the game simply runs chromeless.
    pub fn new(handles: WgpuHandles, format: wgpu::TextureFormat, world: &World) -> Option<Self> {
        let device = handles.device.clone();
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
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());

        Some(Self {
            net,
            overlay: Overlay::new(&device, format),
            leaf: mesocosm_views::minimap_leaf(world),
            view,
            _texture: texture,
        })
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
    }

    /// Composites into a capture frame, which uses the offscreen format
    /// rather than the surface's. Builds a one-shot overlay pipeline; capture
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
        let overlay = Overlay::new(device, format);
        let x = frame.0 as f32 - SIDE as f32 - MARGIN;
        overlay.draw(
            device,
            queue,
            encoder,
            target,
            &self.view,
            (x.max(0.0), MARGIN, SIDE as f32, SIDE as f32),
            frame,
        );
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
        self.overlay.draw(
            device,
            queue,
            encoder,
            target,
            &self.view,
            (x.max(0.0), MARGIN, SIDE as f32, SIDE as f32),
            frame,
        );
    }
}
