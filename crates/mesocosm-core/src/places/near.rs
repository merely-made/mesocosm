// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The near tier's floor: kinematic movement over brick truth, and the
//! tier boundary itself.
//!
//! Advisor-tier locomotion per plan ruling §0.10: owned move-and-slide
//! over [`Ground`] occupancy, integer in and integer out, no physics
//! engine and nothing persisted. Rain World's lesson is that physicality
//! is mostly *constraint*: walls stop you, ledges are climbed, gravity
//! is a fact. All of that is queries against the one truth.
//!
//! The tier boundary lives here too because it is a place-graph fact:
//! places near the played body run embodied agents, far places run the
//! statistical ecology, and the line between them moves with hysteresis
//! so a critter pacing the border does not flicker between minds.

use std::collections::{BTreeMap, VecDeque};

use serde::{Deserialize, Serialize};

use super::Places;
use super::bricks::Ground;

/// How tall a walker is, in voxels, for occupancy checks.
pub const WALKER_HEIGHT: i32 = 2;
/// One step may climb this many voxels of ledge.
pub const CLIMB: i32 = 1;
/// A fall taller than this is not walked off voluntarily by prey-seekers.
pub const COMFORT_DROP: i32 = 4;

/// One kinematic step toward `toward`: slide on block, climb one ledge,
/// then settle under gravity. Returns where the walker ends up; the
/// result is always a standable voxel, and never more than one voxel of
/// horizontal travel from `from`.
pub fn step(ground: &Ground, from: [i32; 3], toward: [i32; 3]) -> [i32; 3] {
    let want = [
        (toward[0] - from[0]).signum(),
        (toward[2] - from[2]).signum(),
    ];
    let vertical = (toward[1] - from[1]).signum();

    // Try the full move, then each axis alone: the slide.
    let tries = [[want[0], want[1]], [want[0], 0], [0, want[1]]];
    let mut forced_drop = None;
    for try_move in tries {
        if try_move == [0, 0] {
            continue;
        }
        let target = [from[0] + try_move[0], from[1], from[2] + try_move[1]];
        // Prefer lifts that close the vertical gap, but preference is an
        // ordering, never a refusal: descending the far side of a hill
        // requires climbing the near side first.
        let lifts = if vertical < 0 {
            [0, -1, CLIMB]
        } else {
            [0, CLIMB, -1]
        };
        for lift in lifts {
            let at = [target[0], from[1] + lift, target[2]];
            if ground.stands(at, WALKER_HEIGHT) {
                return settle(ground, at);
            }
        }
        // The doorway case: air ahead but no floor (a shaft, a burrow
        // mouth). Walk in and let gravity decide, if the drop is short
        // or the quarry is below. A deeper drop is remembered: comfort
        // orders choices, and when the drop is the only way off a cliff
        // edge, standing there forever is not an option.
        let ahead = [target[0], from[1], target[2]];
        if !solid_span(ground, ahead) {
            let landing = settle(ground, ahead);
            if from[1] - landing[1] <= COMFORT_DROP || vertical < 0 {
                return landing;
            }
            if forced_drop.is_none() {
                forced_drop = Some(landing);
            }
        }
    }
    if let Some(landing) = forced_drop {
        return landing;
    }
    // Boxed in horizontally; at least settle where we are.
    settle(ground, from)
}

