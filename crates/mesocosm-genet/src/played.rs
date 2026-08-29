// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a played session leaves behind, and what drives one without a player.
//!
//! A run is a seed, an organism count, and an ordered trace; the hash is what
//! two runs compare. The trace file carries all four so `--replay` can assert
//! rather than merely re-run, and so the assertion needs no second file.

use std::path::{Path, PathBuf};

use mesocosm_core::{Intent, Placement, World};
use mesocosm_mesh::VolumeMap;
use mesocosm_runtime::Runtime;
use serde::{Deserialize, Serialize};

/// Steps the recorded demo runs for. Long enough to grow a body, dig, deposit
/// and spend a stretch on its own instincts; short enough that the headed
/// replay finishes in seconds — **and short enough to end alive**.
///
/// Two hundred, at the TD2 constants and under TD4's hand, ended in a corpse
/// every time: a held critter cannot flee, and by tick 100 this one is the
/// fattest body in the enclosure with six predators feeding on it at once. A
/// receipt of a dead world shows neither the roster nor the vitals, so the run
/// stops while there is still somebody in it. The death is a real finding, not
/// a bug — it is recorded in the terrarium dynamics plan.
pub const DEMO_STEPS: u64 = 120;

/// Move deltas the demo walks, in the order WASD produces them.
const CARDINALS: [[i32; 3]; 4] = [[0, 0, -2], [0, 0, 2], [-2, 0, 0], [2, 0, 0]];

/// Steps the demo spends with its hands off the keys.
///
/// Longer than `INSTINCT_IDLE_TICKS`, deliberately: the recorded run has to
/// cross the threshold and keep going, so the receipt covers both halves of
/// TD4 — the hand holding a body, and the ecology taking it back. Four seconds
/// of nothing at the canonical tempo.
const HANDS_OFF: std::ops::Range<u64> = 60..100;

/// A recorded run, complete enough to reproduce and to judge.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayedTrace {
    pub seed: u64,
    pub organisms: u32,
    pub steps: u64,
    /// What the recording run ended at. A replay that lands elsewhere is a
    /// determinism failure, which is the whole point of writing it down.
    pub state_hash: u64,
    pub intents: Vec<Intent>,
}

/// What a run says about itself on the way out.
#[derive(Clone, Debug, Serialize)]
pub struct PlayedReceipt {
    /// `played` for a session at the keyboard, `replay` for a driven one.
    pub mode: &'static str,
    pub seed: u64,
    pub organisms: u32,
    pub steps: u64,
    pub state_hash: u64,
    /// The hash the driving trace recorded. `None` for a played session,
    /// which is the run that establishes it.
    pub expected_state_hash: Option<u64>,
    pub state_hash_matches: Option<bool>,
    pub adapter: String,
    pub backend: String,
    pub frames: u32,
    pub trace_len: usize,
    /// Advances with every carve and deposit that changed ground, so a
    /// receipt shows whether the section had anything to re-upload.
    pub ground_revision: u64,
    pub body_parts: usize,
    /// Bodies the last traced frame drew beside the played one: every alive
    /// organism the slab window held, capped by the lens's roster limit.
    pub section_roster: u32,
    pub trace: Option<String>,
    pub capture: Option<String>,
}

/// `Code/testing/<repo>/`, the workspace's headed-verify home. Derived from
/// this crate's own location rather than a hardcoded user path; every file
/// below it is overridable from the command line.
pub fn default_out_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .map(|code| code.join("testing").join("mesocosm"))
        .unwrap_or_else(|| PathBuf::from("testing/mesocosm"))
}

pub fn default_trace_path() -> PathBuf {
    default_out_dir().join("ps1_played.trace.json")
}

pub fn default_receipt_path() -> PathBuf {
    default_out_dir().join("ps1_played.json")
}

pub fn default_capture_path() -> PathBuf {
    default_out_dir().join("ps1_played.png")
}

fn ensure_parent(path: &Path) -> Result<(), String> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    std::fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    ensure_parent(path)?;
    let json = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    std::fs::write(path, json).map_err(|error| format!("{}: {error}", path.display()))
}

pub fn read_trace(path: &Path) -> Result<PlayedTrace, String> {
    let bytes = std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("{}: {error}", path.display()))
}

