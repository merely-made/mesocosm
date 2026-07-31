# Execution Waves

**Status: in progress, 2026-07-31.** Wave 1.1 landed; wave 1.2's frame-delivery
half landed. Ruled by Mark. This is the **authority on ordering** across the
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

### 1.2 The Genet host — **partially landed 2026-07-31**

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
- **Remaining for 1.2**: the window and the device. What is left is whether the
  result *looks* legible, which is a judgment for a machine with a display and
  cannot be asserted in a test. Everything that judgment needs is now
  derivable, deterministic, and cheap.

**Why the runtime is shared rather than per-host.** If each host wrote its own
stepping, a divergence between hosts could be a divergence in *stepping*, and
the probe would measure the wrong thing. Both hosts drive the world through
this crate, so any difference is attributable. This is also the extraction
candidate the body pipeline plan named; it stays here, and small, until a
second consumer justifies lifting it.

### 1.3 The Bevy host

Host the **exact same** core, fixture, physics dimension, and body document in
Bevy.

> **The confound rule.** Comparing a Genet 2.5D lane against a Bevy 3D lane
> would measure host *and* perspective at once and attribute the difference to
> whichever the reader already preferred. **Both hosts initially stage the same
> enclosure.** Perspective is a lens choice to be made later, on its own
> evidence.

**Done when** its final state hash matches the Genet run, and concrete receipts
exist for adapter size, frame pacing, iteration quality, asset handling, and
debugging.

### 1.4 The Isometry projection

Teach `isometry-voxel` to consume **`BodyDocument v0`** and emit a sprite plus
a bake receipt. `.vox` remains an **authoring input**, never the interchange
object.

**Done when** one critter completes
`body document → Isometry sprite → body/profile round-trip`
with part provenance intact.

### Where Bones goes

**Not its own lane.** After the host comparison lands, test core-owned storage
against Bones **inside the winning host**. That isolates the real question —
which storage model the core wants — instead of mixing an ECS choice with a
renderer choice and getting one muddy answer for two questions.

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
- **2026-07-31, wave 1.2.** The mesher is the shared organ the body pipeline
  plan predicted, and it arrived earlier than expected: **Isometry's baker
  (wave 1.4) wants the same quads**, so the projection split is
  mesh-once-then-render-many rather than two independent pipelines.

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
