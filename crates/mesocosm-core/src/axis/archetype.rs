// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Authored creatures: the bodies a world is meant to open holding.
//!
//! Sibling of [`catalogue`](super::catalogue) and its opposite in intent. The
//! catalogue's own doc says "these are reference points, not content" — it
//! exists to prove four axial rules reach a centipede. **This module is
//! content**: imagined creatures, each a [`Recipe`] plus the palette entries
//! its shapes need, developed by the same `Recipe -> Soma -> develop_body`
//! path every other body takes. Nothing here builds a [`BodyDocument`]
//! directly; an archetype that did would be the second simulation authority
//! `CLAUDE.md` forbids.
//!
//! # No names
//!
//! The identifiers are role-descriptive. Naming rounds in this repo carry
//! crates.io / game / studio / trademark checks and are Mark's; a founding
//! lineage is also deliberately unnamed (`species.rs`: "they were there before
//! anybody arrived to name them").
//!
//! # The palette is world state and the recipe is the program
//!
//! An archetype is therefore two halves that live in different places. Its
//! *shapes* go in [`palette`], which a world snapshots; its *arrangement* goes
//! in the recipe, which a lineage carries. Every shape below is added in a
//! spare [`RoleShapes`] slot — no default moves — so a tier that still draws
//! from [`seed`](super::seed) develops exactly the body it developed before.

use super::{Appendage, Recipe, Tagma};
use crate::body::VolumeRef;
use crate::development::{PartPalette, PartTemplate};

/// The trunk shape a slim segment is made of: 5x3x3 voxels, 36 mg of ceiling.
const SLIM: [i32; 3] = [2, 1, 1];
/// A broad segment, the one a leg pair hangs off: 5x5x3, 60 mg.
const BROAD: [i32; 3] = [2, 2, 1];
/// The cropping mouth: a flat 5x3x1 blade, 12 mg. `Mass`-classified, which is
/// what makes the body that wears it a [`Grazer`](crate::process::FeedingMode).
const CROP: [i32; 3] = [2, 1, 0];
/// A walking leg: 7x3x3, 50 mg, span 3. Build price 6.00 against the primitive
/// limb's 6.25, so it is admissible under the §2.3 guard; DC1's finding is
/// that at a 3x3 cross-section `[4,1,1]` is the longest leg that would be.
const LEG: [i32; 3] = [3, 1, 1];
/// A working eye, the primitive sensor: 3x3x3, 21 mg, span 1.
const EYE: [i32; 3] = [1, 1, 1];
/// A decorative sense voxel: one voxel, 1 mg, and span **zero**, so it buys
/// detail and no horizon. The plan's §2.2 names this as its own kind of part —
/// "an archetype's decorative eye voxels and its functional sense organs are
/// therefore not the same parts."
const SPECK: [i32; 3] = [0, 0, 0];

/// Palette selector for [`SLIM`] within the `Mass` bank.
const SHAPE_SLIM: u8 = 1;
/// Palette selector for [`BROAD`].
const SHAPE_BROAD: u8 = 2;
/// Palette selector for [`CROP`]. A mouth's selector picks its *role* as well
/// as its shape, and everything below [`JAW_SHAPE`](super::JAW_SHAPE) is drawn
/// from the `Mass` bank — which is the whole of why this body grazes.
const SHAPE_CROP: u8 = 3;
/// Palette selector for [`LEG`] within the `Limb` bank.
const SHAPE_LEG: u8 = 1;
/// Palette selector for [`EYE`]: the `Sensor` bank's default.
const SHAPE_EYE: u8 = 0;
/// Palette selector for [`SPECK`].
const SHAPE_SPECK: u8 = 1;

/// The vocabulary the archetypes are carved from, added to the baseline world
/// palette.
///
/// **Every entry is a spare slot.** `PartPalette::primitive`'s four defaults
/// are untouched, so a recipe that names no shape — which is every recipe
/// [`seed`](super::seed) draws — develops the body it always did, and this
/// palette can be installed on a world whose other two tiers are unchanged.
pub fn palette() -> PartPalette {
    let baseline = PartPalette::primitive();
    // The working eye is the baseline's own sensor, taken rather than added:
    // selector zero already names it, and asserting that keeps the archetype's
    // `SHAPE_EYE` honest if the primitive palette ever moves.
    debug_assert_eq!(baseline.sensor.default.half_extent, EYE);
    PartPalette {
        mass: baseline
            .mass
            .and(shape(5, SLIM))
            .and(shape(6, BROAD))
            .and(shape(7, CROP)),
        limb: baseline.limb.and(shape(8, LEG)),
        sensor: baseline.sensor.and(shape(9, SPECK)),
        ..baseline
    }
}

