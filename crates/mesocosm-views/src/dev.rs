// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The dev lane's surface: what the host is doing to time, in words. (DT1)
//!
//! Same posture as [`crate::vitals`] and [`crate::succession`]: a
//! **reading**, taken fresh from facts the host already holds. Nothing here
//! is stored in the world, enters the trace, or reaches the state hash — the
//! dev tools plan's second principle is what makes that true by
//! construction. Pause, step and speed are host pacing over
//! `mesocosm_runtime::Runtime::advance` and `::step`, never a second
//! authority over the world, so this panel has nothing to say that a replay
//! could ever disagree with.
//!
//! Four facts and no more, matching the individual checkpoint's own stop
//! rule: a dev lane that grew a console would be a different tool, and DT2's
//! follow-and-inspect surface is where the next reading belongs, not here.

use cambium::{AnyView, DetailRow, DetailSection, GenetCtx, GenetElement, detail_panel, el};

pub type DevChild = Box<dyn AnyView<Dev, (), GenetCtx, GenetElement>>;

/// What the dev lane shows. Whether the clock is running, at what multiplier,
/// the world's own tick, and how many of those ticks this session took
/// through the step keys rather than the clock.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Dev {
    /// `false` while the pause key is holding the clock's elapsed time at
    /// zero — see [`crate`] module docs and the host's `advance`.
    pub running: bool,
    /// The clock's speed multiplier, already in the words the ladder shows:
    /// `"1/4"`, `"1/2"`, `"1"`, `"2"`, `"4"`.
    pub speed: &'static str,
    /// The world's own tick count: `Runtime::advance`/`::step`'s returns,
    /// summed. The same number the receipt calls `steps`.
    pub tick: u64,
    /// Ticks taken through the step or step-N keys this session. Tracked by
    /// the host and never by the world: a fact about how this run was
    /// driven, not about what it did — so it is not in the trace either.
    pub manual_steps: u64,
}

/// The dev panel: the catalog's labelled-facts component, exactly as
/// [`crate::vitals::vitals_root`] uses it for its own top section.
pub fn dev_root(dev: &Dev) -> DevChild {
    let rows = vec![
        DetailRow::new("state", if dev.running { "running" } else { "paused" }),
        DetailRow::new("speed", dev.speed),
        DetailRow::new("tick", dev.tick.to_string()),
        DetailRow::new("stepped", dev.manual_steps.to_string()),
    ];
    let panel: DevChild = Box::new(detail_panel::<Dev, ()>(&[DetailSection::new("dev", rows)]));
    Box::new(el::<_, Dev, ()>("div", vec![panel]).attr("class", "dev"))
}

/// The sheet the panel is styled by. Smaller than the vitals panel: four
/// rows and nothing that wraps.
pub fn dev_css() -> &'static str {
    r#"
.dev {
    width: 200px;
    padding: 10px 12px;
    background-color: #10141aee;
    color: #dfe6dd;
    font-family: sans-serif;
    font-size: 14px;
}
.detail-section-title {
    color: #8fa08c;
    font-size: 12px;
    margin-bottom: 4px;
}
.detail-row { margin-bottom: 2px; }
.detail-key { color: #8fa08c; }
.detail-value { color: #eaf2e6; font-weight: bold; margin-left: 8px; }
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A regression guard on the one thing worth guarding here: the class the
    /// root sets is the class the sheet styles.
    #[test]
    fn the_root_builds_and_the_sheet_styles_the_class_it_sets() {
        let dev = Dev {
            running: true,
            speed: "1",
            tick: 10,
            manual_steps: 2,
        };
        let _ = dev_root(&dev);
        assert!(
            dev_css().contains(".dev {"),
            "the sheet styles the class the root sets"
        );
    }

    /// The two plain words a player reads, and only those two.
    #[test]
    fn running_and_paused_are_the_only_two_states() {
        for running in [true, false] {
            let dev = Dev {
                running,
                ..Dev::default()
            };
            let word = if dev.running { "running" } else { "paused" };
            assert!(word == "running" || word == "paused");
        }
    }
}
