// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The window, the surface, and the loop.

use std::sync::Arc;
use std::time::Instant;

use mesocosm_core::Intent;
use mesocosm_mesh::{BodyMesh, VolumeMap, VolumeSource, mesh_body};
use mesocosm_render::{Camera, Renderer, SceneItem};
use mesocosm_runtime::Runtime;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

use crate::fixture;

#[derive(Clone, Debug)]
pub struct HostConfig {
    pub seed: u64,
    pub organisms: u32,
    pub ticks_per_second: u32,
    pub width: u32,
    pub height: u32,
    /// Run this many frames and exit. Makes the windowed path verifiable
    /// without a person sitting in front of it.
    pub frames: Option<u32>,
    /// Write the last frame here before exiting.
    pub capture: Option<std::path::PathBuf>,
    /// Metabolize automatically every N steps, so a capture run has something
    /// to show without keyboard input.
    pub auto_eat_every: Option<u64>,
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            seed: 0x00A7_7AC4,
            organisms: 60,
            ticks_per_second: 60,
            width: 720,
            height: 720,
            frames: None,
            capture: None,
            auto_eat_every: None,
        }
    }
}

/// Half-height of the visible world region, in voxel units. Fixed, so the
/// critter moves across the view rather than the view rescaling around it.
const WORLD_EXTENT: f32 = 26.0;

/// Mirrors the core's reach. Presentation only: the core decides what is
/// actually edible, this only decides what looks edible.
const REACH: i32 = 8;

/// One drawable thing in the world: its geometry, where it sits, and how
/// brightly it reads.
type Placed = (BodyMesh, [i32; 3], f32);

/// Camera orbit state. Presentation only; never reaches the core.
struct View {
    yaw: f32,
    pitch: f32,
    zoom: f32,
}

impl Default for View {
    fn default() -> Self {
        Self { yaw: std::f32::consts::FRAC_PI_4, pitch: 0.6154797, zoom: 1.0 }
    }
}

pub struct Host {
    config: HostConfig,
    runtime: Runtime,
    volumes: VolumeMap,
    view: View,
    window: Option<Arc<Window>>,
    gpu: Option<Gpu>,
    last: Option<Instant>,
    frames: u32,
    steps: u64,
}

struct Gpu {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    renderer: Renderer,
}

impl Host {
    pub fn new(config: HostConfig) -> Self {
        let runtime = Runtime::new(config.seed, config.organisms, config.ticks_per_second);
        Self {
            config,
            runtime,
            volumes: fixture::volumes(),
            view: View::default(),
            window: None,
            gpu: None,
            last: None,
            frames: 0,
            steps: 0,
        }
    }

