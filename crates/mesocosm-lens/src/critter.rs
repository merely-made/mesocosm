// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A critter as a chain, moved the Rain World way.
//!
//! Follow-the-leader constraint chains: the head goes where it is going, and
//! every segment behind swings to stay a fixed spacing from the one ahead.
//! That single rule is most of what makes Rain World's bodies read as alive,
//! and it needs no physics engine: it is a handful of normalised subtractions
//! per step. Undulation rides on top as a lateral sine along the chain,
//! scaled by speed, so a moving body writhes and a still one settles.
//!
//! Presentation-side f32 by design, like every chain here will be: the core
//! stays integer, and a body's *pose* is derived per frame from where the
//! simulation says the critter is. The full game drives chains from anatomy
//! (and seiche remains the named engine if chains ever need real dynamics);
//! this probe drives one from a seeded wander, because its question is
//! visual: does a capsule chain read as an animal.

/// One segment: a centre and a radius. The shader draws capsules between
/// consecutive centres and smooth-unions the lot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Segment {
    pub at: [f32; 3],
    pub radius: f32,
}

/// A chain body: head first, tail last.
#[derive(Clone, Debug)]
pub struct Chain {
    pub segments: Vec<Segment>,
    /// Distance each segment keeps from the one ahead.
    spacing: f32,
    /// Where the head was last step, for speed.
    previous_head: [f32; 3],
}

impl Chain {
    /// A tapered body: a slightly bulbous head, a thick middle, a thin tail.
    /// The taper is most of the difference between an animal and a worm.
    pub fn tapered(length: usize, scale: f32) -> Self {
        let radii = |i: usize| -> f32 {
            let t = i as f32 / (length.max(2) - 1) as f32;
            // Head bulge, shoulder dip, belly, tail taper.
            let profile = 0.85 + 0.35 * (1.0 - t) * (t * 6.0).min(1.0)
                - 0.55 * (t - 0.55).max(0.0) / 0.45;
            scale * profile.max(0.25)
        };
        // Spacing at twice the scale keeps consecutive capsules tangent
        // rather than coincident: an elongated body, not a blob.
        let segments = (0..length)
            .map(|i| Segment { at: [0.0, 0.0, i as f32 * scale * 2.0], radius: radii(i) })
            .collect();
        Self { segments, spacing: scale * 2.0, previous_head: [0.0, 0.0, 0.0] }
    }

    /// Moves the head to `target` and swings the body after it.
    ///
    /// `ground` supplies terrain height at (x, z); segments ride above it by
    /// their radius, so the body drapes over relief instead of tunnelling.
    pub fn step(&mut self, target: [f32; 3], ground: impl Fn(f32, f32) -> f32) {
        self.previous_head = self.segments[0].at;
        self.segments[0].at = target;

        let speed = distance(target, self.previous_head);

        for i in 1..self.segments.len() {
            let ahead = self.segments[i - 1].at;
            let here = self.segments[i].at;
            let d = distance(here, ahead);
            let mut toward = if d > 1e-5 {
                [(here[0] - ahead[0]) / d, (here[1] - ahead[1]) / d, (here[2] - ahead[2]) / d]
            } else {
                [0.0, 0.0, 1.0]
            };

            // Relax toward the leader's own heading, so a still body slowly
            // straightens instead of freezing its last wiggle, and a moving
            // one carries a flowing curve rather than a kink.
            if i >= 2 {
                let ahead2 = self.segments[i - 2].at;
                let lead = distance(ahead, ahead2);
                if lead > 1e-5 {
                    let straighten = 0.18;
                    for axis in 0..3 {
                        toward[axis] = toward[axis] * (1.0 - straighten)
                            + (ahead[axis] - ahead2[axis]) / lead * straighten;
                    }
                    let n = (toward[0] * toward[0] + toward[1] * toward[1] + toward[2] * toward[2])
                        .sqrt()
                        .max(1e-5);
                    for axis in &mut toward {
                        *axis /= n;
                    }
                }
            }
            let mut at = [
                ahead[0] + toward[0] * self.spacing,
                ahead[1] + toward[1] * self.spacing,
                ahead[2] + toward[2] * self.spacing,
            ];

            // Undulation: a travelling wave along the chain, perpendicular to
            // the segment's own direction, fading with stillness. This is the
            // gait; there is no other animation.
            let phase = i as f32 * 0.9;
            let wave = (phase + self.wave_clock(target)) .sin()
                * (speed * 1.4).min(1.0)
                * self.spacing
                * 0.35;
            at[0] += -toward[2] * wave;
            at[2] += toward[0] * wave;

            // Drape over the terrain, then re-assert spacing: the drape can
            // yank a segment down a slope, and a chain that stretches stops
            // being a body. Spacing wins over ground contact on a cliff,
            // which reads as a body bridging rather than one tearing.
            let floor = ground(at[0], at[2]) + self.segments[i].radius;
            at[1] = at[1].max(floor) * 0.35 + floor * 0.65;

            let stretched = distance(at, ahead);
            if stretched > 1e-5 {
                for axis in 0..3 {
                    at[axis] = ahead[axis] + (at[axis] - ahead[axis]) / stretched * self.spacing;
                }
            }

            self.segments[i].at = at;
        }
    }

