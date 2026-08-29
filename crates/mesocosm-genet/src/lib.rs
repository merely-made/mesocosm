// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's windowed host.
//!
//! Owns the window, the device, and the frame loop. Holds **no game state**:
//! input becomes intents, the shared runtime steps the world at a fixed rate,
//! and the world comes back out to be drawn. If a rule ever appears in this
//! crate, it is in the wrong crate.
//!
//! The loop is the standard one, and the reason it is safe is elsewhere:
//! `mesocosm-runtime` converts however much wall time actually elapsed into
//! whole fixed steps, so a stuttering window cannot change what happens.
//!
//! The main view is the lens brick tracer's side-on terrarium section over the
//! live world (PS1). Camera motion is presentation only and never enters the
//! trace, which is what lets a recorded session replay to the same hash.

//! Two chrome lanes ride the frame, both through [`chrome`]: the painted
//! minimap ([`hud`]) and the cambium vitals panel ([`vitals`], landed
//! 2026-08-29). Neither touches the world, so neither can reach the trace.

pub mod app;
pub mod chrome;
pub mod fixture;
pub mod hud;
pub mod played;
pub mod section;
pub mod vitals;

pub use app::{Host, HostConfig};
pub use played::{PlayedReceipt, PlayedTrace};
