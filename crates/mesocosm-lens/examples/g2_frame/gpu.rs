// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::sync::Arc;

use mesocosm_core::places::{Ground, Places};
use mesocosm_lens::{
    BrickFrameInput, BrickMap, BrickRevision, BrickTracer, CritterPose, FRAME_FORMAT, Flight,
    Grade, critter::Capsule,
};
use mesocosm_render::composite::Composite;
use netrender::{
    Compositor, ExternalTextureComposite, ExternalTexturePlacement, NetrenderOptions,
    PresentedFrame, Scene, WgpuHandles, create_netrender_instance,
};
use winit::window::Window;

use crate::receipt::Receipt;

const MASTER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

struct Document {
    map: BrickMap,
    revision: BrickRevision,
    flight: Flight,
    grade: Grade,
    pose: CritterPose,
    bytes: Vec<u8>,
}

pub struct Gpu {
    pub surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    handles: WgpuHandles,
    tracer: BrickTracer,
    trace_target: wgpu::Texture,
    trace_view: wgpu::TextureView,
    net: netrender::Renderer,
    composite: Composite,
    document: Document,
    chrome: Scene,
}

impl Gpu {
    pub async fn new(window: Arc<Window>) -> Result<Self, String> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = instance
            .create_surface(window)
            .map_err(|error| error.to_string())?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                apply_limit_buckets: false,
                compatible_surface: Some(&surface),
            })
            .await
            .map_err(|error| error.to_string())?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Mesocosm G2"),
                required_features: netrender::REQUIRED_FEATURES,
                required_limits: wgpu::Limits {
                    max_inter_stage_shader_variables: 28,
                    ..Default::default()
                },
                ..Default::default()
            })
            .await
            .map_err(|error| error.to_string())?;
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|format| format.is_srgb())
            .unwrap_or(caps.formats[0]);
        let size = [960, 540];
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size[0],
            height: size[1],
            present_mode: caps.present_modes[0],
            // wgpu 30 made surface color space explicit; Auto keeps the pre-30
            // platform-chosen behavior.
            color_space: wgpu::SurfaceColorSpace::Auto,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);
        let handles = WgpuHandles {
            instance,
            adapter,
            device,
            queue,
        };
        let net = create_netrender_instance(
            handles.clone(),
            NetrenderOptions {
                tile_cache_size: Some(64),
                enable_vello: true,
                ..Default::default()
            },
        )
        .map_err(|error| format!("netrender init failed: {error:?}"))?;
        let document = document()?;
        let tracer = BrickTracer::with_format(
            handles.device.clone(),
            handles.queue.clone(),
            size[0],
            size[1],
            FRAME_FORMAT,
        );
        let (trace_target, trace_view) = trace_target(&handles.device, size);
        Ok(Self {
            surface,
            surface_config,
            handles: handles.clone(),
            tracer,
            trace_target,
            trace_view,
            net,
            composite: Composite::new(&handles.device, format),
            document,
            chrome: chrome_scene(size),
        })
    }

    fn configure(&self) {
        self.surface
            .configure(&self.handles.device, &self.surface_config);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let size = [width.max(1), height.max(1)];
        self.surface_config.width = size[0];
        self.surface_config.height = size[1];
        self.configure();
        self.tracer.resize(size[0], size[1]);
        (self.trace_target, self.trace_view) = trace_target(&self.handles.device, size);
        self.chrome = chrome_scene(size);
    }

    pub fn draw(&mut self, frame: u32) -> Result<Option<(Receipt, wgpu::Texture)>, String> {
        let surface = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
                self.configure();
                return Ok(None);
            }
            wgpu::CurrentSurfaceTexture::Timeout => return Ok(None),
            _ => return Err("surface acquisition failed".into()),
        };
        let view = surface.texture.create_view(&Default::default());
        let rendered = self.render(&view, frame);
        // wgpu 30 moved presentation from SurfaceTexture to Queue.
        self.handles.queue.present(surface);
        rendered.map(Some)
    }

    fn render(
        &mut self,
        surface_view: &wgpu::TextureView,
        frame: u32,
    ) -> Result<(Receipt, wgpu::Texture), String> {
        let mut encoder =
            self.handles
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Mesocosm G2 trace"),
                });
        let trace = self
            .tracer
            .encode(
                &mut encoder,
                &self.trace_view,
                BrickFrameInput::new(
                    &self.document.map,
                    self.document.revision,
                    &self.document.flight,
                    &self.document.grade,
                )
                .with_pose(&self.document.pose),
            )
            .map_err(|error| error.to_string())?;
        self.handles.queue.submit([encoder.finish()]);

        let size = [self.surface_config.width, self.surface_config.height];
        let external = [ExternalTextureComposite::new(
            &self.trace_view,
            ExternalTexturePlacement::new([0.0, 0.0, size[0] as f32, size[1] as f32]),
        )
        .with_scene_op_boundary(0)];
        let mut present = FramePresenter {
            target: surface_view,
            composite: &self.composite,
            size,
            master: None,
        };
        self.net.render_with_compositor_and_external_textures(
            &self.chrome,
            MASTER_FORMAT,
            &mut present,
            netrender::peniko::Color::new([0.0, 0.0, 0.0, 0.0]),
            &external,
        );
        let master = present
            .master
            .ok_or("netrender did not present a master texture")?;
        let timings = self
            .net
            .last_frame_timings()
            .ok_or("netrender did not report frame timings")?;
        Ok((
            Receipt::new(
                frame,
                size,
                self.surface_config.format,
                &self.document.bytes,
                &self.handles.adapter,
                trace,
                timings,
                self.net.vello_last_dirty_count().unwrap_or_default(),
            ),
            master,
        ))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn write_native_receipts(
        &self,
        receipt_path: Option<&std::path::Path>,
        capture_path: Option<&std::path::Path>,
        json: &str,
        master: &wgpu::Texture,
    ) -> Result<(), String> {
        if let Some(path) = receipt_path {
            ensure_parent(path)?;
            std::fs::write(path, json).map_err(|error| error.to_string())?;
        }
        let Some(path) = capture_path else {
            return Ok(());
        };
        ensure_parent(path)?;
        let pixels = self.net.wgpu_device.read_rgba8_texture(
            master,
            self.surface_config.width,
            self.surface_config.height,
        );
        let file = std::fs::File::create(path).map_err(|error| error.to_string())?;
        let mut png =
            png::Encoder::new(file, self.surface_config.width, self.surface_config.height);
        png.set_color(png::ColorType::Rgba);
        png.set_depth(png::BitDepth::Eight);
        png.write_header()
            .and_then(|mut writer| writer.write_image_data(&pixels))
            .map_err(|error| error.to_string())
    }
}