    /// A phase clock derived from where the head is, so the gait is a pure
    /// function of position: replaying a path replays the wriggle.
    fn wave_clock(&self, head: [f32; 3]) -> f32 {
        (head[0] + head[2]) * 0.55
    }

    /// The bounding sphere the shader uses to skip rays that miss.
    pub fn bounds(&self) -> ([f32; 3], f32) {
        let mut centre = [0.0f32; 3];
        for segment in &self.segments {
            for (axis, value) in centre.iter_mut().enumerate() {
                *value += segment.at[axis];
            }
        }
        let n = self.segments.len() as f32;
        for axis in &mut centre {
            *axis /= n;
        }
        let radius = self
            .segments
            .iter()
            .map(|s| distance(s.at, centre) + s.radius)
            .fold(0.0, f32::max);
        (centre, radius + 1.0)
    }
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// A seeded wander over the map: a slow arc with a side-to-side hunt, the
/// path a foraging critter walks. Pure function of the step index.
pub fn wander(seed: u64, origin: [f32; 2], step: usize) -> [f32; 2] {
    let t = step as f32 * 0.55;
    let drift = (seed % 977) as f32 * 0.01;
    // Turn radii wider than a body length, or the chain laps itself and a
    // loper reads as a coiler.
    [
        origin[0] + t * 2.4 + (t * 0.13 + drift).sin() * 10.0,
        origin[1] + t * 1.7 + (t * 0.09 + drift).cos() * 13.0,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(_: f32, _: f32) -> f32 {
        0.0
    }

    fn walked(steps: usize) -> Chain {
        let mut chain = Chain::tapered(8, 1.5);
        for step in 0..steps {
            let [x, z] = wander(7, [0.0, 0.0], step);
            chain.step([x, flat(x, z) + 1.5, z], flat);
        }
        chain
    }

    #[test]
    fn the_body_keeps_its_spacing() {
        let chain = walked(60);
        for pair in chain.segments.windows(2) {
            let d = distance(pair[0].at, pair[1].at);
            assert!(
                (d - chain.spacing).abs() < chain.spacing * 0.45,
                "a segment strayed: {d} vs {}",
                chain.spacing
            );
        }
    }

    #[test]
    fn the_body_tapers_head_to_tail() {
        let chain = Chain::tapered(8, 1.5);
        let first = chain.segments[1].radius;
        let last = chain.segments[7].radius;
        assert!(first > last, "the tail is thinner than the shoulders");
    }

    #[test]
    fn a_moving_body_undulates_and_a_still_one_settles() {
        let mut moving = walked(40);
        let lateral_spread = |chain: &Chain| -> f32 {
            // Perpendicular scatter of the chain about its head-tail axis.
            let head = chain.segments[0].at;
            let tail = chain.segments.last().unwrap().at;
            let axis = [tail[0] - head[0], tail[2] - head[2]];
            let len = (axis[0] * axis[0] + axis[1] * axis[1]).sqrt().max(1e-5);
            chain
                .segments
                .iter()
                .map(|s| {
                    let rel = [s.at[0] - head[0], s.at[2] - head[2]];
                    ((rel[0] * axis[1] - rel[1] * axis[0]) / len).abs()
                })
                .fold(0.0, f32::max)
        };
        let wriggle = lateral_spread(&moving);

        // Hold the head still; the wave dies with speed and the relax term
        // straightens the chain. Straightening cascades one segment per
        // step, so a nine-segment body needs tens of steps to settle, which
        // at tick rate is a couple of seconds of stillness.
        let head = moving.segments[0].at;
        for _ in 0..90 {
            moving.step(head, flat);
        }
        let settled = lateral_spread(&moving);
        assert!(
            wriggle > settled * 1.5,
            "moving wriggle {wriggle} did not exceed settled {settled}"
        );
    }

    #[test]
    fn the_walk_is_deterministic() {
        let a = walked(50);
        let b = walked(50);
        assert_eq!(a.segments, b.segments);
    }

    #[test]
    fn the_body_drapes_over_ground() {
        let mut chain = Chain::tapered(8, 1.5);
        let bumpy = |x: f32, z: f32| (x * 0.3).sin() * 4.0 + (z * 0.2).cos() * 3.0;
        for step in 0..60 {
            let [x, z] = wander(3, [0.0, 0.0], step);
            chain.step([x, bumpy(x, z) + 1.5, z], bumpy);
        }
        for segment in &chain.segments[1..] {
            let floor = bumpy(segment.at[0], segment.at[2]);
            assert!(
                segment.at[1] > floor - segment.radius,
                "a segment tunnelled below ground"
            );
        }
    }
}
