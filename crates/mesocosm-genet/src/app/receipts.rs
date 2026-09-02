// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What the host writes on the way out.
//!
//! A run that left nothing behind cannot be judged, and a hash that drifted
//! looks exactly like one that did not until somebody says so. Every exit —
//! the window closing, Escape, a frame limit, a replay reaching the end of its
//! trace — comes through [`Host::finish`].

use winit::event_loop::ActiveEventLoop;

use super::Host;
use crate::played::{self, PlayedReceipt, PlayedTrace};

impl Host {
    /// Writes what the run leaves behind, once, and stops the loop.
    pub(super) fn finish(&mut self, event_loop: &ActiveEventLoop) {
        if !self.finished {
            self.finished = true;
            self.write_capture();
            self.write_trace();
            self.write_receipt();
        }
        event_loop.exit();
    }

    /// The frame the player was last looking at, chrome included.
    fn write_capture(&self) {
        let (Some(path), Some(gpu)) = (&self.config.capture, &self.gpu) else {
            return;
        };
        let frame = (gpu.config.width, gpu.config.height);
        let shot = gpu.section.capture(|encoder, target, format| {
            if let Some(lanes) = &gpu.chrome {
                lanes
                    .hud
                    .capture_composite(&lanes.device, format, encoder, target, frame);
                lanes
                    .vitals
                    .capture_composite(&lanes.device, format, encoder, target, frame);
                if self.config.dev {
                    lanes
                        .dev
                        .capture_composite(&lanes.device, format, encoder, target, frame);
                }
                lanes
                    .checkpoint
                    .capture_composite(&lanes.device, format, encoder, target, frame);
                // Last, as on screen: a capture of a boundary is a capture of
                // the board.
                lanes
                    .board
                    .capture_composite(&lanes.device, format, encoder, target, frame);
            }
        });
        let Some((width, height, pixels)) = shot else {
            eprintln!("capture: the section could not be read back");
            return;
        };
        if let Err(error) = played::write_png(path, width, height, &pixels) {
            eprintln!("capture: {error}");
        }
    }

    /// A replay's trace is an input; only a played session writes one.
    fn write_trace(&self) {
        let (Some(path), None) = (&self.config.trace, &self.config.replay) else {
            return;
        };
        let recorded = PlayedTrace {
            seed: self.config.seed,
            organisms: self.config.organisms,
            steps: self.runtime.trace().len() as u64,
            state_hash: self.runtime.state_hash(),
            intents: self.runtime.trace().to_vec(),
        };
        if let Err(error) = played::write_json(path, &recorded) {
            eprintln!("trace: {error}");
        }
    }

    fn write_receipt(&mut self) {
        let receipt = self.receipt();
        println!(
            "{}{} {} steps over {} frames, hash {:016x}{}",
            // **The label goes first, where it cannot be read past** (DT3).
            // A run that ended an epoch, forced a birth, killed something or
            // placed matter is not an unaided playtest, and the line that
            // reports it should not be able to be skimmed as one.
            if receipt.dev_intents > 0 {
                format!("assisted ({} dev intents) ", receipt.dev_intents)
            } else {
                String::new()
            },
            receipt.mode,
            receipt.steps,
            receipt.frames,
            receipt.state_hash,
            match receipt.state_hash_matches {
                Some(true) => " (matches the recorded hash)".to_string(),
                Some(false) => format!(
                    " (MISMATCH: the trace recorded {:016x})",
                    receipt.expected_state_hash.unwrap_or_default()
                ),
                None => String::new(),
            }
        );
        // Loud, because the alternative this replaces was a body that was
        // simply not there.
        if receipt.body_capsules_dropped > 0 {
            println!(
                "body: {} of {} parts past the lens capsule budget, drawn truncated to its widest",
                receipt.body_capsules_dropped, receipt.body_parts
            );
        }
        // The only route to a nonzero exit: a replay that landed elsewhere.
        if receipt.state_hash_matches == Some(false) {
            self.code = 1;
        }
        let Some(path) = &self.config.receipt else {
            return;
        };
        if let Err(error) = played::write_json(path, &receipt) {
            eprintln!("receipt: {error}");
        }
    }

    fn receipt(&self) -> PlayedReceipt {
        let run = self.runtime.receipt();
        // Only a replay that reached the end of its trace has a hash to be held
        // to. A capture run cut short by `--frames` stopped somewhere in the
        // middle of the recording on purpose, and comparing its world against
        // the recording's *final* hash would report a determinism failure for
        // having taken a photograph.
        let expected = self
            .config
            .replay
            .as_ref()
            .filter(|trace| self.cursor >= trace.intents.len())
            .map(|trace| trace.state_hash);
        let world = self.runtime.world();
        PlayedReceipt {
            mode: if self.config.replay.is_some() {
                "replay"
            } else {
                "played"
            },
            seed: run.seed,
            organisms: run.organisms,
            steps: run.steps,
            state_hash: run.state_hash,
            expected_state_hash: expected,
            state_hash_matches: expected.map(|hash| hash == run.state_hash),
            adapter: self
                .adapter
                .as_ref()
                .map_or_else(|| "none".into(), |info| info.name.clone()),
            backend: self
                .adapter
                .as_ref()
                .map_or_else(|| "none".into(), |info| format!("{:?}", info.backend)),
            frames: self.frames,
            trace_len: self.runtime.trace().len(),
            ground_revision: world.ground().revision(),
            body_parts: world.body().map(|body| body.len()).unwrap_or(0),
            body_capsules_dropped: self.body_capsules_dropped,
            section_roster: self
                .gpu
                .as_ref()
                .map_or(0, |gpu| gpu.section.last_roster_members()),
            roster_capsules_dropped: self
                .gpu
                .as_ref()
                .map_or(0, |gpu| gpu.section.last_roster_capsules_dropped()),
            slab_half_height: self
                .gpu
                .as_ref()
                .map_or(self.config.slab_half_height, |gpu| {
                    gpu.section.half_height()
                }),
            trace: self
                .config
                .trace
                .as_ref()
                .map(|path| path.display().to_string()),
            capture: self
                .config
                .capture
                .as_ref()
                .map(|path| path.display().to_string()),
            dev: self.config.dev,
            // Off the driver, which counts what the world accepted rather than
            // what a key asked for — so a replay of an assisted trace reports
            // the same number, and a refused dev intent reports none. (DT3)
            dev_intents: self.runtime.dev_intents(),
        }
    }
}