struct FramePresenter<'a> {
    target: &'a wgpu::TextureView,
    composite: &'a Composite,
    size: [u32; 2],
    master: Option<wgpu::Texture>,
}

impl Compositor for FramePresenter<'_> {
    fn declare_surface(&mut self, _key: netrender::SurfaceKey, _world_bounds: [f32; 4]) {}

    fn destroy_surface(&mut self, _key: netrender::SurfaceKey) {}

    fn present_frame(&mut self, frame: PresentedFrame<'_>) {
        let master_view = frame.master.create_view(&Default::default());
        let mut encoder =
            frame
                .handles
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Mesocosm G2 present"),
                });
        {
            let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Mesocosm G2 clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: self.target,
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
        }
        self.composite.draw(
            &frame.handles.device,
            &frame.handles.queue,
            &mut encoder,
            self.target,
            &master_view,
            (0.0, 0.0, self.size[0] as f32, self.size[1] as f32),
            (self.size[0], self.size[1]),
        );
        frame.handles.queue.submit([encoder.finish()]);
        self.master = Some(frame.master.clone());
    }
}

fn trace_target(device: &wgpu::Device, size: [u32; 2]) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Mesocosm G2 trace target"),
        size: wgpu::Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FRAME_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    (texture, view)
}

fn chrome_scene(size: [u32; 2]) -> Scene {
    let (width, height) = (size[0] as f32, size[1] as f32);
    let mut scene = Scene::new(size[0], size[1]);
    scene.push_rect(0.0, 0.0, width, 42.0, [0.025, 0.035, 0.055, 0.88]);
    scene.push_rect(18.0, 13.0, 168.0, 29.0, [0.27, 0.85, 0.60, 0.92]);
    scene.push_rect(
        width - 142.0,
        13.0,
        width - 18.0,
        29.0,
        [0.95, 0.68, 0.28, 0.92],
    );
    scene.push_rect(0.0, 0.0, width, 2.0, [0.50, 0.95, 0.72, 1.0]);
    scene.push_rect(0.0, height - 2.0, width, height, [0.50, 0.95, 0.72, 1.0]);
    scene
}

fn document() -> Result<Document, String> {
    let ground = Ground::grow(&Places::grown(4_242, 4, 64), 64);
    let map = BrickMap::from_ground(&ground).map_err(|error| error.to_string())?;
    let eye_top = ground
        .surface(4, 4)
        .ok_or("fixture column is outside Ground")? as f32;
    let body_top = ground
        .surface(4, 18)
        .ok_or("body fixture column is outside Ground")? as f32;
    let body = [4.5, body_top + 1.15, 18.5];
    let eye = [4.5, eye_top + 17.0, 4.5];
    let distance = ((body[0] - eye[0]).powi(2) + (body[2] - eye[2]).powi(2)).sqrt();
    let flight = Flight {
        eye,
        yaw: 0.0,
        pitch: f32::atan2(body[1] - eye[1], distance),
        fov: 0.9,
        far: 48.0,
    };
    let pose = CritterPose::from_capsules(
        vec![Capsule {
            a: [body[0] - 0.7, body[1], body[2]],
            ra: 0.65,
            b: [body[0] + 0.7, body[1], body[2]],
            rb: 0.52,
        }],
        [
            [body[0] - 0.45, body[1] + 0.15, body[2] - 0.35, 0.10],
            [body[0] - 0.45, body[1] - 0.15, body[2] - 0.35, 0.10],
        ],
        [0.15, 0.86, 0.32],
    );
    let grade = Grade::retro(3);
    let bytes = postcard::to_allocvec(&(ground, flight, grade, pose.clone()))
        .map_err(|error| error.to_string())?;
    Ok(Document {
        map,
        revision: BrickRevision(0),
        flight,
        grade,
        pose,
        bytes,
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn ensure_parent(path: &std::path::Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    Ok(())
}
