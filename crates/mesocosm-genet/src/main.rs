// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Runs Mesocosm in a window.
//!
//! ```text
//! cargo run -p mesocosm-genet
//! cargo run -p mesocosm-genet -- --record-demo
//! cargo run -p mesocosm-genet -- --replay <trace>
//! ```
//!
//! Controls: WASD moves, E or Space metabolizes what is in reach, Q deposits,
//! C digs, the arrow keys pan the section, Escape writes the receipts and
//! quits.

use std::path::PathBuf;

use mesocosm_genet::{Host, HostConfig, played};

fn main() {
    let mut config = HostConfig::default();
    let mut args = std::env::args().skip(1);
    let mut replay: Option<PathBuf> = None;
    let mut record_demo = false;
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
            "--record-demo" => record_demo = true,
            "--auto-eat" => config.auto_eat_every = args.next().and_then(|v| v.parse().ok()),
            // Presentation only, and unruled: see `HostConfig::slab_half_height`.
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
            "--help" | "-h" => {
                println!("{}", HELP);
                return;
            }
            other => eprintln!("ignoring unknown argument: {other}"),
        }
    }

    // The workspace's headed-verify home, unless a flag says otherwise.
    let trace_path = trace.unwrap_or_else(played::default_trace_path);
    config.capture = Some(capture.unwrap_or_else(played::default_capture_path));
    config.receipt = Some(receipt.unwrap_or_else(played::default_receipt_path));

    if record_demo {
        // Headless: the demo trace exists so the headed receipt below needs
        // nobody at the keyboard.
        let recorded = played::record_demo(config.seed, config.organisms, config.ticks_per_second);
        if let Err(error) = played::write_json(&trace_path, &recorded) {
            eprintln!("record-demo: {error}");
            std::process::exit(1);
        }
        println!(
            "recorded {} intents to {}, hash {:016x}",
            recorded.intents.len(),
            trace_path.display(),
            recorded.state_hash
        );
        return;
    }

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
  --record-demo   write a scripted trace headlessly and exit
  --auto-eat N    metabolize automatically every N steps
  --seed N        world seed
  --slab H        section slab half-height in voxels (presentation only)

controls: WASD move, E/Space eat, Q deposit, C dig, arrows pan, Esc quit";
