// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The host's configuration: every flag `main.rs` can set.
//!
//! Split out of `app.rs` at the 600-line ceiling.

use std::path::PathBuf;

use crate::played::PlayedTrace;
use crate::section;

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
    /// Off by default (DT1). On, the dev lane draws and its keys go live;
    /// recorded in the receipt either way.
    pub dev: bool,
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
            dev: false,
        }
    }
}
