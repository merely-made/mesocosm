// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The biosphere lens probe.
//!
//! Two passes: the **march** (a fullscreen heightfield raymarch, the Voxel
//! Space lineage with free pitch) renders terrain from two images; the
//! **grade** (fog, palette LUT, ordered dither) turns the same march into any
//! look a [`Grade`] block describes. Retro and clay are two blocks, and
//! everything between them is a space.
//!
//! Everything here is presentation. Nothing reads world state; a caller
//! hands in painted maps and a camera. If the probe earns its keep this
//! becomes a netrender render-graph task, never a second custom pipeline.

pub mod maps;

use bytemuck::{Pod, Zeroable};

/// A look, as data. Worldgen can emit one of these per world or per biome
/// the way it emits a heightmap.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Grade {
    pub fog: [f32; 3],
    /// Distance (0..1 of far) where fog begins.
    pub fog_start: f32,
    /// Palette entries used; 0 disables quantisation (clay).
    pub palette_len: u32,
    /// Ordered-dither strength; 0 disables.
    pub dither: f32,
    /// Fog band count; 0 is smooth.
    pub fog_bands: f32,
    /// Internal render scale denominator: 1 = full res, 4 = quarter res
    /// integer-upscaled, which is most of the retro grain.
    pub downscale: u32,
}

impl Grade {
    /// The Comanche soul: starved palette, ordered dither, banded fog,
    /// quarter-resolution grain.
    pub fn retro(palette_len: u32) -> Self {
        Self {
            fog: [0.66, 0.66, 0.72],
            fog_start: 0.35,
            palette_len,
            dither: 0.10,
            fog_bands: 6.0,
            downscale: 4,
        }
    }

    /// The clay soul: full resolution, smooth ramp, smooth fog.
    pub fn clay() -> Self {
        Self {
            fog: [0.72, 0.74, 0.80],
            fog_start: 0.45,
            palette_len: 0,
            dither: 0.0,
            fog_bands: 0.0,
            downscale: 1,
        }
    }
}

/// A first-person camera over the heightfield, in map units.
#[derive(Clone, Copy, Debug)]
pub struct Flight {
    pub eye: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    pub far: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct MarchParams {
    eye: [f32; 3],
    yaw: f32,
    pitch: f32,
    fov: f32,
    far: f32,
    map_side: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GradeParams {
    fog: [f32; 3],
    fog_start: f32,
    palette_len: u32,
    dither: f32,
    fog_bands: f32,
    _pad: f32,
}

/// The probe renderer: headless, deterministic, capture-first.
pub struct Lens {
    device: wgpu::Device,
    queue: wgpu::Queue,
    march: wgpu::RenderPipeline,
    march_layout: wgpu::BindGroupLayout,
    grade: wgpu::RenderPipeline,
    grade_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    nearest: wgpu::Sampler,
    width: u32,
    height: u32,
}

const FRAME_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

impl Lens {
    pub fn headless(width: u32, height: u32) -> Option<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .ok()?;
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).ok()?;
        Some(Self::with_device(device, queue, width, height))
    }

    pub fn with_device(
        device: wgpu::Device,
        queue: wgpu::Queue,
        width: u32,
        height: u32,
    ) -> Self {
        let uniform_entry = |binding: u32| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let texture_entry = |binding: u32| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let sampler_entry = |binding: u32| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        };

