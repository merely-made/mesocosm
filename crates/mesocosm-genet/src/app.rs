// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The window, the surface, and the loop.

use std::path::{Path, PathBuf};
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
use crate::input;
use crate::played::PlayedTrace;
use crate::section::{self, Pan, Section, SectionFrame};

mod receipts;
mod setup;

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
    /// The default is ruled ([`section::SLAB_HALF_HEIGHT`], 28 since
    /// 2026-08-29); the knob stays so every framing remains reproducible from
    /// the tree. Presentation only: it never reaches an intent, so it cannot
    /// move a replay hash.
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
    /// Parts of the played body the lens's capsule budget could not carry on
    /// the last drawn frame. Nonzero means a truncated player, which the
    /// receipt says out loud — before DC3 it meant no player at all.
    body_capsules_dropped: u32,
    /// Which row of the trait board the cursor is on. (PE3b)
    ///
    /// Host state and nothing else: moving it sends no intent, so it cannot
    /// reach the trace or the hash. It resets whenever a review opens, because
    /// a cursor left pointing at a row from the last boundary would be pointing
    /// at a candidate this one may not offer.
    board_row: usize,
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

/// The four chrome lanes over one netrender instance and one blend pass: the
/// painted minimap, the cambium vitals panel, the individual checkpoint, and
/// the trait board. The last two draw only while the world is holding at a
/// question, and never both: a lineage checkpoint is the board's.
pub(crate) struct Lanes {
    device: Chrome,
    hud: crate::hud::Hud,
    vitals: crate::vitals::VitalsChrome,
    checkpoint: crate::succession::SuccessionChrome,
    board: crate::review::BoardChrome,
}

/// The shipped pack's root, derived from this crate's own location rather than
/// a hardcoded path. (PE3b)
///
/// `<repo>/crates/mesocosm-genet` up two, then down into the pack the game
/// ships. What the host reads out of it is the review's second proposal source
/// and nothing else: the ruleset a world runs is still the core's own.
fn pack_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .map(|repo| repo.join("packs").join("mesocosm"))
        .unwrap_or_else(|| PathBuf::from("packs/mesocosm"))
}

impl Host {
    pub fn new(config: HostConfig) -> Self {
        let runtime = Runtime::new(config.seed, config.organisms, config.ticks_per_second);
        // The pack's authored expressions, for the review. A pack that will not
        // load is a diagnostic and not a reason to refuse to run: the board
        // simply shows one proposal source per row, which is what it does
        // wherever no pack expression applies anyway.
        let runtime = match mesocosm_runtime::Authored::load(&pack_root()) {
            Ok(authored) => runtime.with_authored(authored),
            Err(why) => {
                eprintln!("pack: {}", why.words());
                runtime
            }
        };
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
            body_capsules_dropped: 0,
            board_row: 0,
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
            // A checkpoint may hold part of a batch, so count what actually
            // ran. The recording answered its own questions, so the answer is
            // already the next intent in the queue and the hold lifts on the
            // step it is reached.
            let ran = self.runtime.step(batch as u64);
            self.steps += ran;
            // A trace that ends still holding at a question ends there, and the
            // frame that ends it is the question on screen.
            return self.cursor >= replay.intents.len()
                && (ran == 0 || self.runtime.queued_len() == 0);
        }

        let now = Instant::now();
        let elapsed_us = self
            .last
            .map(|then| now.duration_since(then).as_micros() as u64)
            .unwrap_or(0);
        self.last = Some(now);

        // Auto-eat lets a capture run grow a body with nobody at the keyboard.
        // It stops at a checkpoint like a hand would: an unattended run reaches
        // the question and stays there, which is how a capture of one is taken.
        if let Some(every) = self.config.auto_eat_every
            && self.runtime.checkpoint().is_none()
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