    pub fn run(config: HostConfig) -> Result<(), winit::error::EventLoopError> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);
        let mut host = Self::new(config);
        event_loop.run_app(&mut host)
    }

    /// Frames the world around the critter at a fixed scale, so moving reads
    /// as travelling rather than as the camera zooming.
    fn camera(&self) -> Camera {
        let aspect = self.config.width as f32 / self.config.height.max(1) as f32;
        let mut camera = Camera::following(
            self.runtime.world().position,
            WORLD_EXTENT * self.view.zoom,
            aspect,
        );
        camera.yaw = self.view.yaw;
        camera.pitch = self.view.pitch;
        camera
    }

    /// Builds the drawable scene: the critter where it stands, plus every
    /// organism where it lies. Organisms out of reach are dimmed, so what can be
    /// eaten reads without a UI element.
    fn scene(&self) -> Option<(BodyMesh, Vec<Placed>)> {
        let world = self.runtime.world();
        let body = mesh_body(&world.body, &self.volumes).ok()?;

        let mut loose = Vec::with_capacity(world.organisms.len());
        for organism in &world.organisms {
            let Some(volume) = self.volumes.volume(organism.volume) else {
                continue;
            };
            let in_reach = (0..3).all(|a| {
                (organism.position[a] - world.position[a]).abs() <= REACH
            });
            loose.push((
                BodyMesh::single(organism.volume, volume),
                organism.position,
                if in_reach { 1.0 } else { 0.45 },
            ));
        }
        Some((body, loose))
    }

    /// Turns a key into an intent. The host does not decide whether the intent
    /// is legal; the core does, and reports a rejection.
    fn intent_for(&self, key: &Key) -> Option<Intent> {
        let step = 2;
        match key {
            Key::Character(c) => match c.as_str() {
                "w" | "W" => Some(Intent::Move { delta: [0, 0, -step] }),
                "s" | "S" => Some(Intent::Move { delta: [0, 0, step] }),
                "a" | "A" => Some(Intent::Move { delta: [-step, 0, 0] }),
                "d" | "D" => Some(Intent::Move { delta: [step, 0, 0] }),
                "e" | "E" => {
                    let world = self.runtime.world();
                    fixture::reachable(world).map(|m| fixture::metabolize(world, m, &self.volumes))
                }
                "q" | "Q" => Some(Intent::Deposit { mass_mg: 60 }),
                _ => None,
            },
            Key::Named(NamedKey::Space) => {
                let world = self.runtime.world();
                fixture::reachable(world).map(|m| fixture::metabolize(world, m, &self.volumes))
            }
            _ => None,
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        if let Some(gpu) = &mut self.gpu {
            gpu.config.width = self.config.width;
            gpu.config.height = self.config.height;
            gpu.surface.configure(gpu.renderer.device(), &gpu.config);
            gpu.renderer.resize(self.config.width, self.config.height);
        }
    }

    fn frame(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let elapsed_us = self
            .last
            .map(|then| now.duration_since(then).as_micros() as u64)
            .unwrap_or(0);
        self.last = Some(now);

        // Auto-eat lets a capture run grow a body with nobody at the keyboard.
        if let Some(every) = self.config.auto_eat_every
            && self.steps / every > (self.steps.saturating_sub(1)) / every
            && self.runtime.queued_len() == 0
        {
            let world = self.runtime.world();
            if let Some(target) = fixture::reachable(world) {
                let intent = fixture::metabolize(world, target, &self.volumes);
                self.runtime.queue(intent);
            }
        }

        self.steps += self.runtime.advance(elapsed_us);

        let camera = self.camera();
        let Some((body, loose)) = self.scene() else { return };
        let critter_at = self.runtime.world().position;

        let Some(gpu) = &mut self.gpu else { return };
        // wgpu 29 returns an enum rather than a Result here: a suboptimal
        // texture is still drawable, and a lost or outdated surface wants
        // reconfiguring rather than an error.
        let surface_texture = match gpu.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
                gpu.surface.configure(gpu.renderer.device(), &gpu.config);
                return;
            }
            _ => return,
        };

        let view = surface_texture.texture.create_view(&Default::default());
        let mut encoder = gpu.renderer.device().create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("frame") },
        );
        let mut items = Vec::with_capacity(loose.len() + 1);
        items.push(SceneItem::new(&body, critter_at));
        for (mesh, at, tint) in &loose {
            items.push(SceneItem::tinted(mesh, *at, *tint));
        }
        gpu.renderer.draw_scene(&mut encoder, &view, &items, &camera);
        gpu.renderer.queue().submit(Some(encoder.finish()));
        surface_texture.present();

        self.frames += 1;
        if let Some(limit) = self.config.frames
            && self.frames >= limit
        {
            self.capture();
            event_loop.exit();
        }
    }

    /// Renders one offscreen frame and writes it, so a windowed run leaves
    /// evidence a person can look at later.
    fn capture(&self) {
        let Some(path) = &self.config.capture else { return };
        let Some(gpu) = &self.gpu else { return };
        let Some((body, loose)) = self.scene() else { return };
        let critter_at = self.runtime.world().position;

        // The offscreen path wants our own colour format, not the surface's.
        let shot = Renderer::with_device(
            gpu.renderer.device().clone(),
            gpu.renderer.queue().clone(),
            self.config.width,
            self.config.height,
        );
        let mut items = Vec::with_capacity(loose.len() + 1);
        items.push(SceneItem::new(&body, critter_at));
        for (mesh, at, tint) in &loose {
            items.push(SceneItem::tinted(mesh, *at, *tint));
        }
        let Ok(frame) = shot.render_scene(&items, &self.camera()) else {
            return;
        };

        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let Ok(file) = std::fs::File::create(path) else { return };
        let mut encoder = png::Encoder::new(file, frame.width, frame.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        if let Ok(mut writer) = encoder.write_header() {
            let _ = writer.write_image_data(&frame.pixels);
        }
        println!(
            "captured {} after {} frames, {} steps, {} body parts",
            path.display(),
            self.frames,
            self.steps,
            self.runtime.world().body.len()
        );
    }
}

impl ApplicationHandler for Host {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attributes = Window::default_attributes()
            .with_title("Mesocosm")
            .with_inner_size(winit::dpi::LogicalSize::new(
                self.config.width,
                self.config.height,
            ));
        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .expect("a window is available"),
        );

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = instance
            .create_surface(window.clone())
            .expect("a surface for this window");
        let adapter = pollster::block_on(instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            },
        ))
        .expect("an adapter that can present to this surface");
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor { label: Some("mesocosm host"), ..Default::default() },
        ))
        .expect("a device");

        let size = window.inner_size();
        self.config.width = size.width.max(1);
        self.config.height = size.height.max(1);

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: self.config.width,
            height: self.config.height,
            present_mode: caps.present_modes[0],
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // The renderer targets the surface's format, which is usually BGRA
        // rather than the offscreen RGBA the tests use.
        let renderer = Renderer::with_format(
            device,
            queue,
            self.config.width,
            self.config.height,
            format,
        );

        self.gpu = Some(Gpu { surface, config, renderer });
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => self.resize(size.width, size.height),

            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                match &event.logical_key {
                    Key::Named(NamedKey::Escape) => {
                        self.capture();
                        event_loop.exit();
                    }
                    Key::Named(NamedKey::ArrowLeft) => self.view.yaw -= 0.12,
                    Key::Named(NamedKey::ArrowRight) => self.view.yaw += 0.12,
                    Key::Named(NamedKey::ArrowUp) => {
                        self.view.pitch = (self.view.pitch + 0.08).min(1.5)
                    }
                    Key::Named(NamedKey::ArrowDown) => {
                        self.view.pitch = (self.view.pitch - 0.08).max(-1.5)
                    }
                    key => {
                        if let Some(intent) = self.intent_for(key) {
                            self.runtime.queue(intent);
                        }
                    }
                }
            }

            WindowEvent::RedrawRequested => {
                self.frame(event_loop);
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
