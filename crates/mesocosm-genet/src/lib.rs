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

pub mod app;
pub mod fixture;

pub use app::{Host, HostConfig};
