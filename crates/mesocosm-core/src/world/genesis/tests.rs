// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a founding must be true of. Split out of `genesis.rs` at the 600-line
//! ceiling before DC2 added the archetype arm.

use super::*;

// Before the kingdom floor (2026-08-29, TD2b), 2 of these 10 seeds
// founded zero producer species -- guaranteed collapse under any
// constants. Every seed must now found all three kingdoms among the
// non-played species.
// The founding cohort every test below reads. Derived, not typed: S1 tied
// it to the enclosure's area, and a test that kept saying 60 would stop
// being about the world that ships.
use super::super::FOUNDERS;

// **The DC1.5 census.** The pyramid no longer authors a kingdom onto a
// founder; it picks which body that founder draws, and the world reads the
// kingdom back off the body's feeding organs. So the transitional draw owes
// a receipt that the two agree for *every* founder — one body that read the
// wrong tier would be a producer that cannot fix or a consumer that cannot
// eat, and the whole ecology stands on the pyramid being real.
#[test]
fn every_founding_body_reads_the_kingdom_its_tier_drew() {
    for seed in 1u64..=10 {
        let world = World::new(seed, FOUNDERS);
        let intended = intended_kingdoms(seed, FOUNDERS);
        let mut census: BTreeMap<Kingdom, u32> = BTreeMap::new();
        for organism in &world.organisms {
            let drew = intended[organism.id.0 as usize];
            assert_eq!(
                organism.kingdom(),
                drew,
                "seed {seed}: founder {:?} drew {drew:?} and its body reads {:?}",
                organism.id,
                organism.kingdom()
            );
            *census.entry(drew).or_default() += 1;
        }
        assert_eq!(
            census.values().sum::<u32>(),
            FOUNDERS + 1,
            "seed {seed} censused {census:?} of {} founders",
            FOUNDERS + 1
        );
    }
}

/// The tiers genesis drew, in founder order, replaying the same seeded
/// draws `World::with_development_palette` makes above them. Kept beside
/// the code it mirrors so the census asserts against the *intent* rather
/// than against the reading it is checking.
fn intended_kingdoms(seed: u64, organism_count: u32) -> Vec<Kingdom> {
    let mut rng = Rng::from_seed(seed);
    let mut floor = [Kingdom::Producer, Kingdom::Consumer, Kingdom::Decomposer];
    for i in (1..floor.len()).rev() {
        floor.swap(i, rng.below(i as u64 + 1) as usize);
    }
    let mut kingdoms = pyramid(organism_count as usize);
    for i in (1..kingdoms.len()).rev() {
        kingdoms.swap(i, rng.below(i as u64 + 1) as usize);
    }
    // The played critter founds first and is always a consumer.
    std::iter::once(Kingdom::Consumer).chain(kingdoms).collect()
}

// Both readings of a consumer must be reachable at founding. Under the
// symmetry bijection they were not: every founding consumer drew a limbed
// recipe and read Predator, and Grazer was only a state a line fell into by
// losing its limbs. Mouth geometry is what makes them two bodies.
#[test]
fn founding_reaches_both_a_jaw_and_a_crop() {
    let mut modes: std::collections::BTreeSet<crate::process::FeedingMode> = Default::default();
    for seed in 1u64..=10 {
        let world = World::new(seed, FOUNDERS);
        for organism in &world.organisms {
            if organism.kingdom() == Kingdom::Consumer {
                modes.insert(organism.feeding_mode());
            }
        }
    }
    assert_eq!(
        modes.len(),
        2,
        "ten seeds founded only {modes:?} of the two consumer readings"
    );
}

#[test]
fn every_seed_founds_all_three_kingdoms() {
    for seed in 1u64..=10 {
        let world = World::new(seed, FOUNDERS);
        let mut kingdoms: BTreeMap<Kingdom, u32> = BTreeMap::new();
        for organism in &world.organisms {
            if organism.species != SpeciesId(1) {
                *kingdoms.entry(organism.kingdom()).or_default() += 1;
            }
        }
        for kingdom in [Kingdom::Producer, Kingdom::Consumer, Kingdom::Decomposer] {
            assert!(
                kingdoms.get(&kingdom).is_some_and(|&count| count > 0),
                "seed {seed} founded no {kingdom:?} among the non-played species: {kingdoms:?}"
            );
        }
    }
}

// The founding composition is a pyramid, not equal thirds (2026-08-29,
// TD7): many producers, fewer consumers, few decomposers, exactly and in
// every seed rather than on average.
#[test]
fn founding_is_a_pyramid_in_every_seed() {
    for seed in 1u64..=10 {
        let world = World::new(seed, FOUNDERS);
        let mut kingdoms: BTreeMap<Kingdom, u32> = BTreeMap::new();
        for organism in &world.organisms {
            if organism.species != SpeciesId(1) {
                *kingdoms.entry(organism.kingdom()).or_default() += 1;
            }
        }
        assert_eq!(
            (
                kingdoms.get(&Kingdom::Producer).copied(),
                kingdoms.get(&Kingdom::Consumer).copied(),
                kingdoms.get(&Kingdom::Decomposer).copied(),
            ),
            // 916 founders: 610 producers (2/3), 229 consumers (1/4), 77
            // decomposers (the rest). The shares are TD7's; the counts moved
            // with S1's area-scaled cohort and the pyramid survived it.
            (Some(610), Some(229), Some(77)),
            "seed {seed} founded {kingdoms:?} rather than the 2/3 : 1/4 : rest pyramid"
        );
    }
}