    /// The trait board's own keys, while it is standing. (PE3b)
    ///
    /// `None` means the board did not take this key and it falls through to the
    /// checkpoint's answers. `Some(None)` means the board took it and produced
    /// no intent — moving the cursor is presentation, and putting a cursor
    /// move in the queue would be putting it in the trace.
    fn board_key(&mut self, key: &winit::keyboard::Key) -> Option<Option<Intent>> {
        let action = input::board_key(key)?;
        let review = self.runtime.review()?;
        match action {
            // `Review::commit` refuses the status quo and every untakeable row,
            // so the key cannot send a revision the world would only reject.
            input::BoardKey::Commit => Some(review.commit(self.board_row)),
            input::BoardKey::Next => {
                // Wrapping, and over every row including the untakeable ones:
                // a candidate you cannot take yet is a thing to read, and
                // skipping it would hide the reason it is there for.
                let rows = review.rows.len();
                self.board_row = if rows == 0 {
                    0
                } else {
                    (self.board_row + 1) % rows
                };
                Some(None)
            }
        }
    }

    fn frame(&mut self, event_loop: &ActiveEventLoop) {
        let replay_done = self.advance();

        // Presentation reads of the stepped world, taken before the device is
        // borrowed: the section follows the critter, the HUD backdrop wants
        // the meshed scene, and neither is world state.
        let at = self.runtime.world().position().unwrap_or([0, 0, 0]);
        let half = section::half_height_or_default(self.config.slab_half_height);
        let centre = section::centre_on(at, self.pan, half);
        let tint = self
            .runtime
            .world()
            .controlled()
            .map_or([0.42, 0.62, 0.46], |organism| look_of(organism).0);
        let (pose, dropped) = match section::pose_of(self.runtime.world(), tint) {
            Some((pose, dropped)) => (Some(pose), dropped),
            None => (None, 0),
        };
        self.body_capsules_dropped = dropped;
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
        // The bounded ecology windows, read off the driver that reduced them.
        // Presentation only: the panel shows what the reducer found and writes
        // nothing back, so the trace and the hash never learn it exists.
        let trend = self.runtime.trend();
        // The question the driver is holding at, if it is holding at one.
        // Cloned off the driver so the world can be borrowed below; presentation
        // only, like everything else on this side of the frame.
        let checkpoint = self.runtime.checkpoint().cloned();
        // The played line's turn, at a lineage checkpoint. The cursor is kept in
        // range here rather than in the key handler, so a review that came back
        // shorter than the last one cannot leave it pointing past the table.
        let review = self.runtime.review().cloned();
        let board_row = match &review {
            Some(review) if !review.rows.is_empty() => self.board_row.min(review.rows.len() - 1),
            _ => 0,
        };
        self.board_row = board_row;

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
            lanes
                .vitals
                .refresh(&lanes.device, world, &outcomes, steps, &trend);
            lanes
                .vitals
                .composite(&lanes.device, &mut encoder, &view, frame);
            // Last, and over everything: while either of these is up the world
            // is not running, and it should look that way. The board takes the
            // lineage checkpoint and the checkpoint panel takes the rest, so
            // they never overlap.
            lanes
                .board
                .refresh(&lanes.device, review.as_ref(), board_row);
            let held = checkpoint.as_ref().filter(|_| !lanes.board.standing());
            lanes.checkpoint.refresh(&lanes.device, held);
            lanes
                .checkpoint
                .composite(&lanes.device, &mut encoder, &view, frame);
            lanes
                .board
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
        self.boot(event_loop);
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
                    //
                    // `event.repeat` filters OS auto-repeat (see `input`'s
                    // module docs for the backlog it used to build): a held
                    // key contributes its initial keydown and nothing more
                    // while it stays down.
                    key if self.config.replay.is_none() && !event.repeat => {
                        // At a checkpoint the keyboard narrows to the answers:
                        // the world is stopped, so a move has nothing to move
                        // and would only go stale in the queue. At a lineage
                        // checkpoint the board's own two keys come first — one
                        // of which sends no intent at all.
                        let intent = match self.board_key(key) {
                            Some(taken) => taken,
                            None => match self.runtime.checkpoint() {
                                Some(checkpoint) => input::answer_for(checkpoint, key),
                                None => input::intent_for(self.runtime.world(), &self.volumes, key),
                            },
                        };
                        if let Some(intent) = intent {
                            let urgency = input::urgency_of(&intent);
                            if input::admits(urgency, self.runtime.queued_len()) {
                                self.runtime.queue(intent);
                            }
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
