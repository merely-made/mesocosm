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

use crate::cohort;
use crate::development::PartPalette;
use crate::places::{Ground, Places, Tier, TierLine};
use crate::process::FeedingMode;
use crate::rng::Rng;
use crate::species::Lineages;

use crate::history::Event;

use super::{Organism, OrganismId, Stage, Tally};

mod movement;

use movement::{
    CarrionTarget, LivingTarget, carrion_cells, choose_carrion_target, choose_living_target,
    disperse, living_cells, surface_stance,
};

/// Reference body mass for the allometric rates below.
const REFERENCE_MASS_MG: u64 = 100;
/// The integer fourth root of the reference mass, used to normalize quarter
/// power life-history rates.
const REFERENCE_MASS_QRT: u64 = 3;
/// Reference rates at 100 mg. These are model parameters, not organism facts.
/// The 2026-08-29 numbers are the TD2 retune, swept against
/// `examples/population_instrument.rs` over seeds 1-10 at 61 founders; the
/// receipt is `Code/testing/mesocosm/td2_retune.json`.
///
/// 2026-08-29 retune: 3x, stretched with lifespan so the juvenile share of a
/// life is unchanged at the slower tempo.
const MATURITY_BASE: u32 = 270;
/// 2026-08-29 retune: 3x, putting a 1,000 mg starter's life at 3,000 ticks —
/// five minutes at the canonical 10 ticks/second rather than 100 seconds.
const LIFESPAN_BASE: u32 = 1800;
/// 2026-08-29 retune: 4x against lifespan's 3x, one brood fewer per life. This
/// is the knob that decides boil against breathe: 360 boils twice over the ten
/// seeds and 420 once, 480 none.
const GESTATION_BASE: u32 = 480;
/// 2026-08-29 retune: producer income cut a third, the bloom's own throttle.
const FIXES_BASE_MG: u64 = 2;
/// 2026-08-29 retune: halved, so a grazer crops its pasture instead of
/// stripping it and starving behind it. Consumers persist at 2 and die out at
/// 4.
const GRAZES_BASE_MG: u64 = 2;
/// 2026-08-29 retune: scavenger income raised against the stretched tempo. It
/// does not rescue decomposers on its own — carrion is too sparse to find.
const DECAYS_BASE_MG: u64 = 4;
/// The basal cost of being alive.
const UPKEEP_BASE_MG: u64 = 1;
/// The allometric share of the body's mass paid as upkeep.
/// 2026-08-29 retune: halved rent, taking a 1,000 mg starter's energy budget
/// from 166 ticks of upkeep to 333.
const UPKEEP_SCALE: u64 = 62;
/// Edge of a crowding cell, in voxel units.
/// 2026-08-29 retune: doubled, so shading still registers once bodies spread
/// past the ground the enclosure grew. `population_instrument.rs` mirrors this.
const CROWD_CELL: i32 = 16;
/// Neighbours a cell supports before its occupants start shading each other
/// out. Beyond this a producer's income falls away and self-thinning begins.
/// 2026-08-29 retune: halved, which halves the enclosure's producer capacity.
const CROWD_COMFORT: u32 = 2;
/// Fraction of a parent's mass an offspring costs, as a divisor.
pub(crate) const OFFSPRING_COST: u64 = 4;
/// Mass below which an organism cannot sustain itself.
pub(crate) const STARVATION_MG: u64 = 20;

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

pub(crate) fn upkeep_for_mass(mass_mg: u64) -> u64 {
    UPKEEP_BASE_MG + three_quarter_power(mass_mg) / UPKEEP_SCALE
}

/// One graph step's dispersal budget. Contractile geometry gives larger bodies
/// more options, while hunger makes leaving an exhausted place worthwhile.
pub(crate) fn dispersal_for(organism: &Organism) -> u32 {
    (organism.locomotion() / 4).max(1) + u32::from(organism.energy_mg == 0)
}

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
    step_inner(
        organisms, next_id, rng, events, lineages, palette, None, None, None,
    )
}

/// Advances the enclosure with place-graph ownership enabled. The plain
/// [`step`] entry point remains useful for isolated ecology fixtures; worlds
/// use this path so dispersal and tier hysteresis are part of replayed state.
#[allow(clippy::too_many_arguments)]
pub fn step_with_places(
    organisms: &mut Vec<Organism>,
    next_id: &mut u32,
    rng: &mut Rng,
    events: &mut Vec<Event>,
    lineages: &Lineages,
    palette: PartPalette,
    places: &Places,
    focus: Option<[i32; 3]>,
) -> Tally {
    step_inner(
        organisms,
        next_id,
        rng,
        events,
        lineages,
        palette,
        Some(places),
        None,
        focus,
    )
}

