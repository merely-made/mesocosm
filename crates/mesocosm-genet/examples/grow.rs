// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Grows a critter headlessly and writes frames, using the host's real
//! placement policy.
//!
//! Exists so the growth policy can be looked at without opening a window,
//! which matters when a window is already open: on Windows a running instance
//! holds the binary and the linker cannot replace it.
//!
//! ```text
//! cargo run -p mesocosm-genet --example grow -- <output-dir> [meals]
//! ```

use mesocosm_core::World;
use mesocosm_mesh::{VolumeSource, flatten, mesh_body};
use mesocosm_render::{Camera, Renderer, SceneItem};

use mesocosm_genet::fixture;

const SIZE: u32 = 640;

fn main() {
    let mut args = std::env::args().skip(1);
    let dir: std::path::PathBuf = args.next().unwrap_or_else(|| ".".into()).into();
    let meals: usize = args
        .next()
        .and_then(|v| v.parse().ok())
        .unwrap_or(18);
    std::fs::create_dir_all(&dir).expect("output directory");

    let volumes = fixture::volumes();
    let renderer = Renderer::headless(SIZE, SIZE).expect("a GPU adapter");

    let mut world = World::new(0x00A7_7AC4, 90);
    let mut eaten = 0;

    for _ in 0..meals {
        let Some(target) = fixture::reachable(&world) else {
            break;
        };
        let intent = fixture::metabolize(&world, target, &volumes);
        if matches!(
            world.apply(intent),
            mesocosm_core::Outcome::Incorporated { .. }
        ) {
            eaten += 1;
        }
    }

    let mesh = mesh_body(&world.body, &volumes).expect("every part resolves");
    let (min, max) = mesh.bounds().expect("geometry");
    let camera = Camera::framing(min, max, 1.0);
    let frame = renderer
        .render_scene(&[SceneItem::new(&mesh, [0, 0, 0])], &camera)
        .expect("render");

    let path = dir.join("06_grown.png");
    let file = std::fs::File::create(&path).expect("create png");
    let mut encoder = png::Encoder::new(file, frame.width, frame.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .expect("header")
        .write_image_data(&frame.pixels)
        .expect("data");

    // The receipt that matters: no voxel was lost to an overlap.
    let flat = flatten(&world.body, &volumes).expect("flatten");
    let expected: usize = world
        .body
        .parts
        .iter()
        .filter_map(|p| volumes.volume(p.volume))
        .map(|v| v.solid_count())
        .sum();

    // Lateral balance is the check that matters for a bilateral plan: a
    // mirrored body keeps its centre of mass on the midline.
    let centre = world.body.centre_of_mass();
    let pairs = world
        .body
        .parts
        .iter()
        .filter(|p| {
            let Some(at) = world.body.world_offset(p.id) else {
                return false;
            };
            at[0] != 0
        })
        .count();

    println!("ate {eaten}, body has {} parts", world.body.len());
    // With pivots the root is centred on the origin, so the midline is zero.
    let midline = 0;
    println!(
        "centre of mass {centre:?}; midline x={midline} -> {}",
        if centre[0] == midline { "balanced" } else { "DRIFTING" }
    );
    println!("{pairs} parts sit off the midline");
    println!("mass {} mg", world.total_mass_mg());
    println!(
        "voxels: {} placed, {} expected{}",
        flat.volume.solid_count(),
        expected,
        if flat.volume.solid_count() == expected {
            " (no overlap)"
        } else {
            " (OVERLAP: parts are stacking)"
        }
    );
    println!("wrote {} ({} px drawn)", path.display(), frame.covered());
}
