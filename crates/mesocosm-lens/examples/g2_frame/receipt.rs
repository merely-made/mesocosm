// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use mesocosm_lens::BrickDiagnostics;
use netrender::profiling::FrameTimings;

#[derive(Debug, serde::Serialize)]
pub struct Receipt {
    pub gate: &'static str,
    pub target: &'static str,
    pub profile: &'static str,
    pub frame: u32,
    pub size: [u32; 2],
    pub surface_format: String,
    pub scene_digest: String,
    pub scene_bytes: usize,
    pub adapter: AdapterReceipt,
    pub tracer: TracerReceipt,
    pub netrender: NetrenderReceipt,
}

#[derive(Debug, serde::Serialize)]
pub struct AdapterReceipt {
    pub name: String,
    pub backend: String,
    pub max_texture_dimension_2d: u32,
    pub max_bind_groups: u32,
}

#[derive(Debug, serde::Serialize)]
pub struct TracerReceipt {
    pub cpu_prepare_us: u64,
    pub brick_upload_bytes: u64,
    pub uniform_upload_bytes: u64,
    pub resource_creations: u32,
    pub bind_group_rebuilds: u32,
    pub map_recreated: bool,
    pub trace_passes: u32,
}

#[derive(Debug, serde::Serialize)]
pub struct NetrenderReceipt {
    pub total_us: u64,
    pub dirty_tiles: usize,
    pub spans: Vec<TimingReceipt>,
}

#[derive(Debug, serde::Serialize)]
pub struct TimingReceipt {
    pub name: &'static str,
    pub duration_us: u64,
}

impl Receipt {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        frame: u32,
        size: [u32; 2],
        format: wgpu::TextureFormat,
        scene: &[u8],
        adapter: &wgpu::Adapter,
        tracer: BrickDiagnostics,
        netrender: FrameTimings,
        dirty_tiles: usize,
    ) -> Self {
        let info = adapter.get_info();
        let limits = adapter.limits();
        Self {
            gate: "G2",
            target: if cfg!(target_arch = "wasm32") {
                "browser"
            } else {
                "native"
            },
            profile: "raster-baseline",
            frame,
            size,
            surface_format: format!("{format:?}"),
            scene_digest: format!("fnv1a64:{:016x}", digest(scene)),
            scene_bytes: scene.len(),
            adapter: AdapterReceipt {
                name: info.name,
                backend: format!("{:?}", info.backend),
                max_texture_dimension_2d: limits.max_texture_dimension_2d,
                max_bind_groups: limits.max_bind_groups,
            },
            tracer: TracerReceipt {
                cpu_prepare_us: tracer.cpu_prepare_us,
                brick_upload_bytes: tracer.brick_upload_bytes,
                uniform_upload_bytes: tracer.uniform_upload_bytes,
                resource_creations: tracer.resource_creations,
                bind_group_rebuilds: tracer.bind_group_rebuilds,
                map_recreated: tracer.map_recreated,
                trace_passes: tracer.trace_passes,
            },
            netrender: NetrenderReceipt {
                total_us: netrender.total.as_micros() as u64,
                dirty_tiles,
                spans: netrender
                    .spans
                    .into_iter()
                    .map(|span| TimingReceipt {
                        name: span.name,
                        duration_us: span.duration.as_micros() as u64,
                    })
                    .collect(),
            },
        }
    }
}

fn digest(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}
