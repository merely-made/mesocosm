// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! PD2's headed receipt: the vitals panel, in each of the gate's four states,
//! rasterized through the real cambium/netrender pipeline with no window.
//!
//! **A temporary receipt tool, same as the rest of PD2's authoring path.**
//! `Intent::Rearrange` is the gate's one editor operation and it has no
//! keyboard binding — the plan permits exactly a native fixture or an
//! explicit editor operation, not a finished UI — so there is no ordinary
//! `--replay` trace that reaches a gland. This drives the same
//! `mesocosm_genet::vitals::VitalsChrome` the interactive host composites,
//! over a headless device, so the pixels are the engine's own rather than a
//! description of them. Deleted alongside the rest of the temporary path at
//! PD3.
//!
//! Four captures, four states, in `Code/testing/mesocosm/`:
//! - `pd2_process_1_allocated.png` — the moment a development takes tissue
//!   off the frond and pays for it. The panel shows the "rebuilt" notice, the
//!   cells and the part, and the rent that starts from here.
//! - `pd2_process_2_useful.png` — the same body, its notice faded, reading a
//!   live bite cost off ground it built the gland on.
//! - `pd2_process_3_dormant.png` — the body one column over: the tissue and
//!   the rent are unchanged, and the bite is not, because this ground cannot
//!   supply what the gland holds.
//! - `pd2_process_4_severed.png` — the frond gone. No sting, no rent, and the
//!   branch can still say what it used to carry.
//!
//! ```text
//! cargo run -p mesocosm-genet --release --example pd2_receipt
//! ```

use std::path::Path;

use mesocosm_core::{
    Allocate, Attachment, CellId, Intent, Kingdom, Organism, Outcome, PartId, Process, ProcessRef,
    Provenance, Registry, Stage, Trend, VolumeRef, World, Yaw,
};
use mesocosm_genet::chrome::Chrome;
use mesocosm_genet::vitals::VitalsChrome;
use mesocosm_genet::{hud, played};

/// Panel plus margin, not a game window: enough canvas to hold the
/// bottom-left-placed panel with its usual clearance on every side.
const FRAME: (u32, u32) = (330, 220);
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

/// A plate wide enough that its gland can outgrow fresh ground — the same
/// fixture `tests/embodied/gland.rs` uses, for the same reason: `[6, 4, 1]`
/// lattices to twelve cells, so a gland on five of them holds more than a
/// fresh soil column supplies, which is what lets one fixture show a charged
/// gland and a dry one without waiting for the enclosure to draw itself down.
const FROND: [i32; 3] = [6, 4, 1];
const SEED: u64 = 4_242;
const FOUNDERS: u32 = 24;

fn gland_ref() -> ProcessRef {
    Registry::native().of_native(Process::Secrete).reference()
}

fn fixing_ref() -> ProcessRef {
    Registry::native().of_native(Process::Fix).reference()
}

/// The played critter, reset to a plain bulk consumer so the fixture below is
/// the only anatomy on the body that matters.
fn bulk_world(seed: u64, founders: u32) -> World {
    let mut world = World::new(seed, founders);
    let me = world.controlled_id().expect("embodied");
    let organism = world.organisms.iter_mut().find(|o| o.id == me).unwrap();
    let (species, position) = (organism.species, organism.position);
    *organism = Organism {
        stage: Stage::Mature,
        ..Organism::founding(
            me,
            species,
            Kingdom::Consumer,
            VolumeRef::from_tag(1),
            [2, 2, 2],
            position,
            1_500,
        )
    };
    world
}

