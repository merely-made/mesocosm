// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Retained fragment-only rendering over [`conatus_brick::BrickMap`].
//!
//! The tracer owns GPU copies, never voxel authority. A caller gives it a
//! revision and the slots changed by a projection drain; it encodes into a
//! caller-owned target just like [`crate::Lens`].

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

mod types;

use conatus_brick::{BRICK_DDA_WGSL, BrickMap, BrickProjectionRevision};

use crate::FRAME_FORMAT;
use types::{TraceParams, validates_change, validates_pose};

pub use types::{
    BrickCapture, BrickChange, BrickDiagnostics, BrickFrameInput, BrickRevision, BrickTraceError,
    LeasedAtlas, TraceCamera,
};

struct ResidentMap {
    pointer_extent: [u32; 3],
    atlas_extent: [u32; 3],
    revision: BrickRevision,
    projection_revision: BrickProjectionRevision,
    pointer: wgpu::Texture,
    atlas: wgpu::Texture,
    bind: wgpu::BindGroup,
}

struct CaptureResources {
    width: u32,
    height: u32,
    target: wgpu::Texture,
    view: wgpu::TextureView,
    staging: wgpu::Buffer,
    padded_bytes_per_row: u32,
}

pub struct BrickTracer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
    pipeline: wgpu::RenderPipeline,
    /// The depth-join variant: `fs_depth` against a caller-owned
    /// `Depth32Float` attachment. Built on first use so plain tracing pays
    /// nothing for it.
    depth_pipeline: Option<wgpu::RenderPipeline>,
    shader: wgpu::ShaderModule,
    pipeline_layout: wgpu::PipelineLayout,
    layout: wgpu::BindGroupLayout,
    params: wgpu::Buffer,
    map: Option<ResidentMap>,
    capture: Option<CaptureResources>,
    pending_resource_creations: u32,
    last_diagnostics: Option<BrickDiagnostics>,
    last_params: Option<TraceParams>,
}

