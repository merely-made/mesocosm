# Execution Waves

**Status: in progress, 2026-07-31.** **Waves 1.1 and 1.2 complete**: core,
runtime, mesher, renderer, and a windowed host, with every done-condition met
and tested. **Wave 1.3 dropped**: the render lane is decided (custom wgpu body
renderer, netrender owning the device), so there is no second host to compare
against. Ruled by Mark. This is the **authority on ordering** across the
games wing. It does not restate design;
it sequences the work the governing plans already specify and adds the
constraints that only appear once the order is fixed.

Governing plans, which own the *what*:

- [body pipeline and host probe](2026-07-30_body_pipeline_and_host_probe_plan.md) — the shared organ, the body document, R-phases
- [Mesocosm founding plan](2026-07-30_mesocosm_founding_plan.md) — vessel 1's design and M-phases
- [games wing founding record](2026-07-30_games_wing_founding.md) — the laws, and the proof pair as the next architectural threshold

---

## Wave 1 — Architecture and receipts

The purpose of this wave is to make later decisions cheap by producing
evidence rather than argument. Nothing here is a game yet, and that is
deliberate — but see the standing caveat in Wave 2.1, which is the reason this
wave is not allowed to expand.

### 1.1 `mesocosm-core` — **LANDED 2026-07-31**

Own the core: seeded simulation, ordered inputs, snapshots, state hashing, the
body graph, provenance, and the smallest possible metabolize/attach operation.

**The core owns game state; hosts only project it.** That sentence is the
whole architecture of this wave, and every later comparison depends on it
holding literally.

**Done when** the fixture replays identically, provenance round-trips, and
attaching a part changes the core's mass and collision state.

**All three met.** `crates/mesocosm-core`, 30 tests, clippy clean, no host or
graphics dependencies (serde and postcard only). The shared fixture lives in
`tests/replay.rs` and is what waves 1.2 and 1.3 replay.

The design decision worth carrying forward: **the core is integer-only.**
Voxel coordinates, masses in milligrams, and quarter-turn rotations are exact
on every platform, so a replay cannot diverge on a different machine's
floating-point behaviour. This is a stronger guarantee than the R0 note
anticipated, which had float physics behind a seam but still inside the
determinism argument. Float physics now sits wholly outside the core, and
`rapier`'s `enhanced-determinism` stops being load-bearing for replay
equivalence: a host may use floats freely, provided it does not feed derived
float state back into the core.

### 1.2 The Genet host — **COMPLETE 2026-07-31**

Build the custom host over the existing winit / wgpu / Cambium stack, probing
**Renderling** for voxel bodies. It consumes the core's state and intents
**without duplicating rules**.

**Done when** the new part is visible and physically legible, the host uses the
intended wgpu device, and uneven frame delivery does not change simulation
results.

**Split on landing, because the three conditions carry very different risk.**
The frame-delivery condition is a logic property provable without a window; the
other two need a GPU and a display. Doing the provable part first also serves
the confound rule, since the stepping it defines is shared by both hosts.

- **`mesocosm-runtime` landed** (`crates/mesocosm-runtime`, 12 tests, clippy
  clean): the fixed-step clock, the intent queue, the applied-intent trace, and
  `Receipt`. **Uneven frame delivery does not change simulation results** is
  met and tested: the same total elapsed time delivered in ragged chunks
  produces the same trace and the same state hash, and a step cap defers work
  rather than dropping it.
- **`mesocosm-mesh` landed** (`crates/mesocosm-mesh`, 24 tests, clippy clean):
  per-part greedy voxel meshing plus rigid placement, headless and integer-only.
  **The attachment hypothesis is met on everything derivable without a screen.**
  An end-to-end test eats a real morsel and checks all four claims at once:
  mass grows, the centre of mass moves toward the new part, the collision box
  grows, and the drawn body gains placements and reaches further, with
  provenance intact alongside the geometry.