        let march_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("march"),
            entries: &[
                texture_entry(0),
                texture_entry(1),
                sampler_entry(2),
                uniform_entry(3),
            ],
        });
        let grade_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("grade"),
            entries: &[
                texture_entry(0),
                sampler_entry(1),
                texture_entry(2),
                uniform_entry(3),
            ],
        });

        let pipeline = |label: &str,
                        source: &str,
                        layout: &wgpu::BindGroupLayout|
         -> wgpu::RenderPipeline {
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(label),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });
            let pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some(label),
                    bind_group_layouts: &[Some(layout)],
                    immediate_size: 0,
                });
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: FRAME_FORMAT,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            })
        };

        let march = pipeline("march", include_str!("march.wgsl"), &march_layout);
        let grade = pipeline("grade", include_str!("grade.wgsl"), &grade_layout);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("maps"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            ..Default::default()
        });
        let nearest = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("upscale"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            device,
            queue,
            march,
            march_layout,
            grade,
            grade_layout,
            sampler,
            nearest,
            width,
            height,
        }
    }

    fn upload(
        &self,
        label: &str,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        bytes_per_texel: u32,
        data: &[u8],
    ) -> wgpu::TextureView {
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self.queue.write_texture(
            texture.as_image_copy(),
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * bytes_per_texel),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        );
        texture.create_view(&Default::default())
    }

    /// Renders one frame through march and grade, returning RGBA pixels.
    pub fn render(&self, maps: &maps::BiomeMaps, flight: &Flight, look: &Grade) -> Vec<u8> {
        let height_view =
            self.upload("height", wgpu::TextureFormat::R8Unorm, maps.side, maps.side, 1, &maps.height);
        let color_view =
            self.upload("color", FRAME_FORMAT, maps.side, maps.side, 4, &maps.color);

        let mut palette_bytes = Vec::with_capacity(256 * 4);
        for entry in &maps.palette {
            for channel in entry {
                palette_bytes.push((channel * 255.0) as u8);
            }
            palette_bytes.push(255);
        }
        palette_bytes.resize(256 * 4, 0);
        let palette_view =
            self.upload("palette", FRAME_FORMAT, 256, 1, 4, &palette_bytes);

        // The march renders at the grade's internal resolution; the grade
        // pass reads it at the output resolution with nearest sampling, so
        // downscale is the retro grain rather than a blur.
        let scale = look.downscale.max(1);
        let (inner_w, inner_h) = (self.width / scale, self.height / scale);
        let make_target = |label: &str, w: u32, h: u32| {
            self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: FRAME_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            })
        };
        let marched = make_target("marched", inner_w.max(1), inner_h.max(1));
        let marched_view = marched.create_view(&Default::default());
        let graded = make_target("graded", self.width, self.height);
        let graded_view = graded.create_view(&Default::default());

        let march_params = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("march params"),
            size: std::mem::size_of::<MarchParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(
            &march_params,
            0,
            bytemuck::bytes_of(&MarchParams {
                eye: flight.eye,
                yaw: flight.yaw,
                pitch: flight.pitch,
                fov: flight.fov,
                far: flight.far,
                map_side: maps.side as f32,
            }),
        );
        let grade_params = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("grade params"),
            size: std::mem::size_of::<GradeParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.queue.write_buffer(
            &grade_params,
            0,
            bytemuck::bytes_of(&GradeParams {
                fog: look.fog,
                fog_start: look.fog_start,
                palette_len: look.palette_len.min(maps.palette.len() as u32),
                dither: look.dither,
                fog_bands: look.fog_bands,
                _pad: 0.0,
            }),
        );

        let march_bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("march"),
            layout: &self.march_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&height_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&color_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                wgpu::BindGroupEntry { binding: 3, resource: march_params.as_entire_binding() },
            ],
        });
        let grade_bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("grade"),
            layout: &self.grade_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&marched_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.nearest) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&palette_view) },
                wgpu::BindGroupEntry { binding: 3, resource: grade_params.as_entire_binding() },
            ],
        });

        let mut encoder = self.device.create_command_encoder(&Default::default());
        for (pipeline, bind, target) in [
            (&self.march, &march_bind, &marched_view),
            (&self.grade, &grade_bind, &graded_view),
        ] {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, bind, &[]);
            pass.draw(0..3, 0..1);
        }

        // Read back.
        let padded = (self.width * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: (padded * self.height) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &graded,
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
            wgpu::Extent3d { width: self.width, height: self.height, depth_or_array_layers: 1 },
        );
        self.queue.submit(Some(encoder.finish()));

        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("device poll");
        let data = slice.get_mapped_range();
        let mut pixels = Vec::with_capacity((self.width * self.height * 4) as usize);
        for row in 0..self.height {
            let start = (row * padded) as usize;
            pixels.extend_from_slice(&data[start..start + (self.width * 4) as usize]);
        }
        pixels
    }
}
