// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a played session leaves behind, and what drives one without a player.
//!
//! A run is a seed, an organism count, and an ordered trace; the hash is what
//! two runs compare. The trace file carries all four so `--replay` can assert
//! rather than merely re-run, and so the assertion needs no second file.
//!
//! The recorded demo answers its own checkpoints, because a checkpoint holds the
//! world and a recording that ignored one would simply stop advancing. Those
//! answers are ordinary intents, so they sit in the trace beside the moves and
//! the meals and replay with them.

use std::path::{Path, PathBuf};

use mesocosm_core::{Intent, Placement, World};
use mesocosm_mesh::VolumeMap;
use mesocosm_runtime::{Checkpoint, Occasion, Runtime};
use serde::{Deserialize, Serialize};

/// Steps the recorded demo runs for.
///
/// **Long enough for the loop the game is named around** (PE1). It was 120 —
/// enough to grow a body, dig, deposit and spend a stretch on instincts, and
/// far short of a critter that lives long enough to breed. A demo that never
/// reaches a birth or a death cannot receipt the checkpoint, so the recording
/// now runs to the far side of both: three births under one hand, the parent's
/// own death at the end of its natural life, and the line continued through a
/// descendant. It still stops while somebody is alive, because a receipt of a
/// corpse shows neither the roster nor the vitals.
pub const DEMO_STEPS: u64 = 3_100;

/// The world the demo is recorded in.
///
/// **A fixture picks the world that shows the thing.** The interactive default
/// seed is untouched; this one is chosen because its played critter survives
/// its own lifespan and leaves exactly one living descendant behind, so a
/// recorded run reaches both halves of the individual checkpoint instead of
/// being eaten in the first twenty seconds (which is what most seeds do — a
/// real finding, and the terrarium dynamics plan's, not this one's). A replay
/// reads the seed out of the trace, so nothing downstream needs to know it.
pub const DEMO_SEED: u64 = 7;

/// Move deltas the demo walks, in the order WASD produces them.
const CARDINALS: [[i32; 3]; 4] = [[0, 0, -2], [0, 0, 2], [-2, 0, 0], [2, 0, 0]];

/// Steps the demo spends with its hands off the keys.
///
/// Longer than `INSTINCT_IDLE_TICKS`, deliberately: the recorded run has to
/// cross the threshold and keep going, so the receipt covers both halves of
/// TD4 — the hand holding a body, and the ecology taking it back. Four seconds
/// of nothing at the canonical tempo.
const HANDS_OFF: std::ops::Range<u64> = 300..340;

/// Steps the demo deliberately goes without eating. (PE2)
///
/// **The non-food discovery, scripted.** Every other route into the record
/// happens on its own — the demo eats constantly, and each meal is an
/// observation that unlocks nothing — but coming through a *stress* needs the
/// script to stop reaching for food and stay stopped: the condition is
/// `discovery::HUNGER_TICKS` consecutive ticks under the starved line with a
/// hand still on the body, and one meal in the middle of it resets the run.
///
/// It runs long, and after the ten meals the demo grows on, because a critter
/// that has never eaten is not enduring anything — it is simply small. The
/// window closes well before the first birth, so nothing downstream in the
/// recording depends on it.
const ENDURING: std::ops::Range<u64> = 120..340;

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
    /// Parts of the played body the lens's capsule budget could not carry on
    /// the last drawn frame.
    ///
    /// **This number exists because a body used to vanish instead.** The host
    /// swallowed the lens's refusal with `.ok()`, so a critter that grew past
    /// the cap simply stopped being drawn while alive. Since DC3 the widest
    /// capsules are kept, the body is drawn truncated, and the loss is
    /// reported here rather than shown as an absence.
    pub body_capsules_dropped: u32,
    /// Bodies the last traced frame drew beside the played one: every alive
    /// organism the slab window held, capped by the lens's roster limit.
    pub section_roster: u32,
    /// Capsules the drawn roster members carried past their own smaller
    /// budget, widest kept.
    pub roster_capsules_dropped: u32,
    /// How much world the section framed, in voxels of slab half-height. A
    /// capture that does not say what it framed cannot be compared with the
    /// next one. (S1; the number was ruled at 28 on 2026-08-29.)
    pub slab_half_height: f32,
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
pub fn record_demo(seed: u64, organisms: u32, ticks_per_second: u32, steps: u64) -> PlayedTrace {
    let volumes = crate::fixture::volumes();
    let mut runtime = Runtime::new(seed, organisms, ticks_per_second);
    let mut meals = 0u32;
    for step in 0..steps {
        // A checkpoint holds the world, so a recording that ignored one would
        // simply stop advancing. Answering it is the demo's whole point: the
        // choice is an ordinary intent, so it lands in the trace beside the
        // moves and the meals and replays with them.
        let intent = match runtime.checkpoint() {
            Some(checkpoint) => answer(checkpoint),
            None => demo_intent(runtime.world(), &volumes, step, &mut meals),
        };
        runtime.queue(intent);
        // One intent per step, off the clock: a recording has no frame rate.
        runtime.step(1);
    }
    PlayedTrace {
        seed,
        organisms,
        steps,
        state_hash: runtime.state_hash(),
        intents: runtime.trace().to_vec(),
    }
}

