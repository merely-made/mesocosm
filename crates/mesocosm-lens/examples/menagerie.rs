// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The generator's receipt: one method, several animals.
//!
//! Renders catalogue plans through the same lens with nothing per-creature in
//! the renderer. If a centipede, an insect, and a snake come out visibly
//! different, the axial recipe is doing the work rather than a sculpt.
//!
//! ```text
//! cargo run -p mesocosm-lens --example menagerie -- <out_dir>
//! ```

use mesocosm_core::axis::{Soma, catalogue};
use mesocosm_lens::{CritterPose, Flight, Grade, Lens, critter, maps};

fn write_png(path: &std::path::Path, width: u32, height: u32, pixels: &[u8]) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let file = std::fs::File::create(path).expect("capture file");
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .expect("png header")
        .write_image_data(pixels)
        .expect("png data");
}

fn main() {
    let out = std::env::args().nth(1).unwrap_or_else(|| "captures".into());
    let out = std::path::Path::new(&out);

    let (width, height) = (1280, 720);
    let Some(lens) = Lens::headless(width, height) else {
        eprintln!("no adapter; the probe needs a GPU");
        std::process::exit(1);
    };
    let world = maps::synthesize(4_242, 1024);
    let ground = |x: f32, z: f32| -> f32 {
        let side = world.side;
        let i = (z.max(0.0) as u32 % side) * side + (x.max(0.0) as u32 % side);
        world.height[i as usize] as f32
    };

    // The flattest patch, so silhouettes read against ground rather than
    // against a cliff.
    let mut origin = [430.0, 350.0];
    let mut best = f32::MAX;
    for row in (64..(world.side - 112)).step_by(32) {
        for col in (64..(world.side - 112)).step_by(32) {
            let (mut lo, mut hi) = (255u8, 0u8);
            for dz in (0..48).step_by(8) {
                for dx in (0..48).step_by(8) {
                    let h = world.height[((row + dz) * world.side + col + dx) as usize];
                    lo = lo.min(h);
                    hi = hi.max(h);
                }
            }
            if hi >= 250 || lo <= 5 {
                continue;
            }
            if ((hi - lo) as f32) < best {
                best = (hi - lo) as f32;
                origin = [col as f32 + 8.0, row as f32 + 8.0];
            }
        }
    }

    let subjects = [
        ("centipede", catalogue::centipede(14), 1.1),
        ("insect", catalogue::insect(), 1.5),
        ("spider", catalogue::spider(), 1.6),
        ("tetrapod", catalogue::tetrapod(6), 1.6),
        ("snake", catalogue::snake(16), 1.2),
    ];
    let look = Grade::clay();

    for (name, plan, scale) in subjects {
        let soma = Soma::develop(&plan, 9);
        let mut body = critter::Body::from_plan(&plan, &soma, scale);
        for step in 0..14 {
            let [x, z] = critter::wander(4_242, origin, step);
            body.step([x, ground(x, z) + 2.2, z], ground);
        }
        let pose = CritterPose::from_body(&body, ground, [0.36, 0.62, 0.42]);

        // Framed to the body's own size, side-on, so plans compare fairly.
        let head = body.chain.segments[0].at;
        let tail = body.chain.segments.last().unwrap().at;
        let mid = [
            (head[0] + tail[0]) * 0.5,
            (head[1] + tail[1]) * 0.5,
            (head[2] + tail[2]) * 0.5,
        ];
        let heading = f32::atan2(head[0] - tail[0], head[2] - tail[2]);
        let flank = heading + std::f32::consts::FRAC_PI_2;
        let off = pose.bounds_radius * 2.6 + 8.0;
        let mut eye = [
            mid[0] + flank.sin() * off,
            mid[1] + off * 0.28,
            mid[2] + flank.cos() * off,
        ];
        eye[1] = eye[1].max(ground(eye[0], eye[2]) + 4.0);
        let dist = ((mid[0] - eye[0]).powi(2) + (mid[2] - eye[2]).powi(2)).sqrt();
        let flight = Flight {
            eye,
            yaw: f32::atan2(mid[0] - eye[0], mid[2] - eye[2]),
            pitch: f32::atan2(mid[1] - 2.0 - eye[1], dist),
            fov: 0.9,
            far: 700.0,
        };

        let pixels = lens.render_with(&world, &flight, &look, Some(&pose));
        let path = out.join(format!("18_plan_{name}.png"));
        write_png(&path, width, height, &pixels);
        println!(
            "captured {} ({} segments, {} appendages, complexity {})",
            path.display(),
            plan.segments(),
            plan.appendages(),
            plan.complexity()
        );
    }
}