pub fn write_png(path: &Path, width: u32, height: u32, pixels: &[u8]) -> Result<(), String> {
    ensure_parent(path)?;
    let file =
        std::fs::File::create(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .and_then(|mut writer| writer.write_image_data(pixels))
        .map_err(|error| error.to_string())
}

/// Builds a trace headlessly, so the headed receipt runs with nobody at the
/// keyboard. Every verb the slice claims appears in it: moves in each
/// direction, metabolize, deposit, carve — and a stretch of doing nothing at
/// all, which since TD4 is a verb too.
pub fn record_demo(seed: u64, organisms: u32, ticks_per_second: u32) -> PlayedTrace {
    let volumes = crate::fixture::volumes();
    let mut runtime = Runtime::new(seed, organisms, ticks_per_second);
    let mut meals = 0u32;
    for step in 0..DEMO_STEPS {
        let intent = demo_intent(runtime.world(), &volumes, step, &mut meals);
        runtime.queue(intent);
        // One intent per step, off the clock: a recording has no frame rate.
        runtime.step(1);
    }
    PlayedTrace {
        seed,
        organisms,
        steps: DEMO_STEPS,
        state_hash: runtime.state_hash(),
        intents: runtime.trace().to_vec(),
    }
}

fn demo_intent(world: &World, volumes: &VolumeMap, step: u64, meals: &mut u32) -> Intent {
    // Open with a lap in every direction, so each movement key is in the
    // trace whether or not the hunt below happens to use it.
    if step < 16 {
        return Intent::Move {
            delta: CARDINALS[(step / 4) as usize % 4],
        };
    }
    // Walking away, and staying away. Checked before everything below so the
    // run of idles is unbroken — a single carve in the middle of it would
    // reset the world's idle count and put the hand straight back on.
    if HANDS_OFF.contains(&step) {
        return Intent::Idle;
    }
    // Digging at your own feet: reach is anatomy, and one voxel down is
    // inside the shortest reach a starting critter has.
    if step % 40 == 19
        && let Some(at) = world.position()
    {
        return Intent::Carve {
            at: [at[0], at[1] - 1, at[2]],
            radius: 1,
        };
    }
    if step % 40 == 39 {
        return Intent::Deposit { mass_mg: 60 };
    }
    // One verb, no route. Whether these meals burn or build is the demo's
    // *budget* talking, not its script — which is the point of recording it:
    // the trace exercises the decision without carrying it.
    //
    // Ten meals to grow on, then whenever the budget says so. Growing raises
    // the rent, which is what pushes a well-fed critter back over the starved
    // line; a script that stopped eating at its quota simply starved to death
    // partway through the run, and a receipt of a corpse shows nothing.
    if (*meals < 10 || world.is_starved())
        && let Some(target) = crate::fixture::reachable(world)
    {
        *meals += 1;
        return crate::fixture::metabolize(world, target, volumes, Placement::Planned);
    }
    if let Some(delta) = crate::fixture::toward_prey(world) {
        return Intent::Move { delta };
    }
    Intent::Move {
        delta: CARDINALS[(step / 3) as usize % 4],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mesocosm_core::INSTINCT_IDLE_TICKS;

    #[test]
    fn the_demo_trace_exercises_every_verb_the_slice_claims() {
        let trace = record_demo(0x00A7_7AC4, 60, 10);
        assert_eq!(trace.intents.len(), DEMO_STEPS as usize);

        let moves: std::collections::BTreeSet<_> = trace
            .intents
            .iter()
            .filter_map(|intent| match intent {
                Intent::Move { delta } => Some([delta[0].signum(), delta[2].signum()]),
                _ => None,
            })
            .collect();
        assert!(
            moves.len() >= 4,
            "the trace walks in more than one direction: {moves:?}"
        );
        for verb in ["Metabolize", "Deposit", "Carve", "Idle"] {
            assert!(
                trace
                    .intents
                    .iter()
                    .any(|intent| format!("{intent:?}").starts_with(verb)),
                "the trace contains a {verb}"
            );
        }
    }

    /// The claim `--replay` makes: the recorded hash is reachable from the
    /// seed and the trace alone, with no host in the loop.
    #[test]
    fn the_demo_trace_replays_to_its_recorded_hash() {
        let trace = record_demo(0x00A7_7AC4, 60, 10);
        let (world, _) = Runtime::replay(trace.seed, trace.organisms, &trace.intents);
        assert_eq!(mesocosm_core::state_hash(&world), trace.state_hash);
    }

    /// TD4's other half, receipted: the recorded run must actually let go, or
    /// the fixture proves nothing about instincts under idleness.
    #[test]
    fn the_demo_trace_puts_its_hands_down_long_enough_to_lose_the_body() {
        let trace = record_demo(0x00A7_7AC4, 60, 10);
        let longest = trace
            .intents
            .iter()
            .fold((0u32, 0u32), |(run, best), intent| {
                let run = if matches!(intent, Intent::Idle) {
                    run + 1
                } else {
                    0
                };
                (run, best.max(run))
            })
            .1;
        assert!(
            longest >= INSTINCT_IDLE_TICKS,
            "the demo idles {longest} in a row, short of the {INSTINCT_IDLE_TICKS} \
             it takes for the ecology to take the body back"
        );

        let (world, _) = Runtime::replay(trace.seed, trace.organisms, &trace.intents);
        assert!(
            world.controlled_id().is_none() || world.idle_run() < INSTINCT_IDLE_TICKS,
            "and the run ends back under the hand, so the capture is of a played world"
        );
    }

    /// A carve that removed nothing would leave the section with no dirty
    /// bricks to drain, and the whole refresh path untested by the receipt.
    #[test]
    fn the_demo_trace_changes_the_ground() {
        let trace = record_demo(0x00A7_7AC4, 60, 10);
        let (world, _) = Runtime::replay(trace.seed, trace.organisms, &trace.intents);
        assert!(world.ground().revision() > 0, "the digging removed voxels");
    }
}