// The pyramid never costs the TD2b floor: a founding too small to give a
// tier its share still gives it a founder.
#[test]
fn a_small_pyramid_still_founds_every_kingdom() {
    for count in 3..=12 {
        let tiers = pyramid(count);
        assert_eq!(tiers.len(), count);
        for kingdom in [Kingdom::Producer, Kingdom::Consumer, Kingdom::Decomposer] {
            assert!(
                tiers.contains(&kingdom),
                "a founding of {count} left out {kingdom:?}: {tiers:?}"
            );
        }
    }
}

// Before the stagger (2026-08-29, TD2b), every founder read
// since_offspring 0, gating a world's whole first brood behind one full
// gestation. Founders beyond the played critter must now spread across
// the range, the way `age` already does.
#[test]
fn since_offspring_is_staggered_like_age() {
    let world = World::new(4_242, 60);
    let distinct: std::collections::BTreeSet<u32> = world
        .organisms
        .iter()
        .filter(|o| o.species != SpeciesId(1))
        .map(|o| o.since_offspring)
        .collect();
    assert!(
        distinct.len() > 1,
        "every non-played founder read the same since_offspring: {distinct:?}"
    );
    assert!(
        distinct.iter().any(|&v| v > 0),
        "no founder started with a head start on gestation"
    );
}

// Before the mid-life stagger (2026-08-29, TD5b), every founder's age was
// rng.below(200) against a lifespan in the thousands, so nothing died of
// old age until deep into the run and the enclosure held no real carrion
// until then (TD5's corpse-drought finding). Age must now range well
// past the old flat cap, and the played critter must stay a newborn --
// the player's life should start near its beginning, not drawn from the
// same distribution as the ecology around it.
#[test]
fn ages_are_staggered_across_the_founders_own_lifespan() {
    let world = World::new(4_242, 60);
    let max_age = world
        .organisms
        .iter()
        .filter(|o| o.species != SpeciesId(1))
        .map(|o| o.age)
        .max()
        .expect("60 non-played founders");
    assert!(
        max_age > 200,
        "no founder aged past the old flat rng.below(200) cap: {max_age}"
    );
    let played = world
        .organisms
        .iter()
        .find(|o| o.id == OrganismId(0))
        .expect("the played critter founds as organism 0");
    assert_eq!(played.age, 0, "the played critter did not start a newborn");
}

// **The DC2 arm.** The consumer tier founds from the authored browsing
// hexapod; the other two still draw. Three things have to hold at once for the
// arm to be isolable and for the archetype to be the creature it was authored
// as.

#[test]
fn the_authored_tier_founds_mobile_grazers() {
    // The body this world has never had: a consumer that crops *and* walks.
    // Before DC1.5 grazing and sessility were one reading, so this founding is
    // new ecology rather than new geometry, which is why the instrument runs
    // both arms.
    for seed in 1u64..=10 {
        let world = World::founded(seed, FOUNDERS, Founding::BrowsingConsumer)
            .expect("the archetype palette is admissible");
        let mut consumers = 0;
        for organism in &world.organisms {
            if organism.kingdom() != Kingdom::Consumer {
                continue;
            }
            consumers += 1;
            assert_eq!(
                organism.feeding_mode(),
                crate::process::FeedingMode::Grazer,
                "seed {seed}: founder {:?} is not a grazer",
                organism.id
            );
            assert!(
                organism.actuator_span() > 0,
                "seed {seed}: founder {:?} cannot walk",
                organism.id
            );
        }
        assert_eq!(
            consumers, 230,
            "seed {seed}: 229 founders plus the played one"
        );
    }
}

// The full-bodied archetype reads §2.4's carving-B column off a *founded*
// organism, not off a bench fixture. Developmental absence still takes an
// appendage pair from some individuals; those are the ones that read lower,
// and every one of them still reads the tier it drew.
#[test]
fn a_founded_archetype_reads_the_carving_b_column() {
    let world = World::founded(1, FOUNDERS, Founding::BrowsingConsumer)
        .expect("the archetype palette is admissible");
    let mut whole = 0;
    for organism in &world.organisms {
        if organism.kingdom() != Kingdom::Consumer {
            continue;
        }
        if organism.body.living().count() != 33 {
            // An absence took one appendage pair; the recipe is the same.
            assert!(organism.body.living().count() < 33);
            continue;
        }
        whole += 1;
        assert_eq!(organism.mass_ceiling_mg(), 1_284);
        assert_eq!(organism.actuator_span(), 18);
        assert_eq!(organism.sensor_span(), 2);
    }
    assert!(whole > 0, "no founder developed the whole archetype");
}

// The arm is isolable: producers and decomposers found bit-identical bodies
// under both foundings. The archetype palette only fills spare slots and an
// authored tier leaves its own salted stream unspent, so nothing the other two
// tiers draw can move.
#[test]
fn the_arm_leaves_the_other_two_tiers_alone() {
    for seed in 1u64..=10 {
        let drawn = World::new(seed, FOUNDERS);
        let armed = World::founded(seed, FOUNDERS, Founding::BrowsingConsumer)
            .expect("the archetype palette is admissible");
        let consumer_species: BTreeMap<SpeciesId, ()> = drawn
            .organisms
            .iter()
            .filter(|o| o.kingdom() == Kingdom::Consumer)
            .map(|o| (o.species, ()))
            .collect();
        let mut compared = 0;
        for (before, after) in drawn.organisms.iter().zip(&armed.organisms) {
            assert_eq!(before.id, after.id);
            if consumer_species.contains_key(&before.species) {
                continue;
            }
            assert_eq!(
                crate::snapshot::encode(&before.body).expect("a body encodes"),
                crate::snapshot::encode(&after.body).expect("a body encodes"),
                "seed {seed}: founder {:?} changed body under the archetype arm",
                before.id
            );
            assert_eq!(before.position, after.position);
            compared += 1;
        }
        assert_eq!(
            compared, 687,
            "seed {seed}: 610 producers and 77 decomposers"
        );
    }
}
