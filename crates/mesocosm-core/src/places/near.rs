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

use crate::body::Aabb;

use super::Places;
use super::bricks::Ground;

/// How tall a walker is, in voxels, for occupancy checks.
pub const WALKER_HEIGHT: i32 = 2;
/// Body-space voxels represented by one Ground voxel for locomotion.
///
/// The primitive developmental palette makes one mass segment four body
/// voxels across. Treating that as one Ground voxel keeps a root segment at
/// the old one-column scale while still letting lateral anatomy widen the
/// passage a body needs.
pub const BODY_VOXELS_PER_GROUND_VOXEL: i32 = 4;
/// One step may climb this many voxels of ledge.
pub const CLIMB: i32 = 1;
/// A fall taller than this is not walked off voluntarily by prey-seekers.
pub const COMFORT_DROP: i32 = 4;

/// The turning cross-section a body presents to Ground.
///
/// Movement does not yet retain facing, so the shorter horizontal axis is the
/// honest passage width: an elongated body may align itself with a corridor,
/// but broad anatomy still cannot. Height and width are derived readings of
/// [`Aabb`], never a second body shape stored on an organism.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WalkerShape {
    radius: i32,
    height: i32,
}

impl WalkerShape {
    pub const STANDARD: Self = Self {
        radius: 0,
        height: WALKER_HEIGHT,
    };

    pub fn from_aabb(box_: Aabb) -> Self {
        let extent = box_.extent().map(i32::abs);
        let cross_section = extent[0].min(extent[2]);
        let scale = BODY_VOXELS_PER_GROUND_VOXEL;
        let radius = cross_section
            .saturating_sub(scale)
            .saturating_add(2 * scale - 1)
            / (2 * scale);
        let height = extent[1].saturating_add(scale - 1).div_euclid(scale).max(1);
        Self { radius, height }
    }

    pub fn radius(self) -> i32 {
        self.radius
    }

    pub fn height(self) -> i32 {
        self.height
    }

    /// The top-centre point this body uses for terrain sight.
    ///
    /// A one-voxel body keeps the established head-height ray one voxel above
    /// its stance. Anatomy taller than the old two-voxel walker lifts the ray;
    /// the visual horizon itself does not grow.
    pub fn sight_point(self, stance: [i32; 3]) -> [i32; 3] {
        [
            stance[0],
            stance[1] + self.height.max(WALKER_HEIGHT) - 1,
            stance[2],
        ]
    }

    /// Whether the whole cross-section is clear and at least one column has
    /// footing. Wide bodies may bridge a small hollow; they may not intersect
    /// its rim.
    pub fn stands(self, ground: &Ground, at: [i32; 3]) -> bool {
        self.clear(ground, at) && self.supported(ground, at)
    }

    fn clear(self, ground: &Ground, at: [i32; 3]) -> bool {
        (-self.radius..=self.radius).all(|dx| {
            (-self.radius..=self.radius).all(|dz| {
                (0..self.height).all(|dy| !ground.solid([at[0] + dx, at[1] + dy, at[2] + dz]))
            })
        })
    }

    fn supported(self, ground: &Ground, at: [i32; 3]) -> bool {
        (-self.radius..=self.radius).any(|dx| {
            (-self.radius..=self.radius).any(|dz| ground.solid([at[0] + dx, at[1] - 1, at[2] + dz]))
        })
    }
}

/// One kinematic step toward `toward`: slide on block, climb one ledge,
/// then settle under gravity. Returns where the walker ends up; the
/// result is always a standable voxel, and never more than one voxel of
/// horizontal travel from `from`.
pub fn step(ground: &Ground, from: [i32; 3], toward: [i32; 3]) -> [i32; 3] {
    step_for(ground, WalkerShape::STANDARD, from, toward)
}

/// [`step`] with occupancy derived from the body that is moving.
pub fn step_for(ground: &Ground, shape: WalkerShape, from: [i32; 3], toward: [i32; 3]) -> [i32; 3] {
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
            if shape.stands(ground, at) {
                return settle(ground, shape, at);
            }
        }
        // The doorway case: air ahead but no floor (a shaft, a burrow
        // mouth). Walk in and let gravity decide, if the drop is short
        // or the quarry is below. A deeper drop is remembered: comfort
        // orders choices, and when the drop is the only way off a cliff
        // edge, standing there forever is not an option.
        let ahead = [target[0], from[1], target[2]];
        if !solid_span(ground, shape, ahead) {
            let landing = settle(ground, shape, ahead);
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
    settle(ground, shape, from)
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
    route_step_for(ground, WalkerShape::STANDARD, from, target, budget)
}

