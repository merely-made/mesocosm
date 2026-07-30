# Engine and Render Lane: Landscape

**Status: research, 2026-07-30. No decision.** Written because the founding
record reduced this to "Bevy versus custom wgpu," which is a false binary.
There are other pure-Rust engines, some forkable in the way this stack
normally forks things, and "custom wgpu" is not one option but an assembly
with many joints. This doc maps the space so a probe can be aimed.

Facts here were checked on 2026-07-30. Version numbers move; recheck before
committing.

---

## 0. What is actually being chosen

Three vessels, three different needs, and they do not have to share an
engine — only a substrate (§5).

Working dimensionality, proposed by Mark 2026-07-30 and **not yet ruled**:

| Vessel | Proposed | Notes |
| ------ | -------- | ----- |
| Mesocosm | 2D or 2.5D, rapier2d | A large simplification. Voxel bodies still work in 2.5D, and physics-legible bodies survive: mass, balance, and reach all read fine in 2D. |
| Paredros | 3D, first person | The heaviest renderer requirement in the wing. |
| Isometry | 3D, third person | **Conflicts with a standing ruling** — see below. |

**Isometry caveat, flagged rather than adopted.** Isometry's `CLAUDE.md`
says camera freedom is explicitly not a near-term rendering task: the locked
isometric 2D lens is the shipped one, and later 2.5D/3D modes are allowed
(voxel source models dissolve the facing-art explosion) *but need their own
plan and render lane*. Isometry also renders through genet's DOM today, so a
3D lens is a different renderer, not a camera change. Moving Isometry to 3D
third person is a real scope change to a shipping project and should be its
own plan in its own repo, not a side effect of this wing's choices.

If Mesocosm lands at 2.5D, the wing's hardest renderer is Paredros, not
Mesocosm — which inverts the founding record's assumption that vessel 1 is
the render pressure vessel.

---

## 1. Pure-Rust engines

Checked 2026-07-30.

| Engine | Version | Shape | License | Fork posture |
| ------ | ------- | ----- | ------- | ------------ |
| **Bevy** | 0.18 | Data-driven ECS, 2D + 3D, ~44k stars, large plugin ecosystem | MIT/Apache | Too large to fork; the idiom is plugins, and that is genuinely how the ecosystem works |
| **Fyrox** | 0.36.2 | 3D-focused full engine, **the only one with a real visual editor** (scene hierarchy, property inspector, asset browser, 3D viewport) | MIT | **Forkable.** Pure Rust, single-vendor, editor included |
| **Macroquad** | 0.4.14 | Minimal-friction 2D, tiny API surface | MIT/Apache | Small enough to fork or simply outgrow |
| **ggez** | 0.9.3 | Comfortable 2D defaults, LÖVE-shaped | MIT | Small |
| Amethyst | — | Archived; its ECS lineage fed Bevy | — | Dead, do not start here |
| Piston | — | Legacy modular experiment | MIT | Effectively dormant |
| Ambient | — | Rust/WASM multiplayer engine; **shut down** | — | Dead |
| Nannou, Blue Engine, Tetra | — | Creative-coding / niche 2D | — | Not a fit |
| rend3 | — | 3D renderer (not an engine) on wgpu | — | Verify maintenance before use |

**The two that deserve a real look are Bevy and Fyrox**, for opposite
reasons. Bevy is the ecosystem bet: voxel crates, physics integrations, and
asset pipelines already exist against it, and the plugin idiom means adopting
it is not all-or-nothing. Fyrox is the fork bet: it is the only Rust engine
that ships an editor, which is the single most expensive thing to build and
the thing a solo-maintained stack most often does without.