/// A content address and an exact shape. The addresses are fixture tags while
/// the project has no pack loader, continuing the primitive palette's 1-4.
fn shape(tag: u8, half_extent: [i32; 3]) -> PartTemplate {
    PartTemplate {
        volume: VolumeRef::from_tag(tag),
        half_extent,
    }
}

/// A browsing hexapod: the consumer tier's authored body. (Plan §2.4, carving B)
///
/// Thirty-three parts — a slim head carrying a broad cropping mouth, two eyes
/// and two decorative specks, a five-segment neck, three broad segments bearing
/// six legs around a deep chest, and a ten-segment tail. It reads
/// [`Kingdom::Consumer`](crate::organism::Kingdom) because the head bears a
/// mouth and nothing on it fixes, and [`FeedingMode::Grazer`] because that
/// mouth is bulk rather than a jaw: it crops what stands still. **A mobile
/// grazer is a body this world has never founded** — before DC1.5, grazing and
/// sessility were the same reading — so this archetype is the deliberate
/// version of the founding DC1.5 measured and declined to ship.
///
/// # Variance is zero, deliberately
///
/// The plan asks an archetype to set [`Recipe::variance`] rather than inherit
/// the draw's `1 + rng.below(2)`: "a hexapod with a varying leg count is a
/// different creature, while a frond with a varying frond count is the same
/// plant." Kin still are not clones — [`Soma::develop`](super::Soma::develop)
/// keeps its developmental absence, so an individual may be born without a leg
/// pair or without its eyes. What absence may never take is the mouth, which
/// is guarded, so no individual can develop out of its line's kingdom.
///
/// [`FeedingMode::Grazer`]: crate::process::FeedingMode::Grazer
pub fn consumer_browser() -> Recipe {
    let mut recipe = Recipe::of(vec![
        // The head, and the crop borne under it: this one attachment is what
        // the world reads the whole body's living off.
        Tagma::new(1, Appendage::Mouth).with_shapes(SHAPE_SLIM, SHAPE_CROP),
        // Eyes, then specks. Two tagmata because a stretch grows one kind of
        // appendage from one shape, and these are two different parts doing
        // two different jobs.
        Tagma::new(1, Appendage::Feeler).with_shapes(SHAPE_SLIM, SHAPE_EYE),
        Tagma::new(1, Appendage::Feeler).with_shapes(SHAPE_SLIM, SHAPE_SPECK),
        // A browser's reach.
        Tagma::bare(5).with_shapes(SHAPE_SLIM, 0),
        // Six legs on three broad segments, around a deep chest.
        Tagma::new(1, Appendage::Limb).with_shapes(SHAPE_BROAD, SHAPE_LEG),
        Tagma::bare(1).with_shapes(0, 0),
        Tagma::new(2, Appendage::Limb).with_shapes(SHAPE_BROAD, SHAPE_LEG),
        Tagma::bare(10).with_shapes(SHAPE_SLIM, 0),
    ]);
    recipe.variance = 0;
    recipe
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::SpeciesId;
    use crate::development::develop_body;
    use crate::organism::Kingdom;
    use crate::plan::{Role, classify};
    use crate::process::{FeedingMode, Process};

    /// The adult mass the recipe implies, and the mass every reading below is
    /// taken at. Plan §2.4's carving-B column.
    const CEILING_MG: u64 = 1_284;

    fn browser(seed: u64) -> crate::body::BodyDocument {
        let recipe = consumer_browser();
        let soma = super::super::Soma::develop(&recipe, seed);
        develop_body(SpeciesId(2), &recipe, &soma, CEILING_MG, palette())
            .expect("the archetype develops at its own adult mass")
    }

    fn ceiling(body: &crate::body::BodyDocument) -> u64 {
        body.living()
            .map(|part| crate::organism::ecology::part_ceiling_mg(part.half_extent))
            .sum()
    }

    fn span(body: &crate::body::BodyDocument, process: Process) -> u32 {
        body.living()
            .filter(|part| body.processes(part.id).contains(&process))
            .map(|part| {
                part.half_extent
                    .iter()
                    .map(|v| v.unsigned_abs())
                    .max()
                    .unwrap_or(0)
            })
            .sum()
    }

    /// **The DC2 receipt.** Every economy number §2.4 predicted for carving B,
    /// measured off a developed archetype body rather than derived on paper.
    #[test]
    fn the_browser_reads_the_carving_b_column() {
        let body = browser(1);
        assert_eq!(body.living().count(), 33, "part count");
        assert_eq!(ceiling(&body), CEILING_MG, "mass_ceiling_mg");
        assert_eq!(span(&body, Process::Contract), 18, "actuator_span");
        assert_eq!(span(&body, Process::Sense), 2, "sensor_span");

        // build multiple 2.40: kept as the exact rational the economy uses so
        // the assertion is the formula rather than a rounded copy of it.
        assert_eq!(CEILING_MG + 18 * 100, 3_084);

        let ecology = crate::organism::ecology::upkeep_for_body(CEILING_MG, 18, CEILING_MG);
        assert_eq!(ecology, 9, "rent at adult mass, mg/tick");
        assert_eq!(
            crate::organism::ecology::feeding_rate_for_body(CEILING_MG, 18, CEILING_MG),
            49,
            "bite at adult mass, mg"
        );
        assert_eq!(
            crate::organism::ecology::sight_for_body(8, 2, CEILING_MG),
            9,
            "sight horizon, voxels"
        );
        assert_eq!(CEILING_MG * 33 / 100, 423, "breeding gate");
        assert_eq!(CEILING_MG / ecology, 142, "ticks of reserve at full");
    }

    /// The two invariants §2.3 asks an archetype palette to hold, asserted
    /// rather than commented: every shape classifies as the role whose bank it
    /// sits in, and no `Limb` or `Sensor` is priced past the primitive
    /// palette's. `PartPalette::validate` refuses either, so developing at all
    /// is the proof.
    #[test]
    fn the_archetype_palette_is_admissible() {
        assert!(browser(1).living().count() > 0);
        assert_eq!(classify(CROP), Role::Mass, "the crop must be bulk");
        assert_eq!(classify(LEG), Role::Limb);
        assert_eq!(classify(EYE), Role::Sensor);
        assert_eq!(classify(SPECK), Role::Sensor);
        assert_eq!(classify(SLIM), Role::Mass);
        assert_eq!(classify(BROAD), Role::Mass);
    }

    /// A decorative voxel is not a sense organ. Both are `Sensor`-classified
    /// and both perform `Sense`; only the span tells them apart, and the whole
    /// sight horizon rests on that.
    #[test]
    fn a_speck_sees_nothing() {
        let body = browser(1);
        let sensing = body
            .living()
            .filter(|part| body.processes(part.id).contains(&Process::Sense))
            .count();
        assert_eq!(sensing, 4, "two eyes and two specks");
        assert_eq!(span(&body, Process::Sense), 2, "and only the eyes carry it");
    }

    /// **Variance cannot mutate the body out of its kingdom.** The reading
    /// hangs off two facts: the head bears a `Mass` mouth, and nothing on the
    /// body fixes. Absence is guarded from taking a mouth, and the archetype
    /// carries no `Plate` at all, so neither fact is reachable by development.
    #[test]
    fn no_individual_develops_out_of_being_a_grazer() {
        for seed in 0u64..256 {
            let body = browser(seed);
            assert_eq!(Kingdom::of_body(&body), Kingdom::Consumer, "seed {seed}");
            assert_eq!(
                FeedingMode::of_body(&body),
                FeedingMode::Grazer,
                "seed {seed}"
            );
            assert!(
                !body.performs(Process::Fix),
                "seed {seed} grew something that fixes"
            );
        }
    }

    /// Kin resemble without cloning: at variance zero the segment counts are
    /// fixed, and what still differs between individuals is a developmental
    /// absence taking one appendage pair.
    #[test]
    fn kin_resemble_and_are_not_clones() {
        let counts: std::collections::BTreeSet<usize> = (0u64..256)
            .map(|seed| browser(seed).living().count())
            .collect();
        assert!(
            counts.len() > 1,
            "every individual developed identically: {counts:?}"
        );
        assert!(
            counts.contains(&33),
            "the archetype's own body is not among the ones it develops: {counts:?}"
        );
    }
}
