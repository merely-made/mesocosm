// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What locomotion and target choice have to keep true.
//!
//! Split out of `movement.rs` in TD10, which is the standing move here: the
//! file was at 585 lines against this repo's six-hundred ceiling and kinship
//! had both a rule and a behaviour to state.

use super::*;
use crate::body::{SpeciesId, VolumeRef};
use crate::organism::{Kingdom, Signal};
use crate::species::Lineages;

fn target(id: u32, species: u32, at: [i32; 3], mass_mg: u64) -> LivingTarget {
    LivingTarget {
        id: OrganismId(id),
        position: at,
        organism_index: id as usize,
        kingdom: Kingdom::Consumer,
        species: SpeciesId(species),
        mass_mg,
        signal: Signal::Plain,
        shape: WalkerShape::STANDARD,
    }
}

/// The half-extent that makes a consumer a `Predator`: a long-and-thin plan
/// grows a Limb-role part, which performs `Contract`.
fn predator(species: u32, mass_mg: u64) -> Organism {
    Organism::founding(
        OrganismId(0),
        SpeciesId(species),
        Kingdom::Consumer,
        VolumeRef::from_tag(16),
        [8, 2, 2],
        [0, 0, 0],
        mass_mg,
    )
}

#[test]
fn a_predator_passes_over_its_own_line_for_a_stranger_but_still_takes_kin_alone() {
    // TD10's whole rule as one behaviour. Two founding lines with no common
    // ancestor, a sibling of the eater's own species standing *nearer* and
    // *fatter* than a stranger — every term of the old score preferred it —
    // and the kin remove is what turns it down. Take the stranger away and the
    // same predator takes the sibling: rare, not impossible.
    let mut lineages = Lineages::new();
    lineages.found(SpeciesId(2));
    lineages.found(SpeciesId(9));
    let kin = Kin::new(&lineages);
    let hunter = predator(2, 300);

    let sibling = target(1, 2, [1, 0, 0], 4_000);
    let stranger = target(2, 9, [3, 0, 0], 100);
    let both = vec![sibling, stranger];
    assert_eq!(
        choose_living_target(&hunter, &both, &living_cells(&both), None, &kin),
        Some(2),
        "the stranger, though it is further off and smaller"
    );

    let alone = vec![sibling];
    assert_eq!(
        choose_living_target(&hunter, &alone, &living_cells(&alone), None, &kin),
        Some(1),
        "and the sibling when there is nothing else in reach"
    );
}

#[test]
fn an_unrelated_line_is_eaten_exactly_as_it_was_before_kinship() {
    // The undefined-distance decision where it is spent: two founding lines
    // share no ancestor, so nothing about a stranger's ranking moved and the
    // old score's own preference — nearer, then fatter — still decides.
    let mut lineages = Lineages::new();
    for id in [2, 8, 9] {
        lineages.found(SpeciesId(id));
    }
    let kin = Kin::new(&lineages);
    let hunter = predator(2, 300);

    let far_fat = target(1, 8, [4, 0, 0], 4_000);
    let near_thin = target(2, 9, [1, 0, 0], 100);
    let living = vec![far_fat, near_thin];
    assert_eq!(
        choose_living_target(&hunter, &living, &living_cells(&living), None, &kin),
        Some(2),
        "the nearer stranger, undiscounted"
    );
}

#[test]
fn lost_sight_memory_expires_and_cannot_cross_the_tier_line() {
    let mut hunter = Organism::founding(
        OrganismId(0),
        SpeciesId(2),
        Kingdom::Consumer,
        VolumeRef::from_tag(16),
        [3, 1, 1],
        [0, 1, 0],
        300,
    );
    let target = OrganismId(900);
    let living = vec![LivingTarget {
        id: target,
        position: [4, 1, 0],
        organism_index: 0,
        kingdom: Kingdom::Producer,
        species: SpeciesId(7),
        mass_mg: 300,
        signal: Signal::Plain,
        shape: WalkerShape::STANDARD,
    }];
    let ground = Ground::default();
    hunter.last_seen = Some(LastSeen {
        target,
        position: [4, 1, 0],
        ticks_left: 1,
    });

    assert_eq!(
        remembered_target(&mut hunter, &living, Some(&ground)),
        Some([4, 1, 0])
    );
    assert_eq!(hunter.last_seen.unwrap().ticks_left, 0);
    assert_eq!(remembered_target(&mut hunter, &living, Some(&ground)), None);
    assert_eq!(hunter.last_seen, None);

    hunter.last_seen = Some(LastSeen {
        target,
        position: [4, 1, 0],
        ticks_left: MEMORY_TICKS,
    });
    hunter.tier = Tier::Far;
    assert_eq!(remembered_target(&mut hunter, &living, Some(&ground)), None);
    assert_eq!(hunter.last_seen, None);
}
