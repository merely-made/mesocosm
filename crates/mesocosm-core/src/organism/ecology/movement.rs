// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Local affordance lookup and locomotion for ecology ticks.
//!
//! The spatial buckets are rebuilt from the tick's immutable reading. They
//! are an acceleration structure, never state: their only job is to avoid
//! asking every embodied body about every other body before a sight query.

use std::cmp::Reverse;
use std::collections::BTreeMap;

use crate::history::Event;
use crate::organism::Signal;
use crate::places::{Ground, Places, Tier, WALKER_HEIGHT, route_step, spot, step as grounded_step};
use crate::process::FeedingMode;
use crate::rng::Rng;

use crate::organism::{
    FaunaDecisionTrace, FaunaDrive, FaunaSenses, FaunaTraits, Kingdom, LastSeen, Organism,
    OrganismId,
};

use super::dispersal_for;

/// How far a consumer reaches for a meal, in voxel units.
pub(super) const GRAZE_RANGE: i32 = 5;
/// How far a decomposer reaches for the dead, in voxel units.
pub(super) const DECOMPOSE_RANGE: i32 = 6;
/// An embodied mind's local visual horizon. Anatomy still decides what it can
/// reach after it sees a target; an enormous body does not make terrain sight
/// globally omniscient.
const NEAR_SIGHT_RANGE: i32 = 8;
/// A remembered sight line can take a short detour, but never authorizes a
/// global navigation search in an ecology tick.
const MEMORY_ROUTE_BUDGET: i32 = 8;
/// Direct observation is fresh for this many failed perception ticks.
const MEMORY_TICKS: u8 = 8;
/// Target-query cells are finer than crowding cells: they bound perception
/// work, while crowding deliberately groups a wider ecological neighbourhood.
const SENSORY_CELL: i32 = 4;

pub(super) type LivingTarget = (OrganismId, [i32; 3], usize, Kingdom, u64, Signal);
pub(super) type CarrionTarget = ([i32; 3], usize);

type Cells = BTreeMap<(i32, i32), Vec<usize>>;

fn sensory_cell(position: [i32; 3]) -> (i32, i32) {
    (
        position[0].div_euclid(SENSORY_CELL),
        position[2].div_euclid(SENSORY_CELL),
    )
}

pub(super) fn living_cells(living: &[LivingTarget]) -> Cells {
    let mut cells = Cells::new();
    for (index, (_, at, ..)) in living.iter().enumerate() {
        cells.entry(sensory_cell(*at)).or_default().push(index);
    }
    cells
}

pub(super) fn carrion_cells(carrion: &[CarrionTarget]) -> Cells {
    let mut cells = Cells::new();
    for (index, (at, _)) in carrion.iter().enumerate() {
        cells.entry(sensory_cell(*at)).or_default().push(index);
    }
    cells
}

/// Candidate indexes in the horizontal cells intersecting a local range.
/// Buckets are BTree-ordered; the original vector order is still added to
/// target tie-breaks below, so this changes cost rather than preference.
fn nearby_indexes(
    cells: &Cells,
    position: [i32; 3],
    range: i32,
) -> impl Iterator<Item = usize> + '_ {
    let (min_x, min_z) = sensory_cell([position[0] - range, position[1], position[2] - range]);
    let (max_x, max_z) = sensory_cell([position[0] + range, position[1], position[2] + range]);
    cells
        .range((min_x, i32::MIN)..=(max_x, i32::MAX))
        .filter(move |((_, z), _)| *z >= min_z && *z <= max_z)
        .flat_map(|(_, indexes)| indexes.iter().copied())
}

/// Chooses a food source within the body's actual reach. This is local for
/// both tiers, so it never needs a global scan.
pub(super) fn choose_living_target(
    organism: &Organism,
    living: &[LivingTarget],
    cells: &Cells,
    ground: Option<&Ground>,
) -> Option<usize> {
    let mode = organism.feeding_mode();
    let reach = GRAZE_RANGE + organism.body.reach();
    let sight = sight_range(organism, reach, ground);
    let mut candidates: Vec<(u64, usize, usize)> = Vec::new();
    for order in nearby_indexes(cells, organism.position, sight) {
        let Some((id, at, index, kingdom, mass, signal)) = living.get(order) else {
            continue;
        };
        if *id == organism.id
            || !matches!(mode, FeedingMode::Grazer | FeedingMode::Predator)
            || (mode == FeedingMode::Grazer && *kingdom != Kingdom::Producer)
            || chebyshev(organism.position, *at) > reach
            || (*signal != Signal::Plain && mode != FeedingMode::Grazer)
        {
            continue;
        }
        let distance = chebyshev(organism.position, *at) as u64;
        let danger = u64::from(*signal == Signal::Warning) * 4;
        let score = (distance.saturating_mul(16) + danger).saturating_sub((*mass).min(256) / 64);
        candidates.push((score, order, *index));
    }
    candidates.sort_unstable();
    candidates.into_iter().find_map(|(_, order, index)| {
        living
            .get(order)
            .is_some_and(|(_, at, ..)| can_perceive(organism, *at, sight, ground))
            .then_some(index)
    })
}