impl BrickTracer {
    pub fn headless(width: u32, height: u32) -> Option<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .ok()?;
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).ok()?;
        Some(Self::with_device(device, queue, width, height))
    }

    /// The device this tracer renders on, so a resident producer can
    /// allocate its lease against the same one. A lease from another
    /// device cannot be copied into this tracer's atlas.
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn with_device(device: wgpu::Device, queue: wgpu::Queue, width: u32, height: u32) -> Self {
        Self::with_format(device, queue, width, height, FRAME_FORMAT)
    }

    pub fn with_format(
        device: wgpu::Device,
        queue: wgpu::Queue,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Self {
        let texture = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Uint,
                view_dimension: wgpu::TextureViewDimension::D3,
                multisampled: false,
            },
            count: None,
        };
        let uniform = wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("brick tracer layout"),
            entries: &[texture(0), texture(1), uniform],
        });
        let shader_source = format!("{BRICK_DDA_WGSL}\n{}", include_str!("tracer.wgsl"));
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("brick tracer"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("brick tracer"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("brick tracer"),
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
                    format,
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
        });
        let params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("brick tracer parameters"),
            size: size_of::<TraceParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            device,
            queue,
            format,
            width: width.max(1),
            height: height.max(1),
            pipeline,
            depth_pipeline: None,
            shader,
            pipeline_layout,
            layout,
            params,
            map: None,
            capture: None,
            pending_resource_creations: 1,
            last_diagnostics: None,
            last_params: None,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width.max(1);
        self.height = height.max(1);
        self.capture = None;
    }

    pub fn last_diagnostics(&self) -> Option<BrickDiagnostics> {
        self.last_diagnostics
    }

    pub fn encode(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        input: BrickFrameInput<'_>,
    ) -> Result<BrickDiagnostics, BrickTraceError> {
        let mut diagnostics = self.prepare_frame(input)?;
        let bind = &self.map.as_ref().expect("map ensured").bind;
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("brick trace"),
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
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, bind, &[]);
        pass.draw(0..3, 0..1);
        drop(pass);
        diagnostics.trace_passes = 1;
        self.last_diagnostics = Some(diagnostics);
        Ok(diagnostics)
    }

    /// The depth join: trace into a raster tenant's colour target against
    /// its stored `Depth32Float` depth, so the two occlude each other per
    /// pixel.
    ///
    /// The caller renders its raster pass first with depth stored, then
    /// this pass loads both attachments, writes `@builtin(frag_depth)` from
    /// the frame's `clip_from_world`, and tests `LessEqual` with depth
    /// write on. Standard-z is assumed, matching a depth cleared to 1.0
    /// under a `Less` raster compare; `LessEqual` here lets the traced sky,
    /// clamped to the far plane, replace the raster background while never
    /// covering nearer geometry.
    pub fn encode_with_depth(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        input: BrickFrameInput<'_>,
    ) -> Result<BrickDiagnostics, BrickTraceError> {
        if input.clip_from_world.is_none() {
            return Err(BrickTraceError::MissingClipFromWorld);
        }
        if self.depth_pipeline.is_none() {
            self.depth_pipeline = Some(self.create_depth_pipeline());
            self.pending_resource_creations += 1;
        }
        let mut diagnostics = self.prepare_frame(input)?;
        let bind = &self.map.as_ref().expect("map ensured").bind;
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("brick trace depth join"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(self.depth_pipeline.as_ref().expect("depth pipeline built"));
        pass.set_bind_group(0, bind, &[]);
        pass.draw(0..3, 0..1);
        drop(pass);
        diagnostics.trace_passes = 1;
        self.last_diagnostics = Some(diagnostics);
        Ok(diagnostics)
    }

    /// Everything a trace pass needs before it begins: validation, resident
    /// textures, and the uniform, shared by both encode paths.
    fn prepare_frame(
        &mut self,
        input: BrickFrameInput<'_>,
    ) -> Result<BrickDiagnostics, BrickTraceError> {
        let started = Instant::now();
        validates_pose(input)?;
        validates_change(input)?;
        let mut diagnostics = BrickDiagnostics {
            resource_creations: std::mem::take(&mut self.pending_resource_creations),
            ..Default::default()
        };
        self.ensure_map(input, &mut diagnostics);
        let params = TraceParams::from_input(input);
        if self.last_params != Some(params) {
            self.queue
                .write_buffer(&self.params, 0, bytemuck::bytes_of(&params));
            diagnostics.uniform_upload_bytes = size_of::<TraceParams>() as u64;
            self.last_params = Some(params);
        }
        diagnostics.cpu_prepare_us = started.elapsed().as_micros() as u64;
        Ok(diagnostics)
    }

    fn create_depth_pipeline(&self) -> wgpu::RenderPipeline {
        self.device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("brick tracer depth join"),
                layout: Some(&self.pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &self.shader,
                    entry_point: Some("vs"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &self.shader,
                    entry_point: Some("fs_depth"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: self.format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::LessEqual),
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            })
    }

    pub fn capture(&mut self, input: BrickFrameInput<'_>) -> Result<BrickCapture, BrickTraceError> {
        if self.format != FRAME_FORMAT {
            return Err(BrickTraceError::CaptureFormat(self.format));
        }
        self.ensure_capture();
        let capture = self.capture.as_ref().expect("capture ensured");
        let view = capture.view.clone();
        let target = capture.target.clone();
        let staging = capture.staging.clone();
        let padded = capture.padded_bytes_per_row;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("brick trace capture"),
            });
        let mut diagnostics = self.encode(&mut encoder, &view, input)?;
        encoder.copy_texture_to_buffer(
            target.as_image_copy(),
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
        self.queue.submit([encoder.finish()]);
        let slice = staging.slice(..);
        let (send, receive) = std::sync::mpsc::sync_channel(1);
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = send.send(result);
        });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|error| BrickTraceError::DevicePoll(error.to_string()))?;
        receive
            .recv()
            .map_err(|error| BrickTraceError::Readback(error.to_string()))?
            .map_err(|error| BrickTraceError::Readback(error.to_string()))?;
        let data = slice.get_mapped_range().expect("map range");
        let mut pixels = Vec::with_capacity((self.width * self.height * 4) as usize);
        for row in 0..self.height {
            let start = (row * padded) as usize;
            pixels.extend_from_slice(&data[start..start + (self.width * 4) as usize]);
        }
        drop(data);
        staging.unmap();
        diagnostics.readback_bytes = pixels.len() as u64;
        self.last_diagnostics = Some(diagnostics);
        Ok(BrickCapture {
            width: self.width,
            height: self.height,
            pixels,
            diagnostics,
        })
    }

    fn ensure_map(&mut self, input: BrickFrameInput<'_>, diagnostics: &mut BrickDiagnostics) {
        let recreate = self.map.as_ref().is_none_or(|resident| {
            resident.pointer_extent != input.map.pointer_extent()
                || resident.atlas_extent != input.map.atlas_extent()
        });
        if recreate {
            self.map = Some(self.create_map(input.map));
            diagnostics.resource_creations += 2;
            diagnostics.bind_group_rebuilds += 1;
            diagnostics.map_recreated = true;
        }
        let resident = self.map.as_mut().expect("map created");
        let projection_changed = resident.projection_revision != input.map.projection_revision();
        if resident.revision == input.revision && !projection_changed && !recreate {
            return;
        }
        diagnostics.projection_replaced = projection_changed && !recreate;
        // A leased atlas replaces the CPU upload for the voxels it
        // covers: the producer already has them resident, so they move
        // GPU-side. The pointer volume still uploads from the map,
        // which is tiny and identifies slots rather than carrying
        // material.
        let atlas_from_lease = if let Some(leased) = input.leased_atlas {
            if leased.revision != input.revision {
                diagnostics.stale_lease_rejections += 1;
                false
            } else if leased.projection_revision != input.map.projection_revision() {
                diagnostics.projection_lease_rejections += 1;
                false
            } else if !leased.copyable_into(resident.atlas_extent) {
                // An invalid strided range could copy a neighbouring
                // allocation out of the producer's pool or cross the atlas.
                diagnostics.misfit_lease_rejections += 1;
                false
            } else if !lease_covers_change(
                leased,
                input.map,
                input.change,
                recreate || projection_changed,
            ) {
                // A valid partial lease must not suppress uploads for changed
                // slots it does not actually contain.
                diagnostics.incomplete_lease_rejections += 1;
                false
            } else {
                copy_leased_atlas(&self.device, &self.queue, &resident.atlas, leased);
                diagnostics.leased_atlas_bytes += leased.byte_len();
                diagnostics.observed_read_epoch = Some(leased.read_epoch);
                true
            }
        } else {
            false
        };

        // The CPU upload stands unless a lease actually took the atlas:
        // a refused lease must not leave the
        // atlas unwritten, or the frame shows whatever was there before.
        let atlas_from_cpu = !atlas_from_lease;
        if recreate || projection_changed || matches!(input.change, BrickChange::Full) {
            write_texture_3d(
                &self.queue,
                &resident.pointer,
                [0, 0, 0],
                input.map.pointer_extent(),
                4,
                bytemuck::cast_slice(input.map.pointers()),
            );
            diagnostics.brick_upload_bytes += size_of_val(input.map.pointers()) as u64;
            if atlas_from_cpu {
                write_texture_3d(
                    &self.queue,
                    &resident.atlas,
                    [0, 0, 0],
                    input.map.atlas_extent(),
                    1,
                    input.map.atlas(),
                );
                diagnostics.brick_upload_bytes += input.map.atlas().len() as u64;
            }
        } else if let BrickChange::Slots(slots) = input.change {
            for slot in slots {
                let Some(pointer_coord) = input.map.pointer_coord(*slot) else {
                    continue;
                };
                let pointer = input.map.pointer_at(pointer_coord).expect("in bounds");
                write_texture_3d(
                    &self.queue,
                    &resident.pointer,
                    pointer_coord,
                    [1, 1, 1],
                    4,
                    bytemuck::bytes_of(&pointer),
                );
                diagnostics.brick_upload_bytes += size_of::<u32>() as u64;
                if atlas_from_cpu {
                    let atlas_origin = input.map.atlas_slot_origin(*slot).expect("assigned slot");
                    let texels = input.map.slot_texels(*slot).expect("assigned slot");
                    write_texture_3d(
                        &self.queue,
                        &resident.atlas,
                        atlas_origin,
                        [8, 8, 8],
                        1,
                        &texels,
                    );
                    diagnostics.brick_upload_bytes += texels.len() as u64;
                }
            }
        }
        resident.revision = input.revision;
        resident.projection_revision = input.map.projection_revision();
    }

    fn create_map(&self, map: &BrickMap) -> ResidentMap {
        let texture = |label: &str, extent: [u32; 3], format: wgpu::TextureFormat| {
            self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: extent[0],
                    height: extent[1],
                    depth_or_array_layers: extent[2],
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D3,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            })
        };
        let pointer = texture(
            "brick pointers",
            map.pointer_extent(),
            wgpu::TextureFormat::R32Uint,
        );
        let pointer_view = pointer.create_view(&Default::default());
        let atlas = texture(
            "brick atlas",
            map.atlas_extent(),
            wgpu::TextureFormat::R8Uint,
        );
        let atlas_view = atlas.create_view(&Default::default());
        let bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("brick tracer map"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&pointer_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.params.as_entire_binding(),
                },
            ],
        });
        ResidentMap {
            pointer_extent: map.pointer_extent(),
            atlas_extent: map.atlas_extent(),
            revision: BrickRevision(u64::MAX),
            projection_revision: BrickProjectionRevision(u64::MAX),
            pointer,
            atlas,
            bind,
        }
    }

    fn ensure_capture(&mut self) {
        if self
            .capture
            .as_ref()
            .is_some_and(|capture| (capture.width, capture.height) == (self.width, self.height))
        {
            return;
        }
        let target = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("brick trace capture"),
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let padded_bytes_per_row =
            (self.width * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("brick trace readback"),
            size: u64::from(padded_bytes_per_row) * u64::from(self.height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        self.capture = Some(CaptureResources {
            width: self.width,
            height: self.height,
            view: target.create_view(&Default::default()),
            target,
            staging,
            padded_bytes_per_row,
        });
    }
}

fn lease_covers_change(
    leased: LeasedAtlas<'_>,
    map: &BrickMap,
    change: BrickChange<'_>,
    recreate: bool,
) -> bool {
    if recreate || matches!(change, BrickChange::Full) {
        return leased.covers([0; 3], map.atlas_extent());
    }
    let BrickChange::Slots(slots) = change else {
        return false;
    };
    slots.iter().all(|slot| {
        map.atlas_slot_origin(*slot)
            .is_some_and(|origin| leased.covers(origin, [8; 3]))
    })
}

/// Copy a producer's resident voxels into the atlas texture, GPU-side.
///
/// Buffer-to-texture copies require `COPY_BYTES_PER_ROW_ALIGNMENT`-byte
/// rows and a brick row is eight bytes, so the leased rows are repacked
/// into an aligned staging buffer first. That repack is device-local:
/// the CPU still never sees a voxel.
fn copy_leased_atlas(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    atlas: &wgpu::Texture,
    leased: LeasedAtlas<'_>,
) {
    let [width, height, depth] = leased.extent;
    if width == 0 || height == 0 || depth == 0 {
        return;
    }
    let aligned_row = width.next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
    let rows = height * depth;
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("leased atlas repack"),
    });
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("leased atlas staging"),
        size: (aligned_row * rows) as u64,
        usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    for z in 0..depth {
        for y in 0..height {
            let destination_row = z * height + y;
            let source_row = (leased.source_origin[2] + z) * leased.source_rows_per_image
                + leased.source_origin[1]
                + y;
            let source_offset = leased.offset
                + u64::from(source_row) * u64::from(leased.source_bytes_per_row)
                + u64::from(leased.source_origin[0]);
            encoder.copy_buffer_to_buffer(
                leased.buffer,
                source_offset,
                &staging,
                (destination_row * aligned_row) as u64,
                width as u64,
            );
        }
    }
    encoder.copy_buffer_to_texture(
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(aligned_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::TexelCopyTextureInfo {
            texture: atlas,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: leased.slot_origin[0],
                y: leased.slot_origin[1],
                z: leased.slot_origin[2],
            },
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: depth,
        },
    );
    queue.submit([encoder.finish()]);
}

fn write_texture_3d(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    origin: [u32; 3],
    extent: [u32; 3],
    bytes_per_texel: u32,
    data: &[u8],
) {
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: origin[0],
                y: origin[1],
                z: origin[2],
            },
            aspect: wgpu::TextureAspect::All,
        },
        data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(extent[0] * bytes_per_texel),
            rows_per_image: Some(extent[1]),
        },
        wgpu::Extent3d {
            width: extent[0],
            height: extent[1],
            depth_or_array_layers: extent[2],
        },
    );
}
