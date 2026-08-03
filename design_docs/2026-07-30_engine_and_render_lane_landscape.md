# Engine and Render Lane: Landscape

**Status: research, 2026-07-30. No decision.** Rewritten the same day after a
review found two systematic faults in the first draft: it counted an
ingredient shelf as a nearly finished engine, and it let "engine" mean two
different things in one document. Both are fixed below. The decision itself
and its probe live in
[the body pipeline and host probe plan](2026-07-30_body_pipeline_and_host_probe_plan.md).

External facts were re-verified on 2026-07-30 against crates.io and GitHub.
Version numbers move; recheck before committing.

---

## 0. What is actually being chosen

Three vessels with three different presentation needs. They do not have to
share a renderer — see §5 for what they *must* share.

Working dimensionality, proposed by Mark and **not yet ruled**:

| Vessel | Proposed | Notes |
| ------ | -------- | ----- |
| Mesocosm | 2D or 2.5D, rapier2d | A large simplification, and it rides rendering the stack already has. |
| Paredros | 3D, close camera | The heaviest renderer requirement in the wing. |
| Isometry | 3D, distant camera | **Conflicts with a standing ruling.** See below. |

**Camera is not person.** The first draft wrote "Paredros: 3D, first person",
which collides two vocabularies. The wing's person grammar describes *agency*
— Paredros is second person because companions are peers you address rather
than units you command. That is orthogonal to where the camera sits. A
second-person game can use a first-person camera; saying "first person" about
Paredros invites exactly the drift the person grammar exists to detect. Say
**camera distance** when discussing renderers, and **person** only when
discussing agency.

**Isometry caveat, flagged rather than adopted.** Isometry's `CLAUDE.md` says
camera freedom is explicitly not a near-term rendering task: the locked
isometric 2D lens is the shipped one, later 2.5D/3D modes are allowed because
voxel source models dissolve the facing-art explosion, *but they need their
own plan and render lane*. Isometry also renders through genet's DOM today, so
a 3D lens is a different renderer rather than a camera change. Moving Isometry
to 3D is a real scope change to a shipping project and belongs in a plan in
that repo, not as a side effect of this wing's choices.

If Mesocosm lands at 2.5D, the wing's hardest renderer is Paredros, which
inverts the founding record's assumption that vessel 1 is the render pressure
vessel.

---

## 1. Pure-Rust engines

Verified 2026-07-30.

