// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The hunter: the wing's first embodied antagonist.
//!
//! A three-state mind over the near tier's queries: search until seen,
//! stalk what is seen, and when sight breaks, go to where the quarry
//! was last known before giving up. Nothing here reads the ecology or
//! the world; a hunter is position, memory, and the ground's answers,
//! which is what makes it portable to every vessel that has all three.

use crate::rng::Rng;

use super::bricks::Ground;
use super::near::{spot, step};

/// How far a hunter can see, in voxels.
pub const SIGHT: i32 = 24;
/// Ticks of blindness before a stalk decays into a search of the last
/// known position, and ticks of fruitless memory before giving up.
pub const PATIENCE: u32 = 90;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mind {
    /// No quarry: wander, deterministically.
    Search,
    /// Quarry in sight, closing.
    Stalk,
    /// Sight broken: heading for where it was, counting down.
    Memory { since: u32 },
}

/// One hunter. Advisor-tier state: positions are integer facts, the
/// mind is three variants, and every transition is caused by a query.
#[derive(Clone, Debug)]
pub struct Hunter {
    pub at: [i32; 3],
    pub mind: Mind,
    pub last_seen: Option<[i32; 3]>,
    /// The quarry's heading while it was visible, for extrapolation.
    heading: [i32; 2],
    rng: Rng,
    wander: [i32; 3],
}

impl Hunter {
    pub fn new(at: [i32; 3], seed: u64) -> Self {
        Self {
            at,
            mind: Mind::Search,
            last_seen: None,
            heading: [0, 0],
            rng: Rng::from_seed(seed),
            wander: at,
        }
    }

    /// One tick against the ground and the quarry's true position.
    /// Movement is at most one voxel of horizontal travel; the mind
    /// transitions are: seen -> Stalk, sight broken -> Memory, memory
    /// exhausted or arrived-with-nothing -> Search.
    pub fn tick(&mut self, ground: &Ground, quarry: [i32; 3]) {
        let seen = spot(ground, self.at, quarry, SIGHT);
        if seen {
            self.mind = Mind::Stalk;
            if let Some(prev) = self.last_seen
                && prev != quarry
            {
                self.heading = [
                    (quarry[0] - prev[0]).signum(),
                    (quarry[2] - prev[2]).signum(),
                ];
            }
            self.last_seen = Some(quarry);
        } else {
            self.mind = match self.mind {
                Mind::Stalk => {
                    // Sight just broke: project the anchor along the
                    // quarry's last heading. It was going somewhere; a
                    // predator searches where it was going, not where it
                    // stopped being visible.
                    if let Some(last) = self.last_seen {
                        self.last_seen = Some([
                            last[0] + self.heading[0] * 8,
                            last[1],
                            last[2] + self.heading[1] * 8,
                        ]);
                    }
                    Mind::Memory { since: 0 }
                }
                Mind::Memory { since } if since >= PATIENCE => {
                    self.last_seen = None;
                    Mind::Search
                }
                Mind::Memory { since } => Mind::Memory { since: since + 1 },
                Mind::Search => Mind::Search,
            };
        }

        let target = match (self.mind, self.last_seen) {
            (Mind::Stalk, Some(at)) => at,
            (Mind::Memory { .. }, Some(at)) => {
                let arrived = (self.at[0] - at[0]).abs() <= 1 && (self.at[2] - at[2]).abs() <= 1;
                if arrived {
                    // Cast around the last known position: a den mouth or
                    // a shadowed hollow is found by prowling, not by
                    // standing on the spot it was last seen.
                    let reached = (self.at[0] - self.wander[0]).abs() <= 1
                        && (self.at[2] - self.wander[2]).abs() <= 1;
                    let stale = (self.wander[0] - at[0]).abs() > 5
                        || (self.wander[2] - at[2]).abs() > 5;
                    if reached || stale {
                        self.wander = [
                            at[0] + self.rng.range_i32(-4, 4),
                            at[1],
                            at[2] + self.rng.range_i32(-4, 4),
                        ];
                    }
                    self.wander
                } else {
                    at
                }
            }
            _ => {
                // A wander target refreshed when reached or stale, drawn
                // from the hunter's own stream: deterministic restlessness.
                let arrived = (self.at[0] - self.wander[0]).abs() <= 1
                    && (self.at[2] - self.wander[2]).abs() <= 1;
                if arrived {
                    self.wander = [
                        self.at[0] + self.rng.range_i32(-12, 12),
                        self.at[1],
                        self.at[2] + self.rng.range_i32(-12, 12),
                    ];
                }
                self.wander
            }
        };
        self.at = step(ground, self.at, target);
    }
}

#[cfg(test)]
mod tests {
    use super::super::Places;
    use super::super::bricks::SURFACE_BAND;
    use super::super::near::{Tier, TierLine, WALKER_HEIGHT};
    use super::*;

    /// The G3 receipt: acquire, pursue, lose, re-acquire, and follow into
    /// a burrow, continuously, on the real ground.
    #[test]
    fn the_chase() {
        let grown = Places::grown(4_242, 4, 64);
        let mut ground = Ground::grow(&grown, 64);

        // Stage: a hillside with a bored den behind it. The prey will
        // round the hill (breaking sight) and go to ground inside.
        let mut stage = None;
        'scan: for z in -44..44 {
            for x in -44..30 {
                let (Some(a), Some(b)) = (ground.surface(x, z), ground.surface(x + 10, z))
                else {
                    continue;
                };
                let (open, behind) = ([x, a + 1, z], [x + 10, b + 1, z]);
                // Two stances the terrain hides from each other: whatever
                // stands between them is the hill.
                if ground.stands(open, WALKER_HEIGHT)
                    && ground.stands(behind, WALKER_HEIGHT)
                    && !ground.sees(
                        [open[0], open[1] + 1, open[2]],
                        [behind[0], behind[1] + 1, behind[2]],
                    )
                {
                    stage = Some((open, behind));
                    break 'scan;
                }
            }
        }
        let (open, behind) = stage.expect("a hill with two flanks");
        // The den: bored into the hill from the far flank.
        for depth in 0..4 {
            ground.carve([behind[0] - 2 - depth, behind[1], behind[2]], 1);
        }
        let den = [behind[0] - 4, behind[1], behind[2]];