- **`mesocosm-genet` landed** (`crates/mesocosm-genet`): a winit window, a wgpu
  surface, input as intents, and the shared runtime stepping the world. It
  holds no game state; if a rule ever appears in it, it is in the wrong crate.
  **All three done-conditions are now met.** A capture run of 300 frames
  produced **111 steps** — the window drew at roughly 162 fps while the
  simulation ticked at exactly 60 Hz, which is frame-delivery independence
  observed live rather than only unit-tested. The critter grew to five parts
  during the run and the frame was captured on exit.
- **`--frames N --capture PATH` keeps the windowed path verifiable** without a
  person sitting in front of it, which is the same discipline as the headless
  tests: run the real loop, leave evidence.

**Wave 1.2 is complete.** What remains open is not a condition but a judgment
already given: Mark's read of the first renders was that the parts read as
attached rather than floating, with the caveat that a still frame cannot show
what is *joined* — that wants motion, which the window now supplies.

**Why the runtime is shared rather than per-host.** If each host wrote its own
stepping, a divergence between hosts could be a divergence in *stepping*, and
the probe would measure the wrong thing. Both hosts drive the world through
this crate, so any difference is attributable. This is also the extraction
candidate the body pipeline plan named; it stays here, and small, until a
second consumer justifies lifting it.

### 1.3 ~~The Bevy host~~ — **DROPPED 2026-07-31**

The engine lane is cancelled. Mesocosm renders through a small custom wgpu
body renderer with netrender owning the device and compositing.

**The reason is stronger than preference, and worth keeping so the decision is
defensible later.** Weighing the field established that **our rendering need is
tiny and specialised**: the mesher emits flat-shaded palette quads with no
textures, no interpolated normals, no skinning (parts are rigid by ruling), and
no authored materials. Engines are optimised for the opposite problem, authored
content with PBR materials, image-based lighting, and skeletal animation.
Renderling's entire feature set, and most of Bevy's rendering value, sits in the
part of the problem this game does not have. Roughly six hundred lines of
pipeline covers what we do need.

**What dropping the comparison costs, stated honestly.** We give up a *measured*
answer to "would an engine have been faster to work in." The receipts R2 was to
produce, adapter size, frame pacing, iteration quality, asset handling and
debugging, become absolute observations rather than comparisons. They are still
worth recording, and if the custom lane turns out to fight us, the engine
question can be reopened with real evidence from having built the thing once.

**The confound rule retires with the comparison, but its insight does not.**
"Do not change two variables at once" still applies to any later A/B, and most
immediately to a 2.5D-versus-3D presentation choice, which must not be decided
inside some other change.

**Rejected with reasons, for the record:**

- **Renderling.** Alpha, self-described work in progress, and its shaders need
  a *specific nightly* through rust-gpu. `cargo-gpu` isolates that toolchain so
  the rest of the project stays stable, which is a real mitigation, but it is
  still a pinned nightly tracking behind latest, paid for glTF, IBL, PBR,
  shadows and bloom that this game will not use. Its registry staleness is not
  the problem; the toolchain is. **Two ideas are worth stealing without the
  dependency**: headless rendering with image-diff tests (see below), and
  shaders in Rust so quad and material types are defined once instead of
  drifting between Rust and WGSL.
- **Vello, for now.** If Mesocosm settles into a genuinely 2.5D stylised look,
  vello becomes very attractive, and it is already owned. It is not the probe
  target because **vello has no depth buffer**: a 2.5D vello lane would depth
  sort by painter's algorithm, which is a different rendering approach rather
  than a different host, and the geometry here is separate rigid boxes that a
  depth buffer handles without sorting artifacts. Revisit once the look is
  known.

**Adopted from the Renderling read: golden-image testing.** The visual half of
1.2 was going to be pure judgment. It need not be. wgpu renders headless to a
texture, so the renderer should be built headless-first and wrapped in a window,
which makes "the new part is visible" an assertable property rather than an
opinion. The opinion that remains is whether it looks *good*, which is the right
thing to leave to a human at a screen.

### 1.4 The Isometry projection — **in progress 2026-07-31**

Teach `isometry-voxel` to consume **`BodyDocument v0`** and emit a sprite plus
a bake receipt. `.vox` remains an **authoring input**, never the interchange
object.

