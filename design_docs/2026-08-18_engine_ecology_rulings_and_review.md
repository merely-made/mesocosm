# Engine and Ecology Rulings: Critical Review (2026-08-18)

**Status: provisional engine rulings, reviewed against the live code on
2026-08-18.** The vessel brief remains authority for product identity,
camera, and presentation. The resident-views composition plan remains
authority for the shared-device seam and the completed A-F proofs. This
document records the consequences of those decisions, corrects the
overclaims that accumulated in discussion, and names the next proof gates.

The short conclusion is that the architecture survives review. Its useful
shape is a record with several projections, one shared GPU allocation path,
a hybrid renderer, and deterministic resolution around learned or evolved
proposals. The claims that fail are the claims that turn a reusable mechanism
into a finished subsystem.

## 1. Evidence labels

- **Standing** means an existing ratified document or landed proof already
  carries the decision.
- **Working ruling** means this is the recommended shape, but a named proof
  may still overturn it.
- **Correction** means the discussion stated more than the evidence supports.

## 2. Conclusions that survive

### 2.1 One record, several projections

**Standing.** A vessel consumes facts, places, motifs, bodies, and provenance
from the shared record, then realizes them at its own scale. Mesocosm,
Paredros, and Isometry do not need a permanent interchange format for their
render chunks. Their voxel resolutions and presentation policies may differ.

This does not abolish shared spatial semantics. The record still needs stable
place and region identities, coordinates and scale transforms, material and
biome vocabulary, provenance, and projection-version identity. Exact player
edits such as a tunnel or base also remain exact facts even when a later
vessel renders them differently.

There are therefore two fidelity regimes:

1. **Deep-time projection.** Facts and motifs survive; each vessel regenerates
   its own spatial realization.
2. **Contemporaneous construction.** Exact committed edits survive; another
   vessel may simplify their appearance but must not silently change their
   meaning or topology.

The cross-vessel render-chunk problem is narrowed to a shared spatial and
historical contract. It is not deleted.

### 2.2 One resident allocation path

**Standing.** `ResidentChunk` is the seam between exact world state, compute,
rendering, and persistence. CubeCL owns allocation outward; raw wgpu and Burn
views refer to the same allocation; revision and valid-read epochs guard use.
The seed crystal proved CubeCL growth and DDA rendering over one resident
chunk with zero CPU voxel or pixel transfer per frame.

The plane type matters:

- exact occupancy and material planes remain integer or fixed-point authority
  and are consumed by raw CubeCL kernels;
- Burn is a good producer and consumer of fixed-shape floating-point fields,
  including density, distance, light, learned growth, and candidate state;
- constraints and the record decide which candidate result becomes an exact
  committed fact.

"Burn produces the field" is accurate for learned and derived floating-point
fields. It is not the ownership rule for every voxel plane.

### 2.3 Hybrid rendering

**Standing direction, incomplete product proof.** The engine should share the
brick layout and DDA traversal core, then give each vessel its own camera,
lighting, LOD, composition, picking, and readability policy.

- Mesocosm uses an orthographic terrarium section with a near/far slab and
  organism presentation composed over it.
- Paredros uses perspective traversal, sparse residency, and clipmap or mip
  policy for continuous pull-back.
- Isometry may use orthographic traversal where live volume matters and baked
  sprites or meshes where a stable tabletop asset is cheaper.

Ground remains raymarched on the hot path. Renderling remains the mesh tenant
for dynamic bodies, baked surfaces, and impostors. Netrender owns composition.
This is shared traversal, not one complete renderer with three camera
matrices.

### 2.4 Fields, extraction, and draw

**Working ruling.** For a floating-point candidate or derived field, the bake
path is:

```text
Burn tensor field
    -> CubeCL count / prefix / scatter extraction
    -> GPU vertex range plus draw count
    -> renderling draw
```

The direction is correct and keeps data on one device. Stock renderling does
not yet expose the required uninitialized GPU-only vertex allocation. The
missing surface also crosses craballoc/crabslab concerns: contiguous capacity,
slab growth and lease invalidation, overflow reporting, compacted count, and
indirect or otherwise bounded draw submission.

Renderling's stock `Vertex` is 26 words. A procedural bake should measure
whether that ABI is acceptable before making it the universal extraction
target. A compact procedural vertex ABI may be the better fork seam.

This path is for bakes, colliders that require meshes, Isometry assets, distant
LOD, and forms that cannot stay raymarched. It is not a reason to mesh the
resident ground every frame.

### 2.5 Collision and physics

