// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Runs Mesocosm in a window.
//!
//! ```text
//! cargo run -p mesocosm-genet
//! cargo run -p mesocosm-genet -- --replay <trace>
//! cargo run -p mesocosm-genet -- --scenario <scenario>
//! ```
//!
//! With nothing named on the command line the trace, receipt and capture go to
//! `<Code>/testing/mesocosm/scratch_played.*`. The golden `ps1_played.*`
//! fixture is written only when a flag names it, ruled 2026-09-02 — before
//! that the defaults were the fixture, and an unqualified run overwrote it.
//!
//! `--scenario` drives the run from a text scenario through genet-probe's
//! shared driver (DT4). It is where `--record-demo` and `--auto-eat` went: both
//! are now actions a scenario asks for by name. See [`mesocosm_genet::app::drive`]
//! for the verbs and [`mesocosm_genet::app::actions`] for the names.
//!
//! Controls: WASD moves, E or Space metabolizes what is in reach, Q deposits,
//! C digs, the arrow keys pan the section, Escape writes the receipts and
//! quits. At a checkpoint — a birth involving your critter, or its death — the
//! world stops and the keys narrow to Enter (carry on) and T (take the body on
//! offer). At the epoch boundary the trait board comes up instead: Tab moves
//! among the candidates, R commits the selected one, Enter goes back to the
//! terrarium.
//!
//! `--dev` adds a fifth chrome lane and twelve keys, live only while it is set.
//! DT1's five drive time: P pauses or unpauses the clock, `.` steps once and
//! `,` steps ten, both off the clock, and `[`/`]` move the clock's speed down
//! or up a rung. DT2's three drive the camera: N and B cycle the follow target
//! through the living roster in id order, and M snaps it back to the critter
//! under your hand. Following moves the camera and nothing else — control
//! stays where it is.
//!
//! DT3's four change the world, and are the only dev keys that do: X ends the
//! epoch now, F forces a birth from the followed critter, K kills it, and G
//! puts matter into the ground under it. Each queues an ordinary intent, so it
//! is in the trace, it replays, and the receipt counts it — a run that used one
//! prints as **assisted**. See [`mesocosm_genet::input`] for the exact mapping.

use std::path::PathBuf;

use mesocosm_genet::{Host, HostConfig, played};

fn main() {
    let mut config = HostConfig::default();
    let mut args = std::env::args().skip(1);
    let mut replay: Option<PathBuf> = None;
    let mut trace = None;
    let mut receipt = None;
    let mut capture = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--frames" => config.frames = args.next().and_then(|v| v.parse().ok()),
            "--capture" => capture = args.next().map(PathBuf::from),
            "--trace" => trace = args.next().map(PathBuf::from),
            "--receipt" => receipt = args.next().map(PathBuf::from),
            "--replay" => replay = args.next().map(PathBuf::from),
            // The scenario driver (DT4). It replaces `--record-demo` and
            // `--auto-eat`, which are now the `demo` and `hunt` actions.
            "--scenario" => {
                let Some(path) = args.next().map(PathBuf::from) else {
                    eprintln!("--scenario wants a path");
                    std::process::exit(1);
                };
                match std::fs::read_to_string(&path) {
                    Ok(text) => config.scenario = Some(text),
                    Err(error) => {
                        eprintln!("scenario: {}: {error}", path.display());
                        std::process::exit(1);
                    }
                }
            }
            // Presentation only; the default is ruled and this varies it.
            "--slab" => {
                if let Some(half) = args.next().and_then(|v| v.parse::<f32>().ok()) {
                    config.slab_half_height = half;
                }
            }
            "--seed" => {
                if let Some(seed) = args.next().and_then(|v| v.parse().ok()) {
                    config.seed = seed;
                }
            }
            // The dev lane and its keys (DT1). Off by default; recorded in
            // the receipt either way.
            "--dev" => config.dev = true,
            // Where the camera starts (DT2). Presentation only, and only a
            // starting point: the follow keys move it from here.
            "--follow" => config.follow = args.next().and_then(|v| v.parse().ok()),
            "--help" | "-h" => {
                println!("{}", HELP);
                return;
            }
            other => eprintln!("ignoring unknown argument: {other}"),
        }
    }

    // A scratch name under the workspace's headed-verify home, unless a flag
    // says otherwise. **Scratch, deliberately** (ruled 2026-09-02): these
    // defaulted to `ps1_played.*` until 2026-09-04, which is the golden fixture
    // `--replay` is checked against, so running this binary with no arguments
    // destroyed it. See `played::DEFAULT_STEM`.
    let trace_path = trace.unwrap_or_else(played::default_trace_path);
    config.capture = Some(capture.unwrap_or_else(played::default_capture_path));
    config.receipt = Some(receipt.unwrap_or_else(played::default_receipt_path));

    if let Some(path) = replay {
        match played::read_trace(&path) {
            Ok(recorded) => {
                // The replay is the recording's run, so it is the recording's
                // seed and roster too; a flag that disagreed would be a
                // different world wearing the same trace.
                config.seed = recorded.seed;
                config.organisms = recorded.organisms;
                config.replay = Some(recorded);
            }
            Err(error) => {
                eprintln!("replay: {error}");
                std::process::exit(1);
            }
        }
    } else {
        config.trace = Some(trace_path);
    }

    match Host::run(config) {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("host failed: {error}");
            std::process::exit(1);
        }
    }
}