**Done when** one critter completes
`body document → Isometry sprite → body/profile round-trip`
with part provenance intact.

**Landed: the flattening half.** `mesocosm-mesh::flatten` composes a whole body
into one occupancy grid with a body-space origin, handling negative placements
by shifting the origin rather than clipping. That is precisely what
`isometry-voxel::bake_facing` consumes, so **Isometry's baker needs no change**;
what was missing was the adapter, not a feature.

**Open, and it is a decision rather than work: which side holds the seam.**
`Voxels` and `Palette` live in `isometry-voxel`; `Flattened` lives here. One of
four has to happen, and the choice is Mark's because it sets a coupling
direction for the whole wing:

1. **Mesocosm depends on `isometry-voxel`** (git dep). Smallest diff. Couples a
   game to a VTT's asset crate, which is backwards but survivable since
   `isometry-voxel` is nearly standalone.
2. **Isometry depends on `mesocosm-core`.** Worse: a general VTT would depend on
   one specific game.
3. **Extract the body document into a neutral crate** both consume. Correct by
   the wing's own extraction rule, since two real consumers now exist, and it
   would be MIT/Apache under the licensing split as a genuinely reusable
   library. Costs a naming round and a new published crate.
4. **Couple by data, not types.** Mesocosm writes the grid through
   `mere.pack/v1`; Isometry reads it with its own thirty-line adapter. Most
   consistent with the wing's law that games interoperate through data rather
   than type dependencies, and the only option that scales to a third game.

**Ruled 2026-07-31: option 4, couple by data.** Mesocosm writes the body
document and its flattened grid through `mere.pack/v1`; Isometry reads them
with its own small adapter into `Voxels` and `Palette`. No shared type, no
cross-repo dependency, no naming round.

This is what the wing's interop law already required — games interoperate
through data rather than type dependencies — and it is the only option that
scales to a third consumer without rework. It also follows from the
presentation ruling: if projection negotiates by capability, the pack must
ship the *document*, so the seam is data by construction.

Revisit option 3, a neutral extracted crate, when a third consumer appears and
the schema has stopped moving.

**Booked cost, honestly:** a schema and a reader instead of a type the compiler
checks. Drift between writer and reader becomes a runtime failure rather than a
build failure, so the profile needs a version field and a refusal path from the
first commit. That is the trade the interop model makes everywhere else.

### Where Bones goes

**Not its own lane**, and now **unblocked earlier**. The original rule was to
test core-owned storage against Bones inside the *winning* host, so the ECS
choice would not be mixed with a renderer choice. With one host, there is
nothing left to wait for: the question is whether `mesocosm-core`'s own storage
or Bones ECS serves the core better, and it is independent of rendering.

It stays deferred behind playfeel rather than behind the host, because a
storage model chosen before there is a game to store is chosen against
guesses.

---

## Wave 2 — Playfeel and proof

Begins once one host can make metabolize feel promising.

### 2.1 Mesocosm M0 playfeel

One enclosure, one critter, meaningful movement, eating, deposition,
incorporation, and a metabolic budget.

> **The standing caveat, and the reason Wave 1 is bounded:** architecture
> receipts cannot substitute for the founding condition. **Somebody wants
> another run.** No quantity of state-hash equivalence redeems a verb that is
> not worth repeating.

### 2.2 Epoch and ecology lab

A headless or lightly visualised model of the epoch loop's turn structure:
complex-first initiative, simpler-lineage response, complexity-frontier
switching, and autonomous inactive lineages.

**Three authored worlds from the Exocosm-informed grammar**, not procedural
world generation. Authored worlds are legible enough to debug an ecology
against; generated ones move the question before it is answered.

### 2.3 Paredros social proof

**P0/P1 only**: one place, three companions, offers, confidence bands, one
standing agreement, and bounded tag-in. **Placeholder bodies** — this proof
does not wait on the body pipeline.

The proof is whether unfamiliar requests and trusted assignments *feel
socially different* without becoming friction theatre. That is a judgment, and
it is the right kind of judgment for this vessel: the failure mode is a
negotiation minigame attached to routine work.

### 2.4 The full Mesocosm–Isometry proof pair

