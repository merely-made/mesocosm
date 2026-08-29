// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The window, the surface, and the loop.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use mesocosm_core::{Intent, Kingdom, Organism, Placement, Signal, Stage};
use mesocosm_mesh::{BodyMesh, VolumeMap, VolumeSource, mesh_body};
use mesocosm_render::{SceneItem, deadened, kingdom_colour};
use mesocosm_runtime::Runtime;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

use crate::chrome::Chrome;
use crate::fixture;
use crate::played::PlayedTrace;
use crate::section::{self, Pan, Section, SectionFrame};

mod receipts;

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
    pub capture: Option<PathBuf>,
    /// Write the session's intent trace here before exiting. Skipped on a
    /// replay, whose trace is an input rather than a result.
    pub trace: Option<PathBuf>,
    /// Write the run's receipt here before exiting.
    pub receipt: Option<PathBuf>,
    /// Drive the run from this trace instead of the keyboard. The self-driving
    /// receipt: exactly one recorded intent per fixed step, then a hash
    /// assertion against what the trace recorded.
    pub replay: Option<PlayedTrace>,
    /// Metabolize automatically every N steps, so a capture run has something
    /// to show without keyboard input.
    pub auto_eat_every: Option<u64>,
    /// Half the height of the section's orthographic slab, in voxels — how much
    /// world the terrarium view frames.
    ///
    /// **A knob rather than a constant because the number is unruled.** S1
    /// widened the world to 129 voxels and proposed a value with the
    /// arithmetic; until Mark rules, the default stays what shipped and the
    /// proposal is one `--slab` away, so a capture of either is reproducible
    /// from the tree. Presentation only: it never reaches an intent, so it
    /// cannot move a replay hash. (2026-08-29 S1.)
    pub slab_half_height: f32,
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            seed: 0x00A7_7AC4,
            // The world's own area-scaled cohort, not a literal: S1 tied the
            // founding population to the enclosure's floor area so a wider
            // terrarium is bigger rather than emptier.
            organisms: mesocosm_core::world::FOUNDERS,
            // The canonical played tempo (TD2, ruled 2026-08-29). Sixty was
            // never chosen; it was the frame rate, and driving the ecology's
            // tick-tuned life history at it mapped a whole lifetime onto
            // seventeen seconds. Ten gives 100ms input granularity and puts a
            // starter's life at about five minutes. Headless labs and the
            // population instrument keep their own rates.
            ticks_per_second: 10,
            width: 960,
            height: 540,
            frames: None,
            capture: None,
            trace: None,
            receipt: None,
            replay: None,
            auto_eat_every: None,
            slab_half_height: section::SLAB_HALF_HEIGHT,
        }
    }
}

/// Recorded steps a replay frame drives. Exact, not throttled: the queue is
/// topped up with precisely this many intents and precisely this many steps
/// run, so a replay's trace is the recording's trace and nothing else.
const REPLAY_STEPS_PER_FRAME: usize = 4;

/// One drawable thing in the world: its geometry, where it sits, and how
/// brightly it reads.
type Placed = (BodyMesh, [i32; 3], f32, bool, [f32; 3], f32);

/// How an organism should read on screen.
///
/// Colour comes from its **guise**, never its kingdom, so a simulacrum is
/// drawn as the thing it pretends to be. Size tracks mass, so growth is
/// visible and a sapling does not look like a giant. The dead drain toward
/// grey.
fn look_of(organism: &Organism) -> ([f32; 3], f32) {
    let guise = match organism.guise {
        Kingdom::Producer => 0,
        Kingdom::Consumer => 1,
        Kingdom::Decomposer => 2,
    };
    let colour = kingdom_colour(guise);

    match organism.stage {
        // Growth you can see: a juvenile is visibly smaller than what it will
        // become, which is what makes waiting a decision.
        Stage::Juvenile => (
            [
                colour[0] * 0.85 + 0.1,
                colour[1] * 0.85 + 0.1,
                colour[2] * 0.85 + 0.1,
            ],
            0.55,
        ),
        Stage::Mature => (colour, 1.0),
        Stage::Carrion => (deadened(colour), 0.8),
        Stage::Spent => (deadened(colour), 0.3),
    }
}