const HELP: &str = "\
mesocosm-genet: run Mesocosm in a window

  --frames N      run N frames and exit
  --capture PATH  write the final frame as a PNG
  --trace PATH    write (or, with --replay, read) the intent trace
  --receipt PATH  write the run's receipt
  --replay PATH   drive the run from a recorded trace and assert its hash
  --scenario PATH drive the run from a text scenario and exit 1 if it fails
  --seed N        world seed
  --slab H        section slab half-height in voxels (presentation only, default 28)
  --dev           enable the dev lane and its keys (DT1, DT2, DT3); off by
                  default
  --follow ID     start the camera on this critter (DT2; needs --dev to be
                  worth anything, presentation only)

--capture, --trace and --receipt default to scratch names under the workspace's
headed-verify home: <Code>/testing/mesocosm/scratch_played.png, .trace.json and
.json. They are never the golden ps1_played.* fixture, which is written only
when one of those flags names it.

controls: WASD move, E/Space eat, Q deposit, C dig, arrows pan, Esc quit
at a checkpoint the world stops and the keys narrow:
  Enter  carry on unchanged
  T      take the body on offer (the newborn, or your eldest descendant)
at the epoch boundary the trait board comes up instead:
  Tab    move among the candidates
  R      commit the selected candidate to your line
  Enter  back to the terrarium
with --dev, eight host-only keys are live (none of them ever reaches the
trace, so a replay's hash cannot move because of one):
  P      pause or unpause the clock
  .      step once, off the clock
  ,      step ten, off the clock
  [      one rung slower on the speed ladder (1/4, 1/2, 1, 2, 4)
  ]      one rung faster
  N      follow the next living critter in id order, wrapping
  B      follow the previous one
  M      snap the camera back to the critter under your hand
following moves the camera and nothing else: control stays where it is
and four world-changing ones, which queue ordinary intents and so do enter
the trace, replay with it, and are counted on the receipt:
  X      end the epoch now (refused where the world's epoch rule says so)
  F      force a birth from the followed critter
  K      kill the followed critter
  G      put matter into the ground under the followed critter
a run that applied any of the four prints as assisted

a scenario is one verb a line (blank lines and # comments skipped):
  act NAME        one of the key letters above (w, e, x, ...), or one of five
                  host actions: follow ID, follow-nearest, follow-child,
                  hunt EVERY, demo STEPS
  settle N        pump N frames
  wait [CAP]      hold until the host reports quiet (a replay spent, a demo or
                  hunt finished, the queue empty); CAP is a hang-stop
  assert text S   S is on a chrome lane that is on screen
  assert snap F OP V   a run field: hash, expected, matches, mode, tick, steps,
                  frames, epoch, dev, dev-intents, assisted, queued, controlled,
                  follow, living, checkpoint, boundary, paused. OP is == >= <= ~
                  (assisted reads 'unassisted' or 'assisted (N dev intents)')
  assert event S  S is in what the world answered
  capture NAME    a PNG, at NAME if it is a path or beside the fixtures if not
  log WORDS       into the run's log
the process exits 1 if any assertion fails or the scenario runs out of frames";
