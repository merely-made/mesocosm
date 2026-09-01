// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The in-epoch vitals surface: what the played critter has, and what the
//! world just said back — a refusal, or what the body did with a meal.
//!
//! The cambium lane (ruled 2026-08-29), and Mesocosm's first consumer of it.
//! Words and numbers are expected here; the painted lane keeps its own
//! discipline and grows no lettering. The first real playtest ran with energy,
//! refusals and death all unsurfaced, which is what this exists against.
//!
//! Host-agnostic, like the paint leaves beside it: this crate says what the
//! panel *is*, and the host says where its raster lands.

use cambium::{AnyView, DetailRow, DetailSection, GenetCtx, GenetElement, detail_panel, el, text};
use mesocosm_core::{Ineligible, Outcome, Rejection, Trend, World};

/// A view in the vitals tree. Inert: nothing here takes a click, because
/// during an epoch you act on the world, not on a panel.
pub type VitalsChild = Box<dyn AnyView<Vitals, (), GenetCtx, GenetElement>>;

/// The energy bar's full width, in logical pixels. The fill is measured
/// against this rather than against a percentage so the bar reads the same
/// whatever box the panel ends up in.
const BAR_WIDTH: f32 = 168.0;

/// What the panel shows. A **reading**, taken fresh from the world each time:
/// nothing here is stored in the world, enters the trace, or reaches the state
/// hash. The host owns how long a notice stays up, because that is a clock,
/// not a fact.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Vitals {
    /// The played critter's budget. `None` once control is lost, which is the
    /// persistent death state: the panel says so and empties.
    pub energy_mg: Option<u64>,
    /// Where the budget sits against the most it has held this session,
    /// clamped to `0..=1`. Presentation only — the world has no capacity, so
    /// there is no other honest denominator.
    pub fullness: f32,
    /// The most recent thing the world said back, in plain words, while the
    /// host is still showing it: a refusal, or — since TD4 took the meal's
    /// destination out of the player's hands — what the body did with a meal.
    /// The state that decided is the number right above it.
    pub notice: Option<&'static str>,
    /// The first ecology reading: maturation against mortality, and the window
    /// it covers. **A fact with its window on it**, never a bare ratio.
    pub replacement: Option<String>,
    /// The one warning PE0 ships, when the support path has actually run short.
    /// It says what moved and over how long; it never presents a collapse
    /// percentage, and it never repeats the population instrument's test
    /// verdicts, which are not player language.
    pub warning: Option<String>,
}

impl Vitals {
    pub fn is_dead(&self) -> bool {
        self.energy_mg.is_none()
    }
}

/// Reads the vitals out of a world. `high_water` is the host's running
/// maximum, passed in because the bar's scale is a session fact rather than a
/// world one; `trend` is the driver's bounded windows, passed in for the same
/// reason — a world cannot say what happened, only what is.
pub fn vitals_of(
    world: &World,
    high_water: u64,
    notice: Option<&'static str>,
    trend: Option<&Trend>,
) -> Vitals {
    reading(
        world.controlled().map(|critter| critter.energy_mg),
        high_water,
        notice,
        trend,
    )
}

/// The reading itself, off the world. Separated so the disembodied case is
/// testable: nothing public drops control, and the world it happens in is a
/// thousand ticks away.
fn reading(
    energy_mg: Option<u64>,
    high_water: u64,
    notice: Option<&'static str>,
    trend: Option<&Trend>,
) -> Vitals {
    let fullness = match (energy_mg, high_water) {
        (Some(energy), top) if top > 0 => (energy as f32 / top as f32).clamp(0.0, 1.0),
        _ => 0.0,
    };
    Vitals {
        energy_mg,
        fullness,
        // A dead critter refuses nothing and eats nothing; the death state
        // stands alone.
        notice: notice.filter(|_| energy_mg.is_some()),
        // The enclosure keeps happening whether or not anyone is in it, so
        // these two survive losing a body.
        replacement: trend.map(replacement_words),
        warning: trend.and_then(warning_words),
    }
}

/// Maturation against mortality, and the window it covers.
///
/// Counts rather than a ratio, because a ratio hides how much evidence it is
/// made of: three deaths in two hundred ticks and three hundred are the same
/// number and not the same fact.
pub fn replacement_words(trend: &Trend) -> String {
    format!(
        "{} matured, {} died in {} ticks",
        trend.matured, trend.died, trend.replacement_ticks
    )
}

