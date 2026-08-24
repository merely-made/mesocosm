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

use crate::history::Event;
use crate::organism::{Kingdom, Signal};
use crate::places::{
    Ground, Places, Tier, WalkerShape, route_step_for, step_for as grounded_step,
    surface_stance_for,
};
use crate::process::FeedingMode;
use crate::rng::Rng;

use crate::organism::{
    FaunaDecisionTrace, FaunaDrive, FaunaSenses, FaunaTraits, LastSeen, Organism, OrganismId,
};

use super::dispersal_for;

mod perception;

pub(super) use perception::{CarrionTarget, LivingTarget, carrion_cells, living_cells};
use perception::{Cells, can_perceive, can_perceive_position, nearby_indexes, sight_range};

/// How far a consumer reaches for a meal, in voxel units.
pub(super) const GRAZE_RANGE: i32 = 5;
/// How far a decomposer reaches for the dead, in voxel units.
pub(super) const DECOMPOSE_RANGE: i32 = 6;
/// A remembered sight line can take a short detour, but never authorizes a
/// global navigation search in an ecology tick.
const MEMORY_ROUTE_BUDGET: i32 = 8;
/// Direct observation is fresh for this many failed perception ticks.
const MEMORY_TICKS: u8 = 8;

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
    let observer_shape = organism.walker_shape();
    let mut candidates: Vec<(u64, usize, usize)> = Vec::new();
    for order in nearby_indexes(cells, organism.position, sight) {
        let Some(target) = living.get(order) else {
            continue;
        };
        if target.id == organism.id
            || !matches!(mode, FeedingMode::Grazer | FeedingMode::Predator)
            || (mode == FeedingMode::Grazer && target.kingdom != Kingdom::Producer)
            || chebyshev(organism.position, target.position) > reach
            || (target.signal != Signal::Plain && mode != FeedingMode::Grazer)
        {
            continue;
        }
        let distance = chebyshev(organism.position, target.position) as u64;
        let danger = u64::from(target.signal == Signal::Warning) * 4;
        let score =
            (distance.saturating_mul(16) + danger).saturating_sub(target.mass_mg.min(256) / 64);
        candidates.push((score, order, target.organism_index));
    }
    candidates.sort_unstable();
    candidates.into_iter().find_map(|(_, order, index)| {
        living
            .get(order)
            .is_some_and(|target| can_perceive(organism, observer_shape, target, sight, ground))
            .then_some(index)
    })
}

/// Carrion feeding is local too. Returning the original organism index keeps
/// the drain pass independent from the derived bucket representation.
pub(super) fn choose_carrion_target(
    organism: &Organism,
    carrion: &[CarrionTarget],
    cells: &Cells,
    ground: Option<&Ground>,
) -> Option<usize> {
    let observer_shape = organism.walker_shape();
    nearby_indexes(cells, organism.position, DECOMPOSE_RANGE)
        .filter_map(|order| carrion.get(order).map(|target| (order, target)))
        .filter(|(_, target)| {
            (0..3).all(|axis| {
                (target.position[axis] - organism.position[axis]).abs() <= DECOMPOSE_RANGE
            }) && can_perceive_position(
                organism,
                observer_shape,
                target.position,
                target.shape,
                DECOMPOSE_RANGE,
                ground,
            )
        })
        .min_by_key(|(order, _)| *order)
        .map(|(_, target)| target.organism_index)
}

fn chebyshev(from: [i32; 3], to: [i32; 3]) -> i32 {
    (0..3)
        .map(|axis| (from[axis] - to[axis]).abs())
        .max()
        .unwrap_or(0)
}

/// Finds a valid surface stance for a graph position, if it is resident in the
/// grown ground. Callers choose the fallback: a near-tier birth remains beside
/// its grounded parent when its scatter leaves the grown enclosure.
pub(super) fn surface_stance(
    ground: &Ground,
    shape: WalkerShape,
    position: [i32; 3],
) -> Option<[i32; 3]> {
    surface_stance_for(ground, shape, position)
}