fn sight_range(organism: &Organism, reach: i32, ground: Option<&Ground>) -> i32 {
    if organism.tier == Tier::Near && ground.is_some() {
        reach.min(NEAR_SIGHT_RANGE)
    } else {
        reach
    }
}

/// Carrion feeding is local too. Returning the original organism index keeps
/// the drain pass independent from the derived bucket representation.
pub(super) fn choose_carrion_target(
    organism: &Organism,
    carrion: &[CarrionTarget],
    cells: &Cells,
    ground: Option<&Ground>,
) -> Option<usize> {
    nearby_indexes(cells, organism.position, DECOMPOSE_RANGE)
        .filter_map(|order| carrion.get(order).map(|target| (order, target)))
        .filter(|(_, (at, _))| {
            (0..3).all(|axis| (at[axis] - organism.position[axis]).abs() <= DECOMPOSE_RANGE)
                && can_perceive(organism, *at, DECOMPOSE_RANGE, ground)
        })
        .min_by_key(|(order, _)| *order)
        .map(|(_, (_, index))| *index)
}

fn chebyshev(from: [i32; 3], to: [i32; 3]) -> i32 {
    (0..3)
        .map(|axis| (from[axis] - to[axis]).abs())
        .max()
        .unwrap_or(0)
}

/// Terrain perception belongs to embodied agents. Far cohorts retain their
/// aggregate place-graph affordances until their promotion boundary is crossed.
fn can_perceive(
    organism: &Organism,
    target: [i32; 3],
    range: i32,
    ground: Option<&Ground>,
) -> bool {
    match (organism.tier, ground) {
        (Tier::Near, Some(ground)) => spot(ground, organism.position, target, range),
        _ => true,
    }
}

/// Finds a valid surface stance for a graph position, if it is resident in the
/// grown ground. Callers choose the fallback: a near-tier birth remains beside
/// its grounded parent when its scatter leaves the grown enclosure.
pub(super) fn surface_stance(ground: &Ground, position: [i32; 3]) -> Option<[i32; 3]> {
    ground
        .surface(position[0], position[2])
        .map(|surface| [position[0], surface + 1, position[2]])
        .filter(|at| ground.stands(*at, WALKER_HEIGHT))
}