pub struct Host {
    config: HostConfig,
    runtime: Runtime,
    volumes: VolumeMap,
    /// Where the section sits relative to the critter it follows. Presentation
    /// only; it never reaches an intent, so it cannot reach the trace.
    pan: Pan,
    window: Option<Arc<Window>>,
    gpu: Option<Gpu>,
    adapter: Option<wgpu::AdapterInfo>,
    last: Option<Instant>,
    frames: u32,
    steps: u64,
    /// How much of a replay's trace has been fed in.
    cursor: usize,
    finished: bool,
    /// Process exit code. Nonzero only when a replay landed on a different
    /// hash than the one its trace recorded.
    code: i32,
}

struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    /// The main view: the ruled side-on section over the live Ground.
    section: Section,
    /// Both chrome lanes and the device they share. `None` when netrender
    /// declined the device; the game runs chromeless rather than not at all.
    chrome: Option<Lanes>,
}

/// The two chrome lanes over one netrender instance and one blend pass: the
/// painted minimap, and the cambium vitals panel.
pub(crate) struct Lanes {
    device: Chrome,
    hud: crate::hud::Hud,
    vitals: crate::vitals::VitalsChrome,
}

impl Host {
    pub fn new(config: HostConfig) -> Self {
        let runtime = Runtime::new(config.seed, config.organisms, config.ticks_per_second);
        Self {
            config,
            runtime,
            volumes: fixture::volumes(),
            pan: Pan::default(),
            window: None,
            gpu: None,
            adapter: None,
            last: None,
            frames: 0,
            steps: 0,
            cursor: 0,
            finished: false,
            code: 0,
        }
    }