| Engine | Version (registry) | Shape | License | Posture |
| ------ | ------------------ | ----- | ------- | ------- |
| **Bevy** | **0.19.0** (2026-06-19) | Data-driven ECS, 2D + 3D, ~44k stars, large plugin ecosystem | MIT/Apache | Too large to fork; the idiom is plugins. **Cost to book: Bevy documents frequent breaking releases**, and two minor versions landed in the months around this survey. |
| **Fyrox** | **1.0.1** (2026-03-28) | 2D/3D engine, **the only Rust engine shipping a real scene editor** | MIT | Forkable in principle. But its current renderer is `fyrox-graphics-gl`, which makes shared-device composition with genet a poor fit. Treat as a possible **Paredros** engine, not a reusable stack organ. |
| **ggez** | **0.10.0** (2026-06-03) | 2D framework, LÖVE-shaped; now explicitly wgpu-based with event, timing, resource, and audio defaults | MIT | An alternative *host*, not a missing component. |
| **Bones** | `bones_framework` 0.4.0 on crates.io (2024-09-12) — **but git-active, pushed 2026-04-24**, 305 stars, not archived | **Renderer-neutral game logic**: deterministic ECS, snapshots, asset server, schema reflection, Piccolo (Lua) integration. Bevy renderer optional | MIT/Apache | **The most interesting row.** Addresses the missing middle (§2) rather than the renderer. Piccolo is already Isometry's scripting engine, which is an uncanny adjacency. |
| **Renderling** | 0.4.9 on crates.io (2024-09-20) — **git-active, pushed 2026-07-19**, 238 stars | GPU-driven wgpu + rust-gpu renderer: PBR, glTF scene machinery, headless image tests | MIT/Apache | Young, but a far better custom-3D probe target than a bare `rend3` reference. |
| **Macroquad** | — | Minimal 2D | MIT/Apache | **Ruled out.** [RUSTSEC-2025-0035](https://rustsec.org/advisories/RUSTSEC-2025-0035.html): multiple soundness issues, unprincipled mutable statics enabling use-after-free from safe code, **all versions affected, no patched release**, and the advisory notes fixing them is not treated as a priority. |
| Ambient | — | Rust/WASM multiplayer engine | — | **Paused indefinitely.** An architectural *donor* to read, not a candidate to adopt. |
| Amethyst, Piston | — | Legacy | — | Dead. Amethyst's ECS lineage fed Bevy. |
| rend3, Nannou, Blue Engine, Tetra | — | Renderer / creative-coding / niche 2D | — | Superseded here by Renderling, or not a fit. |

**A note on registry-stale, git-active projects.** Bones and Renderling both
look abandoned on crates.io and are not: their GitHub repositories saw pushes
in April and July 2026 respectively. This stack already has the convention for
that situation — the cambium family is git-only by ruling, tracked by branch
rather than pinned — so a stale registry version is not disqualifying here the
way it would be elsewhere. It *is* a maintenance signal worth weighing.

---

## 2. The correction: the shelf is not an engine

The first draft claimed "eleven of thirteen components are already owned" and
concluded the stack owns most of a game engine's parts. That is
inventory-true and architecture-misleading, and the corrected claim is:

> The stack already owns most of the **platform-facing boundary** and several
> important simulation primitives. **The missing middle is a coherent game
> runtime.**

That is still a strong position, and worth stating precisely.

### What genuinely exists — a credible host skeleton

- winit window, input, device, and presentation ownership (`cambium-winit`, genet's winit host)
- a real 2D renderer with external-texture composition (`netrender`, vello backend shipped)
- native UI, layout, and text (`cambium`, `sprigging`, `genet-layout`, `parley`)
- a host-neutral actor boundary (`armillary`)
- audio proven in a real interactive genet application (Firewheel, via Hocket)
- persistence and append-only history primitives (`muniment`, `codicil`)
- rapier2d already in the tree (`seiche`)
- field algebra and evaluation (`numen`, `quint`)
- voxel recipes and sprite baking (`isometry-voxel`)
- two working application patterns to copy (Hocket; Isometry's pure-core / views / native-host split)

### What is missing, and is load-bearing

Not one of these is supplied by the shelf:

- fixed-timestep simulation and clock policy
- an authoritative gameplay world or ECS
- snapshot, checkpoint, and deterministic replay lifecycle
- input actions, rebinding, controllers, device churn
- asset dependency graph, hot reload, content addressing
- scene, prefab, and level representation
- camera, animation, particles, lighting, materials
- navigation and spatial queries above raw collision
- game audio mixing, emitters, streaming, spatialization
- diagnostics, inspection, simulation stepping
- packaging and content-build tooling

And three specific over-claims from the first draft, corrected:

- **`armillary` is an actor harness, not a game scheduler.** It does not
  supply a fixed timestep or a simulation clock policy.
- **`codicil` is a linear replay primitive, not a game deed log.** A typed
  deed log with schema evolution, indexes, checkpoints, and forks is work on
  top of it, and Law A's `(context, chosen, foregone, cause-link)` record is
  exactly that work.
- **`muniment` ships an in-memory backend and expects the host to supply
  durable storage.** Isometry supplies redb; a game must supply its own.
- **Firewheel is proven for Hocket's audio graph**, which is not the same as
  positional game audio for Paredros.
- **`numen`/`quint` are field mathematics**, not ecological scheduling or AI.

### The opportunity this reveals

Better than the first draft's version. Mesocosm can be the consumer that
extracts a small reusable **game runtime layer** sitting between the existing
host boundary and a game's rules — fixed step, input intents, snapshot and
replay, asset graph — while Paredros stays free to choose a heavier 3D engine
later. That is the stack's normal extraction pattern applied one layer up.

---

## 3. Approaches worth probing

1. **Bevy as-is.** Fastest to a playable M0; accept that it owns the loop, and
   book the breaking-release cadence as a real cost.
2. **Fyrox for Paredros specifically.** Gets an editor. Its GL renderer makes
   it a poor genet-composition citizen, so treat it as a self-contained host
   for the heavy-3D vessel rather than a shared organ.
3. **Custom loop, 2.5D, on the existing host skeleton.** `winit` +
   `netrender`/vello + `rapier2d`, with bodies rendered to a shared wgpu
   texture that netrender composites and cambium decorates. Maximum reuse,
   minimum new rendering, and it fits Mesocosm's proposed dimensionality.
4. **Custom loop, 3D.** `winit` + `wgpu` + a mesher + `rapier3d`, with
   **Renderling** as the probe target rather than a from-scratch renderer.
5. **Bones as the missing middle.** Renderer-neutral: it could supply the
   deterministic ECS, snapshots, and asset server *inside* lane 3, which is
   the layer that is actually undecided.
6. **Split the bet.** Mesocosm custom-2.5D, Paredros on a heavier engine
   later. Permitted by §5.

The architecture lane 3 implies, which the review sketched and I endorse:

```text
genet / winit kernel  (device, window, input, presentation)
    |-- cambium UI, inspection, chrome
    |-- mesocosm fixed-step world
    |     |-- rapier2d bodies
    |     |-- ecology + lineage systems
    |     `-- numen/quint fields
    `-- body renderer -> shared wgpu texture -> netrender composite
```

**The hidden gap in that diagram is the body renderer**, and it is the whole
subject of the plan doc. `isometry-voxel` bakes appearances; it has never
shown that an incorporated part can attach *during play*, acquire collision
and mass, rotate correctly, move the center of balance, and stay visually
legible. "Voxel bodies still work in 2.5D" is a hypothesis, not a finding.

---

## 4. How to decide

By probe, with one correction that matters: **both hosts must consume the same
`mesocosm-core`**, or the experiment compares two implementations of the game
instead of two hosts.

```text
mesocosm-core          (deterministic rules, traits, metabolism, lineage,
                        input intents — no rendering, no host)
    |-- mesocosm-genet  (custom 2.5D lane)
    `-- mesocosm-bevy   (engine lane)
```

Same seed, same recorded input trace, and record:

- final simulation-state hash
- save/reload and replay equivalence
- fixed-tick behaviour under uneven frame delivery
- input-to-present latency and frame pacing
- **whether one new body part can be attached and metabolized mid-run**
- rapier reconciliation and collider rebuilding cost
- lines of adapter code, and any duplicated ownership
- whether the host creates a second wgpu device
- headed inspection and debugging quality
- asset and content iteration behaviour

"Does the verb feel right?" stays the product judgment. These receipts say
whether a lane is helping or fighting it.

A **third, smaller experiment belongs inside the genet lane**: `mesocosm-core`
storage versus Bones ECS. Bevy-versus-genet mostly tests host ownership;
core-versus-Bones tests the layer that is genuinely undecided.

---

## 5. What must stay shared, restated

The first draft said "renderer choice is free" and then listed a shared
*world model*, which was too strong in one direction and too vague in the
other. Corrected, with the sharing rule made explicit per layer:

| Layer | Sharing rule |
| ----- | ------------ |
| World identity and fact substrate | **Shared** |
| Identity, provenance, deed vocabulary | **Shared** |
| Portable critter profile | **Extracted from two consumers** |
| Game simulation runtime | May share crates, may differ |
| Host shell and event loop | Per vessel |
| Renderer and camera | Per vessel |

Two consequences:

- **"Shared world model" becomes "shared world identity and fact substrate."**
  Mesocosm's live ecology, Paredros' settlement, and Isometry's campaign state
  will never be one in-memory model. They append compatible facts to one
  world.
- **`isometry-voxel` recipes are no longer canonical.** They are an excellent
  first projection codec and probably Isometry's projection, but making them
  *the* appearance format lets today's renderer leak into the substrate. The
  portable artifact carries **body topology, incorporated-part provenance,
  loud inherited signatures, and optional projection recipes**; each vessel
  derives its own presentation.

### The anti-Spore law, restated

The founding record's one-substrate rule and this document's "renderers need
not be shared" appear to conflict. They do not, once the law is stated at the
right altitude:

> **A vessel must not create a private replacement for shared world identity,
> provenance, and causal history.**

A vessel may absolutely have its own renderer, event loop, ECS, or physics
dimensionality. What hollowed Spore was five stages that shared no world, not
five stages that shared no renderer.

---

## 6. Findings

- **2026-07-30**: the `wgpu-*` siblings are a web-embedding family
  (`wgpu-graft` = Servo texture grafting, `wgpu-weld` = CEF accelerated OSR,
  `wgpu-scry` = system-webview capture). None renders geometry, so none
  shortens this path. `netrender` and vello apply, but to the 2D lane only.
- **2026-07-30**: `seiche` wraps rapier2d 0.33, so the physics family is
  already in the tree and understood.
- **2026-07-30, verified versions**: Bevy 0.19.0 (2026-06-19), Fyrox 1.0.1
  (2026-03-28), ggez 0.10.0 (2026-06-03). Macroquad carries an unpatched
  soundness advisory affecting all versions. Bones (`bones_framework` 0.4.0,
  registry 2024-09-12) and Renderling (0.4.9, registry 2024-09-20) are both
  **registry-stale but git-active** (pushes 2026-04-24 and 2026-07-19), which
  this stack already has a convention for.
- **2026-07-30**: `block-mesh` remains the meshing reference —
  `visible_block_faces` ≈ 40M quads/s on one core, `greedy_quads` ≈ ⅓ the
  triangles at ~3× the time.

---

## 7. The person ladder and the hybrid lens (research round 2026-08-03)

Prompted by the pivot: the custom prism pipeline dies, the game wants a
Barony-flavoured first-person view of a No Man's Sky-flavoured biosphere, the
player should feel like slugcat, and Mark asked whether the stack's 2D
strengths can carry it, and whether sprites can be more than billboards,
cheap. Research, no decision; probes decide.

### The contradiction dissolves into three dimensionalities

"Sacrifice 3D" and "height and gravity should count for wings" stop
conflicting once simulation, world rendering, and body rendering are allowed
different answers:

- **Simulation: 2.5D columns.** Terrain is an integer heightfield per
  column plus an air band above it. Height and gravity are real rules
  (staying aloft costs energy; wings occupy the band; falling is a thing),
  but nothing stacks, which is what places already ruled ("height is not a
  place") and what worldgen, crowding, and dispersal already assume.
- **World rendering: raycast the heightfield.** The Voxel Space lineage
  (Comanche 1992): each screen column marches a ray across a heightmap and
  fills vertical spans. The modern GPU form is one fullscreen shader (a
  hobby implementation reports ~500fps), and the engine's entire input is
  **two images: a heightmap and a colormap**. That is the load-bearing fact
  for this stack: the vello lane can *paint* those images. Worldgen
  generates biome maps per place; the renderer just marches them. Rolling
  organic terrain, fog, no meshes, no scene graph.
- **Bodies: the parts graph, not a billboard.** Three candidate forms, in
  ascending ambition, all driven by the same anatomy:
  1. **Per-part cards**: one small quad per part, placed by its attachment
     frame in 3D, texture painted by vello, chains driven host-side by
     seiche. Parts occlude and parallax as the body turns; all art stays
     2D. The Rain World technique is exactly this (2D chains, IK feet)
     lifted into a 3D placement.
  2. **Sprite stacking**: a part as a stack of horizontal slices (the
     MagicaVoxel-to-slices pipeline; SpriteStack, many GameMaker/PICO-8
     games). Genuinely 3D-reading rotation from 2D draws, thousands of
     sprites cheap. Honest limit: it reads from above-ish and degrades at
     eye level, so it serves Paredros' over-shoulder and Isometry's
     pulled-back distances better than Mesocosm's first person.
  3. **Skeleton-driven SDF capsules**: model a critter as smooth-unioned
     capsules along its part chains and raymarch it (iq's canon; the
     technique behind Claybook-style clay). Squash, stretch, breathing,
     and morphing are math on the field rather than rig work, and
     **smooth-union is kleptoplasty made visible**: a grafted part blends
     into the body exactly the way the mechanic means it. Cost: a
     fullscreen march is real GPU work and an art-direction commitment
     (the clay look), and debugging lives in shaders.

### The unified-raymarch candidate

The three lanes above share one loop. A single fullscreen pass can march
terrain heightfield and critter capsule-fields together: the same ray, the
same fog, one engine measured in hundreds of lines of WGSL rather than a
renderer. Everything it consumes is 2D data the stack generates (height and
colour maps from vello; capsule chains from the parts graph via seiche).
The camera ladder becomes a parameter: Mesocosm holds the camera at a
critter's eye, Paredros pulls it over a shoulder, Isometry lifts it to the
scene, and no vessel grows an engine, which is the anti-Spore law satisfied
by construction rather than discipline.

Netrender is the frame owner in this shape: its render graph takes encode
callbacks as tasks, so the march is a `Task` between vello layers, the HUD
composites above, and `mesocosm-render`'s custom pipeline dies into a graph
node rather than being replaced by a second custom pipeline.

### What the stack contributes, named

vello paints every input (terrain maps, part textures, HUD); seiche drives
chains host-side (f32 presentation, integer core untouched); netrender owns
the frame and the composite; genet's cambium lane arrives for the epoch
screen; the sprite-hull tracer turns rendered art into collision hulls. The
raymarch shader is the one genuinely new piece, and it is small enough to
become a shared component (a biosphere lens) if Paredros pulls.

### Probes, with targets

- ~~**Terrain probe**~~ **LANDED 2026-08-03** as `crates/mesocosm-lens`:
  two passes (march + grade), maps synthesised from a `Places` partition
  (one biome per place, golden-angle tints, fbm relief), both souls
  captured along a flight at `testing/mesocosm/16_lens_*.png`. The vista
  shots show rolling multi-biome terrain into banded dithered fog (retro)
  and smooth haze (clay) from the same march. The probe confirms: the
  engine is two small WGSL files, the world is two images, a soul is a
  uniform block, and place borders read as biome borders. CPU map
  synthesis stands in for the vello lane without changing the contract.
  Two findings: the march's step-growth budget silently capped view
  distance at ~155 units until the growth rate matched the far plane (a
  vista rendered as pure sky), and banded fog plus starved palette carries
  most of the retro soul on its own. Dither-in-motion remains a headed
  judgement; three consecutive frames per soul are captured for it.
- ~~**Critter probe**~~ **LANDED 2026-08-03**: a follow-the-leader capsule
  chain (`lens::critter`, the Rain World constraint technique, ~100 lines,
  no physics engine) smooth-unioned into the same march, walking a wander
  over the terrain. Receipts at `testing/mesocosm/17_critter_*.png`: a
  tapered caterpillar-form body cresting a ridge with visible ground
  contact, fully legible under both grades; the retro dither reads as body
  texture rather than noise. **The gate passes: it reads as an animal.**
  Chain findings: spacing must be ~2x segment radius or the body blends
  into a blob; the smooth-min k must be small against the radii; a relax
  term toward the leader's heading is what lets a still body settle
  instead of freezing its wiggle; and turn radii must exceed a body length
  or a loper reads as a coiler. One shader bug worth remembering: seeding
  a smooth-min from a 1e9 sentinel makes `mix(1e9, d, 1.0)` cancel
  catastrophically at f32 and the whole field reads as a hit -- the
  giant-sphere artefact. Seed from the first primitive, never a sentinel.
  Deliberately absent yet: legs, eyes, contact shadows, and the
  parts-graph binding (the probe chain is hand-shaped; the game's chains
  derive from anatomy).
- **Fallback**: if the march disappoints on perf or art, per-part cards on
  the same chains are the retreat, and sprite stacking stays evaluated for
  the two pulled-back vessels.

### Rulings (Mark, 2026-08-03)

1. **The soul question is a styling matter.** The march yields colour,
   depth, and normals; a parametric **grade** stage (internal resolution,
   palette LUT, dither matrix, lighting ramp, fog curve) turns every look
   into a small data block worldgen can emit the way it emits a heightmap.
   Retro and clay are two grade blocks, and everything between them is a
   space. Mark leans retro; the probe renders both.
2. **Lineage grades: yes.** Successful lineages may shift their territory's
   grade, so a region can be felt on entering. Same derived-not-stored rule
   as every look: computed from world state, never in the snapshot.
3. **HUD inside or above the grade is a setting**, because it touches
   accessibility. Not a design fork; both stay cheap.
4. The layering is **march (geometry) → field fx (world-driven, the same
   ScalarField that feeds physics, so fx cannot lie) → grade (the soul)**.
5. Probes proceed, with the dither-in-motion check flagged (Obra Dinn's
   lesson: ordered dither shimmers under camera motion; judge the retro
   grade while flying, not from stills).