/// The first legal step on a bounded route to `target`. It expands the same
/// owned [`step`] transitions the player and ecology use, so routing cannot
/// invent a second collision representation. `None` means the target lies
/// outside the local budget or no route was found there.
pub fn route_step(
    ground: &Ground,
    from: [i32; 3],
    target: [i32; 3],
    budget: i32,
) -> Option<[i32; 3]> {
    if from == target || budget < 1 || chebyshev(from, target) > budget {
        return None;
    }
    const DIRECTIONS: [[i32; 2]; 4] = [[1, 0], [-1, 0], [0, 1], [0, -1]];
    const MAX_ROUTE_STANCES: usize = 256;
    let mut frontier = VecDeque::from([from]);
    let mut previous = BTreeMap::from([(from, from)]);
    while let Some(at) = frontier.pop_front() {
        for [dx, dz] in DIRECTIONS {
            let next = step(ground, at, [at[0] + dx, at[1], at[2] + dz]);
            if next == at || chebyshev(from, next) > budget || previous.contains_key(&next) {
                continue;
            }
            if previous.len() >= MAX_ROUTE_STANCES {
                return None;
            }
            previous.insert(next, at);
            if next == target {
                let mut first = target;
                while previous[&first] != from {
                    first = previous[&first];
                }
                return Some(first);
            }
            frontier.push_back(next);
        }
    }
    None
}

fn chebyshev(a: [i32; 3], b: [i32; 3]) -> i32 {
    (0..3)
        .map(|axis| (a[axis] - b[axis]).abs())
        .max()
        .unwrap_or(0)
}

fn solid_span(ground: &Ground, at: [i32; 3]) -> bool {
    (0..WALKER_HEIGHT).any(|dy| ground.solid([at[0], at[1] + dy, at[2]]))
}

/// Fall until footing. Bedrock guarantees termination.
fn settle(ground: &Ground, at: [i32; 3]) -> [i32; 3] {
    let mut here = at;
    while !ground.solid([here[0], here[1] - 1, here[2]]) && here[1] > 0 {
        here[1] -= 1;
    }
    here
}

/// Which mind runs an agent: embodied, or the statistical ecology.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    #[default]
    Near,
    Far,
}

/// The tier boundary with hysteresis. Promote when an agent's place
/// comes within `promote_hops` of the focus; demote only past
/// `demote_hops`. The band between is memory.
#[derive(Clone, Copy, Debug)]
pub struct TierLine {
    pub promote_hops: u32,
    pub demote_hops: u32,
}

impl Default for TierLine {
    fn default() -> Self {
        Self {
            promote_hops: 1,
            // The standard enclosure is a 3x3 graph with diameter two. A
            // threshold of three would make the far tier unreachable in the
            // shipped world, so the outer ring is the demotion boundary.
            demote_hops: 2,
        }
    }
}

impl TierLine {
    /// The next tier, given where the agent and the focus are.
    pub fn tick(&self, places: &Places, current: Tier, agent: [i32; 3], focus: [i32; 3]) -> Tier {
        let (Some(a), Some(f)) = (places.at(agent), places.at(focus)) else {
            return current;
        };
        let hops = places.hops(a, f).unwrap_or(u32::MAX);
        match current {
            Tier::Far if hops <= self.promote_hops => Tier::Near,
            Tier::Near if hops >= self.demote_hops => Tier::Far,
            _ => current,
        }
    }
}

/// Whether `eye` can see `target`: distance-capped line of sight from
/// head height to head height.
pub fn spot(ground: &Ground, eye: [i32; 3], target: [i32; 3], range: i32) -> bool {
    let dx = (eye[0] - target[0]).abs();
    let dy = (eye[1] - target[1]).abs();
    let dz = (eye[2] - target[2]).abs();
    if dx.max(dy).max(dz) > range {
        return false;
    }
    ground.sees(
        [eye[0], eye[1] + 1, eye[2]],
        [target[0], target[1] + 1, target[2]],
    )
}

#[cfg(test)]
mod tests {
    use super::super::{Places, SURFACE_BAND};
    use super::*;

    fn ground() -> Ground {
        let grown = Places::grown(4_242, 4, 64);
        Ground::grow(&grown, 64)
    }

    fn a_stance(ground: &Ground) -> [i32; 3] {
        for z in -40..40 {
            for x in -40..40 {
                if let Some(top) = ground.surface(x, z) {
                    let at = [x, top + 1, z];
                    if ground.stands(at, WALKER_HEIGHT) {
                        return at;
                    }
                }
            }
        }
        unreachable!("a world with no footing");
    }