/// The warning, when the support path has run short long enough to say so.
///
/// **What moved, over what window.** Never a collapse percentage, and never one
/// of the population instrument's verdicts: those classify a test run, and
/// turning one into player language is a separate interaction ruling.
///
/// The two numbers sit side by side rather than one inside the other. Grazing is
/// one of several ways a stand loses matter, and it can exceed the net loss when
/// the survivors are still growing; saying "of which" would be a causal claim
/// the record does not make.
pub fn warning_words(trend: &Trend) -> Option<String> {
    trend.warns().then(|| {
        format!(
            "the stand has been shrinking for {} ticks: {} mg lost over the last {}; mouths took {} mg in the same window",
            trend.shortfall_ticks,
            trend.stand_change_mg.unsigned_abs(),
            trend.stand_ticks,
            trend.grazed_mg
        )
    })
}

/// The plain words for a rejection. Short because they are read in motion, and
/// stated as what happened rather than as an error code.
pub fn refusal_words(rejection: &Rejection) -> &'static str {
    match rejection {
        Rejection::InsufficientMass => "not enough energy",
        Rejection::OutOfReach(_) => "out of reach",
        Rejection::Disembodied => "no body",
        Rejection::Itself => "that is you",
        Rejection::NoRoom => "no room for it",
        Rejection::NoSuchOrganism(_) => "nothing there",
        Rejection::NoSuchParent(_) => "nowhere to attach",
        Rejection::Ineligible(Ineligible::NotAlive) => "that one is dead",
        Rejection::Ineligible(Ineligible::AboveTheFrontier { .. }) => "beyond you",
        Rejection::Ineligible(Ineligible::NoSuchOrganism) => "nothing there",
    }
}

/// The first thing worth saying in a batch of outcomes, in plain words.
///
/// First rather than last: a frame's worth of steps can refuse the same intent
/// several times, and the one that arrived is the one worth saying.
///
/// A landed meal counts. Since TD4 the body decides what a meal becomes, so
/// the player is reading a decision rather than confirming one — which makes
/// the difference between "burned" and "grew" the whole feedback for the verb.
/// Nothing new is built for it: the outcome is already here, and these are the
/// same three words the refusals are.
pub fn notice_in(outcomes: &[Outcome]) -> Option<&'static str> {
    outcomes.iter().find_map(|outcome| match outcome {
        Outcome::Rejected(rejection) => Some(refusal_words(rejection)),
        Outcome::Burned { .. } => Some("burned"),
        Outcome::Incorporated { .. } | Outcome::IncorporatedPair { .. } => Some("grew"),
        _ => None,
    })
}

/// The vitals panel.
pub fn vitals_root(vitals: &Vitals) -> VitalsChild {
    let mut children: Vec<VitalsChild> = Vec::new();

    // The catalog's labelled-facts component, which is exactly what a vital
    // sign is: an inert key and its value. Not a hand-rolled row.
    let mut rows = match vitals.energy_mg {
        Some(energy) => vec![DetailRow::new("energy", format!("{energy} mg"))],
        None => vec![DetailRow::new("state", "dead")],
    };
    if let Some(replacement) = &vitals.replacement {
        rows.push(DetailRow::new("replacement", replacement.clone()));
    }
    children.push(Box::new(detail_panel::<Vitals, ()>(&[DetailSection::new(
        "vitals", rows,
    )])));

    // The bar is a box, not a drawing: its fill is a width, and the engine
    // paints it. A dead critter has no bar at all.
    if !vitals.is_dead() {
        let fill = el::<_, Vitals, ()>("div", ())
            .attr("class", "vital-bar-fill")
            .attr(
                "style",
                format!("width: {:.0}px", vitals.fullness * BAR_WIDTH),
            );
        children.push(Box::new(
            el::<_, Vitals, ()>("div", Box::new(fill) as VitalsChild).attr("class", "vital-bar"),
        ));
    }

    if let Some(words) = vitals.notice {
        children.push(Box::new(
            el::<_, Vitals, ()>("div", text(words)).attr("class", "vital-notice"),
        ));
    }

    // Last, and only when it is true. A warning that is always on screen is a
    // decoration.
    if let Some(words) = &vitals.warning {
        children.push(Box::new(
            el::<_, Vitals, ()>("div", text(words.clone())).attr("class", "vital-warning"),
        ));
    }

    Box::new(el::<_, Vitals, ()>("div", children).attr("class", "vitals"))
}

