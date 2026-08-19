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
| Burn to CubeCL to renderling needs only a few fork lines. | Direction valid, scope understated. | The allocation API is small, but lifetime, capacity, count, ABI, and draw submission form the actual proof. |
| Parry or Nexus can collide with these forms. | Split answer. | Parry can query committed voxel truth. Nexus can later simulate baked dynamic shapes after an interop proof. |
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

### R1. Shared traversal profile

Run one brick/DDA data contract under a Mesocosm slab camera and a Paredros
perspective camera. Vessel-specific lighting and LOD policy may differ.

**Done when:** both headed views consume the same traversal implementation and
brick ABI; captures and timings are recorded; no renderer-wide abstraction is
introduced merely to hide their policy differences.

### R2. GPU mesh bake into renderling

Add the smallest safe GPU-only range allocation seam, extract one resident
field with CubeCL, and draw the resulting compacted vertices through
renderling.

**Done when:** the receipt proves buffer identity, contiguous capacity,
overflow behavior, compacted draw count, slab-growth invalidation, zero CPU
vertex staging, and a headed capture. The receipt decides full `Vertex` versus
a compact procedural ABI.

### R3. Committed voxel collision

Project one Ground revision into Parry `Voxels`, apply a later Ground delta,
and compare ray, point, and contact results with exact occupancy.

**Done when:** unchanged regions remain untouched, changed voxels alter the
expected queries, revision mismatch is refused, and replay produces the same
collision answers. Only then test a learned field through propose, constrain,
commit, and collision projection.

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

- whether procedural extraction targets renderling's full `Vertex` or a
  compact fork-owned ABI;
- whether the first evolved controller runs fixed-point on the authority path
  or proposes recorded intents from a floating-point batch;
- the exact shared spatial contract for contemporaneous construction across
  vessels;
- the Paredros scene and scale that define the first residency budget.

These are not prerequisites for the Mesocosm epoch slice. Each has a consumer
and a receipt that can decide it without inventing an engine in advance.