    /// Runs the host to completion. The `i32` is the process exit code.
    pub fn run(config: HostConfig) -> Result<i32, winit::error::EventLoopError> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);
        let mut host = Self::new(config);
        event_loop.run_app(&mut host)?;
        Ok(host.code)
    }

    /// Builds the scene the HUD's backdrop draws: the critter where it stands,
    /// plus every organism where it lies. Organisms out of reach are dimmed, so
    /// what can be eaten reads without a UI element.
    fn scene(&self) -> Option<(BodyMesh, Vec<Placed>)> {
        let world = self.runtime.world();
        let body = mesh_body(world.body()?, &self.volumes).ok()?;

        let mut loose = Vec::with_capacity(world.organisms.len());
        for organism in &world.organisms {
            let Some(volume) = self.volumes.volume(organism.volume()) else {
                continue;
            };
            // Ask the world rather than mirroring a constant. The host used to
            // keep its own copy of the reach rule, which was wrong the moment
            // reach became anatomy.
            let in_reach = world.in_reach(organism.position);
            let reach_tint = if in_reach { 1.0 } else { 0.45 };
            let (colour, scale) = look_of(organism);
            loose.push((
                BodyMesh::single(organism.volume(), volume),
                organism.position,
                reach_tint,
                organism.signal == Signal::Warning,
                colour,
                scale,
            ));
        }
        Some((body, loose))
    }

    /// The next meal in reach. `None` when nothing is close enough.
    ///
    /// One meal, one key. Where it goes is the body's answer, not a second
    /// keystroke (TD4): a starved critter burns what it eats and a provisioned
    /// one builds with it, and the budget that decides is on the panel in the
    /// corner.
    fn meal(&self) -> Option<Intent> {
        let world = self.runtime.world();
        fixture::reachable(world)
            .map(|m| fixture::metabolize(world, m, &self.volumes, Placement::Planned))
    }

    /// Turns a key into an intent. The host does not decide whether the intent
    /// is legal; the core does, and reports a rejection.
    fn intent_for(&self, key: &Key) -> Option<Intent> {
        let step = 2;
        match key {
            Key::Character(c) => match c.as_str() {
                "w" | "W" => Some(Intent::Move {
                    delta: [0, 0, -step],
                }),
                "s" | "S" => Some(Intent::Move {
                    delta: [0, 0, step],
                }),
                "a" | "A" => Some(Intent::Move {
                    delta: [-step, 0, 0],
                }),
                "d" | "D" => Some(Intent::Move {
                    delta: [step, 0, 0],
                }),
                // The one verb, one key. The second one (F, for burning) is
                // gone: Mark ruled the hotkey pair unworkable as an interface
                // and the destination diegetic, so there is nothing left for a
                // second key to say.
                "e" | "E" => self.meal(),
                "q" | "Q" => Some(Intent::Deposit { mass_mg: 60 }),
                // Digging at your own feet. Legality is embodiment plus
                // reach, and one voxel down is inside the shortest reach.
                "c" | "C" => self.runtime.world().position().map(|at| Intent::Carve {
                    at: [at[0], at[1] - 1, at[2]],
                    radius: 1,
                }),
                _ => None,
            },
            Key::Named(NamedKey::Space) => self.meal(),
            _ => None,
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        if let Some(gpu) = &mut self.gpu {
            gpu.config.width = self.config.width;
            gpu.config.height = self.config.height;
            gpu.surface.configure(&gpu.device, &gpu.config);
            gpu.section.resize(self.config.width, self.config.height);
        }
    }

    /// Steps the world. A replay feeds recorded intents and ignores the clock;
    /// a played session converts elapsed wall time into whole fixed steps.
    /// Returns true when a replay has reached the end of its trace.
    fn advance(&mut self) -> bool {
        if let Some(replay) = &self.config.replay {
            let end = (self.cursor + REPLAY_STEPS_PER_FRAME).min(replay.intents.len());
            let batch = end - self.cursor;
            for intent in &replay.intents[self.cursor..end] {
                self.runtime.queue(intent.clone());
            }
            self.cursor = end;
            self.runtime.step(batch as u64);
            self.steps += batch as u64;
            return self.cursor >= replay.intents.len();
        }

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
                // An unattended run eats what it can reach; whether that grows
                // a body or refills a budget is the body's to decide.
                let intent = fixture::metabolize(world, target, &self.volumes, Placement::Planned);
                self.runtime.queue(intent);
            } else if let Some(step) = fixture::toward_prey(world) {
                // **Hunt, do not wait.** Reach became anatomy in P2, so a
                // starting critter touches about three voxels. An unattended
                // run that stood still grew nothing in nine hundred frames.
                self.runtime.queue(Intent::Move { delta: step });
            }
        }

        self.steps += self.runtime.advance(elapsed_us);
        false
    }

    fn frame(&mut self, event_loop: &ActiveEventLoop) {
        let replay_done = self.advance();

        // Presentation reads of the stepped world, taken before the device is
        // borrowed: the section follows the critter, the HUD backdrop wants
        // the meshed scene, and neither is world state.
        let at = self.runtime.world().position().unwrap_or([0, 0, 0]);
        let centre = section::centre_on(at, self.pan);
        let tint = self
            .runtime
            .world()
            .controlled()
            .map_or([0.42, 0.62, 0.46], |organism| look_of(organism).0);
        let pose = section::pose_of(self.runtime.world(), tint);
        // An ant farm needs the ants: everything else alive in the slab the
        // camera already cuts, posed the same way and coloured by its own
        // guise. Nothing here reaches an intent.
        let roster = self
            .gpu
            .as_ref()
            .map(|gpu| gpu.section.slab_window(centre))
            .map(|window| {
                section::roster_of(self.runtime.world(), window, |organism| look_of(organism).0)
            })
            .unwrap_or_default();
        let scene = self.scene();
        let dirty = self.runtime.drain_ground_dirty();
        let steps = self.steps;
        // What the world said about the intents this frame fed it. Refusals
        // were polite inside `World::apply` and silent outside it until the
        // cambium lane landed; this is the whole of their route to a screen.
        let outcomes = self.runtime.last_outcomes().to_vec();

        let world = self.runtime.world();
        let Some(gpu) = &mut self.gpu else { return };
        // wgpu 29 returns an enum rather than a Result here: a suboptimal
        // texture is still drawable, and a lost or outdated surface wants
        // reconfiguring rather than an error.
        let surface_texture = match gpu.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
                gpu.surface.configure(&gpu.device, &gpu.config);
                return;
            }
            _ => return,
        };

        let view = surface_texture.texture.create_view(&Default::default());
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });
        if let Err(error) = gpu.section.draw(
            &mut encoder,
            &view,
            SectionFrame {
                ground: world.ground(),
                dirty: &dirty,
                centre,
                pose: pose.as_ref(),
                roster: &roster,
            },
        ) {
            eprintln!("section: {error}");
        }
        if let Some(lanes) = &mut gpu.chrome {
            let frame = (gpu.config.width, gpu.config.height);
            if let Some((body, loose)) = &scene {
                let mut items = Vec::with_capacity(loose.len() + 1);
                items.push(SceneItem::new(body, at));
                for (mesh, place, tint, warns, colour, scale) in loose {
                    items.push(SceneItem::creature(
                        mesh, *place, *tint, *warns, *colour, *scale,
                    ));
                }
                lanes.hud.render_backdrop(&items, steps);
            }
            lanes.hud.refresh(&lanes.device, world);
            lanes
                .hud
                .composite(&lanes.device, &mut encoder, &view, frame);
            // After the minimap, so a refusal reads over the section rather
            // than under it. Presentation only: nothing it reads is written
            // back, so the trace and the hash never learn it exists.
            lanes.vitals.refresh(&lanes.device, world, &outcomes, steps);
            lanes
                .vitals
                .composite(&lanes.device, &mut encoder, &view, frame);
        }
        gpu.queue.submit(Some(encoder.finish()));
        // wgpu 30 moved presentation from SurfaceTexture to Queue.
        gpu.queue.present(surface_texture);

        self.frames += 1;
        let hit_limit = self.config.frames.is_some_and(|limit| self.frames >= limit);
        if replay_done || hit_limit {
            self.finish(event_loop);
        }
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
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            apply_limit_buckets: false,
            compatible_surface: Some(&surface),
        }))
        .expect("an adapter that can present to this surface");
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("mesocosm host"),
            ..Default::default()
        }))
        .expect("a device");
        self.adapter = Some(adapter.get_info());

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
            // wgpu 30 made surface color space explicit; Auto keeps the
            // pre-30 platform-chosen behavior.
            color_space: wgpu::SurfaceColorSpace::Auto,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // The section binds the live Ground at genesis and refreshes from the
        // world's own dirty drain thereafter.
        let section = match Section::new(
            device.clone(),
            queue.clone(),
            self.config.width,
            self.config.height,
            format,
            self.runtime.world().ground(),
            self.config.slab_half_height,
        ) {
            Ok(section) => section,
            Err(error) => {
                eprintln!("section: {error}");
                self.code = 1;
                event_loop.exit();
                return;
            }
        };

        // The chrome shares the game's device rather than creating a second
        // one, which is the arrangement the workspace's wgpu pin exists for.
        // One netrender instance carries both lanes.
        let chrome = Chrome::new(
            netrender::WgpuHandles {
                instance,
                adapter,
                device: device.clone(),
                queue: queue.clone(),
            },
            format,
            crate::hud::SIDE,
        )
        .map(|device| Lanes {
            hud: crate::hud::Hud::new(&device, self.runtime.world()),
            vitals: crate::vitals::VitalsChrome::new(&device),
            device,
        });

        self.gpu = Some(Gpu {
            device,
            queue,
            surface,
            config,
            section,
            chrome,
        });
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => self.finish(event_loop),

            WindowEvent::Resized(size) => self.resize(size.width, size.height),

            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                match &event.logical_key {
                    Key::Named(NamedKey::Escape) => self.finish(event_loop),
                    // The section pans; it does not orbit. A ruled side-on
                    // view that could be rotated would stop being one.
                    Key::Named(NamedKey::ArrowLeft) => self.pan.x -= section::PAN_STEP,
                    Key::Named(NamedKey::ArrowRight) => self.pan.x += section::PAN_STEP,
                    Key::Named(NamedKey::ArrowUp) => self.pan.y += section::PAN_STEP,
                    Key::Named(NamedKey::ArrowDown) => self.pan.y -= section::PAN_STEP,
                    // A replay drives itself. Accepting a key would put an
                    // intent in the trace that the recording never had.
                    key if self.config.replay.is_none() => {
                        if let Some(intent) = self.intent_for(key) {
                            self.runtime.queue(intent);
                        }
                    }
                    _ => {}
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