/// The sheet the panel is styled by. Dark and translucent so the section
/// reads through it, and plain: this is a status panel, not a flourish.
pub fn vitals_css() -> &'static str {
    r#"
.vitals {
    /* The host's raster is 300 wide and this is a content box, so the twelve
       pixels of padding on each side come out of it. Setting the two equal
       clipped the last character off every line that reached the edge. */
    width: 276px;
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
.vital-bar {
    width: 168px;
    height: 8px;
    margin-top: 6px;
    background-color: #263026;
}
.vital-bar-fill {
    height: 8px;
    background-color: #7fc46a;
}
.vital-notice {
    margin-top: 8px;
    color: #e2a06a;
    font-size: 13px;
}
.vital-warning {
    margin-top: 8px;
    color: #d8776a;
    font-size: 12px;
}
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_world_with_nobody_in_it_reads_dead() {
        let vitals = reading(None, 1_000, Some("not enough energy"), None);
        assert!(vitals.is_dead());
        assert_eq!(vitals.fullness, 0.0);
        // The death state stands alone: a notice from the same batch is not
        // shown beside "dead", because a dead critter refused nothing.
        assert_eq!(vitals.notice, None);
    }

    #[test]
    fn the_bar_measures_against_the_session_high_water() {
        let world = World::new(0x00A7_7AC4, 8);
        let energy = world.energy_mg().expect("the world starts embodied");
        let full = vitals_of(&world, energy, None, None);
        assert_eq!(full.energy_mg, Some(energy));
        assert!((full.fullness - 1.0).abs() < f32::EPSILON);
        let half = vitals_of(&world, energy * 2, None, None);
        assert!((half.fullness - 0.5).abs() < 0.001);
    }

    /// The playtest's three silences, each with a word for it.
    #[test]
    fn the_refusals_the_playtest_hit_have_plain_words() {
        assert_eq!(
            notice_in(&[Outcome::Rejected(Rejection::InsufficientMass)]),
            Some("not enough energy")
        );
        assert_eq!(
            notice_in(&[Outcome::Rejected(Rejection::Disembodied)]),
            Some("no body")
        );
        assert_eq!(notice_in(&[Outcome::Moved, Outcome::Idled]), None);
    }

    /// The warning says what moved and over what window, or says nothing.
    #[test]
    fn a_warning_carries_its_evidence_and_only_arrives_when_it_is_true() {
        let quiet = Trend {
            replacement_ticks: 240,
            matured: 4,
            died: 2,
            stand_ticks: 60,
            stand_change_mg: 900,
            grazed_mg: 300,
            shortfall_ticks: 0,
        };
        assert_eq!(warning_words(&quiet), None);
        assert_eq!(replacement_words(&quiet), "4 matured, 2 died in 240 ticks");

        let short = Trend {
            stand_change_mg: -7_930,
            grazed_mg: 15_771,
            shortfall_ticks: mesocosm_core::WARN_AFTER_TICKS,
            ..quiet
        };
        let words = warning_words(&short).expect("a real shortfall says so");
        assert!(words.contains("7930 mg lost over the last 60"));
        assert!(words.contains("mouths took 15771 mg in the same window"));
        assert!(words.contains("ticks"), "and the window it moved over");
        assert!(
            !words.contains('%'),
            "never an unexplained percentage: {words}"
        );
        for verdict in ["breathes", "thins", "boils", "collapses"] {
            assert!(
                !words.contains(verdict),
                "the instrument's verdicts are not player language: {words}"
            );
        }
    }

    /// TD4's half of the feedback: the player no longer chooses what a meal
    /// becomes, so the panel has to say what it became.
    #[test]
    fn a_landed_meal_says_which_way_the_body_took_it() {
        assert_eq!(
            notice_in(&[Outcome::Burned {
                organism: mesocosm_core::OrganismId(3),
                energy_mg: 120,
            }]),
            Some("burned")
        );
        assert_eq!(
            notice_in(&[Outcome::Incorporated {
                part: mesocosm_core::PartId(1),
            }]),
            Some("grew")
        );
    }
}
