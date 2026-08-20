// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The biosphere lens.
//!
//! Two passes project presentation data supplied by a caller. The **march**
//! renders a heightfield and optional SDF/capsule body; the **grade** applies
//! fog, palette quantisation, and ordered dither. Simulation authority stays
//! outside this crate.
//!
//! [`Lens::encode`] is the live boundary: retained resources, a caller-owned
//! encoder, and a caller-owned target. [`Lens::capture`] is the receipt adapter
//! over that same path.

mod body;
pub mod bricks;
pub mod critter;
pub mod maps;
mod renderer;
mod scene;
mod tracer;

#[cfg(test)]
mod netrender_tests;
#[cfg(test)]
mod tracer_tests;

pub use body::{BodyLensProjection, BodyPlacement, BodyProjectionError, BodyRevision, LensPart};
pub use bricks::{BrickMap, BrickMapError};
pub use renderer::{
    Capture, DirtyRect, FRAME_FORMAT, FrameDiagnostics, FrameInput, Lens, LensError, MapChange,
    MapRevision,
};
pub use scene::{LensScene, SceneCodecError};
pub use tracer::{
    BrickCapture, BrickChange, BrickDiagnostics, BrickFrameInput, BrickRevision, BrickTraceError,
    BrickTracer, LeasedAtlas, TraceCamera,
};

/// Hard admission limit imposed by the baseline uniform layout.
pub const MAX_CAPSULES: usize = 96;

/// A look, as data. Worldgen can emit one of these per world or per biome
/// the way it emits a heightmap.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Grade {
    pub fog: [f32; 3],
    /// Distance (0..1 of far) where fog begins.
    pub fog_start: f32,
    /// Palette entries used; 0 disables quantisation (clay).
    pub palette_len: u32,
    /// Ordered-dither strength; 0 disables.
    pub dither: f32,
    /// Fog band count; 0 is smooth.
    pub fog_bands: f32,
    /// Internal render scale denominator: 1 = full res, 4 = quarter res
    /// integer-upscaled, which is most of the retro grain.
    pub downscale: u32,
}

impl Grade {
    /// The Comanche soul: starved palette, ordered dither, banded fog,
    /// quarter-resolution grain.
    pub fn retro(palette_len: u32) -> Self {
        Self {
            fog: [0.66, 0.66, 0.72],
            fog_start: 0.35,
            palette_len,
            dither: 0.10,
            fog_bands: 6.0,
            downscale: 4,
        }
    }

    /// The clay soul: full resolution, smooth ramp, smooth fog.
    pub fn clay() -> Self {
        Self {
            fog: [0.72, 0.74, 0.80],
            fog_start: 0.45,
            palette_len: 0,
            dither: 0.0,
            fog_bands: 0.0,
            downscale: 1,
        }
    }
}

/// A first-person camera over the heightfield, in map units.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Flight {
    pub eye: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    pub far: f32,
}

/// The body in frame, if any, as the shader wants it.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CritterPose {
    pub capsules: Vec<critter::Capsule>,
    pub eyes: [[f32; 4]; 2],
    pub bounds_centre: [f32; 3],
    pub bounds_radius: f32,
    pub tint: [f32; 3],
}

impl CritterPose {
    pub fn from_capsules(
        capsules: Vec<critter::Capsule>,
        eyes: [[f32; 4]; 2],
        tint: [f32; 3],
    ) -> Self {
        let mut centre = [0.0f32; 3];
        for capsule in &capsules {
            for (axis, value) in centre.iter_mut().enumerate() {
                *value += (capsule.a[axis] + capsule.b[axis]) * 0.5;
            }
        }
        let n = capsules.len().max(1) as f32;
        for value in &mut centre {
            *value /= n;
        }
        let bounds_radius = capsules
            .iter()
            .flat_map(|c| [(c.a, c.ra), (c.b, c.rb)])
            .map(|(at, r)| {
                let d = [at[0] - centre[0], at[1] - centre[1], at[2] - centre[2]];
                (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() + r
            })
            .fold(0.0, f32::max)
            + 1.0;
        Self {
            capsules,
            eyes,
            bounds_centre: centre,
            bounds_radius,
            tint,
        }
    }

    pub fn from_body(
        body: &critter::Body,
        ground: impl Fn(f32, f32) -> f32,
        tint: [f32; 3],
    ) -> Self {
        let capsules = body.capsules(ground);
        Self::from_capsules(capsules, body.eyes(), tint)
    }
}