/// Grows the frond directly on the played critter's phenotype: the "native
/// developmental fixture" the plan names as an acceptable PD2 proof vehicle,
/// exactly as `tests/embodied/gland.rs` uses it. What follows —
/// `Intent::Rearrange` — is the real, validated editor verb; only the
/// anatomy this fixture starts from is not itself something an ordinary meal
/// produced here.
fn frond_on(world: &mut World) -> PartId {
    let me = world.controlled_id().expect("embodied");
    let organism = world.organisms.iter_mut().find(|o| o.id == me).unwrap();
    let root = organism.body().root;
    organism
        .phenotype
        .attach(
            VolumeRef::from_tag(7),
            300,
            FROND,
            Attachment {
                parent: root,
                offset: [0, 7, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .expect("a frond attaches above the root")
}

/// Asks for the frond split between fixing and a gland of `cells` cells,
/// taken off the high end of the lattice — the real `Intent::Rearrange` a
/// host would send.
fn split(world: &World, part: PartId, cells: u32) -> Intent {
    let capacity = world
        .phenotype()
        .expect("embodied")
        .mosaic(part)
        .expect("a living part carries a mosaic")
        .capacity();
    let kept: Vec<CellId> = (0..capacity - cells).map(|i| CellId(i as u16)).collect();
    let taken: Vec<CellId> = (capacity - cells..capacity)
        .map(|i| CellId(i as u16))
        .collect();
    let mut sites = Vec::new();
    if !kept.is_empty() {
        sites.push(Allocate {
            process: fixing_ref(),
            cells: kept,
        });
    }
    sites.push(Allocate {
        process: gland_ref(),
        cells: taken,
    });
    Intent::Rearrange { part, sites }
}

fn main() {
    let out_dir = played::default_out_dir();
    std::fs::create_dir_all(&out_dir).expect("out dir");

    let chrome = pollster::block_on(headless_chrome());
    let mut vitals = VitalsChrome::new(&chrome);

    let mut world = bulk_world(SEED, FOUNDERS);
    let part = frond_on(&mut world);

    // State 1: allocated, and paid. Five of the frond's twelve cells become
    // the gland; the other seven keep fixing.
    let outcome = world.apply(split(&world.clone(), part, 5));
    let Outcome::Rearranged { cost_mg, .. } = outcome else {
        panic!("the development did not validate: {outcome:?}");
    };
    println!(
        "state 1: rearranged, cost {cost_mg} mg, tick {}",
        world.tick
    );
    capture(
        &chrome,
        &mut vitals,
        &world,
        &[outcome],
        &out_dir.join("pd2_process_1_allocated.png"),
    );

    // State 3: dormant, branched off state 1 immediately — mirroring
    // `tests/embodied/gland.rs` exactly, one tick away from the development —
    // so the ecology has no time to enrich the column the body walks onto.
    // One column over, the gland's own ground stays behind and this one has
    // never been enriched, so it cannot supply what five cells of gland hold.
    // Nothing about the allocation moves.
    let mut dormant = world.clone();
    dormant.apply(Intent::Move { delta: [2, 0, 0] });
    let dry = dormant.gland().expect("still has one");
    assert!(!dry.charged, "the fixture wants a dry gland here");
    println!(
        "state 3: dry, ground {} mg against {} mg needed, tick {}",
        dry.ground_mg, dry.potency_mg, dormant.tick
    );
    capture(
        &chrome,
        &mut vitals,
        &dormant,
        &[],
        &out_dir.join("pd2_process_3_dormant.png"),
    );

    // State 2: useful, continuing the original (unmoved) body. Wait out the
    // notice so the panel reads its steady state rather than the moment of
    // installation — the gland is charged by its own spoil the whole time,
    // since the development's price landed in the column under it.
    for _ in 0..30 {
        world.apply(Intent::Idle);
    }
    let charged = world.gland().expect("built one");
    assert!(charged.charged, "the fixture wants a charged gland here");
    println!(
        "state 2: charged, {} mg a bite, tick {}",
        charged.potency_mg, world.tick
    );
    capture(
        &chrome,
        &mut vitals,
        &world,
        &[],
        &out_dir.join("pd2_process_2_useful.png"),
    );

    // State 4: severed, and gone. No `Intent` removes a part yet — that is
    // phenotype D3a's gate, not PD2's — so this is the same direct call
    // `tests/embodied/gland.rs` uses to prove the branch's consequence goes
    // with it while the branch itself stays explainable.
    let me = world.controlled_id().expect("embodied");
    world
        .organisms
        .iter_mut()
        .find(|o| o.id == me)
        .unwrap()
        .phenotype
        .sever(part);
    let gone = world.gland().expect("the loss is still readable");
    assert!(gone.sites.is_empty());
    assert_eq!(gone.lost, vec![part]);
    println!("state 4: severed, tick {}", world.tick);
    capture(
        &chrome,
        &mut vitals,
        &world,
        &[],
        &out_dir.join("pd2_process_4_severed.png"),
    );
}

async fn headless_chrome() -> Chrome {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            apply_limit_buckets: false,
            compatible_surface: None,
        })
        .await
        .expect("an adapter");
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("pd2 receipt"),
            ..Default::default()
        })
        .await
        .expect("a device");
    Chrome::new(
        netrender::WgpuHandles {
            instance,
            adapter,
            device,
            queue,
        },
        FORMAT,
        hud::SIDE,
    )
    .expect("the chrome device")
}

/// Refreshes the panel off `world` and writes it, alone, to `path`. Bottom-
/// left placed on a plain cleared background rather than over a traced
/// section, because the reading is the receipt here, not the terrarium —
/// `mesocosm-views` and `mesocosm-genet`'s vitals lanes are exactly the
/// surface the plan names for this.
fn capture(
    chrome: &Chrome,
    vitals: &mut VitalsChrome,
    world: &World,
    outcomes: &[Outcome],
    path: &Path,
) {
    vitals.refresh(chrome, world, outcomes, world.tick, &Trend::default());

    let device = chrome.device();
    let queue = chrome.queue();
    let (width, height) = FRAME;
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("pd2 receipt target"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("pd2 receipt"),
    });
    {
        // A plain ground to composite the panel over — a dark neutral rather
        // than transparent, so the panel's own edges read in a plain PNG
        // viewer.
        let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("pd2 receipt clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.05,
                        g: 0.07,
                        b: 0.05,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }
    vitals.capture_composite(chrome, FORMAT, &mut encoder, &view, FRAME);

    let unpadded = width * 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = unpadded.div_ceil(align) * align;
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("pd2 receipt readback"),
        size: (padded * height) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(Some(encoder.finish()));

    let slice = staging.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("device poll");
    let mapped = slice.get_mapped_range().expect("mapped range");
    let mut pixels = Vec::with_capacity((unpadded * height) as usize);
    for row in 0..height {
        let start = (row * padded) as usize;
        pixels.extend_from_slice(&mapped[start..start + unpadded as usize]);
    }
    drop(mapped);
    staging.unmap();

    played::write_png(path, width, height, &pixels).expect("write png");
    println!("wrote {}", path.display());
}