        let mut hunter = Hunter::new(open, 7);
        let mut quarry = open;
        // The quarry's script: away from the hunter around the hill,
        // then into the den.
        let waypoints = [behind, den];
        let mut leg = 0;

        let mut acquired = false;
        let mut lost_after_acquire = false;
        let mut reacquired = false;
        let mut followed_in = false;
        let mut previous = hunter.at;

        for tick in 0..400 {
            // Quarry moves every tick; the hunter lumbers at half speed,
            // so quarry can actually break away around the hill.
            if leg < waypoints.len() {
                let next = step(&ground, quarry, waypoints[leg]);
                quarry = next;
                if (quarry[0] - waypoints[leg][0]).abs() <= 1
                    && (quarry[1] - waypoints[leg][1]).abs() <= 1
                    && (quarry[2] - waypoints[leg][2]).abs() <= 1
                {
                    leg += 1;
                }
            }
            let mind_before = hunter.mind;
            if tick % 2 == 0 {
                hunter.tick(&ground, quarry);
            }
            if std::env::var("CHASE_TRACE").is_ok()
                && (hunter.mind != mind_before || tick % 20 == 0)
            {
                println!(
                    "t{tick:3} hunter {:?} {:?} quarry {quarry:?} leg {leg} last {:?}",
                    hunter.at, hunter.mind, hunter.last_seen
                );
            }

            // Continuity: never more than one voxel of horizontal travel.
            assert!(
                (hunter.at[0] - previous[0]).abs() <= 1
                    && (hunter.at[2] - previous[2]).abs() <= 1,
                "hunter teleported {previous:?} -> {:?}",
                hunter.at
            );
            previous = hunter.at;

            match hunter.mind {
                Mind::Stalk if !acquired => acquired = true,
                Mind::Stalk if lost_after_acquire => reacquired = true,
                Mind::Memory { .. } if acquired => lost_after_acquire = true,
                _ => {}
            }
            // Followed in: the hunter stands in the den's carved dark,
            // at den height, not on its roof.
            if (hunter.at[0] - den[0]).abs() <= 1
                && (hunter.at[1] - den[1]).abs() <= 1
                && (hunter.at[2] - den[2]).abs() <= 2
            {
                followed_in = true;
                break;
            }
        }

        assert!(acquired, "the hunter never saw its quarry");
        assert!(
            lost_after_acquire,
            "sight never broke: hunter {:?} quarry {quarry:?} leg {leg}",
            hunter.at
        );
        if !(reacquired || followed_in) {
            panic!(
                "trail died: open {open:?} behind {behind:?} den {den:?} hunter {:?}                  mind {:?} last_seen {:?} quarry {quarry:?} leg {leg}",
                hunter.at, hunter.mind, hunter.last_seen
            );
        }
        assert!(reacquired || followed_in, "the trail died completely");
        assert!(followed_in, "the hunter never reached the den");
    }

    #[test]
    fn the_chase_is_deterministic() {
        let run = || {
            let grown = Places::grown(4_242, 4, 64);
            let ground = Ground::grow(&grown, 64);
            let mut hunter = Hunter::new([0, SURFACE_BAND, 0], 7);
            let mut trace = Vec::new();
            for tick in 0..120 {
                hunter.tick(&ground, [tick % 30 - 15, 10, 20]);
                trace.push(hunter.at);
            }
            trace
        };
        assert_eq!(run(), run());
    }

    #[test]
    #[ignore]
    fn perception_cost_at_population() {
        // The plan's number: N critters doing sight-lines per tick.
        let grown = Places::grown(4_242, 4, 64);
        let ground = Ground::grow(&grown, 64);
        let mut hunters: Vec<Hunter> = (0..300)
            .map(|i| Hunter::new([(i % 40) - 20, SURFACE_BAND, (i / 40) - 20], i as u64))
            .collect();
        let start = std::time::Instant::now();
        let ticks = 100;
        for tick in 0..ticks {
            let quarry = [tick % 40 - 20, 12, tick % 30 - 15];
            for hunter in &mut hunters {
                hunter.tick(&ground, quarry);
            }
        }
        let per_tick = start.elapsed() / ticks as u32;
        println!("300 hunters: {per_tick:?} per tick");
    }

    #[test]
    fn tiers_and_hunters_compose() {
        // A far-tier hunter is not ticked; crossing the line wakes it.
        let grown = Places::grown(4_242, 4, 64);
        let ground = Ground::grow(&grown, 64);
        let line = TierLine::default();
        let focus = [0, 12, 0];
        let mut tier = Tier::Far;
        let mut hunter = Hunter::new([50, SURFACE_BAND, 50], 3);
        for d in (0..50).rev() {
            let agent = [d, hunter.at[1], d];
            hunter.at = agent;
            tier = line.tick(&grown.places, tier, agent, focus);
            if tier == Tier::Near {
                hunter.tick(&ground, focus);
                break;
            }
        }
        assert_eq!(tier, Tier::Near, "closing distance never woke the hunter");
    }
}
