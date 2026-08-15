// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use mesocosm_core::places::{Ground, Places};

use crate::{
    BrickChange, BrickFrameInput, BrickMap, BrickRevision, BrickTracer, CritterPose, Flight, Grade,
    critter::Capsule,
};

fn ground() -> Ground {
    let grown = Places::grown(4_242, 4, 64);
    Ground::grow(&grown, 64)
}

fn flight(ground: &Ground) -> Flight {
    let top = ground.surface(4, 4).expect("ground column");
    Flight {
        eye: [4.5, top as f32 + 14.0, 4.5],
        yaw: 0.0,
        pitch: -1.52,
        fov: 0.15,
        far: 48.0,
    }
}

#[test]
fn a_carved_ground_updates_only_its_brick_slots_in_the_live_tracer() {
    let mut ground = ground();
    let mut map = BrickMap::from_ground(&ground).expect("atlas capacity");
    let Some(mut tracer) = BrickTracer::headless(96, 64) else {
        eprintln!("no adapter; skipping brick tracer receipt");
        return;
    };
    let camera = flight(&ground);
    let grade = Grade::clay();
    let before = tracer
        .capture(BrickFrameInput::new(
            &map,
            BrickRevision(ground.revision()),
            &camera,
            &grade,
        ))
        .expect("initial brick frame");
    assert!(before.diagnostics.brick_upload_bytes > 0);
    assert_eq!(before.diagnostics.trace_passes, 1);

    let top = ground.surface(4, 4).expect("ground column");
    assert!(ground.carve([4, top, 4], 2) > 0);
    let dirty = ground.drain_dirty();
    let slots = map
        .refresh(&ground, dirty)
        .expect("a carve preserves the brick map shape");
    assert!(!slots.is_empty());
    let after = tracer
        .capture(
            BrickFrameInput::new(&map, BrickRevision(ground.revision()), &camera, &grade)
                .changed(BrickChange::Slots(&slots)),
        )
        .expect("updated brick frame");

    assert_eq!(
        after.diagnostics.brick_upload_bytes,
        slots.len() as u64 * (8 * 8 * 8 + size_of::<u32>()) as u64,
        "a carve uploads just its changed brick slots"
    );
    assert_ne!(
        before.pixels, after.pixels,
        "the opened material is visible"
    );
}

#[test]
fn a_steady_brick_frame_has_no_upload_churn() {
    let ground = ground();
    let map = BrickMap::from_ground(&ground).expect("atlas capacity");
    let Some(mut tracer) = BrickTracer::headless(64, 64) else {
        eprintln!("no adapter; skipping brick tracer receipt");
        return;
    };
    let camera = flight(&ground);
    let grade = Grade::retro(3);
    let input = BrickFrameInput::new(&map, BrickRevision(ground.revision()), &camera, &grade);
    let first = tracer.capture(input).expect("first brick frame");
    let steady = tracer.capture(input).expect("steady brick frame");

    assert_eq!(steady.diagnostics.brick_upload_bytes, 0);
    assert_eq!(steady.diagnostics.uniform_upload_bytes, 0);
    assert_eq!(steady.diagnostics.resource_creations, 0);
    assert_eq!(steady.pixels, first.pixels);
}

#[test]
fn a_nearer_sdf_body_composes_in_front_of_ground() {
    let ground = ground();
    let map = BrickMap::from_ground(&ground).expect("atlas capacity");
    let Some(mut tracer) = BrickTracer::headless(96, 64) else {
        eprintln!("no adapter; skipping brick/body composition receipt");
        return;
    };
    let camera = flight(&ground);
    let grade = Grade::clay();
    let terrain = tracer
        .capture(BrickFrameInput::new(
            &map,
            BrickRevision(ground.revision()),
            &camera,
            &grade,
        ))
        .expect("terrain frame");
    let top = ground.surface(4, 4).expect("ground column") as f32;
    let pose = CritterPose::from_capsules(
        vec![Capsule {
            a: [4.5, top + 7.0, 4.5],
            ra: 1.4,
            b: [4.5, top + 5.0, 4.5],
            rb: 1.1,
        }],
        [[4.5, top + 6.6, 4.5, 0.18], [4.5, top + 6.2, 4.5, 0.18]],
        [0.15, 0.86, 0.32],
    );
    let composed = tracer
        .capture(
            BrickFrameInput::new(&map, BrickRevision(ground.revision()), &camera, &grade)
                .with_pose(&pose),
        )
        .expect("composed frame");

    assert_eq!(composed.diagnostics.brick_upload_bytes, 0);
    assert!(composed.diagnostics.uniform_upload_bytes > 0);
    assert_ne!(
        terrain.pixels, composed.pixels,
        "body wins its nearer pixels"
    );
}
