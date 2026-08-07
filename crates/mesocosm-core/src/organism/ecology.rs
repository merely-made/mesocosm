// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The enclosure living on its own schedule.
//!
//! Split out of `organism.rs` on 2026-08-01, because that file had grown past
//! this repo's six-hundred-line ceiling and the rule is to split before adding
//! rather than after. The types live next door; this is what happens to them
//! every tick.
//!
//! Nothing here knows which organism is being played. That is not an oversight;
//! it is the wing's third law holding at the level of the simulation.

use std::collections::BTreeMap;

use crate::development::PartPalette;
use crate::rng::Rng;
use crate::species::Lineages;

use crate::history::Event;

use super::{Kingdom, Organism, OrganismId, Stage, Tally};

/// Ticks of growth before an organism can reproduce.
pub(crate) const MATURITY: u32 = 90;
/// Ticks of life before an organism dies of age.
pub(crate) const LIFESPAN: u32 = 600;
/// Ticks between one offspring and the next.
pub(crate) const GESTATION: u32 = 120;
/// Milligrams a producer fixes per tick in open ground.
const FIXES_MG: u64 = 3;
/// Milligrams of upkeep every living thing owes before its size is counted.
///
/// The floor. What a body actually pays is this plus a share of what it
/// weighs, so being large is a standing cost rather than a free upgrade.
///
/// Everything pays rent. Without this a producer's income is free and the
/// population grows without bound, which is what the first run of this did:
/// 75 organisms became 1530 in 600 ticks. Biomass share is meaningless when
/// everyone's share grows.
pub(crate) const UPKEEP_MG: u64 = 1;
/// Milligrams of body a creature can carry per extra milligram of upkeep.
///
/// Tuned so a starting critter pays a few milligrams a tick and a
/// well-fed one pays enough to notice. This is the number that decides
/// whether growing is worth it.
pub(crate) const UPKEEP_SHARE: u64 = 250;
/// Edge of a crowding cell, in voxel units.
const CROWD_CELL: i32 = 8;
/// Neighbours a cell supports before its occupants start shading each other
/// out. Beyond this a producer's income falls away and self-thinning begins.
const CROWD_COMFORT: u32 = 4;
/// Milligrams a decomposer draws from nearby carrion per tick.
const DECAYS_MG: u64 = 3;
/// How far a decomposer reaches for the dead, in voxel units.
const DECOMPOSE_RANGE: i32 = 6;
/// Milligrams a consumer takes from living prey per tick.
const GRAZES_MG: u64 = 4;
/// How far a consumer reaches for a meal, in voxel units.
const GRAZE_RANGE: i32 = 5;
/// Fraction of a parent's mass an offspring costs, as a divisor.
pub(crate) const OFFSPRING_COST: u64 = 4;
/// Mass below which an organism cannot sustain itself.
pub(crate) const STARVATION_MG: u64 = 20;

/// Which crowding cell a position falls in.
fn cell_of(position: [i32; 3]) -> (i32, i32) {
    (
        position[0].div_euclid(CROWD_CELL),
        position[2].div_euclid(CROWD_CELL),
    )
}

