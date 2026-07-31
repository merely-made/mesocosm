# Body Pipeline and Host Probe

**Status: plan, 2026-07-30. Nothing implemented.** Answers the question "can
we plan a render lane usable by all three vessels?" — and the answer is no,
not at that layer, but there is a shared organ underneath it that is worth
planning now. Landscape and candidate inventory:
[engine and render lane landscape](2026-07-30_engine_and_render_lane_landscape.md).

---

## 1. Why not a shared render lane

The three vessels currently point toward genuinely different renderers: a
2.5D or 3D Mesocosm world, a close-camera 3D Paredros world, and a locked
isometric 2D Isometry board drawn through a DOM today. Perspective is now a
lens choice rather than a person rule, so later convergence is welcome if a
second consumer proves it. Forcing one renderer before that would either
cripple Paredros or drag Isometry off a shipped lens, and the sharing rules say
plainly that renderer and camera are per-vessel.

The instinct behind the question is still right, though. Something *is* shared
and it is one layer down:

> **Every vessel has to draw the same creatures.** A critter bred in Mesocosm
> appears as a companion in Paredros and a token in Isometry, and it must be
> recognisably itself in all three.

That shared organ is the **body pipeline**: a presentation-neutral description
of a critter's body, plus per-vessel projections of it. Not a renderer. A
format and a small amount of geometry work, with three backends.

This also happens to be where the wing's biggest unproven assumption lives, so
planning it first buys down the most risk per unit of effort.

---

## 2. The unproven assumption

`isometry-voxel` ingests `.vox`, applies recipes and palette swaps, and bakes
isometric sprites. That is a *build-time appearance pipeline* and it is
shipped and real.

Nothing in it demonstrates the thing Mesocosm's keystone requires:

> An incorporated part attaches to a living body **during play**, acquires
> collision and mass, rotates with its parent, moves the center of balance,
> and stays visually legible.

"Voxel bodies still work in 2.5D" is a hypothesis. Until a part can be eaten
and worn in the same session, incorporation-with-provenance is a design on
paper. **M0 must exercise attachment and changed physics, not merely walk one
prebuilt critter around.**

Three plausible techniques, to be chosen by probe rather than argument:
jointed voxel-part sprites; layered textured meshes with per-part transforms;
or rebaking the body during the adaptation phase, which is cheap precisely
because the epoch loop already has a natural seam where bodies change.

---

## 3. The portable artifact

What crosses vessels is **not** a mesh and **not** a sprite:

| Field | Why |
| ----- | --- |
| Body topology | Parts, attachment frames, parent/child structure |
| Per-part provenance | What each part used to be — the keystone, and Law A's raw material |
| Mass and collision hints | So physics is derivable without shipping a collider format |
| Loud inherited signatures | Law B's pointable few, so a village reads as *yours* |
| Optional projection recipes | `isometry-voxel` recipes among them, as a hint rather than a requirement |

`isometry-voxel` recipes are **not canonical**. They are an excellent first
projection codec and probably Isometry's, but making them the format lets
today's renderer leak into the substrate. Each vessel derives its own
presentation from topology.

This artifact is the concrete content of interchange profile v0, which the
wing founding record names as the next architectural threshold. Planning the
body pipeline and planning the profile are the same work.

### Authoring input, canonical body document, and caches

**Ruled 2026-07-31 after reviewing the rigid-part rendering proposal.**
MagicaVoxel `.vox` is the first authoring and import format, not the
interchange schema. The importer may read reserved marker voxels to discover
sockets and orientation, but it strips those markers and writes explicit
attachment frames into the canonical body document. This avoids making
MagicaVoxel palette behavior part of the protocol.

The canonical shape is:

- content-addressed part-volume references
- an explicit parts graph with attachment frames
- palette/material roles and overrides
- physical hints and capability references
- per-part and cross-part provenance
- biological-lineage and world-provenance references

A physical part is a useful unit of inheritance, animation, collision, and
mesh invalidation, but it is not the only semantic unit. One trait may span
several parts; one symbiont may inhabit a whole body; one part may carry
several layers of provenance. The body document therefore refers to traits
and capabilities rather than forcing each to equal exactly one meshable part.

Merged meshes, colliders, sprite sheets, and PNGs are content-addressed
**derived artifacts**. They may be distributed and reused, but none becomes
the editable source of truth.