The cost that has to be weighed honestly against both: **an engine owns the
app loop.** That sits awkwardly beside armillary (the stack's actor runtime),
cambium's host patterns, and the pattern seiche demonstrates of a
continuously-reconciled world the host owns. For a game this is normal and
usually fine; for *this* stack it is the real friction, and it is worth
knowing before falling in love with either.

---

## 2. "Custom wgpu" is an assembly, not an option

The useful reframing: the stack already owns most of a game engine's parts.
What is missing is 3D geometry rendering, a mesher, and glue.

| Concern | Candidate | Already owned? |
| ------- | --------- | -------------- |
| Windowing + input | `winit` | Yes, via `cambium-winit` and genet's winit host |
| 2D GPU render | `vello`, and `netrender` (webrender-wgpu fork, vello backend shipped) | **Yes** — and this is what makes a 2.5D Mesocosm cheap |
| 3D render | `wgpu` directly, or `rend3` | No. This is the actual gap |
| Voxel meshing | `block-mesh` (`visible_block_faces` ≈ 40M quads/s single-core; `greedy_quads` ≈ ⅓ the triangles at ~3× the time), surface-nets, binary-greedy-meshing ports | No, but these are small, well-scoped crates |
| Physics | `rapier2d` / `rapier3d`, `parry` | Partly — `seiche` already wraps rapier2d 0.33 |
| Voxel asset ingest | `isometry-voxel` (.vox ingest, recipes, palette swaps, bakes) | **Yes**, and it is already the wing's "recipe not image" pipeline |
| Audio | **Firewheel** | **Yes** — already the audio lane via Hocket/Strophe |
| Actor runtime | `armillary` | Yes |
| Persistence | `muniment`, `codicil` | Yes, and codicil is already the deed-log shape |
| Influence fields / AI gradients | `numen` + `quint` | Yes (R² today; R³ is parked) |
| UI | `cambium`, `xilem_serval`, `sprigging` | Yes |
| Text | `parley` | Yes (endorsed over cosmic-text) |
| ML, if ever | `burn` | Yes (endorsed direction) |

Eleven of thirteen rows are already owned or trivially available. The custom
path is therefore not "write an engine"; it is **write a 3D renderer and a
game loop, and wire in things that already exist.** That is a materially
different proposition from the founding record's framing, and it is the
strongest argument for the custom lane: the synergies Mark asked about are
real and they are mostly already paid for.

The honest counterweight: an engine supplies not just these boxes but the
*integration*, plus an asset pipeline, a scene format, tooling, and a
community answering questions. Eleven owned components still have to be made
to cohere, and nobody else has done that combination.

---

## 3. Approaches worth probing, not just the two

1. **Bevy as-is.** Fastest to a playable M0. Accept the loop.
2. **Fyrox, with fork intent.** Gets an editor. Smaller community; a fork is
   a real maintenance commitment, which this stack has taken on before
   (stylo, xilem, webrender) and knows the cost of.
3. **Custom loop, 2.5D.** `winit` + `netrender`/`vello` + `rapier2d` +
   voxel bake through `isometry-voxel`. Maximum reuse, minimum new
   rendering, and it fits Mesocosm's proposed dimensionality exactly.
4. **Custom loop, 3D.** `winit` + `wgpu` + `block-mesh` + `rapier3d`. The
   real new work is a voxel renderer, which is one of the more tractable
   custom renderers to write (chunked meshing, no skeletal-animation
   pipeline needed if bodies are voxel-rigid).
5. **Split the bet.** Mesocosm on the custom 2.5D lane (cheap, high reuse,
   proves the substrate), Paredros on an engine (its 3D first-person needs
   are conventional and an engine serves them well). The wing shares a
   *substrate*, not a renderer — which the founding record already requires
   for other reasons.

Option 5 deserves more attention than it first appears to. The one-substrate
law binds the world model, the deed log, and the interchange profile. It
says nothing about renderers, and pretending it does would be the same
category error as letting a stage grow its own engine.

---

## 4. How to decide

**By probe, and the probe already exists.** M0 is "one critter, one
enclosure, metabolize" — the phase that must feel good regardless. Build it
more than once:

- M0 on Bevy
- M0 on the custom 2.5D lane

**Done when** the verb feels right in one of them and the difference in
effort is measured rather than argued. Fyrox enters the probe only if the
editor turns out to be the deciding factor, since that is its distinguishing
claim.

What to record from each probe: time to first playable, how the app loop
fought or fit the stack's actor/host patterns, how much owned code was
actually reusable, and whether voxel bodies read as physics-legible.

---

## 5. What must stay shared regardless

Renderer choice is free. These are not:

- The world model, deed log, and interchange profile (the one-substrate law)
- `codicil` / `muniment` for persistence
- The `mere.pack/v1` envelope for anything crossing games
- `isometry-voxel` recipes as the appearance format, so a critter can look
  like itself in every vessel that renders it

---

## 6. Findings

- **2026-07-30**: the `wgpu-*` sibling repos are a web-embedding family
  (`wgpu-graft` = Servo texture grafting, `wgpu-weld` = CEF accelerated OSR,
  `wgpu-scry` = system-webview capture). None renders geometry, so none of
  them shortens this path. `netrender` and `vello` *do* apply, but to the 2D
  lane only.
- **2026-07-30**: `seiche` wraps rapier2d 0.33 already, so the physics
  dependency family is in the tree and understood.
