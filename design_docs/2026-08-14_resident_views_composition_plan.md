# Resident Views Composition Plan (2026-08-14)

**Status: founded 2026-08-14; proof gate open.** How the voxel world,
Burn/CubeCL, the tracer, the mesher, collision, and persistence compose
on one device. Ratified direction (Mark, 2026-08-14): the voxel world
is Burn-addressable state while remaining voxel-authoritative state.
Burn does not import voxels into a second world representation; every
consumer receives a resident view of the same facts.

The platform half of this ruling (lanes are program shapes, the
allocator direction, the resident-views law, the tensor-bundle plane
taxonomy) lives in mere
(`design_docs/mere_docs/technical_architecture/2026-08-13_spatial_compute_plan.md`
section 0.5). This doc is the wing application: the seam type, the
ownership rows, and the first proof.

This is the graph architecture applied to spatial state. The graph is
not inside layout, rendering, search, or ML; it stays authoritative
while each consumer gets an appropriate resident reading. `Ground` is
not inside the tracer, Burn, or parry; same law. The wing had already
grown the authoritative half before the seam was named: `Ground`
carries `revision: u64` and a dirty-brick queue whose doc comment
states the law ("projection work queue, not world authority; the
revision and brick bytes carry the authoritative change").

## The seam

```
ResidentChunk
  identity
  world position and extent
  authoritative revision (mirrors Ground revision at materialization)
  typed channel planes
  device-buffer handles
  shape, strides, element types
  valid-read epoch
  dirty regions
```

Views produced without copying: Burn tensor view, CubeCL kernel view,
tracer/material view, mesher input, collision-refresh input,
persistence/checkpoint view. A view is one arrangement of the chunk,
never a replacement for it.

**Planes are a bundle, not one tensor.** Exact planes (occupancy,
palette id, provenance, flags), fixed-point simulation planes
(temperature, moisture, nutrient, pressure; replay-exact), learned and
derived planes (latents, light estimates; projections, drift is a
shrug), temporary planes (inference output, constraint masks, work
queues). The plane taxonomy is the fact-bearing ruling: exact and
fixed-point planes belong to the record's side of the doctrine;
everything else is projection.

**Allocation flows compute-side out.** Burn/CubeCL can hand its
allocations outward but cannot cheaply adopt a foreign buffer, so
planes are allocated through the CubeCL client (conatus's resident
allocations) and leased to tracer, mesher, and render tenants.

## The transition path

```
resident chunk
    | tensor view
Burn model or CubeCL process
    | proposed channel deltas (candidate planes)
constraint and game-rule projection
    | accepted delta
authoritative commit + revision bump (Ground)
    |
tracer / mesher / collision / persistence observe the revision
```

**Burn proposes; the record disposes.** Inference stays GPU-resident.
The CPU reads accepted deltas, checkpoints, hashes, and small
reductions at durability boundaries, never whole planes per tick (the
4-byte readback pattern from the resident lane). A commitment that
reads a projected channel snapshots it into the fact at commitment
time.

## Ownership

- **Mesocosm** owns channel meanings, authoritative voxel facts
  (`Ground`), constraints, and commits.
- **Conatus** owns the shared device schedule, resident allocations,
  leases, and execution order.
- **Burn/CubeCL** owns tensor evaluation and learned or numerical
  transforms.
- **Renderling/netrender** consumes resident buffers and revisions as
  tenants.
- **Parry** derives tactile query state from committed voxel revisions
  (never from candidate planes).
- **Persistence and peers** exchange canonical chunk facts and deltas,
  never GPU allocation details.

## The first proof (gate open)

Use one existing `Ground` region; invent no second chunk model.

1. Materialize its authoritative bricks into one resident chunk bundle.
2. Construct a Burn tensor view over a material-derived plane
   (temperature or moisture).
3. Run a simple 3D diffusion or NCA pass through Burn/CubeCL.
4. Have the brick tracer visualize that field from the same resident
   allocation.
5. Commit one constrained field delta back into world history.
6. Receipts: one device; one allocation per plane (proved by buffer
   identity, the P4 epoch style); zero per-tick bulk copies; revision
   coherence (every consumer observes the same revision); replay
   (facts reproduce the committed state with no simulation present);
   unchanged-frame silence (a static world causes no recomputation,
   gated on elapsed time, never frame counts).

Generation, batched brains, fluids, and learned growth then stack on
this seam rather than each negotiating its own import path.

**Sequencing:** the place-graph engine work (G2+) is live in this tree
and owns `Ground`'s internals right now. This proof starts after that
work settles; the seam consumes `Ground`'s public revision contract
and does not reach into brick internals.

## Lanes (2026-08-14)

The proof gate above is one sequence, but the work under it is not one
sequence. These are the independent lanes, each written so a cold
session can pick it up. Lane briefs are handoffs; the ownership column
is what makes two of them safe to run at once.

**The safety rule: disjoint file ownership, not good intentions.** This
tree and mere both had concurrent sessions editing them the day this
was written. Lanes B, C, and D live in `mere/crates/probes/`, which
`.gitignore` excludes wholesale, so they touch no tracked file and can
never appear in another session's staging. Lane A owns quint's source
alone. While lanes run concurrently, each commits its own paths and
names what it left unstaged, which is the concurrent-work exception to
the usual commit-the-whole-tree default.

### Lane A. The seam (keystone, unblocked)

`ResidentChunk` and its view constructors, in mere at
`crates/conatus/quint/src/`, behind the existing `field-gpu` feature.

**A module, not a crate.** quint already owns resident GPU allocations
(`quint::resident`), already gates wgpu optionally, and feature gating
already gives the wing its subset without dragging the field algebra.
Split it out only if chunk substrates outgrow that neighborhood or a
consumer needs the seam with none of quint's algebra.

**Done when:** a synthetic chunk yields a Burn tensor view and a raw
kernel view over the same allocation, proved by buffer identity rather
than equal contents; revision and valid-read epoch surface to a
consumer; planes allocate through the CubeCL client (the allocator
direction); no CPU whole-plane read exists in the path.

Blocks lanes D, E, F. Owns quint's source.

### Lane B. Harvest (unblocked, independent)

Port nexus's GPU radix sort and Karras LBVH build/refit into our
carriage, in an untracked probe. Carry the two caveats their source
records: the Windows and NVIDIA atomic-load workaround (wgpu#9221) and
the radix-sort bounds-check note.

**Done when:** sort correctness against a CPU reference on adversarial
keys (duplicates, already sorted, reversed, single value); tree
correctness (every leaf reachable, each node's box containing its
children's); a timing curve against the CPU Barnes-Hut reference; and
a positive control proving the GPU path actually ran, per the
".spv existing is not .spv running" finding.

Promotion into conatus is a separate decision after receipts.

### Lane C. The carriage decision (unblocked, independent)

Port quint's repulsion kernel to a CubeCL raw kernel in an untracked
probe, with Burn sharing the client so the handoff bridge is absent by
construction rather than by effort.

**Done when:** numeric agreement with `forces::repulsion_reference` at
the tolerance the existing handoff receipt used; timing measured
against the resident lane's 50k figure; and a plain statement of
whether the bridge code disappears. Either outcome is an answer: it
decides whether the explicit lane consolidates on CubeCL-JIT or keeps
both carriages on purpose.

### Lane D. Mechanism proof (complete 2026-08-14)

Burn tensor view over a synthetic chunk plane, one diffusion or NCA
pass, one constrained delta committed in the commit shape. No `Ground`,
no tracer. This clears proof steps 2, 3, and 5 without waiting on the
wing, so only the wing-specific receipts remain.

Receipt: Mere's ignored `crates/probes/resident-diffusion` constructs a
`3x3x3` Burn tensor view from `ResidentChunk`, runs one six-neighbor
diffusion pass on the RTX 4060, and reads one four-byte candidate value.
The centre moved `10.000 -> 4.000`; the record constraint reduced the
proposed `-6000` milli-unit delta to `-1000`, committed revision
`41 -> 42`, then reproduced the resulting `9000` value from serialized
integer history with Burn absent. This clears steps 2, 3, and 5 only.

### Lane E. Wing materialization (mechanism receipt; stable base now available)

`Ground` into a resident chunk bundle, consuming only `Ground`'s public
revision contract. Blocked on lane A and on the live place-graph work,
which owns `Ground`'s internals.

The public surface proved sufficient without reaching into those
internals, so the mechanism ran while G2 remained dirty. Mere's ignored
`crates/probes/ground-resident` consumes only `Ground::revision`,
`drain_dirty`, `keys`, `brick_materials`, and `Brick::{get,raw}`. One
radius-zero carve produced Ground revision 1 and dirty brick
`[-4, 0, -4]`; that public brick became one `ResidentChunk` with world
origin `[-32, 0, -32]`, `8x8x8` extent, one 512-byte exact-U8 CubeCL
allocation, matching revision 1, and host read epoch 1 on the RTX 4060.
Ground's raw order is y/z/x, so materialization performs one
revision-gated staging transpose into canonical world x/y/z order. It
is not a per-frame consumer bridge. Because the receipt used the live
dirty G2 tree based at `7b68db0`, it must be repeated before the exact
material histogram becomes a stable fixture. **Unblocked 2026-08-14:** that
G2 session landed at `7869c96` (verified green first: 316 core tests, 25 lens
tests, fmt, and clippy `-D warnings`), so the stable base now exists and the
rerun is ordinary work.

### Lane F. Tracer lease (mechanism receipt; adoption now scopable)

The brick tracer reads the leased allocation; revision coherence and
unchanged-frame silence receipts. The permanent `mesocosm-lens`
integration was blocked on its tracer sources landing, and the public resident
seam was sufficient to prove the mechanism without editing that in-flight
work. **Unblocked 2026-08-14:** those sources landed at `7869c96`
(`BrickTracer`, `tracer.wgsl`, `BrickMap` in `bricks.rs`), so adoption is now
scopable against real code. The shape to reconcile: `BrickMap` owns an
`R8Uint` material atlas it rebuilds from `Ground`, while the lease offers the
same bytes as a `RawKernelView` on a CubeCL sub-allocation. Adoption means
choosing which of the two owns residency, not keeping both.

**Blocked again, on a wgpu major split (found 2026-08-15).** netrender main
moved to wgpu 30 (`7aa8c88d3`, "Move to wgpu 30 and vello 0.10", pushed),
while this workspace pins wgpu 29 and quint's resident lane is wgpu 29 as
well. A `wgpu::Buffer` from a 29 device is a different *type* than a 30 one,
so the lease cannot cross that boundary at all: this is a compile error, not
a performance question, and no amount of contract design routes around it.
Adoption waits on the row.

This tree builds today only because its lock is behind: it holds netrender at
`c7cedd2a`, a pre-30 commit, resolving wgpu 29.0.4. The next `cargo update`
that touches netrender breaks the workspace. Pinning netrender to the stale
commit is the wrong repair (branches are tracked, not pinned around); the
repair is to move the row.

Moving the row has a price worth knowing before choosing: wgpu 30.0.0 is
stable, but Burn and CubeCL reach it only through prereleases
(`cubecl-wgpu 0.11.0-pre.2` requires `^30.0.0`; `burn 0.22.0-pre.2`), against
today's stable `burn 0.21` / `cubecl 0.10` on wgpu 29. Both prereleases are
already vendored locally, so someone has scouted this. The composition thesis
does not depend on *which* wgpu version, only on everyone sharing one, so
either row works; what does not work is the current split.

Mere's ignored `crates/probes/resident-tracer-lease` binds the exact 512-byte
U8 `RawKernelView` allocation directly into a buffer-backed voxel DDA. On the
RTX 4060, it recorded zero tracer-owned material-upload bytes. An unchanged
16 ms frame performed zero dispatches, bind-group rebuilds, uploads, or
readbacks; the same allocation refreshed after the configured 60-second
elapsed interval without reattachment or upload. A public `Ground` carve
advanced the observed revision/read epoch from `0/10` to `1/11`, changed
2973/4096 pixels, and caused the older revision to be rejected. This proves
the lease, coherence, and elapsed-time silence mechanism. It does not claim
that the in-flight `BrickTracer` has adopted the lease.

### Execution receipts (2026-08-14)

- **A complete.** `ResidentChunk` landed in Mere commit `986fdb91`
  (four quint paths; the concurrent epoch-carriage session swept them
  into its wider commit; audit confirms the paths are in that commit).
  Real-adapter allocation-identity and metadata receipts pass, and the
  identity receipt is the strong form: it resolves the CubeCL handle
  carried by the public Burn tensor and compares buffer, offset, and
  size against the raw view, rather than comparing two metadata values
  quint minted itself. `wgpu::Buffer` equality proxies to the inner
  handle, so that assertion is identity. A second receipt fixes a
  constraint worth stating plainly in the seam's own terms: exact U8
  planes are kernel-viewable but **not** Burn-tensor-viewable
  (`burn_f32_view` returns `BurnElementType`), so "Burn-addressable"
  means the float planes. Audit gap: the brief's "no CPU whole-plane
  read exists in the path" is untested; the test header describes the
  test's method, not a property of the seam. Lane F shows the right
  instrument (counted upload bytes, asserted zero). Add that counter or
  strike the condition.
- **B complete, local ignored probe.** Nexus `3083a43` radix/LBVH
  harvest passes duplicate/sorted/reversed/single sort cases, 4096-leaf
  reachability and containment, and the GPU canary. At 50k, GPU LBVH
  construction measured 5.042 ms beside the differently scoped 305.451
  ms CPU Barnes-Hut reference (a live in-process `seiche` control, not a
  remembered figure). This does not replace mass aggregation. Two audit
  notes: traversal and force accumulation are unmeasured, so no
  end-to-end claim exists and the two figures should not be read as a
  ratio; and the harvest landed as hand-written WGSL (`lbvh.wgsl`,
  `radix.wgsl`), so if lane C's consolidation is eventually made, the
  algorithm knowledge transfers but the artifact needs rewriting. B and
  C were briefed as independent; they are not, and the brief was wrong
  to say so.
- **C: numeric half complete; the carriage verdict is withdrawn
  pending a control (audit, 2026-08-14).** Raw CubeCL repulsion agrees
  with `quint::forces::repulsion_reference` at `1.26e-7` mean relative
  error, and a warmed repulsion-only pass measured 9.55-9.71 ms at 50k.
  Both facts stand. The verdict drawn from them does not: the probe
  compares that repulsion-only pass against `RESIDENT_50K_MS = 12.8`, a
  constant hardcoded from an earlier session whose own printout names it
  the resident lane's *full* repulse-plus-springs-plus-integrate-plus-
  readback frame. Unequal scopes, and no incumbent measurement on this
  host in this run, so the numbers are equally consistent with CubeCL
  being slower at repulsion. The control was one call away and unused:
  `quint::resident::Resident::repulse_only` (resident.rs:445); the probe
  imports only the CPU reference, never `Resident`. **Rerun before
  consolidating:** incumbent and CubeCL repulsion, same host, same
  process, both warmed, with JIT first-launch cost reported separately
  since that cost is the actual tradeoff.
- **D complete.** Synthetic tensor/process/commit mechanism above.
- **E mechanism complete against the live G2 tree.** Stable-base rerun
  remains.
- **F mechanism complete against the live G2 tree.** Direct leased-buffer
  DDA, revision/read-epoch coherence, stale-lease rejection, and elapsed-time
  silence pass. Permanent `BrickTracer` adoption waits for its in-flight
  sources to land.

## Prior art, harvest, and standards (2026-08-14)

Per the borrowing doctrine: lean on mature references and standards,
harvest license-clean technique, and never adopt a tool for a
capability the stack intends to own. Licenses marked **verified** were
read from source today; everything else is verify-at-adoption.

- **Device and kernel language.** wgpu/WGSL is the W3C WebGPU standard
  (fetch the spec, cite the section, as always). CubeCL and Burn are
  MIT/Apache and already in the row. The rust-gpu AOT carriage is
  working in-tree.
- **Far field and spatial queries: harvest nexus.** MIT OR Apache-2.0
  (**verified**, workspace manifest; checkout at `Code/crates/nexus`,
  commit `3083a43`). Lift as kernels and patterns into conatus lanes,
  never as an engine dependency (conatus owns the capability). The
  shopping list: Karras-2012 LBVH (`src_rbd_shaders/broad_phase/lbvh.rs`,
  723 lines, paper linked in-source), the complete GPU radix sort
  (`src_rbd_shaders/utils/radix_sort/`, 9 files, 757 lines), the
  bottom-up refit with atomic arrival counters, the Windows+NVIDIA
  atomic-load workaround (wgpu#9221), and the radix-sort bounds-check
  caveat noted in `nexus_rbd2d/build.rs`.
- **Tracer.** In-house (the brick tracer exists). Its algorithm
  standard is Amanatides & Woo 1987 (voxel DDA); brickmap residency
  follows the van Wingerden brickmap thesis; NanoVDB is the reference
  layout for GPU-resident sparse voxel grids, to align with or diverge
  from knowingly.
- **Meshing (bake paths only).** The bonsairobo family
  (`fast-surface-nets-rs`, `block-mesh-rs`, `ndshape`) is the pure-Rust
  prior art; surface nets per Gibson 1998. Raymarch stays the hot
  path; meshing is for bakes and Isometry sprites.
- **Worldgen.** `noise` 0.9 is Apache-2.0/MIT (**verified**);
  `fastnoise-lite` as alternative. WFC per Gumin (MIT) with the
  in-house Isometry experience as the hard-rule layer under any
  learned prior. NCA per Mordvintsev et al. (distill.pub 2020).
  Diffusion worldgen stays refused for shipping; NCA plus decoder is
  the shippable shape.
- **Field simulation and fluids.** DIY on the standards: Stam's stable
  fluids and Bridson's course notes for grids; lattice Boltzmann as
  the grid-native alternative. salva stays refused (CPU SPH against a
  resident-grid design, and it lags rapier).
- **Agents.** Burn-native modules (GRU, autodiff in dev builds);
  evolution strategies per Salimans et al. 2017. Semantics are
  governed by the general-model plan, never imported from an RL crate.
- **Content interchange.** `dot_vox` is MIT (**verified**);
  MagicaVoxel `.vox` is the de facto interchange standard for voxel
  content packs.
- **Persistence and replication.** Owned: muniment/codicil for the
  store, retinue/personae for peers and identity (the dogfood rule;
  the Syncthing precedent). Minecraft's documented palette-section
  encoding is prior art for the idea of palette compression; the
  format itself is not adopted. Content addressing on the BLAKE3 row.
- **Physics.** parry `Voxels` derives tactile queries from committed
  revisions, under this wing's own three-tier ruling (integer
  authority, parry advisor, rapier in reserve). The mere canvas side
  (seiche) is unaffected.

Three flags:

- **Veloren is GPL-3.0**: read its worldgen writeups for technique,
  never copy code.
- **ECS adoption is deferred.** mesocosm-core owns entity state with
  replay hashes; adopting bevy_ecs or hecs now would risk exactly the
  second world representation the stop rules refuse. SoA columns
  aliased as tensors is a view over owned state, not a storage
  adoption; revisit only if a concrete need survives that framing.
- **Nothing else in the critique's component list is auto-adopted.**
  Each entry composes through the seam or waits for a consumer.

## Stop rules

- No second world representation inside any consumer; a consumer that
  wants one is asking for a view type instead.
- No CPU whole-plane reads in the per-tick path; durability reads are
  deltas, hashes, checkpoints, reductions.
- Candidate planes never feed collision, persistence, or peers; only
  committed revisions do.
- A view is promoted to a shared contract only after multiple real
  consumers prove it (the lease's promotion rule).