### Rendering posture

These are defaults for the probe, not permanent renderer law:

- **Greedy-meshed voxel geometry is the leading live-render candidate.**
  Remesh a dirty part when mutation or damage changes its volume. Per-frame
  remeshing is not assumed and every performance claim is measured on the
  target hardware. Raymarching remains a replaceable projection experiment
  for a future workload that actually benefits from massive static volume.
- **Rigid-part transforms are the baseline animation model.** Sockets,
  pivots, oscillators, and procedural IK fit unpredictable bodies and let one
  part mesh be reused. Skinning or another deformation projection remains
  available for tentacles, soft bodies, cloth, faces, and characters whose
  expression proves the need. The portable body document does not prohibit
  it.
- **Sprite baking is projection and distribution.** Isometry's CPU baker,
  an on-demand local bake, a curated touched-up sheet, and a live orthographic
  projection may all derive from the same body document. A cache key includes
  the canonical source digest, palette/material roles, resolved transforms,
  projection parameters, and baker version.
- **Determinism is stated honestly.** The existing reference CPU baker can
  be required to emit byte-identical pixels. A GPU bake on heterogeneous
  drivers is not presumed pixel-identical. Peers may share a signed baked PNG
  and its bake receipt, or generate a semantically equivalent local cache.
  Gameplay and artifact identity depend on the body document and receipt,
  never on accidental GPU raster equivalence.

Chunked destructible world terrain is a Paredros or Mesocosm world-renderer
decision, not part of the shared body pipeline. The first proof moves one
body across vessels before it generalizes the level format.

---

## 4. The missing middle, and who extracts it

The landscape's §2 correction stands: the stack owns a host skeleton, not a
game runtime. Missing and load-bearing are fixed-timestep simulation, an
authoritative world, snapshot/replay, input actions, an asset graph, scene
representation, camera and animation, spatial queries, game audio, and
inspection.

**Mesocosm builds these for itself first.** No shared runtime crate is minted
in advance — the wing's own extraction discipline says a shared component is
pulled out when a second consumer pulls on it, and Paredros is not yet a
consumer of anything. What Mesocosm should do is build them *behind a seam*
so extraction is later possible:

```text
mesocosm-core     deterministic rules, traits, metabolism, lineage,
                  input intents. No rendering, no host, no wgpu.
mesocosm-runtime  fixed step, snapshot/replay, asset graph, input actions.
                  The extraction candidate.
mesocosm-<host>   winit/genet or engine. Per-lane, disposable.
```

If Paredros later wants `mesocosm-runtime`, it gets renamed and extracted
then, with two real consumers justifying it. If it never does, nothing was
over-built.

---

## 5. Phases

Done-conditions, not estimates. R-phases interleave with the M-phases in the
founding plan; R0–R2 are prerequisites for a meaningful M0.

> **Ordering authority moved 2026-07-31** to the
> [execution waves plan](2026-07-31_execution_waves_plan.md). This section still
> owns *what each phase is*; that plan owns *when they happen and in what
> order*, and adds two constraints that only appear once the order is fixed:
> the **confound rule** (both hosts initially stage the same enclosure, so the
> probe measures host rather than host-plus-perspective) and the placement of
> the Bones experiment **inside the winning host** rather than as its own lane.
> Where the two disagree on sequence, the waves plan wins.

### R0 — The body format, and the determinism constraint
Presentation-neutral topology: parts, attachment frames, per-part provenance,
mass hints. Serde, no host dependencies, no wgpu.

