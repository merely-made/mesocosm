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
            if let Some(hud) = &gpu.hud {
                hud.capture_composite(&gpu.device, &gpu.queue, format, encoder, target, frame);
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
            "{} {} steps over {} frames, hash {:016x}{}",
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
        let expected = self.config.replay.as_ref().map(|trace| trace.state_hash);
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
        }
    }
}
