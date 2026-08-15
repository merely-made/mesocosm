// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Retained fragment-only DDA rendering of [`crate::BrickMap`].
//!
//! The tracer owns GPU copies, never voxel authority. A caller gives it a
//! revision and the slots changed by a projection drain; it encodes into a
//! caller-owned target just like [`crate::Lens`].

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

mod types;

use crate::{BrickMap, FRAME_FORMAT};
use types::{TraceParams, validates_pose};

pub use types::{
    BrickCapture, BrickChange, BrickDiagnostics, BrickFrameInput, BrickRevision, BrickTraceError,
};

struct ResidentMap {
    pointer_extent: [u32; 3],
    atlas_extent: [u32; 3],
    revision: BrickRevision,
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
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("brick tracer"),
            source: wgpu::ShaderSource::Wgsl(include_str!("tracer.wgsl").into()),
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
        let started = Instant::now();
        validates_pose(input)?;
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
        diagnostics.trace_passes = 1;
        self.last_diagnostics = Some(diagnostics);
        Ok(diagnostics)
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
        let data = slice.get_mapped_range();
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
        if resident.revision == input.revision && !recreate {
            return;
        }
        if recreate || matches!(input.change, BrickChange::Full) {
            write_texture_3d(
                &self.queue,
                &resident.pointer,
                [0, 0, 0],
                input.map.pointer_extent(),
                4,
                bytemuck::cast_slice(input.map.pointers()),
            );
            write_texture_3d(
                &self.queue,
                &resident.atlas,
                [0, 0, 0],
                input.map.atlas_extent(),
                1,
                input.map.atlas(),
            );
            diagnostics.brick_upload_bytes +=
                (size_of_val(input.map.pointers()) + input.map.atlas().len()) as u64;
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
                diagnostics.brick_upload_bytes += (size_of::<u32>() + texels.len()) as u64;
            }
        }
        resident.revision = input.revision;
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