fn preferred_living<'a>(
    organism: &Organism,
    candidates: impl Iterator<Item = (usize, &'a LivingTarget)>,
    ground: Option<&Ground>,
    sight: i32,
) -> Option<(OrganismId, [i32; 3])> {
    let observer_shape = organism.walker_shape();
    let mut ranked: Vec<(i32, u64, usize, &'a LivingTarget)> = candidates
        .filter(|(_, target)| {
            target.id != organism.id
                && (organism.feeding_mode() == FeedingMode::Predator
                    || target.kingdom == Kingdom::Producer)
        })
        .map(|(order, target)| {
            (
                chebyshev(organism.position, target.position),
                target.mass_mg,
                order,
                target,
            )
        })
        .collect();
    ranked.sort_unstable_by_key(|(distance, mass, order, _)| (*distance, Reverse(*mass), *order));
    ranked.into_iter().find_map(|(_, _, _, target)| {
        can_perceive(organism, observer_shape, target, sight, ground)
            .then_some((target.id, target.position))
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
    let observer_shape = organism.walker_shape();
    let candidate = candidates
        .filter(|(_, target)| {
            target.id != organism.id
                && (traits.feeding_mode == FeedingMode::Predator
                    || target.kingdom == Kingdom::Producer)
        })
        .filter_map(|(order, target)| {
            let distance = chebyshev(organism.position, target.position);
            if !can_perceive(organism, observer_shape, target, sight, Some(ground)) {
                return None;
            }
            let senses = FaunaSenses::read(
                organism,
                traits,
                target.id,
                distance,
                target.mass_mg,
                target.signal,
            );
            let scores = policy.score(senses, own_mass, sight);
            let drive = scores.selected();
            let rank = (
                scores.score(drive),
                Reverse(distance),
                target.mass_mg,
                Reverse(order),
            );
            Some((rank, target, senses, scores, drive))
        })
        .max_by_key(|(rank, ..)| *rank);

    let Some((_, target, senses, scores, drive)) = candidate else {
        organism.last_fauna_decision = None;
        return None;
    };
    organism.fauna_policy.remember(scores);
    organism.last_fauna_decision = Some(FaunaDecisionTrace {
        traits,
        senses,
        selected_drive: drive,
        selected_target: Some(target.id),
        scores,
    });
    Some(match drive {
        FaunaDrive::Pursue => MovementTarget::Seen(target.id, target.position),
        FaunaDrive::Avoid => MovementTarget::Avoid(target.id, target.position),
        FaunaDrive::Hold => MovementTarget::Hold(target.id, target.position),
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
    let observer_shape = organism.walker_shape();
    let mut ranked: Vec<(i32, usize, &'a CarrionTarget)> = candidates
        .map(|(order, target)| (chebyshev(organism.position, target.position), order, target))
        .collect();
    ranked.sort_unstable_by_key(|(distance, order, _)| (*distance, *order));
    ranked.into_iter().find_map(|(_, _, target)| {
        can_perceive_position(
            organism,
            observer_shape,
            target.position,
            target.shape,
            DECOMPOSE_RANGE,
            ground,
        )
        .then_some(target.position)
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
    if memory.ticks_left == 0 || !living.iter().any(|target| target.id == memory.target) {
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
    let shape = organism.walker_shape();
    let next = if let Some(target) = target {
        if organism.tier == Tier::Far {
            let next = graph_step(places, organism.position, target);
            ground
                .and_then(|ground| surface_stance(ground, shape, next))
                .unwrap_or(next)
        } else if let Some(ground) = ground {
            let mut at = organism.position;
            for _ in 0..dispersal_for(organism) {
                let next = if pursuing_memory {
                    route_step_for(ground, shape, at, target, MEMORY_ROUTE_BUDGET)
                        .unwrap_or_else(|| grounded_step(ground, shape, at, target))
                } else {
                    grounded_step(ground, shape, at, target)
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
                .and_then(|ground| surface_stance(ground, shape, next))
                .unwrap_or(next)
        } else if let Some(ground) = ground {
            const WANDER: [[i32; 2]; 4] = [[1, 0], [-1, 0], [0, 1], [0, -1]];
            let [dx, dz] = WANDER[rng.below(WANDER.len() as u64) as usize];
            grounded_step(
                ground,
                shape,
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
    use crate::organism::{Kingdom, Signal};

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
}