/// Advances a world-owned ecology against its voxel truth. Graph-only
/// fixtures keep using [`step_with_places`]; a replayed world calls this path
/// so embodied bodies obey the same footing and sight rules as the player.
#[allow(clippy::too_many_arguments)]
pub fn step_with_ground(
    organisms: &mut Vec<Organism>,
    next_id: &mut u32,
    rng: &mut Rng,
    events: &mut Vec<Event>,
    lineages: &Lineages,
    palette: PartPalette,
    places: &Places,
    ground: &Ground,
    focus: Option<[i32; 3]>,
) -> Tally {
    step_inner(
        organisms,
        next_id,
        rng,
        events,
        lineages,
        palette,
        Some(places),
        Some(ground),
        focus,
    )
}

#[allow(clippy::too_many_arguments)]
fn step_inner(
    organisms: &mut Vec<Organism>,
    next_id: &mut u32,
    rng: &mut Rng,
    events: &mut Vec<Event>,
    lineages: &Lineages,
    palette: PartPalette,
    places: Option<&Places>,
    ground: Option<&Ground>,
    focus: Option<[i32; 3]>,
) -> Tally {
    let mut tally = Tally::default();

    // Tier ownership is hysteretic state. Forming the far summary here makes
    // cohort conservation part of the tick's receipt rather than an
    // unaccounted host cache.
    if let (Some(places), Some(focus)) = (places, focus) {
        for organism in organisms.iter_mut().filter(|o| o.is_alive()) {
            let previous = organism.tier;
            organism.tier = TierLine::default().tick(places, previous, organism.position, focus);
            match (previous, organism.tier) {
                (Tier::Far, Tier::Near) => tally.promoted += 1,
                (Tier::Near, Tier::Far) => tally.demoted += 1,
                _ => {}
            }
        }
        let far = cohort::from_organisms(organisms, places);
        let (members, biomass, _) = cohort::conserved_totals(&far);
        tally.far_cohorts = far.len() as u32;
        tally.far_members = members as u32;
        tally.far_biomass_mg = biomass;
    }

    // Crowding, counted once per tick on a coarse grid. A BTreeMap keeps
    // iteration ordered, and bucketing keeps this O(n) rather than O(n^2),
    // which matters at the population sizes this produces.
    let mut density: BTreeMap<(i32, i32), u32> = BTreeMap::new();
    for organism in organisms.iter().filter(|o| o.is_alive()) {
        *density.entry(cell_of(organism.position)).or_default() += 1;
    }

    // Carrion positions are read before anything changes, so decomposers all
    // see the same world within a tick rather than racing each other.
    let carrion: Vec<CarrionTarget> = organisms
        .iter()
        .enumerate()
        .filter(|(_, o)| o.stage == Stage::Carrion)
        .map(|(index, o)| CarrionTarget {
            position: o.position,
            organism_index: index,
            shape: o.walker_shape(),
        })
        .collect();
    let carrion_cells = carrion_cells(&carrion);

    // Living bodies, read before anything changes, so feeding decisions all
    // see the same enclosure within a tick rather than racing each other.
    let living: Vec<LivingTarget> = organisms
        .iter()
        .enumerate()
        .filter(|(_, o)| o.is_alive())
        .map(|(index, o)| LivingTarget {
            id: o.id,
            position: o.position,
            organism_index: index,
            kingdom: o.kingdom(),
            mass_mg: o.biomass_mg(),
            signal: o.signal,
            shape: o.walker_shape(),
        })
        .collect();
    let living_cells = living_cells(&living);

    let mut drained: Vec<(usize, u64)> = Vec::new();
    // Feeding is recorded by index and resolved to ids after the pass, because
    // the borrow that reads the pasture cannot also name the eaten.
    let mut fed: Vec<(OrganismId, usize, u64, crate::history::MealKind)> = Vec::new();
    let mut newborns: Vec<Organism> = Vec::new();

    for organism in organisms.iter_mut() {
        organism.age = organism.age.saturating_add(1);

        match organism.stage {
            Stage::Juvenile | Stage::Mature => {
                // Everything alive pays rent, every tick, **and the rent
                // scales with the body**. Budget first, then the body itself:
                // a creature with nothing left to spend eats itself.
                organism.pay_upkeep();

                match organism.feeding_mode() {
                    // Producers make biomass from the world, but they shade
                    // each other out. Income falls with crowding, so a stand
                    // thins itself instead of growing without bound.
                    FeedingMode::Producer => {
                        let crowd = density
                            .get(&cell_of(organism.position))
                            .copied()
                            .unwrap_or(1);
                        let share = producer_income_for_mass(organism.biomass_mg())
                            .saturating_mul(CROWD_COMFORT as u64)
                            / crowd.max(1) as u64;
                        // Floored at rent. A shaded-out producer stagnates
                        // rather than starving, because otherwise an entire
                        // stand of identical plants crosses the starvation
                        // line on the same tick and the patch goes extinct
                        // instead of thinning.
                        let share = share.clamp(
                            UPKEEP_BASE_MG,
                            producer_income_for_mass(organism.biomass_mg()),
                        );
                        organism.gain_mass(share);
                    }
                    FeedingMode::Grazer | FeedingMode::Predator => {
                        if let Some(prey) =
                            choose_living_target(organism, &living, &living_cells, ground)
                        {
                            let amount = feeding_rate_for_mass(organism.biomass_mg());
                            let kind = if organism.feeding_mode() == FeedingMode::Predator {
                                crate::history::MealKind::Predation
                            } else {
                                crate::history::MealKind::Grazing
                            };
                            organism.gain_mass(amount);
                            drained.push((prey, amount));
                            fed.push((organism.id, prey, amount, kind));
                        }
                    }
                    // Decomposers only earn where something has died.
                    FeedingMode::Scavenger => {
                        if let Some(source) =
                            choose_carrion_target(organism, &carrion, &carrion_cells, ground)
                        {
                            let amount = decay_rate_for_mass(organism.biomass_mg());
                            organism.gain_mass(amount);
                            drained.push((source, amount));
                            fed.push((
                                organism.id,
                                source,
                                amount,
                                crate::history::MealKind::Scavenging,
                            ));
                        }
                    }
                }

                if let Some(places) = places
                    && disperse(
                        organism,
                        places,
                        ground,
                        rng,
                        &living,
                        &living_cells,
                        &carrion,
                        &carrion_cells,
                        events,
                    )
                {
                    tally.moved += 1;
                }

                if organism.stage == Stage::Juvenile
                    && organism.age >= maturity_for_mass(organism.life_history_mass_mg())
                {
                    organism.stage = Stage::Mature;
                    events.push(Event::Matured {
                        organism: organism.id,
                    });
                    tally.matured += 1;
                }

                organism.since_offspring = organism.since_offspring.saturating_add(1);

                let starved = organism.biomass_mg() <= STARVATION_MG;
                let aged = organism.age >= lifespan_for_mass(organism.life_history_mass_mg());
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
    for (eater, index, mass_mg, kind) in fed {
        if let Some(from) = organisms.get(index).map(|o| o.id) {
            events.push(Event::Fed {
                eater,
                from,
                mass_mg,
                kind,
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
        let mut body = body;
        body.plan.symmetry = parent.body.plan.symmetry;
        // Wide enough to leave a crowded cell. Dispersal is how a stand
        // escapes its own shade, so a short throw would trap every offspring
        // in the same competition its parent is already losing.
        let scatter = [rng.range_i32(-12, 12), 0, rng.range_i32(-12, 12)];
        let position = [
            parent.position[0] + scatter[0],
            parent.position[1],
            parent.position[2] + scatter[2],
        ];
        let walker_shape = crate::places::WalkerShape::from_aabb(body.aabb());
        let child = Organism {
            id: child_id,
            species: parent.species,
            // A child starts small but structurally filial: the lineage recipe
            // grew this body under the current world's palette, and the whole
            // graph contains exactly what the parent paid.
            body,
            development_seed,
            life_history_mass_mg: cost,
            energy_mg: cost,
            // A near-tier child is an embodied body immediately, rather
            // than an abstract point that has to be repaired next tick.
            position: match (ground, parent.tier) {
                (Some(ground), Tier::Near) => surface_stance(ground, walker_shape, position)
                    .or_else(|| surface_stance(ground, walker_shape, parent.position))
                    .unwrap_or(parent.position),
                _ => position,
            },
            tier: parent.tier,
            last_seen: None,
            fauna_policy: parent.fauna_policy.inherited(development_seed),
            last_fauna_decision: None,
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
