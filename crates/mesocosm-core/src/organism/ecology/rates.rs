// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The life-history constants, and the allometry that scales them.
//!
//! Split out of `ecology.rs` on 2026-08-29 when TD4's idle gating pushed that
//! file at the six-hundred-line ceiling — the same split-before-adding move
//! that put `ecology::tests` and `ecology::movement` in their own files. What
//! stayed next door is the tick itself; what moved here is every number the
//! tick reads and the pure arithmetic that shapes them, which is also the
//! retune's written record.

use super::Organism;

/// Reference body mass for the allometric rates below.
const REFERENCE_MASS_MG: u64 = 100;
/// The integer fourth root of the reference mass, used to normalize quarter
/// power life-history rates.
const REFERENCE_MASS_QRT: u64 = 3;
/// Reference rates at 100 mg. These are model parameters, not organism facts.
/// TD2 set the tempo (life-history times, 3-4x); TD2c reset the trophic rates
/// against TD2b's walled enclosure and balanced founding, which is a different
/// world from the one TD2 was swept in. Receipt:
/// `Code/testing/mesocosm/td2c_persistence.json`.
///
/// TD2: 3x, stretched with lifespan so the juvenile share of a life is
/// unchanged at the slower tempo.
const MATURITY_BASE: u32 = 270;
/// TD2: 3x, putting a 1,000 mg starter's life at 3,000 ticks — five minutes
/// at the canonical 10 ticks/second rather than 100 seconds.
const LIFESPAN_BASE: u32 = 1800;
/// TD2: 4x against lifespan's 3x, one brood fewer per life. Still the knob
/// that decides boil against breathe — 360 boils, 480 does not.
const GESTATION_BASE: u32 = 480;
/// TD2c: 2 -> 5. Balanced founding puts ~22 consumers on ~20 producers, and
/// at 2 the base fixed less than the grazers drew (80 mg/tick against 88), so
/// the whole chain starved. The base has to out-produce its grazing pressure.
const FIXES_BASE_MG: u64 = 5;
/// TD2c: 2 -> 3. A grazer paying upkeep plus movement netted +1 on a fed tick
/// at 2, so it starved below a ~75% prey hit rate; 3 halves the rate it needs.
const GRAZES_BASE_MG: u64 = 3;
/// Scavenger income. Raising this does not rescue decomposers: they starve
/// with carrion in the enclosure but outside the radius they can search
/// (see the TD2c structural finding), so the yield per corpse is not the
/// binding constraint.
const DECAYS_BASE_MG: u64 = 4;
/// The basal cost of being alive.
pub(super) const UPKEEP_BASE_MG: u64 = 1;
/// The allometric share of the body's mass paid as upkeep.
/// TD2: halved rent, taking a 1,000 mg starter's energy budget from 166 ticks
/// of upkeep to 333.
const UPKEEP_SCALE: u64 = 62;
/// Edge of a crowding cell, in voxel units.
/// TD2c: 16 -> 8, undoing TD2's doubling. TD6 kept it: the soil bounds a
/// stand's *mass*, but nothing in a closed cycle bounds its *number*, and a
/// stand answered a finite matter budget by subdividing into ever-smaller
/// plants (620 producers and still rising at the horizon with crowding out).
/// `population_instrument.rs` mirrors this.
pub(super) const CROWD_CELL: i32 = 8;
/// Neighbours a cell supports before its occupants start shading each other
/// out. Beyond this a producer's income falls away and self-thinning begins.
/// TD2c: 2 -> 1, holding the stand's ceiling near 250 -- capacity is
/// `FIXES x COMFORT x UPKEEP_SCALE / 31` per cell.
pub(super) const CROWD_COMFORT: u32 = 1;
/// Voxels in the palette's reference segment (`PartPalette::primitive().mass`
/// is half-extent [2,2,2], so 5x5x5). The unit an adult mass is quoted in.
const REFERENCE_SEGMENT_VOXELS: u64 = 125;
/// The mass one reference segment holds, restated as the unit
/// [`upkeep_for_body`] measures a body's ceiling in — the same
/// `REFERENCE_MASS_MG` above, named for the job so the motility formula reads
/// as "swing per reference segment of body" rather than as arithmetic.
const REFERENCE_SEGMENT_MG: u64 = REFERENCE_MASS_MG;
/// Fraction of a parent's mass an offspring costs, as a divisor.
pub(crate) const OFFSPRING_COST: u64 = 4;
/// Mass below which an organism cannot sustain itself.
pub(crate) const STARVATION_MG: u64 = 20;
/// Ticks of upkeep still held in the budget below which a body without a
/// target starts wandering instead of standing still. TD2d: waiting for the
/// literal last milligram (`energy_mg == 0`) left every kingdom motionless
/// until half dead; this trades a few ticks of margin for a chance to find
/// something before the body starts eating itself. TD5 moved it here from
/// `movement` so the tick's one hunger horizon sits with the tick's numbers.
const HUNGRY_UPKEEP_TICKS: u64 = 8;

