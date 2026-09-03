// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The chrome device: one netrender instance and one blend pass, shared by
//! every chrome surface the frame carries.
//!
//! Both lanes end the same way — a vello scene rasterized into a transparent
//! texture on the game's own device, then blended over the frame. The minimap
//! (painted lane) and the vitals panel (cambium lane) differ only in what
//! builds the scene, so the device, the texture pair, and the composite live
//! here once rather than twice.
//!
//! `Renderer::render_vello` takes `&self`, so two surfaces do not want two
//! netrender instances; they want two targets.

use mesocosm_render::composite::Composite;
use netrender::{
    ColorLoad, Renderer as NetRenderer, Scene, WgpuHandles, create_netrender_instance,
};

pub struct Chrome {
    net: NetRenderer,
    /// The blend pass for the surface's format. A capture writes to the
    /// offscreen format instead and builds its own; captures are rare and
    /// evidence beats economy there.
    composite: Composite,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

/// A vello target and the sRGB-tagged twin it is sampled through.
///
/// Vello writes display-encoded values into a plain `Unorm` texture (its target
/// must be, because it writes through a storage binding and sRGB formats
/// cannot be). Sampling those bytes raw and writing to an sRGB target encodes
/// twice and brightens everything, so each raster is byte-copied into this
/// copy-compatible twin, whose decode-on-sample cancels the target's encode.
pub struct Raster {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sample_texture: wgpu::Texture,
    sample_view: wgpu::TextureView,
    size: (u32, u32),
}

impl Raster {
    pub fn new(device: &wgpu::Device, label: &str, width: u32, height: u32) -> Self {
        let extent = wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: extent,
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
        let sample_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        Self {
            view: texture.create_view(&Default::default()),
            sample_view: sample_texture.create_view(&Default::default()),
            texture,
            sample_texture,
            size: (extent.width, extent.height),
        }
    }

    /// What a composite samples.
    pub fn sample_view(&self) -> &wgpu::TextureView {
        &self.sample_view
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }
}

impl Chrome {
    /// Builds the chrome device. `None` if netrender declines it, in which case
    /// the game runs chromeless rather than not at all.
    pub fn new(handles: WgpuHandles, format: wgpu::TextureFormat, tiles: u32) -> Option<Self> {
        let device = handles.device.clone();
        let queue = handles.queue.clone();
        let net = create_netrender_instance(
            handles,
            netrender::NetrenderOptions {
                tile_cache_size: Some(tiles),
                enable_vello: true,
                ..Default::default()
            },
        )
        .ok()?;
        Some(Self {
            composite: Composite::new(&device, format),
            net,
            device,
            queue,
        })
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// Rasterizes `scene` into `raster` and refreshes its sRGB twin.
    pub fn raster(&self, raster: &Raster, scene: &Scene) {
        self.net.render_vello(
            scene,
            &raster.view,
            ColorLoad::Clear(wgpu::Color::TRANSPARENT),
        );
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("srgb twin"),
            });
        encoder.copy_texture_to_texture(
            raster.texture.as_image_copy(),
            raster.sample_texture.as_image_copy(),
            wgpu::Extent3d {
                width: raster.size.0,
                height: raster.size.1,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit(Some(encoder.finish()));
    }

    /// Blends `content` into the frame at `dest`.
    pub fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        content: &wgpu::TextureView,
        dest: (f32, f32, f32, f32),
        frame: (u32, u32),
    ) {
        self.composite.draw(
            &self.device,
            &self.queue,
            encoder,
            target,
            content,
            dest,
            frame,
        );
    }

    /// The same blend into a capture frame, which uses the offscreen format
    /// rather than the surface's.
    pub fn draw_as(
        &self,
        format: wgpu::TextureFormat,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        content: &wgpu::TextureView,
        dest: (f32, f32, f32, f32),
        frame: (u32, u32),
    ) {
        Composite::new(&self.device, format).draw(
            &self.device,
            &self.queue,
            encoder,
            target,
            content,
            dest,
            frame,
        );
    }
}