**And the constraint that has a deadline.** `mesocosm-core` must be a **pure
function of (seed, ordered inputs)**, behind a boundary whose whole state can
be captured at once. Adopted 2026-07-30 after studying
[Tangle](https://github.com/kettle11/tangle), which gets rollback multiplayer
without the author writing any netcode, because WebAssembly's linear memory
makes "capture the world" a memcpy rather than hand-written serialisation the
author can forget a field of.

One discipline buys five things: **co-op, replay, save/load, time-travel
debugging, and R2's host comparison** are the same mechanism seen from
different angles. Note that R2 already requires state-hash and replay
equivalence between two hosts, so **the engine probe doubles as the co-op
feasibility test** — that was an accident of design worth keeping on purpose.
This keeps rollback feasible; it does not authorize rollback netcode before a
real co-op mode and carrier prove they need it.

Practical implications to hold from the first commit: no ambient clock reads
in core, no unordered iteration affecting simulation, all randomness from the
seeded stream, and physics behind a seam because **cross-platform float
determinism is the classic killer** (rapier's `enhanced-determinism` comes
with caveats rather than guarantees, and is a probe target rather than an
assumption). Whole-heap snapshotting also scales with heap size, and an
ecology with a species roster is precisely the large-mutable-state profile
that makes it expensive — so measure before relying on it.

**Done when** `isometry-voxel` can bake a sprite *from* a topology document,
the document round-trips without losing provenance, and a recorded input
trace replayed against the same seed produces an identical state hash.

### R1 — Live attachment (the hypothesis test)
One critter, one world, in the live projection selected for the probe. Eat a
part; it attaches, gains a collider, changes the body's mass and balance, and
reads clearly on screen.

**Done when** a viewer who did not watch the eating can point at the new part
and say what it used to be — and the critter visibly handles differently
afterwards.

> **R2 and R2a superseded 2026-07-31.** The engine lane is dropped and the
> render lane is decided: a small custom wgpu body renderer with netrender
> owning the device and compositing. There is no second host, so R2 is no
> longer a comparison; its receipts survive as absolute observations of the one
> lane. R2a is unblocked from the host question and deferred behind playfeel
> instead. Reasons, and what dropping the comparison costs, are recorded in the
> [execution waves plan](2026-07-31_execution_waves_plan.md) §1.3. The sections
> below are kept as written because they define what the receipts *are*.

### R2 — The host probe
`mesocosm-core` behind a seam; two hosts over it (a custom Genet lane and an
engine lane); same seed, same recorded input trace. Each host may constrain
the camera differently, but both consume the same body and intent documents.
Record every receipt in the landscape §4 list.

**Done when** the receipts exist and are written into this plan's Findings,
including at minimum: state-hash equivalence, replay equivalence, whether
either host spawns a second wgpu device, and the adapter-code cost of each.

### R2a — The storage experiment
Inside the winning host only: `mesocosm-core`'s own storage versus Bones ECS.
This tests the layer that is actually undecided, where R2 mostly tests host
ownership. **Confirmed 2026-07-31**: Bones does not get its own lane, because
running it as a third host would mix an ECS choice with a renderer choice and
return one muddy answer for two questions.

**Done when** one of them is chosen with a stated reason, or Bones is ruled
out with one.

### R3 — Projection backends
Mesocosm's winning live projection and Isometry's baked-sprite projection,
both deriving from the same R0 topology.

**Done when** one critter appears in both vessels, recognisably itself, with
neither vessel owning the other's renderer.

### R4 — Extraction review
Decide what, if anything, becomes a shared runtime crate — with Paredros as
the second consumer or not at all.

**Done when** the seam is either extracted with two consumers named, or
explicitly declined in writing.

---

## 6. Findings

*Verified facts discovered during the work, dated, with references.*

- **2026-07-31: attachment frames need a per-part pivot.** `Attachment` today
  carries an offset and a yaw, and a part's local origin is its **lowest
  corner**. That makes an offset corner-to-corner rather than socket-to-socket,
  so attaching flush requires knowing the part's size, and a yaw turns the part
  about its corner so a flush limb swings off its joint. §3's portable artifact
  should carry a pivot per part: the point an attachment frame measures from and
  a rotation turns about. Found by rendering, not by testing; every visual
  assertion passed while limbs floated beside the torso. Deferred until real
  authored parts exist, since the convention is easier to choose against them.

- **2026-07-30**: `isometry-voxel` is a build-time bake pipeline (.vox ingest,
  recipes, palette swaps, isometric sprite output). It does not demonstrate
  runtime part attachment, so R1 is a genuine unknown rather than an
  integration task.
- **2026-07-31**: `.vox` is retained as the first authoring format while the
  portable source becomes an explicit parts graph over content-addressed
  volumes. Greedy meshes, rigid transforms, and sprite bakes are first
  projections rather than substrate law; cross-driver pixel identity is not
  required.

---

## 7. Progress

- **2026-07-30**: plan written. No code.
- **2026-07-31**: incorporated the renderer proposal at the body-pipeline
  boundary and kept terrain, deformation policy, and GPU bake identity out of
  the portable schema.
