# Body Pipeline and Host Probe

**Status: plan, 2026-07-30. Nothing implemented.** Answers the question "can
we plan a render lane usable by all three vessels?" — and the answer is no,
not at that layer, but there is a shared organ underneath it that is worth
planning now. Landscape and candidate inventory:
[engine and render lane landscape](2026-07-30_engine_and_render_lane_landscape.md).

---

## 1. Why not a shared render lane

The three vessels want genuinely different renderers: a 2.5D voxel world
composited into a genet host, a close-camera 3D world, and a locked isometric
2D board drawn through a DOM today. Forcing one renderer across those means
either crippling Paredros or dragging Isometry off a shipped lens, and the
sharing rules say plainly that renderer and camera are per-vessel.

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

### R0 — The body format
Presentation-neutral topology: parts, attachment frames, per-part provenance,
mass hints. Serde, no host dependencies, no wgpu.

**Done when** `isometry-voxel` can bake a sprite *from* a topology document,
and the document round-trips without losing provenance.

### R1 — Live attachment (the hypothesis test)
One critter, one world, 2.5D. Eat a part; it attaches, gains a collider,
changes the body's mass and balance, and reads clearly on screen.

**Done when** a viewer who did not watch the eating can point at the new part
and say what it used to be — and the critter visibly handles differently
afterwards.

### R2 — The host probe
`mesocosm-core` behind a seam; two hosts over it (custom genet/2.5D lane and
an engine lane); same seed, same recorded input trace. Record every receipt in
the landscape §4 list.

**Done when** the receipts exist and are written into this plan's Findings,
including at minimum: state-hash equivalence, replay equivalence, whether
either host spawns a second wgpu device, and the adapter-code cost of each.

### R2a — The storage experiment
Inside the winning host only: `mesocosm-core`'s own storage versus Bones ECS.
This tests the layer that is actually undecided, where R2 mostly tests host
ownership.

**Done when** one of them is chosen with a stated reason, or Bones is ruled
out with one.

### R3 — Projection backends
Mesocosm's live 2.5D projection and Isometry's baked-sprite projection, both
deriving from the same R0 topology.

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

- **2026-07-30**: `isometry-voxel` is a build-time bake pipeline (.vox ingest,
  recipes, palette swaps, isometric sprite output). It does not demonstrate
  runtime part attachment, so R1 is a genuine unknown rather than an
  integration task.

---

## 7. Progress

- **2026-07-30**: plan written. No code.