Player-made **and** RNG organisms enter the same Isometry roster slot;
Isometry adds history; Mesocosm reads the descendant back.

This is where **interchange profile v0 becomes real**, and where Law C is
demonstrated rather than asserted: the two organisms must be structurally
indistinguishable in that slot, with only the player able to tell them apart,
by pointing.

---

## Deferred, with reasons

Not "later" as a vague hope — deferred because each becomes much easier to
*name correctly* after M0, the host probe, and the first real cross-game
round-trip:

| Deferred | Why it waits |
| -------- | ------------ |
| Procedural world generation | Authored worlds first (2.2); generation is a question about a grammar that is still being read against real play |
| Paredros city simulation | The settlement's shape depends on what the social proof (2.3) shows is worth simulating |
| A generic storyteller | Extraction discipline: build one inside a vessel, extract at the second consumer |
| Shared game-runtime extraction | Same rule. Paredros is not yet a consumer of anything |

---

## Findings

- **2026-07-31, wave 1.1.** An integer-only core removes float determinism
  from the replay argument entirely, rather than managing it. Voxel units,
  milligram masses, quarter-turn rotations, and an `i128` accumulator for
  centre-of-mass are exact everywhere. Consequence for wave 1.3: the two hosts
  must agree on a state hash, and neither host's physics backend can affect
  that, because host physics never writes to the core. If a host ever needs to
  push a derived float back in, that is the moment this guarantee breaks and
  the decision should be surfaced rather than absorbed.
- **2026-07-31, wave 1.1.** Rejections are recorded outcomes rather than
  errors. An intent that cannot apply still advances the tick and returns
  `Outcome::Rejected`, so a replay that rejects the same intents stays
  identical. This matters for the host probe: a host that silently drops an
  invalid intent instead of submitting it will diverge.
- **2026-07-31, wave 1.1.** The body needed `world_yaw` alongside
  `world_offset`. Position alone would let a projection draw every part
  unrotated, which was a real gap in the portable document rather than a
  rendering detail.
- **2026-07-31, wave 1.2.** The fixed-step clock must accumulate in *rational*
  form, scaling elapsed microseconds by the tick rate rather than dividing by a
  precomputed interval. **1_000_000 is not divisible by 60**, so a precomputed
  16666 us interval runs fast and gains a step roughly every 25 seconds. The
  first implementation here asserted divisibility, which would have refused
  60 Hz outright; the second rounded, which drifts. Scaling is exact for every
  tick rate and refuses none. A host probe that ran long enough would have
  found this as an unexplained divergence between two hosts polling at
  different rates.
- **2026-07-31, wave 1.2.** A step cap must **defer** rather than drop. Capping
  by discarding the remainder would make the simulation depend on how badly the
  host stalled, which is exactly the property the fixed step exists to remove.
- **2026-07-31, wave 1.2.** **Per-part meshing makes attachment cheap, and the
  plan's rigid-part posture is what buys it.** A part's geometry depends only on
  its volume, so it is cacheable by `VolumeRef`: incorporating a part adds one
  placement and, only if the volume is new, one mesh. Nothing already on the
  body is remeshed, which is verified rather than assumed. This also means faces
  are never merged across a joint, which is correct rather than a shortcut,
  since merging across parts would weld the body into one mesh and lose the
  ability to move a limb.
- **2026-07-31, wave 1.2.** A missing volume is a **reported error**, not a
  skipped part. The tempting behaviour is to draw what resolves and move on,
  which yields an invisible limb and a silent divergence between a body's
  physics and its picture. `MeshError::MissingVolume` names the part.
- ~~**2026-07-31, wave 1.2.** The mesher is the shared organ… Isometry's baker
  wants the same quads.~~ **Wrong, corrected 2026-07-31 by reading the code.**
  `isometry-voxel::bake_facing` takes a `Voxels` **occupancy grid** and projects
  it voxel by voxel with depth sorting. It has never wanted quads.

  The corrected shape is better, and smaller. There are **two projections off
  one document**: a live renderer wants quads meshed per part and kept separate
  so limbs can move; a sprite baker wants a single grid, because a baked sprite
  has no moving parts. `mesocosm-mesh::flatten` composes a body into one grid
  and is the adapter that lane needs, so **Isometry's baker requires no change
  at all**. The shared organ is the body *document*, which is what the body
  pipeline plan said in the first place; the quads were my embellishment.
