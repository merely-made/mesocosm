// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! DT3's headed script: a trace that uses all four dev intents, recorded
//! headlessly so the headed run needs nobody at the keyboard.
//!
//! **The same arrangement `--record-demo` uses, and for the same reason.** The
//! dev keys are keys, so an unattended `--frames` run cannot press them; the
//! four intents they queue are ordinary intents, so a recorded trace carries
//! them exactly as it carries a move or a meal. This writes such a trace, and
//! prints the two numbers the headed run needs after it: the forced child's id
//! (for `--follow`, so the dev tile is showing what the script made) and the
//! hash to assert against.
//!
//! ```text
//! cargo run -p mesocosm-genet --release --example dt3_script -- <trace path>
//! cargo run -p mesocosm-genet --release -- --dev --replay <trace path> \
//!     --receipt <receipt path> --capture <png path> --follow <child id>
//! ```
//!
//! The script's shape, and why each part of it is there:
//!
//! - **It eats.** A hand that only ever says "carry on" is a critter paying
//!   rent and earning nothing, and this run's whole point is that somebody is
//!   still holding a body at the end of it: a checkpoint opens only while one
//!   is held (`World::held`, TD4), so a played critter that starved would take
//!   the trait board with it. The filler is the recorded demo's own rule —
//!   metabolize what is in reach, and where nothing is, say carry on.
//! - **`Resume` rather than `Idle` for the rest of the filler.** A run of idles
//!   hands the body back to its instincts after thirty ticks and the hand is
//!   gone; `Resume` is a hand saying carry on.
//! - **The birth is forced from a neighbour, not from the played critter.** A
//!   birth costs its parent a quarter of its body and a matching share of its
//!   budget, which is the ordinary price and is exactly why: charging it to the
//!   played critter cost it the body the boundary needs a hand on. The child
//!   still lands beside its parent in the same frame.
//! - **`EndEpoch` last.** It leaves the world holding at the lineage
//!   checkpoint, so the frame the headed run captures is the trait board over
//!   the terrarium — the boundary PE3a built, opened by a dev key.

use mesocosm_core::{Intent, Outcome, Placement, World};
use mesocosm_genet::app::DEV_PLACE_MG;
use mesocosm_genet::{HostConfig, fixture, played};
use mesocosm_mesh::VolumeMap;
use mesocosm_runtime::Runtime;

/// Ticks the enclosure settles for before the script touches it.
const SETTLE: u64 = 40;
/// Ticks the forced child is left to stand in the terrarium before the
/// boundary, so the capture has something grown enough to see.
const AFTER: u64 = 20;

/// One tick of ordinary play: eat what is in reach, or carry on.
fn filler(world: &World, volumes: &VolumeMap) -> Intent {
    match fixture::reachable(world) {
        Some(target) => fixture::metabolize(world, target, volumes, Placement::Planned),
        None => Intent::Resume,
    }
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .map(std::path::PathBuf::from)
        .expect("usage: dt3_script <trace path>");

    let config = HostConfig::default();
    let volumes = fixture::volumes();
    let mut runtime = Runtime::new(config.seed, config.organisms, config.ticks_per_second);
    for _ in 0..SETTLE {
        let intent = filler(runtime.world(), &volumes);
        runtime.queue(intent);
        runtime.step(1);
    }

    let me = runtime
        .world()
        .controlled_id()
        .expect("the default world starts embodied");
    let here = runtime.world().position().expect("and standing somewhere");

    // The two nearest neighbours, so both dev acts happen where the camera
    // already is: one bears an offspring, the other's life ends.
    let mut near: Vec<_> = runtime
        .world()
        .living()
        .filter(|o| o.id != me && o.biomass_mg() > 400)
        .map(|o| {
            (
                (0..3)
                    .map(|axis| (o.position[axis] - here[axis]).abs())
                    .sum::<i32>(),
                o.id,
            )
        })
        .collect();
    near.sort();
    let parent = near.first().expect("the enclosure is not empty").1;
    let doomed = near.get(1).expect("nor nearly empty").1;

    let mut applied: Vec<(&str, Outcome)> = Vec::new();
    let step = |runtime: &mut Runtime, intent: Intent| -> Outcome {
        runtime.queue(intent);
        runtime.step(1);
        runtime
            .last_outcomes()
            .first()
            .copied()
            .unwrap_or(Outcome::Idled)
    };

    applied.push((
        "PlaceMatter",
        step(
            &mut runtime,
            Intent::PlaceMatter {
                at: here,
                mass_mg: DEV_PLACE_MG,
            },
        ),
    ));
    applied.push((
        "Kill",
        step(&mut runtime, Intent::Kill { organism: doomed }),
    ));
    let bore = step(&mut runtime, Intent::ForceBirth { organism: parent });
    applied.push(("ForceBirth", bore));
    let child = match bore {
        Outcome::Bore { offspring, .. } => offspring,
        other => panic!("the forced birth was refused: {other:?}"),
    };

    // A stretch of ordinary play, so the child is standing in the terrarium
    // and the corpse is lying in it by the time the boundary opens.
    for _ in 0..AFTER {
        let intent = filler(runtime.world(), &volumes);
        runtime.queue(intent);
        runtime.step(1);
    }
    assert!(
        runtime.world().controlled_id().is_some(),
        "somebody has to still be holding a body, or the boundary asks nobody"
    );
    applied.push(("EndEpoch", step(&mut runtime, Intent::EndEpoch)));

    for (name, outcome) in &applied {
        assert!(
            !matches!(outcome, Outcome::Rejected(_)),
            "{name} was refused: {outcome:?}"
        );
        println!("{name}: {outcome:?}");
    }
    assert_eq!(runtime.dev_intents(), 4, "four dev intents applied");

    let recorded = played::PlayedTrace {
        seed: config.seed,
        organisms: config.organisms,
        steps: runtime.trace().len() as u64,
        state_hash: runtime.state_hash(),
        intents: runtime.trace().to_vec(),
    };
    if let Err(error) = played::write_json(&path, &recorded) {
        eprintln!("dt3_script: {error}");
        std::process::exit(1);
    }
    println!(
        "recorded {} intents ({} of them dev) to {}, hash {:016x}",
        recorded.intents.len(),
        runtime.dev_intents(),
        path.display(),
        recorded.state_hash
    );
    println!("forced child: {} (pass as --follow {})", child.0, child.0);
}
