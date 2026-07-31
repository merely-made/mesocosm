// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Runs Mesocosm in a window.
//!
//! ```text
//! cargo run -p mesocosm-genet
//! cargo run -p mesocosm-genet -- --frames 240 --capture shot.png --auto-eat 40
//! ```
//!
//! Controls: WASD moves, E or Space metabolizes what is in reach, Q deposits,
//! the arrow keys orbit the camera, Escape captures and quits.

use mesocosm_genet::{Host, HostConfig};

fn main() {
    let mut config = HostConfig::default();
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--frames" => config.frames = args.next().and_then(|v| v.parse().ok()),
            "--capture" => config.capture = args.next().map(Into::into),
            "--auto-eat" => config.auto_eat_every = args.next().and_then(|v| v.parse().ok()),
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

    if let Err(error) = Host::run(config) {
        eprintln!("host failed: {error}");
        std::process::exit(1);
    }
}

const HELP: &str = "\
mesocosm-genet: run Mesocosm in a window

  --frames N      run N frames and exit
  --capture PATH  write the final frame as a PNG
  --auto-eat N    metabolize automatically every N steps
  --seed N        world seed

controls: WASD move, E/Space eat, Q deposit, arrows orbit, Esc capture+quit";