**Working ruling, with a sharp boundary.** Parry `Voxels` is the direct tactile
projection for committed exact occupancy. It has sparse chunks, an internal
acceleration structure, incremental voxel edits, and point, ray, contact, and
mass-property queries. A Ground revision or delta can update it at a bake
boundary. Terrain collision does not require surface extraction first.

A Burn-generated candidate field cannot become authoritative collision by
remaining GPU-only. It must pass constraints, commit to exact occupancy, and
then materialize the relevant delta for the CPU collision advisor. That
readback is a deliberate bake boundary, not a per-frame world copy.

Nexus remains gated on a real dynamic-body consumer. Its current GPU rigid
body path accepts triangle, polyline, convex, and primitive forms, but it does
not consume Parry `Voxels` directly. Complex body geometry is initialized from
CPU-side Parry shapes, and the current append path is more restricted than the
initial build. Nexus can become the dynamic rigid-body engine after an interop
proof; it is not the terrain-collision answer today.

### 2.6 Evolved behavior

**Working ruling.** The useful part of Job Talle's
[neuroevolution experiment](https://jobtalle.com/neuroevolution_in_squids.html)
is the co-evolution of body affordances and a small inherited controller. Its
current squid network has no sensors and evolves autonomous locomotion against
a short generation score, so it is not yet a model of ecological behavior.

Mesocosm's safe seam is narrower:

```text
body + visible traits
    -> trait-gated senses and drives
    -> inherited, bounded policy
    -> proposed intent
    -> deterministic movement and ecology resolver
    -> authoritative events
```

The first fauna proof should replace target ranking inside the existing
movement policy. It must not replace terrain legality, movement costs,
resource conservation, event emission, or replay authority. Start with a
fixed small recurrent topology, quantized weights, trait-gated sensors and
effectors, and an inspectable decision trace. Let survival and reproduction
provide selection rather than inventing one global fitness score.

Flora should not be forced through the same controller. A developmental
grammar is the better source of form; a slow policy can allocate resources and
choose growth direction. Near fauna may run individual controllers while far
cohorts retain aggregated tendencies. Burn can batch candidate evaluation at
world generation or epoch review, but GPU floats remain proposals unless the
chosen intent or quantized state is recorded deterministically.

The visible trait graph remains the player-facing genome. An evolved policy
may express those traits; opaque weights must not bypass them.

### 2.7 The trait board

**Working ruling.** Mere's graph canvas, typed nodes, springs, exclusions, and
support queries make it a plausible substrate for the epoch review board. It
is not a completed review UI. A real Mesocosm adapter still has to prove trait
adjacency, contradiction and synergy feedback, preview of the compiled body,
undo, commit, keyboard access, and a replay-stable result.

### 2.8 Record, time, and peers

**Standing direction.** The three vessels can share an intent and event
grammar while keeping different schedulers: turn order, fixed real-time
steps, and epoch batches are not interchangeable clocks. A log alone is not a
runtime. Cross-host authority also needs causal order, ruleset and evaluator
version identity, conflict rules, snapshots and compaction, and spatial
invalidation for regeneration.

Integer and fixed-point state is the default for authoritative ecology across
hosts. That is a discipline, not an ontology. Exact rational forms or recorded
resolved outcomes are also valid when they replay identically. Floating-point
GPU results remain derived fields or proposals.

## 3. Critical corrections

| Discussion claim | Verdict | Corrected ruling |
|---|---|---|
| Broad-strokes fidelity kills the cross-vessel chunk problem. | Overstated. | It removes a permanent render-chunk interchange format. Shared spatial semantics and exact committed edits remain. |
| One tracer, three cameras. | Useful shorthand only. | Share brick layout and traversal. Each vessel owns rendering policy beyond camera math. |
| Mesocosm's renderer is nearly free. | Rejected. | Its ground renderer is cheaper than Paredros's, but organism presentation, section readability, picking, and epoch UI remain product work. |
| The trait board is Mere's canvas. | Plausible substrate. | It becomes true only after one Mesocosm review adapter completes a commit and replay receipt. |
| One intent log solves three time models. | Overstated. | The grammar can be shared; scheduling and conflict semantics remain distinct. |
| Every authoritative number is integer or fixed-point. | Good default, too absolute. | Require cross-host replay identity; integers, fixed point, exact forms, or recorded resolution may satisfy it. |
| Burn to CubeCL to renderling needs only a few fork lines. | Direction valid, scope understated. | The allocation API is small, but lifetime, capacity, count, ABI, and draw submission form the actual proof. R2 also found that CubeCL 0.10 cannot import renderling's slab allocation: the live path is one device, two allocators, and one device-local publication copy. |
| Parry or Nexus can collide with these forms. | Split answer. | R3 proves Parry ray, point, and contact-manifold queries over revision-gated committed occupancy. Nexus can later simulate baked dynamic shapes after an interop proof. |
| Squid-style neuroevolution supplies flora and fauna behavior. | Promising mechanism, wrong authority. | Evolve a bounded proposal policy behind existing deterministic ecology resolution; treat flora separately. |

## 4. Authority and projection flow

```text
shared record / Ground exact facts
    -> ResidentChunk exact and fixed planes
       -> CubeCL authoritative transforms
       -> Burn floating-point fields and candidates
          -> constraints + explicit commit
             -> new exact record revision

committed resident revision
    -> brick DDA                 hot ground image and depth
    -> CubeCL extraction         mesh bakes, bodies, LOD assets
       -> renderling             raster tenant
    -> Ground delta
       -> Parry Voxels           tactile and collision advice
    -> later Nexus adapter       dynamic rigid bodies only
```

Neither rendering, Burn, Parry, nor Nexus owns world truth. Each consumes a
revisioned projection with an explicit validity boundary.

## 5. Recommended build order

The next product target should be a Mesocosm epoch slice rather than a generic
engine program: one terrarium section, one controlled organism, legible local
ecology, an end-of-epoch trait-board decision, and deterministic replay. It is
the smallest slice that makes the engine, ecology, and review UI answer to one
consumer.

The engine proofs should be pulled by that slice:

### R1. Shared traversal profile: **COMPLETE 2026-08-20**

Run one brick/DDA data contract under a Mesocosm slab camera and a Paredros
perspective camera. Vessel-specific lighting and LOD policy may differ.

**Done when:** both headed views consume the same traversal implementation and
brick ABI; captures and timings are recorded; no renderer-wide abstraction is
introduced merely to hide their policy differences.

**Receipt, 2026-08-20.** `BrickTracer` now accepts a camera-neutral ray
basis: origin, forward, right, and up, plus projection kind and far cut. Its
WGSL keeps one pointer lookup, material atlas, ray-box intersection, and DDA;
only ray construction differs. The old `Flight` constructor remains a
perspective adapter, so the existing G2, carve, lease, and downlevel receipts
retain their source contract.

Mesocosm's real headed G2 frame now supplies a 16-voxel orthographic slab at
the ratified terrarium angle. On the RTX 4060 Laptop GPU at 1920×1080, frame
32 recorded 3 µs tracer preparation, zero steady brick and uniform upload,
and 1.238 ms in netrender's reported frame total. The inspected capture and
JSON receipt are `Code/testing/mesocosm/r1_terrarium.{png,json}`.

Paredros's real S0 room host consumes the same `mesocosm_lens::BrickTracer`
behind its opt-in `r1-proof` feature while retaining its own close-perspective
eye and target. Its 64-frame 1280×720 headed run recorded 11.601–34.646 ms
overall, 12.552 ms median, zero steady brick upload, 2,357 capture colours,
and the unchanged position-log hash `0x27a905731c6bfc61`. Both receipts name
the same Rust `BrickMap` ABI: origin `[-9,0,-8]`, pointer extent `[18,3,17]`,
atlas extent `[128,16,128]`, 3,672 pointer bytes, and 262,144 atlas bytes.

This closes traversal reuse, not renderer unification. Mesocosm owns the
slab; Paredros owns the perspective rig, torch, and later LOD. The Paredros
dependency is disabled by default and proves that the current
`mesocosm-lens` owner is now too low: the next permanent adoption must lift
the traversal organ to an actual platform owner. R1 does not choose that
owner, create a renderer-wide camera abstraction, or claim the later
raymarch-depth/renderling join.

### R2. GPU mesh bake into renderling: **COMPLETE 2026-08-21**

Add the smallest safe GPU-only range allocation seam, extract one resident
field with CubeCL, and draw the resulting compacted vertices through
renderling.

**Done when:** the receipt proves buffer identity, contiguous capacity,
overflow behavior, compacted draw count, slab-growth invalidation, zero CPU
vertex staging, and a headed capture. The receipt decides full `Vertex` versus
a compact procedural ABI.

**Receipt, 2026-08-21.** Craballoc now allocates a contiguous GPU-only array
without queuing initial CPU values. Renderling exposes that capacity as
`Vertices<GpuOnlyArray>` and refuses a bounded draw count above it while
retaining the full allocation lease.

The probe generated a 48 by 48 floating field through Burn, then ran CubeCL
count, exclusive prefix, and scatter passes. It produced 7,182 vertices, equal
to the CPU count oracle. A deliberate 7,181-vertex capacity raised the GPU
overflow flag and renderling separately refused `capacity + 1`. The retained
range was 13,824 vertices, or 359,424 contiguous words. Forced slab growth
invalidated the old buffer, produced a new identity, and publication targeted
the buffer attached on the following commit. The allocation queued zero CPU
vertex values.

The discussion shorthand was wrong about one allocator. CubeCL 0.10 can reuse
the host's wgpu device, but its compute server cannot import an arbitrary
renderling slab buffer as a CubeCL handle. The proven path is therefore:

```text
Burn field
    -> CubeCL count / prefix / scatter in CubeCL-owned memory
    -> one device-local buffer copy
    -> GPU-only renderling range plus bounded draw count
```

Vertex contents never cross the CPU. Two u32 values, compacted count and
overflow, return at the asynchronous bake boundary. This is one device and two
allocators, not the claimed one-device/one-allocator path.

On the RTX 4060 Laptop GPU the final headed run drew the 7,182 vertices with 63
distinct scene-region colours and 31.6% non-background coverage. Full
renderling `Vertex` cost 746,928 bytes; a ten-word procedural position/colour/
normal ABI would cost 287,280 bytes, 2.6 times less. The ruling is to keep full
`Vertex` for bounded asynchronous bakes because it draws through stock
renderling after the small allocation seam. It is not the universal procedural
ABI. A compact path must be pulled by a high-density bake that proves the
bandwidth and shader complexity are worth a second draw contract.

### R3. Committed voxel collision: **COMPLETE 2026-08-21**

Project one Ground revision into Parry `Voxels`, apply a later Ground delta,
and compare ray, point, and contact results with exact occupancy.

**Done when:** unchanged regions remain untouched, changed voxels alter the
expected queries, revision mismatch is refused, and replay produces the same
collision answers. Only then test a learned field through propose, constrain,
commit, and collision projection.

**Receipt, 2026-08-21.** The standalone `crates/probes/parry-ground` receipt
consumes only public `Ground` revision, dirty-brick, key, material, and
occupancy APIs. It projected 136 stored Ground bricks into Parry 0.29
`Voxels`: 69,632 cells compared and 41,763 occupied voxels. Parry's internal
3D chunks are also 8 cubed, so they align with Ground's brick grid.

One boundary carve committed Ground revision 0 to 1 and removed 19 voxels
across four dirty bricks. The projection rescanned exactly those four regions,
2,048 cells, and changed exactly the 19 committed occupancies. The other 132
region occupancy signatures stayed unchanged. Parry's `set_voxel` maintained
the cross-brick neighbor masks needed to suppress internal collision edges;
the adapter did not rescan adjacent Ground materials.

The committed delta moved a downward ray hit from 2.5 to 4.5 voxel units,
changed point containment from true to false, and cleared a ball contact
manifold from one contact to zero. The ray result matched an exact Ground
occupancy oracle before and after. A stale source revision and a skipped target
revision were both refused before mutation. Re-growing the same seed and
replaying the carve reproduced the whole query receipt bit for bit.

Two API boundaries matter. Ray and point queries are direct `Voxels` traits,
while contact uses Parry's persistent contact-manifold dispatcher; its simpler
single-contact helper does not dispatch voxel shapes in 0.29. The finite
`Voxels` projection covers stored Ground bricks at `y >= 0`. Ground's implicit
solid bedrock below zero is an analytic half-space, not an infinite voxel
allocation.

This remains a host-side projection. `mesocosm-core` stays integer-only and
Parry-free. Promotion waits for the first tactile gameplay consumer to name the
adapter's permanent host crate; R3 proves the contract rather than choosing
that owner early.

### B1. Bounded evolved fauna policy

Insert one quantized recurrent policy at the existing target-choice seam.

**Done when:** fixed seeds replay identically, the decision trace names the
traits, sensed facts, and selected drive, the established behavior receipt
still passes, and distant cohort conservation is unchanged. A useful strange
behavior is evidence; a higher synthetic score alone is not.

### V1. Residency budget under Paredros scale

Let a concrete continuous-zoom scene pull the residency policy. Do not design
a universal pager without the scale pressure that makes its tradeoffs visible.

**Done when:** the scene records visible range, resident bytes, upload and
eviction cadence, camera hitch behavior, and recovery after rapid zoom or
travel.

## 6. Open choices carried to proof

- which tactile gameplay host promotes the proved Ground-to-Parry adapter;
- whether the first evolved controller runs fixed-point on the authority path
  or proposes recorded intents from a floating-point batch;
- the exact shared spatial contract for contemporaneous construction across
  vessels;
- the Paredros scene and scale that define the first residency budget.

These are not prerequisites for the Mesocosm epoch slice. Each has a consumer
and a receipt that can decide it without inventing an engine in advance.