/// [`route_step`] over the transition relation of one body's cross-section.
pub fn route_step_for(
    ground: &Ground,
    shape: WalkerShape,
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
            let next = step_for(ground, shape, at, [at[0] + dx, at[1], at[2] + dz]);
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

fn solid_span(ground: &Ground, shape: WalkerShape, at: [i32; 3]) -> bool {
    !shape.clear(ground, at)
}

/// Fall until footing. Bedrock guarantees termination.
fn settle(ground: &Ground, shape: WalkerShape, at: [i32; 3]) -> [i32; 3] {
    let mut here = at;
    while !shape.supported(ground, here) && here[1] > 0 {
        let below = [here[0], here[1] - 1, here[2]];
        if !shape.clear(ground, below) {
            break;
        }
        here[1] -= 1;
    }
    here
}

/// Places a body on the highest surface under its cross-section.
///
/// This is used when a graph-level position becomes embodied. It does not
/// search for a route or alter Ground; it only realizes that x/z position as
/// a legal stance when its footprint overlaps resident terrain.
pub fn surface_stance_for(
    ground: &Ground,
    shape: WalkerShape,
    position: [i32; 3],
) -> Option<[i32; 3]> {
    let mut surface = None;
    for dx in -shape.radius..=shape.radius {
        for dz in -shape.radius..=shape.radius {
            if let Some(column) = ground.surface(position[0] + dx, position[2] + dz) {
                surface = Some(surface.map_or(column, |top: i32| top.max(column)));
            }
        }
    }
    let at = [position[0], surface? + 1, position[2]];
    shape.stands(ground, at).then_some(at)
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
    spot_for(
        ground,
        WalkerShape::STANDARD,
        eye,
        WalkerShape::STANDARD,
        target,
        range,
    )
}

/// [`spot`] between sight points derived from both live bodies.
///
/// Range remains stance-to-stance. Becoming taller changes which terrain
/// occludes a body, rather than granting a longer visual horizon.
pub fn spot_for(
    ground: &Ground,
    observer_shape: WalkerShape,
    observer: [i32; 3],
    target_shape: WalkerShape,
    target: [i32; 3],
    range: i32,
) -> bool {
    let dx = (observer[0] - target[0]).abs();
    let dy = (observer[1] - target[1]).abs();
    let dz = (observer[2] - target[2]).abs();
    if dx.max(dy).max(dz) > range {
        return false;
    }
    ground.sees(
        observer_shape.sight_point(observer),
        target_shape.sight_point(target),
    )
}

#[cfg(test)]
mod tests {
    use super::super::{Places, SURFACE_BAND};
    use super::*;
    use crate::body::{Attachment, BodyDocument, Provenance, SpeciesId, VolumeRef, Yaw};
    use crate::world::{ENCLOSURE, PLACE_SALT, PLACE_SIDE};

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
    fn live_anatomy_decides_the_turning_cross_section() {
        let mut body = BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 100, [1, 1, 1]);
        let compact = WalkerShape::from_aabb(body.aabb());
        assert_eq!((compact.radius(), compact.height()), (0, 1));

        let plate = body
            .attach(
                VolumeRef::from_tag(2),
                50,
                [3, 1, 3],
                Attachment {
                    parent: body.root,
                    offset: [0, 2, 0],
                    yaw: Yaw::Zero,
                },
                Provenance::founding(),
            )
            .unwrap();
        let broad = WalkerShape::from_aabb(body.aabb());
        assert_eq!(broad.radius(), 1);

        assert_eq!(body.sever(plate), vec![plate]);
        assert_eq!(WalkerShape::from_aabb(body.aabb()), compact);

        let stalk = body
            .attach(
                VolumeRef::from_tag(3),
                1,
                [1, 5, 1],
                Attachment {
                    parent: body.root,
                    offset: [0, 6, 0],
                    yaw: Yaw::Zero,
                },
                Provenance::founding(),
            )
            .unwrap();
        let tall = WalkerShape::from_aabb(body.aabb());
        assert_eq!((tall.radius(), tall.height()), (0, 3));
        assert_eq!(compact.sight_point([4, 7, 9]), [4, 8, 9]);
        assert_eq!(tall.sight_point([4, 7, 9]), [4, 9, 9]);

        assert_eq!(body.sever(stalk), vec![stalk]);
        assert_eq!(WalkerShape::from_aabb(body.aabb()), compact);
    }

    #[test]
    fn one_generated_burrow_admits_a_compact_body_and_excludes_a_broad_one() {
        let grown = Places::grown(PLACE_SALT, PLACE_SIDE, ENCLOSURE);
        let ground = Ground::grow(&grown, ENCLOSURE);
        let route = grown
            .nest_entries(ENCLOSURE)
            .next()
            .expect("seed 0 grows a burrow entry")
            .1
            .route;
        let compact = WalkerShape::STANDARD;
        let broad = WalkerShape::from_aabb(Aabb {
            min: [-3, -2, -3],
            max: [3, 2, 3],
        });
        let threshold = route[1];

        assert!(compact.stands(&ground, threshold));
        assert_eq!(step_for(&ground, compact, route[0], threshold), threshold);
        assert_eq!(broad.radius(), 1);
        assert!(
            !broad.stands(&ground, threshold),
            "the generated one-voxel threshold unexpectedly admits a broad body"
        );
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