/// Advances every organism one tick.
///
/// Deterministic: organisms are visited in id order, offspring are appended in
/// the order their parents produced them, and every random value comes from
/// the seeded stream.
/// One tick of the enclosure.
///
/// `events` collects what happened to *individuals*. [`Tally`] counts, which is
/// what a host shows; the events are what a history records, and significance
/// needs to know who rather than how many.
pub fn step(
    organisms: &mut Vec<Organism>,
    next_id: &mut u32,
    rng: &mut Rng,
    events: &mut Vec<Event>,
    lineages: &Lineages,
    palette: PartPalette,
) -> Tally {
    let mut tally = Tally::default();

    // Crowding, counted once per tick on a coarse grid. A BTreeMap keeps
    // iteration ordered, and bucketing keeps this O(n) rather than O(n^2),
    // which matters at the population sizes this produces.
    let mut density: BTreeMap<(i32, i32), u32> = BTreeMap::new();
    for organism in organisms.iter().filter(|o| o.is_alive()) {
        *density.entry(cell_of(organism.position)).or_default() += 1;
    }

    // Carrion positions are read before anything changes, so decomposers all
    // see the same world within a tick rather than racing each other.
    let carrion: Vec<([i32; 3], usize)> = organisms
        .iter()
        .enumerate()
        .filter(|(_, o)| o.stage == Stage::Carrion)
        .map(|(index, o)| (o.position, index))
        .collect();

    // Living producers, read before anything changes, so grazers all see the
    // same pasture within a tick rather than racing each other.
    let grazeable: Vec<([i32; 3], usize)> = organisms
        .iter()
        .enumerate()
        .filter(|(_, o)| o.is_alive() && o.kingdom == Kingdom::Producer)
        .map(|(index, o)| (o.position, index))
        .collect();

    let mut drained: Vec<(usize, u64)> = Vec::new();
    // Feeding is recorded by index and resolved to ids after the pass, because
    // the borrow that reads the pasture cannot also name the eaten.
    let mut fed: Vec<(OrganismId, usize, u64)> = Vec::new();
    let mut newborns: Vec<Organism> = Vec::new();

    for organism in organisms.iter_mut() {
        organism.age = organism.age.saturating_add(1);

        match organism.stage {
            Stage::Juvenile | Stage::Mature => {
                // Everything alive pays rent, every tick, **and the rent
                // scales with the body**. Budget first, then the body itself:
                // a creature with nothing left to spend eats itself.
                organism.pay_upkeep();

                match organism.kingdom {
                    // Producers make biomass from the world, but they shade
                    // each other out. Income falls with crowding, so a stand
                    // thins itself instead of growing without bound.
                    Kingdom::Producer => {
                        let crowd = density
                            .get(&cell_of(organism.position))
                            .copied()
                            .unwrap_or(1);
                        let share =
                            FIXES_MG.saturating_mul(CROWD_COMFORT as u64) / crowd.max(1) as u64;
                        // Floored at rent. A shaded-out producer stagnates
                        // rather than starving, because otherwise an entire
                        // stand of identical plants crosses the starvation
                        // line on the same tick and the patch goes extinct
                        // instead of thinning.
                        let share = share.clamp(UPKEEP_MG, FIXES_MG);
                        organism.gain_mass(share);
                    }
                    // Consumers eat. Without this they were guaranteed to
                    // starve, so every world converged to producers only and
                    // the trophic cycle had a missing rung.
                    Kingdom::Consumer => {
                        if let Some((_, prey)) = grazeable.iter().copied().find(|(at, _)| {
                            (0..3).all(|a| (at[a] - organism.position[a]).abs() <= GRAZE_RANGE)
                        }) {
                            organism.gain_mass(GRAZES_MG);
                            drained.push((prey, GRAZES_MG));
                            fed.push((organism.id, prey, GRAZES_MG));
                        }
                    }
                    // Decomposers only earn where something has died.
                    Kingdom::Decomposer => {
                        if let Some((_, source)) = carrion.iter().copied().find(|(at, _)| {
                            (0..3).all(|a| (at[a] - organism.position[a]).abs() <= DECOMPOSE_RANGE)
                        }) {
                            organism.gain_mass(DECAYS_MG);
                            drained.push((source, DECAYS_MG));
                            fed.push((organism.id, source, DECAYS_MG));
                        }
                    }
                }

                if organism.stage == Stage::Juvenile && organism.age >= MATURITY {
                    organism.stage = Stage::Mature;
                    events.push(Event::Matured {
                        organism: organism.id,
                    });
                    tally.matured += 1;
                }

                organism.since_offspring = organism.since_offspring.saturating_add(1);

                let starved = organism.biomass_mg() <= STARVATION_MG;
                let aged = organism.age >= LIFESPAN;
                if starved || aged {
                    organism.stage = Stage::Carrion;
                    events.push(Event::Died {
                        organism: organism.id,
                        species: organism.species,
                    });
                    organism.since_offspring = 0;
                    tally.died += 1;
                }
            }

            Stage::Carrion => {
                // The dead return whether or not a decomposer is present, just
                // far more slowly. Locked matter is a real failure mode.
                organism.spend_mass(1);
                if organism.biomass_mg() == 0 {
                    organism.stage = Stage::Spent;
                    events.push(Event::Returned {
                        organism: organism.id,
                    });
                    tally.returned += 1;
                }
            }

            Stage::Spent => {}
        }
    }

    // What was fed on pays for it. Grazed prey can be killed outright, which
    // is how a consumer turns a producer into carrion for a decomposer.
    for (eater, index, mass_mg) in fed {
        if let Some(from) = organisms.get(index).map(|o| o.id) {
            events.push(Event::Fed {
                eater,
                from,
                mass_mg,
            });
        }
    }

    for (index, amount) in drained {
        let eaten = &mut organisms[index];
        eaten.spend_mass(amount);
        if eaten.biomass_mg() == 0 {
            eaten.stage = Stage::Spent;
            events.push(Event::Returned { organism: eaten.id });
            tally.returned += 1;
        } else if eaten.is_alive() && eaten.biomass_mg() <= STARVATION_MG {
            eaten.stage = Stage::Carrion;
            events.push(Event::Died {
                organism: eaten.id,
                species: eaten.species,
            });
            eaten.since_offspring = 0;
            tally.died += 1;
        }
    }

    // Reproduction, after everything has been fed, so a tick's births do not
    // depend on where in the list a parent happened to sit.
    let ready: Vec<usize> = organisms
        .iter()
        .enumerate()
        .filter(|(_, o)| o.can_reproduce())
        .map(|(index, _)| index)
        .collect();
    for index in ready {
        let parent = &organisms[index];
        let cost = parent.biomass_mg() / OFFSPRING_COST;
        let child_id = OrganismId(*next_id);
        let development_seed = filial_seed(parent.development_seed, child_id);
        let Some(lineage) = lineages.get(parent.species) else {
            continue;
        };
        // Provisioning is binding. A complex recipe may need more positive-
        // mass parts than a quarter of this parent can pay for; in that case
        // the birth waits, spending neither matter nor ecology entropy.
        let Ok(body) = lineage.realize(development_seed, cost, palette) else {
            continue;
        };
        // Wide enough to leave a crowded cell. Dispersal is how a stand
        // escapes its own shade, so a short throw would trap every offspring
        // in the same competition its parent is already losing.
        let scatter = [rng.range_i32(-12, 12), 0, rng.range_i32(-12, 12)];
        let child = Organism {
            id: child_id,
            species: parent.species,
            kingdom: parent.kingdom,
            // A child starts small but structurally filial: the lineage recipe
            // grew this body under the current world's palette, and the whole
            // graph contains exactly what the parent paid.
            body,
            development_seed,
            energy_mg: cost,
            position: [
                parent.position[0] + scatter[0],
                parent.position[1],
                parent.position[2] + scatter[2],
            ],
            stage: Stage::Juvenile,
            age: 0,
            since_offspring: 0,
            // A lie is heritable. An offspring wears its parent's colours and
            // carries its parent's bite, which is what makes a mimic lineage a
            // thing you can learn rather than a coin flip per organism.
            signal: parent.signal,
            venom_mg: parent.venom_mg,
            guise: parent.guise,
        };
        *next_id += 1;
        events.push(Event::Born {
            organism: child.id,
            species: child.species,
            parent: Some(parent.id),
        });
        newborns.push(child);

        let parent = &mut organisms[index];
        parent.spend_mass(cost);
        parent.since_offspring = 0;
        tally.born += 1;
    }

    organisms.extend(newborns);
    organisms.retain(|o| o.stage != Stage::Spent);
    tally
}

const FILIAL_SALT: u64 = 0x4649_4C49_414C_0001;

fn filial_seed(parent: u64, child: OrganismId) -> u64 {
    let mut stream = Rng::from_seed(parent ^ FILIAL_SALT ^ u64::from(child.0));
    stream.next_u64()
}

#[cfg(test)]
mod tests;