/// How the recorded demo answers a checkpoint: **continue at a birth, succeed
/// at a death.**
///
/// Continuing is the world default and it is what a birth checkpoint is for —
/// the offspring is alive in the enclosure either way, and the body the run
/// spent nine hundred ticks growing is not thrown away by a script. A death is
/// the other half of the same choice, and taking a descendant there is the
/// whole claim PE1 makes: the line goes on, and the session does not end.
fn answer(checkpoint: &Checkpoint) -> Intent {
    match checkpoint.occasion {
        Occasion::Birth(_) => checkpoint.default_answer(),
        Occasion::Loss(_) => match checkpoint.heir() {
            Some(organism) => Intent::TakeControl { organism },
            // Nothing survived to carry it. Declining is still an answer, and
            // it is the honest one.
            None => checkpoint.default_answer(),
        },
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
    // **Going hungry on purpose** (PE2). Simply not eating was not enough: by
    // this point the demo's critter has grown a canopy and earns about what it
    // spends, so its budget sits flat and it never crosses the starved line at
    // all. So it spends the reserve where a player can — into the ground under
    // it — until the budget sits about half a horizon short, and then holds
    // there while its own income keeps it alive. No new verb, and no scripted
    // world change: `Deposit` is a key the host already has.
    if ENDURING.contains(&step)
        && let Some(me) = world.controlled()
    {
        let target = me.upkeep_mg() * mesocosm_core::STARVED_UPKEEP_TICKS / 2;
        if me.energy_mg > target {
            return Intent::Deposit {
                mass_mg: me.energy_mg - target,
            };
        }
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
    if !ENDURING.contains(&step)
        && (*meals < 10 || world.is_starved())
        && let Some(target) = crate::fixture::reachable(world)
    {
        *meals += 1;
        return crate::fixture::metabolize(world, target, volumes, Placement::Planned);
    }
    if let Some(delta) = crate::fixture::toward_prey(world) {
        return Intent::Move { delta };
    }
    // **A march, not a shuffle.** This used to turn every three steps, which
    // walks a critter in a small circle: it re-crossed ground it had already
    // eaten and starved inside a couple of hundred ticks in every seed
    // measured. Holding a heading covers new enclosure, which is what finding
    // the next meal actually takes.
    Intent::Move {
        delta: CARDINALS[(step / 200) as usize % 4],
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use super::*;
    use mesocosm_core::{INSTINCT_IDLE_TICKS, OrganismId, SpeciesId};
    use mesocosm_runtime::{Birth, Loss};

    /// Steps and founders the assertions below record with.
    ///
    /// **Not the shipped demo's.** Recording 3,100 steps of the shipping cohort
    /// takes minutes unoptimized, and a workspace test run should not. These
    /// tests are about the *script* — that it walks, eats, digs, deposits, puts
    /// its hands down, and replays — and a few hundred ticks says all of that.
    /// The claim the shipped length exists for, that the run reaches a birth
    /// and a death, is receipted by the recording itself and by
    /// `mesocosm-runtime`'s checkpoint tests.
    const TEST_STEPS: u64 = 400;
    const TEST_FOUNDERS: u32 = 60;

    /// One recording, shared. Each of these tests used to pay for its own.
    static TRACE: LazyLock<PlayedTrace> =
        LazyLock::new(|| record_demo(DEMO_SEED, TEST_FOUNDERS, 10, TEST_STEPS));

    #[test]
    fn the_demo_trace_exercises_every_verb_the_slice_claims() {
        let trace = &*TRACE;
        assert_eq!(trace.intents.len(), TEST_STEPS as usize);

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
        let trace = &*TRACE;
        let (world, _) = Runtime::replay(trace.seed, trace.organisms, &trace.intents);
        assert_eq!(mesocosm_core::state_hash(&world), trace.state_hash);
    }

    /// TD4's other half, receipted: the recorded run must actually let go, or
    /// the fixture proves nothing about instincts under idleness.
    #[test]
    fn the_demo_trace_puts_its_hands_down_long_enough_to_lose_the_body() {
        let trace = &*TRACE;
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

    /// The script's whole answer to a checkpoint, in one place: continue at a
    /// birth, succeed at a death, decline only when there is nothing left to
    /// continue through. Pure, so it is asserted directly rather than fished
    /// out of a long recording.
    #[test]
    fn the_demo_continues_at_a_birth_and_succeeds_at_a_death() {
        let heirless = Checkpoint {
            tick: 10,
            occasion: Occasion::Loss(Loss {
                organism: OrganismId(4),
                lineage: SpeciesId(1),
            }),
            heirs: Vec::new(),
        };
        assert_eq!(answer(&heirless), Intent::Resume);

        let carried = Checkpoint {
            heirs: vec![OrganismId(9), OrganismId(12)],
            ..heirless.clone()
        };
        assert_eq!(
            answer(&carried),
            Intent::TakeControl {
                organism: OrganismId(9)
            },
            "the eldest descendant, and only it"
        );

        let birth = Checkpoint {
            tick: 20,
            occasion: Occasion::Birth(Birth {
                parent: OrganismId(4),
                offspring: OrganismId(9),
                lineage: SpeciesId(1),
                substance_mg: 500,
                reserve_mg: 500,
            }),
            heirs: vec![OrganismId(9)],
        };
        assert_eq!(
            answer(&birth),
            Intent::Resume,
            "a birth keeps the body the run has been growing"
        );
    }

    /// PE2's two claims, in the recorded loop rather than only in a fixture.
    ///
    /// **A non-food discovery**, reached by the script putting the food down
    /// long enough to come through the starvation horizon; and **a meal that
    /// refuses an incompatible candidate** — every one of the demo's meals is
    /// an observation, and the record says the endurance condition could not be
    /// reached by any of them because it never declared that lane.
    #[test]
    fn the_demo_reaches_a_non_food_discovery_and_a_meal_that_unlocks_nothing() {
        let trace = &*TRACE;
        let (world, _) = Runtime::replay(trace.seed, trace.organisms, &trace.intents);

        let discoveries = world.discoveries();
        assert_eq!(
            discoveries.len(),
            1,
            "the recorded run comes through exactly one condition: {discoveries:?}"
        );
        let discovery = discoveries[0];
        assert_eq!(
            discovery.route,
            mesocosm_core::Input::Endurance,
            "and it is not a meal that taught it"
        );
        assert!(matches!(
            discovery.evidence,
            mesocosm_core::Evidence::Endured { .. }
        ));
        assert!(
            world.last_observation().is_some(),
            "and the run's last evidence is on the record either way"
        );
    }

    /// The other half, asserted where the routing happens rather than only at
    /// the end of a run: a meal's evidence cannot reach the endurance
    /// condition, and the observation says so in those words.
    #[test]
    fn a_recorded_meal_is_observed_and_unlocks_nothing() {
        let mut runtime = Runtime::new(DEMO_SEED, TEST_FOUNDERS, 10);
        let volumes = crate::fixture::volumes();
        let mut meals = 0u32;
        let mut seen = None;
        for step in 0..TEST_STEPS {
            let intent = match runtime.checkpoint() {
                Some(checkpoint) => answer(checkpoint),
                None => demo_intent(runtime.world(), &volumes, step, &mut meals),
            };
            let ate = matches!(intent, Intent::Metabolize { .. });
            runtime.queue(intent);
            runtime.step(1);
            if ate && let Some(observation) = runtime.world().last_observation() {
                seen = Some(observation.clone());
                break;
            }
        }
        let observation = seen.expect("the script eats early and often");
        assert_eq!(observation.route, mesocosm_core::Input::Meal);
        assert!(
            observation
                .missed
                .iter()
                .any(|(_, miss)| matches!(miss, mesocosm_core::Miss::UndeclaredInput)),
            "a meal cannot be offered to a condition that never asked about \
             meals: {observation:?}"
        );
    }

    /// A carve that removed nothing would leave the section with no dirty
    /// bricks to drain, and the whole refresh path untested by the receipt.
    #[test]
    fn the_demo_trace_changes_the_ground() {
        let trace = &*TRACE;
        let (world, _) = Runtime::replay(trace.seed, trace.organisms, &trace.intents);
        assert!(world.ground().revision() > 0, "the digging removed voxels");
    }
}