- **2026-07-31, wave 1.2. A part's local origin is its lowest corner, not a
  pivot, and the body document needs to say which it wants.** This is the first
  finding that came from *looking at a render* rather than from a test, which is
  the argument for keeping a human in this loop. Every visual test passed while
  the first rendered body had its limbs **floating in space beside the torso**,
  because a test can assert that a part is drawn and that the silhouette
  widened, and both were true of a detached limb.

  Two consequences follow, and they are the same underspecification seen twice.
  Attaching a part flush requires knowing that part's size, so an author cannot
  write an offset without consulting the volume. And **rotation turns a part
  about its corner**, so a limb that was flush swings off its joint when yawed,
  which is why the example holds yaw at zero.

  The fix belongs in the body document as a **per-part pivot**, the origin an
  attachment frame is measured from and a rotation turns about. The plan already
  calls for the `.vox` importer to strip marker voxels and write explicit
  attachment frames; this says what those frames must carry. Deferred rather
  than done, because the right pivot convention is easier to choose once real
  authored parts exist. Recorded so it is not rediscovered as a rendering bug.

---

## Progress

- **2026-07-31**: waves ruled and recorded.
- **2026-07-31**: **wave 1.1 landed.** `crates/mesocosm-core` with body graph,
  provenance, seeded stream, ordered intents, whole-world snapshots, and FNV
  state hashing. 30 tests, clippy clean. Deps: serde, postcard. The fixture in
  `tests/replay.rs` is the shared artifact waves 1.2 and 1.3 compare against.
- **2026-07-31**: **wave 1.2 part one landed.** `crates/mesocosm-runtime`:
  drift-free fixed-step clock, intent queue, applied-intent trace, and
  `Receipt` for host comparison. 12 tests, clippy clean. The frame-delivery
  done-condition is met; the window, device, and body projection remain.
- **2026-07-31**: **wave 1.2 part two landed.** `crates/mesocosm-mesh`:
  per-part greedy voxel meshing, rigid placement, and an end-to-end attachment
  test over the real simulation. 24 tests, clippy clean. 66 tests across the
  workspace. Only the window and the device remain in wave 1.2.
- **2026-07-31**: **wave 1.2 complete.** `crates/mesocosm-genet`: winit window,
  wgpu surface, input as intents, shared runtime stepping the world, and a
  `--frames/--capture` mode that keeps the windowed path verifiable. A 300
  frame run produced 111 steps at 60 Hz, so the loop drew at ~162 fps while
  the simulation ticked fixed. 82 tests across the workspace, clippy clean.
  Five crates: core, runtime, mesh, render, genet.
- **2026-07-31, wave 2.1 (playtest).** Mark ran the window and it found two
  defects no test had. **Drawing only the body made the game unplayable**: the
  camera framed body bounds and morsels were not drawn, so moving did nothing
  visible and there was nothing to act on. Fixed by rendering scenes with a
  following camera and dimming what is out of reach. **And placement stacked
  parts**: the fixture cycled six faces by `body.len() % 6`, so the seventh
  part landed on the first and a well-fed critter collapsed into a z-fighting
  pile. Fixed by checking the flattened body for free space before proposing an
  attachment, with a regression test asserting placed voxels equal expected
  voxels, since an overlap silently loses voxels to overwriting.

  The general lesson: **both defects were invisible to the test suite and
  obvious within seconds of playing.** Tests assert that a part is drawn and
  that the body grew; neither notices that the player cannot see the world or
  that the body is a pile. Wave 2.1's standing caveat is doing real work.
- **2026-07-31.** First free face gives **compact clumping**, not silhouette. A
  real growth policy will want directional bias, symmetry, or trait-driven
  morphology; the current one only guarantees no overlap. Recorded because a
  blob that reads as a blob is a design question, not a bug.