/// Whether a body's reserve has fallen low enough to search rather than wait.
/// The shared predicate lives on [`Organism`]; the horizon is this file's, and
/// **both** the wander and the dispersal bonus ask through it — TD2d moved the
/// wander off literal `energy_mg == 0` and left the bonus behind, which TD5
/// closes.
pub(crate) fn is_hungry(organism: &Organism) -> bool {
    organism.budget_below(HUNGRY_UPKEEP_TICKS)
}

/// The adult mass one part can hold: its own voxel volume, priced so that a
/// reference segment holds the ecology's own reference mass.
///
/// **No new authored number.** The two it reads are already here — the
/// allometry's `REFERENCE_MASS_MG` and the palette's own segment shape — which
/// is what makes this the body plan's ceiling rather than a second knob.
/// Floored at one milligram: development already gives every structural part
/// at least that, so a ceiling below it would make a legal body illegal.
pub(crate) fn part_ceiling_mg(half_extent: [i32; 3]) -> u64 {
    let voxels: u64 = half_extent
        .iter()
        .map(|h| 2 * u64::from(h.unsigned_abs()) + 1)
        .product();
    (voxels * REFERENCE_MASS_MG / REFERENCE_SEGMENT_VOXELS).max(1)
}

/// Integer approximation of `mass^0.75`. It is monotonic, deterministic, and
/// uses no floating point in the authority boundary.
pub(crate) fn three_quarter_power(mass_mg: u64) -> u64 {
    let mass = mass_mg.max(1) as u128;
    integer_sqrt(mass * integer_sqrt(mass)) as u64
}

/// Integer approximation of `mass^0.25` for life-history tempo.
pub(crate) fn quarter_power(mass_mg: u64) -> u64 {
    integer_sqrt(integer_sqrt(mass_mg.max(1) as u128)) as u64
}

fn integer_sqrt(value: u128) -> u128 {
    let mut low = 0u128;
    let mut high = value.saturating_add(1);
    while high - low > 1 {
        let middle = low + (high - low) / 2;
        if middle <= value / middle.max(1) {
            low = middle;
        } else {
            high = middle;
        }
    }
    low
}

fn quarter_rate(base: u32, mass_mg: u64) -> u32 {
    let q = quarter_power(mass_mg).max(1);
    (u64::from(base) * q / REFERENCE_MASS_QRT).max(1) as u32
}

fn allometric_rate(base: u64, mass_mg: u64) -> u64 {
    let reference = three_quarter_power(REFERENCE_MASS_MG).max(1);
    (base * three_quarter_power(mass_mg) / reference).max(1)
}

pub(crate) fn maturity_for_mass(mass_mg: u64) -> u32 {
    quarter_rate(MATURITY_BASE, mass_mg)
}

pub(crate) fn lifespan_for_mass(mass_mg: u64) -> u32 {
    quarter_rate(LIFESPAN_BASE, mass_mg)
}

pub(crate) fn gestation_for_mass(mass_mg: u64) -> u32 {
    quarter_rate(GESTATION_BASE, mass_mg)
}

pub(crate) fn producer_income_for_mass(mass_mg: u64) -> u64 {
    allometric_rate(FIXES_BASE_MG, mass_mg)
}

pub(crate) fn feeding_rate_for_mass(mass_mg: u64) -> u64 {
    allometric_rate(GRAZES_BASE_MG, mass_mg)
}

pub(crate) fn decay_rate_for_mass(mass_mg: u64) -> u64 {
    allometric_rate(DECAYS_BASE_MG, mass_mg)
}