    #[test]
    fn a_step_never_ends_inside_rock_and_never_teleports() {
        let ground = ground();
        let mut at = a_stance(&ground);
        // March hard toward a far corner; every step must stay legal.
        for _ in 0..120 {
            let next = step(&ground, at, [60, at[1], 60]);
            assert!(
                (next[0] - at[0]).abs() <= 1 && (next[2] - at[2]).abs() <= 1,
                "teleported: {at:?} -> {next:?}"
            );
            assert!(
                ground.stands(next, WALKER_HEIGHT),
                "ended unstandable at {next:?}"
            );
            at = next;
        }
    }

    #[test]
    fn walls_slide_and_ledges_climb() {
        let ground = ground();
        let start = a_stance(&ground);
        // Wherever the terrain blocks a heading, the step still makes
        // progress or holds; across a long march it must cover ground.
        let mut at = start;
        for _ in 0..80 {
            at = step(&ground, at, [start[0] + 50, at[1], start[2]]);
        }
        assert!(
            (at[0] - start[0]).abs() + (at[2] - start[2]).abs() > 10,
            "eighty steps went nowhere: {start:?} -> {at:?}"
        );
    }

    #[test]
    fn a_bounded_route_uses_legal_steps_around_generated_ground() {
        let mut ground = ground();
        // An authoritative L-shaped bore is the smallest turning interior the
        // player can make with the same carve primitive the run records.
        for [x, z] in [[0, 0], [4, 0], [4, 4]] {
            let top = ground.surface(x, z).unwrap();
            assert!(ground.carve([x, top, z], 1) > 0);
        }
        let mut stances = Vec::new();
        for z in -2..=6 {
            for x in -2..=6 {
                for y in 1..SURFACE_BAND {
                    let at = [x, y, z];
                    if ground.stands(at, WALKER_HEIGHT) {
                        stances.push(at);
                    }
                }
            }
        }
        for from in &stances {
            for target in &stances {
                let mut greedy = *from;
                for _ in 0..16 {
                    let next = step(&ground, greedy, *target);
                    if next == greedy {
                        break;
                    }
                    greedy = next;
                }
                if greedy == *target {
                    continue;
                }
                let mut routed = *from;
                for _ in 0..16 {
                    let Some(next) = route_step(&ground, routed, *target, 8) else {
                        break;
                    };
                    assert_eq!(step(&ground, routed, next), next);
                    routed = next;
                    if routed == *target {
                        return;
                    }
                }
                if routed != *target {
                    continue;
                }
            }
        }
        panic!("seeded L-shaped bore offered no bounded detour");
    }

    #[test]
    fn the_tier_line_does_not_flap() {
        let grown = Places::grown(4_242, 4, 64);
        let ground = Ground::grow(&grown, 64);
        let line = TierLine::default();
        let focus = a_stance(&ground);
        // Walk an agent straight out and back across the band; count
        // transitions. Hysteresis admits at most one each way.
        let mut tier = Tier::Near;
        let mut flips = 0;
        let mut previous = tier;
        for leg in 0..2 {
            for step_i in 0..60 {
                let d = if leg == 0 { step_i } else { 60 - step_i };
                let agent = [focus[0] + d, focus[1], focus[2]];
                tier = line.tick(&grown.places, tier, agent, focus);
                if tier != previous {
                    flips += 1;
                    previous = tier;
                }
            }
        }
        assert!(flips <= 2, "tier flapped {flips} times");
    }

    #[test]
    fn spotting_respects_walls_and_range() {
        let ground = ground();
        let at = a_stance(&ground);
        assert!(spot(&ground, at, at, 20), "you can see where you stand");
        assert!(
            !spot(&ground, at, [at[0] + 200, at[1], at[2]], 20),
            "range caps sight"
        );
    }
}