fn preferred_living<'a>(
    organism: &Organism,
    candidates: impl Iterator<Item = (usize, &'a LivingTarget)>,
    ground: Option<&Ground>,
    sight: i32,
) -> Option<(OrganismId, [i32; 3])> {
    let mut ranked: Vec<(i32, u64, usize, &'a LivingTarget)> = candidates
        .filter(|(_, (id, _, _, kingdom, _, _))| {
            *id != organism.id
                && (organism.feeding_mode() == FeedingMode::Predator
                    || *kingdom == Kingdom::Producer)
        })
        .map(|(order, target @ (_, at, _, _, mass, _))| {
            (chebyshev(organism.position, *at), *mass, order, target)
        })
        .collect();
    ranked.sort_unstable_by_key(|(distance, mass, order, _)| (*distance, Reverse(*mass), *order));
    ranked
        .into_iter()
        .find_map(|(_, _, _, (id, at, _, _, _, _))| {
            can_perceive(organism, *at, sight, ground).then_some((*id, *at))
        })
}

fn policy_living<'a>(
    organism: &mut Organism,
    candidates: impl Iterator<Item = (usize, &'a LivingTarget)>,
    ground: &Ground,
    sight: i32,
) -> Option<MovementTarget> {
    let traits = FaunaTraits::read(organism);
    let own_mass = organism.biomass_mg();
    let policy = organism.fauna_policy;
    let candidate = candidates
        .filter(|(_, (id, _, _, kingdom, _, _))| {
            *id != organism.id
                && (traits.feeding_mode == FeedingMode::Predator || *kingdom == Kingdom::Producer)
        })
        .filter_map(|(order, target @ (id, at, _, _, mass, signal))| {
            let distance = chebyshev(organism.position, *at);
            if !can_perceive(organism, *at, sight, Some(ground)) {
                return None;
            }
            let senses = FaunaSenses::read(organism, traits, *id, distance, *mass, *signal);
            let scores = policy.score(senses, own_mass, sight);
            let drive = scores.selected();
            let rank = (
                scores.score(drive),
                Reverse(distance),
                *mass,
                Reverse(order),
            );
            Some((rank, target, senses, scores, drive))
        })
        .max_by_key(|(rank, ..)| *rank);

    let Some((_, (id, at, ..), senses, scores, drive)) = candidate else {
        organism.last_fauna_decision = None;
        return None;
    };
    organism.fauna_policy.remember(scores);
    organism.last_fauna_decision = Some(FaunaDecisionTrace {
        traits,
        senses,
        selected_drive: drive,
        selected_target: Some(*id),
        scores,
    });
    Some(match drive {
        FaunaDrive::Pursue => MovementTarget::Seen(*id, *at),
        FaunaDrive::Avoid => MovementTarget::Avoid(*id, *at),
        FaunaDrive::Hold => MovementTarget::Hold(*id, *at),
    })
}

#[derive(Clone, Copy)]
enum MovementTarget {
    Seen(OrganismId, [i32; 3]),
    Avoid(OrganismId, [i32; 3]),
    Hold(OrganismId, [i32; 3]),
    Other([i32; 3]),
}

fn preferred_carrion<'a>(
    organism: &Organism,
    candidates: impl Iterator<Item = (usize, &'a CarrionTarget)>,
    ground: Option<&Ground>,
) -> Option<[i32; 3]> {
    let mut ranked: Vec<(i32, usize, &'a CarrionTarget)> = candidates
        .map(|(order, target @ (at, _))| (chebyshev(organism.position, *at), order, target))
        .collect();
    ranked.sort_unstable_by_key(|(distance, order, _)| (*distance, *order));
    ranked.into_iter().find_map(|(_, _, (at, _))| {
        can_perceive(organism, *at, DECOMPOSE_RANGE, ground).then_some(*at)
    })
}

fn preferred_target(
    organism: &mut Organism,
    living: &[LivingTarget],
    living_cells: &Cells,
    carrion: &[CarrionTarget],
    carrion_cells: &Cells,
    ground: Option<&Ground>,
) -> Option<MovementTarget> {
    match organism.feeding_mode() {
        FeedingMode::Grazer | FeedingMode::Predator => {
            let reach = GRAZE_RANGE + organism.body.reach();
            let sight = sight_range(organism, reach, ground);
            if let (Tier::Near, Some(ground)) = (organism.tier, ground) {
                policy_living(
                    organism,
                    nearby_indexes(living_cells, organism.position, sight)
                        .filter_map(|order| living.get(order).map(|target| (order, target))),
                    ground,
                    sight,
                )
            } else {
                organism.last_fauna_decision = None;
                preferred_living(organism, living.iter().enumerate(), ground, sight)
                    .map(|(id, at)| MovementTarget::Seen(id, at))
            }
        }
        FeedingMode::Scavenger => {
            organism.last_fauna_decision = None;
            if organism.tier == Tier::Near && ground.is_some() {
                preferred_carrion(
                    organism,
                    nearby_indexes(carrion_cells, organism.position, DECOMPOSE_RANGE)
                        .filter_map(|order| carrion.get(order).map(|target| (order, target))),
                    ground,
                )
                .map(MovementTarget::Other)
            } else {
                preferred_carrion(organism, carrion.iter().enumerate(), ground)
                    .map(MovementTarget::Other)
            }
        }
        FeedingMode::Producer => {
            organism.last_fauna_decision = None;
            None
        }
    }
}

fn remembered_target(
    organism: &mut Organism,
    living: &[LivingTarget],
    ground: Option<&Ground>,
) -> Option<[i32; 3]> {
    if organism.tier != Tier::Near || ground.is_none() {
        // A place-tier transition changes the embodied perception model. Do
        // not revive an old local sighting if this body later returns near.
        organism.last_seen = None;
        return None;
    }
    let memory = organism.last_seen?;
    if memory.ticks_left == 0 || !living.iter().any(|(id, ..)| *id == memory.target) {
        organism.last_seen = None;
        return None;
    }
    organism.last_seen = Some(LastSeen {
        ticks_left: memory.ticks_left - 1,
        ..memory
    });
    Some(memory.position)
}

/// Moves an organism toward the affordance it currently needs. Near bodies
/// move by legal integer steps; far bodies traverse one place-graph edge.
#[allow(clippy::too_many_arguments)]
pub(super) fn disperse(
    organism: &mut Organism,
    places: &Places,
    ground: Option<&Ground>,
    rng: &mut Rng,
    living: &[LivingTarget],
    living_cells: &Cells,
    carrion: &[CarrionTarget],
    carrion_cells: &Cells,
    events: &mut Vec<Event>,
) -> bool {
    let target = preferred_target(
        organism,
        living,
        living_cells,
        carrion,
        carrion_cells,
        ground,
    );
    let (target, pursuing_memory) = match target {
        Some(MovementTarget::Seen(id, at)) => {
            organism.last_seen = Some(LastSeen {
                target: id,
                position: at,
                ticks_left: MEMORY_TICKS,
            });
            (Some(at), false)
        }
        Some(MovementTarget::Avoid(id, at)) => {
            organism.last_seen = Some(LastSeen {
                target: id,
                position: at,
                ticks_left: MEMORY_TICKS,
            });
            (
                Some([
                    organism.position[0] + (organism.position[0] - at[0]),
                    organism.position[1],
                    organism.position[2] + (organism.position[2] - at[2]),
                ]),
                false,
            )
        }
        Some(MovementTarget::Hold(id, at)) => {
            organism.last_seen = Some(LastSeen {
                target: id,
                position: at,
                ticks_left: MEMORY_TICKS,
            });
            (None, false)
        }
        Some(MovementTarget::Other(at)) => (Some(at), false),
        None => (remembered_target(organism, living, ground), true),
    };
    let old = organism.position;
    let next = if let Some(target) = target {
        if organism.tier == Tier::Far {
            let next = graph_step(places, organism.position, target);
            ground
                .and_then(|ground| surface_stance(ground, next))
                .unwrap_or(next)
        } else if let Some(ground) = ground {
            let mut at = organism.position;
            for _ in 0..dispersal_for(organism) {
                let next = if pursuing_memory {
                    route_step(ground, at, target, MEMORY_ROUTE_BUDGET)
                        .unwrap_or_else(|| grounded_step(ground, at, target))
                } else {
                    grounded_step(ground, at, target)
                };
                if next == at {
                    break;
                }
                at = next;
                if chebyshev(at, target) <= organism.body.reach() + GRAZE_RANGE {
                    break;
                }
            }
            at
        } else {
            let mut at = organism.position;
            for _ in 0..dispersal_for(organism) {
                at = integer_step(at, target);
                if chebyshev(at, target) <= organism.body.reach() + GRAZE_RANGE {
                    break;
                }
            }
            at
        }
    } else if organism.energy_mg == 0 {
        if organism.tier == Tier::Far {
            let next = diffuse(places, organism.position, rng);
            ground
                .and_then(|ground| surface_stance(ground, next))
                .unwrap_or(next)
        } else if let Some(ground) = ground {
            const WANDER: [[i32; 2]; 4] = [[1, 0], [-1, 0], [0, 1], [0, -1]];
            let [dx, dz] = WANDER[rng.below(WANDER.len() as u64) as usize];
            grounded_step(
                ground,
                organism.position,
                [
                    organism.position[0] + dx,
                    organism.position[1],
                    organism.position[2] + dz,
                ],
            )
        } else {
            diffuse(places, organism.position, rng)
        }
    } else {
        organism.position
    };

    if next != old {
        let distance = chebyshev(old, next) as u64;
        organism.spend_mass(distance.max(1));
        organism.position = next;
        events.push(Event::Moved {
            organism: organism.id,
            from: old,
            to: next,
        });
        true
    } else {
        false
    }
}

fn integer_step(from: [i32; 3], to: [i32; 3]) -> [i32; 3] {
    [
        from[0] + (to[0] - from[0]).signum(),
        from[1] + (to[1] - from[1]).signum(),
        from[2] + (to[2] - from[2]).signum(),
    ]
}

fn graph_step(places: &Places, position: [i32; 3], target: [i32; 3]) -> [i32; 3] {
    let Some(current) = places.at(position) else {
        return integer_step(position, target);
    };
    let Some(goal) = places.at(target) else {
        return integer_step(position, target);
    };
    let Some(next) = places
        .neighbours(current)
        .iter()
        .filter_map(|id| places.get(*id))
        .min_by_key(|place| places.hops(place.id, goal).unwrap_or(u32::MAX))
    else {
        return integer_step(position, target);
    };
    [next.centre[0], position[1], next.centre[1]]
}

fn diffuse(places: &Places, position: [i32; 3], rng: &mut Rng) -> [i32; 3] {
    let Some(current) = places.at(position) else {
        return position;
    };
    let neighbours = places.neighbours(current);
    if neighbours.is_empty() {
        return position;
    }
    let id = neighbours[rng.below(neighbours.len() as u64) as usize];
    let Some(place) = places.get(id) else {
        return position;
    };
    [place.centre[0], position[1], place.centre[1]]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::{SpeciesId, VolumeRef};

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
        let living = vec![(target, [4, 1, 0], 0, Kingdom::Producer, 300, Signal::Plain)];
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
}