/// Rent priced by how a body lives, not only by what it weighs. (TD7)
///
/// **Derived, not tuned — three body-plan numbers and no new constant.**
///
/// ```text
/// rent = UPKEEP_BASE_MG + m^0.75 * (ceiling + span * REFERENCE_SEGMENT_MG)
///                       / (UPKEEP_SCALE * ceiling)
/// ```
///
/// The mass term is unchanged. The motile multiple is
/// `span * REFERENCE_SEGMENT_MG / ceiling`: **the actuator swing this body
/// carries per reference segment of body it carries it on**, where `span` is
/// `Organism::actuator_span` (each contractile part's longest half-extent,
/// summed) and `ceiling` is `Organism::mass_ceiling_mg` (the adult mass the
/// plan describes, itself a sum of per-part voxel volumes).
///
/// Why this is the honest normalizer: the swing has to be measured against
/// *something*, or a long body pays for being long rather than for moving.
/// Dividing by the plan's own adult mass makes the term scale-free — both
/// halves grow with the body — so it reads a body's **build**, not its size.
///
/// The consequences are all readings, not choices. A sessile plan reads span
/// 0 and pays exactly the mass rent it paid before, to the milligram. A
/// palette limb is half-extent `[4,1,1]`, so it swings 4 and holds a 64 mg
/// ceiling while an axial segment holds 100 mg; the seeded consumers and
/// decomposers this world founds land at roughly 2 to 4 swing per reference
/// segment, so they pay 3x to 5x a plant's rent for the same mass. And the
/// multiple is **bounded by construction**: a body made of nothing but limbs
/// reads `4 * 100 / 64` = 6.25, so no anatomy can price itself past ~7x.
///
/// Integer-exact and monotonic: one division, so conservation holds and the
/// rent a body owes is the rent it pays.
pub(crate) fn upkeep_for_body(mass_mg: u64, actuator_span: u32, ceiling_mg: u64) -> u64 {
    // A body with every part severed has no ceiling to normalize against; it
    // also has no actuators, so the mass term is the whole answer.
    let ceiling = ceiling_mg.max(1);
    let priced = ceiling + u64::from(actuator_span) * REFERENCE_SEGMENT_MG;
    UPKEEP_BASE_MG + three_quarter_power(mass_mg) * priced / (UPKEEP_SCALE * ceiling)
}

/// One graph step's dispersal budget. Contractile geometry gives larger bodies
/// more options, while hunger makes leaving an exhausted place worthwhile.
pub(crate) fn dispersal_for(organism: &Organism) -> u32 {
    (organism.locomotion() / 4).max(1) + u32::from(is_hungry(organism))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::organism::Kingdom;

    #[test]
    fn rent_prices_the_body_plan_not_only_the_mass() {
        // TD7's asymmetry, and that it is a reading rather than a constant.
        // Same mass, same adult ceiling, different build.
        let mass = 1_000;
        let ceiling = 2_000;
        let sessile = upkeep_for_body(mass, 0, ceiling);

        assert_eq!(
            sessile,
            upkeep_for_body(mass, 0, 1),
            "a body with no actuator pays the mass rent whatever its ceiling"
        );
        assert_eq!(
            sessile,
            UPKEEP_BASE_MG + three_quarter_power(mass) / UPKEEP_SCALE,
            "and pays exactly what it paid before TD7"
        );

        // Four palette limbs: half-extent [4,1,1], so they swing 4 apiece.
        let motile = upkeep_for_body(mass, 4 * 4, ceiling);
        assert!(
            motile > sessile,
            "moving cost nothing: {motile} against {sessile}"
        );
        assert!(
            upkeep_for_body(mass, 4 * 8, ceiling) > motile,
            "twice the swing did not cost more"
        );
        // Bounded by construction: a limb swings 4 and holds a 64 mg ceiling,
        // so a body of nothing but limbs — the most motile anatomy the palette
        // can express — reads 4 * 100 / 64 swing per reference segment and
        // tops out near 7x however many of them it grows.
        let all_limbs = upkeep_for_body(mass, 4 * 100, 64 * 100);
        assert!(
            all_limbs <= sessile * 8,
            "the surcharge outran the bound the body plan puts on it: \
             {all_limbs} against {sessile}"
        );
    }

    #[test]
    fn a_seeded_producer_is_sessile_and_a_seeded_consumer_is_not() {
        // The rent asymmetry only means anything if the bodies the world
        // actually founds differ in the number it reads. They do, by recipe:
        // `axis::seed` gives an unlimbed line no contractile part at all.
        let world = crate::world::World::new(3, 60);
        let (mut sessile_producers, mut motile_consumers) = (0, 0);
        for organism in world.organisms.iter().filter(|o| o.is_alive()) {
            match organism.kingdom() {
                Kingdom::Producer => {
                    assert_eq!(
                        organism.actuator_span(),
                        0,
                        "a producer grew an actuator: {:?}",
                        organism.id
                    );
                    sessile_producers += 1;
                }
                Kingdom::Consumer if organism.actuator_span() > 0 => motile_consumers += 1,
                _ => {}
            }
        }
        assert!(sessile_producers > 0 && motile_consumers > 0);
    }
}
