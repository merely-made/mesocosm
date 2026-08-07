# Place-Graph Engine Plan (2026-08-05)

**Status: plan, 2026-08-05.** Founded from the 2026-08-04/05 engine
rumination. Sibling to the
[render lane landscape](2026-07-30_engine_and_render_lane_landscape.md),
which owns renderer research and the V-gates, and subordinate to the
[execution waves plan](2026-07-31_execution_waves_plan.md) for ordering.
This plan owns the world substrate: the place graph, volumetric truth, the
two-tier simulation, and the composed slice that proves them together.

## 0. Rulings this plan rests on (2026-08-05)

Recorded here and amended into the founding record, `CLAUDE.md`, and the
landscape doc in the same session:

1. **One authority per capability, stack-wide.** The anti-Spore rule's
   kernel: one simulation authority; projections plural and cheap; refuse a
   second authority, and refuse duplicating functionality the stack already
   owns. (Narrows "refuse a second simulation or a second renderer".)
2. **Shared engine organs are encouraged; sovereignty lives in the verbs.**
   Vessels share nouns (space, bodies, fields, time, provenance); each owns
   its verb and person (metabolize, address, adjudicate). An organ is
   shareable while it stays verb-neutral; when a component encodes what you
   do, it belongs to a game. (Narrows the founding's "no engine" clause,
   which guarded against coupling-as-obligation and genre convergence; that
   guard stands as: no shared genre, no shared schedule, no shared verbs.)
3. **Volumetric world truth is permitted.** The lens-only stop rule opens.
   The anti-Minecraft insight stays: no machinery adopted because prior art
   ships it; acceleration structures are admitted by trace.
4. **Verticality is a simulation affordance.** The third axis must be
   mechanically legible (wings escape ground threats, canopy holds
   resources, burrows hide). Any projection that makes those facts readable
   qualifies; 3D rendering is not the only way.
   *Camera amended 2026-08-06:* Mesocosm's camera is **pulled back, Rain
   World-style**, superseding the 08-05 Barony-march ruling. First person
   survives in agency (person is agency, not camera). The Barony/Delver
   first-person march reference moves to Paredros, which is the brick
   tracer's long-term primary consumer.
5. **World graphs must differ.** No two worlds share topology unless they
   are copies. Distinctness is receipted with graph metrics, not vibes.
6. **Nesting is elective and continuous.** Containers index one global
   coordinate space; an edge is never a portal; a place with no internal
   topology stays a leaf.
7. **Worldgen is hybrid.** Top-down skeleton (relief, watersheds), then
   bottom-up detail within regions.
8. **The unit of proof is a slice.** Engines fail at the joints. No gate is
   proven for the wing until the composed run exercises it beside the
   others.
9. **Design reference targets (2026-08-06).** Mesocosm: Rain World,
   Voxatron, Caves of Qud. Paredros: Barony, Delver, Gotcha Force,
   RimWorld. Isometry: Foundry, the Larian/Owlcat adaptations, Wildermyth,
   tactical RPGs virtual and tabletop. Reading that matters: none of the
   nine demands a heavyweight renderer; all are simulation-deep and
   presentation-modest. Investment order follows: agency, procedural
   bodies, place graph, storyteller over renderer horsepower.
10. **Physics is three tiers (2026-08-06).**
    - **Authority: integer, ours.** Occupancy and movement legality over
      brick truth, inside the replay hash. Never a physics engine.
    - **Advisor: parry queries + bespoke kinematics.** Raycast, shapecast,
      TOI, contacts from parry (no dynamics world, no stepping, no
      persisted handles); move-and-slide and verlet chain gait are owned
      code that quantizes into integer outcomes. The nalgebra seam is one
      adapter around a query library.
    - **Ambience: GPU, outcome-free.** Debris, foliage, cloth, ragdoll
      (nexus-shaped, eventually). GPU float ordering varies by hardware,
      so this tier is constitutionally barred from outcome-bearing facts.
    - Rapier-the-dynamics-engine is **in reserve**: it returns only for a
      proven constraint-dynamics need, and its documented cross-platform
      determinism mode is why it alone may then sit near the fact plane.
      Avian rejected 2026-08-06 (Bevy-coupled by design, parry underneath,
      no determinism story).
11. **Renderer tenancy follows the cohesion contract** (landscape §8.9):
    one device, one frame, declared capability profiles, glam at the
    presentation boundary, receipts. Renderling is the lead mesh-tenant
    *candidate* pending its fork probe; kiss3d is a donor (AOV
    segmentation bakes, 2D GI); nexus is ambience-tier pending its own
    device audit; rust-gpu is welcome, carried by the fork family.

## 1. The model

One continuous integer coordinate space (the core's `[i32; 3]`), with
layers of meaning over it:

- **Place graph (macro).** Places own regions of the space: a generation
  recipe, a revision, dirty state, a simulation tier. Adjacency is derived
  from the landscape (link where travel is possible), never asserted.
  Nested containers are elective: a burrow system is a subgraph on a place;
  a meadow stays a leaf. The chartulary resemblance is noted and not
  depended on; a portable profile follows two real consumers, per the
  standing rule.
- **Brick truth (micro).** Per-region brick maps: coarse grids of pointers
  to dense material bricks; an absent pointer is air or unloaded. Dense
  bricks because they hash, diff, and serialize flat; SVO/DAG stay
  benchmark specimens (landscape §8.3 stands). Interiors are carved bricks
  in the same space.
- **Two-tier simulation.** Places near the played body run embodied agents;
  distant places run the existing statistical ecology.
  `organism/ecology.rs` is the far tier, reframed rather than replaced.
  Promotion and demotion happen at a hops-distance boundary with
  hysteresis; the boundary is a scheduling problem, never geometry.

Presentation stays a family of projections of one truth (landscape §8):
a brick-map DDA raymarch grows out of the landed march for the
first-person lens; `mesocosm-mesh` pointed at world bricks is the raster
projection; the grade is unchanged; bodies ride the landed
`BodyLensProjection` (V2). Fields (numen) attach to places and bricks. The
rapier adapter derives colliders from the same snapshot when it arrives
(§8.4 unchanged).

## 2. The slice: the burrow run

One composed scenario, the standard every gate feeds.

Worldgen lays a place graph over one voxel space; one region generates
with a burrow. A catalogue critter hunts the player by real sight-lines
through real bricks. The player flees or carves, and hides inside
geometry. The hunter follows across the threshold without a stutter.

**Done when**, all in one run, receipts kept:

- the run replay-hashes identically from its recorded intents;
- the brick tracer renders it headed, native and browser, both souls, with
  fps and frame spans recorded beside netrender's (V1 harness reused);
- a carve lands: intent, dirty brick, region re-upload, collision refresh,
  with the latency recorded;
- the burrow needs no special casing in tracer, collision, or perception;
- per-tick sight-line cost is recorded at the target critter population;
- the headed judgment is taken seriously: does hiding from a hunter have
  tension. Wave 2.1's founding condition outranks every receipt here.

The engine document (profile, capability matrix, external references) is
written after this run exists, from its numbers, not before.

## 3. Gates

### G0. A graph worth the name — **constructor LANDED 2026-08-06; adoption pending**

Landed additively as `Places::grown(seed, side, extent) -> Grown` in
`places/relief.rs` (integer diamond-square, 65², own seed, percentile sea)
and `places/grown.rs` (traversability-derived links: climb + ford limits
over sampled crossings, Chebyshev-2 candidates, union-find reconnection
over least-bad blocked passes; `Nest` interiors where ruggedness earns
them). `scatter` and its serialized shape are untouched, because `Places`
rides inside snapshots and world genesis draws from a shared stream after
it; adoption is a one-line swap in `world/genesis.rs` deferred while that
file is in flight. Receipts (all green, strict clippy): same-seed
bit-equality; per-seed topology uniqueness over an 8-seed corpus; the
lattice-regression test (non-uniform interior degree, every seed); edge,
diameter, nest, and **bridge** counts all varying across the corpus with
at least one world growing a chokepoint; whole-world connectivity;
congruence (every unlinked grid-adjacent pair fails the crossing test,
which required making `crossing` direction-canonical after integer stride
rounding made A→B and B→A disagree); serde round-trip. Threshold
calibration kept as an `#[ignore]` spectrum test.

Replace `Places::scatter`'s lattice. Today: stratified jittered sites with
hardcoded 4-connected links, so every world is topologically identical
(uniform interior degree, fixed diameter, zero bridges, hubs, or dead
ends), and `links` disagrees with `at()`'s nearest-site partition, which
can make diagonal Voronoi neighbours the graph denies. Build the hybrid
generator: relief and watersheds top-down, local detail bottom-up,
adjacency derived from traversability, elective nesting where interiors
have topology.

**Done when:** the same seed reproduces the same graph bit-exactly; across
a seed corpus, degree distribution, bridge count, cycle structure,
diameter, and nesting depth all vary, and a lattice-regression test
rejects uniform degree; derived links agree with the partition (no
Voronoi-adjacent pair denied without a landscape reason); the reckoning
consumers (`hops`, `spread`, `scale`) keep their existing receipts.

### G1. Brick truth with a lifecycle — **container LANDED 2026-08-06; world wiring pending**

Landed additively as `places/bricks.rs`: `Ground::grow(&Grown, extent)`
raises dense 8³ bricks (ordered map, serde-flat) from the relief; nests
realize as **roofed burrows anchored at the highest column near their
host** (a low host digs into its hillside rather than cratering; rooms
scale to afforded depth; every chamber keeps a ceiling). `carve` bumps one
revision and marks dirty bricks; `drain_dirty` is the projection's upload
discipline. Occupancy (`solid`, `stands`) and integer line-of-sight
(`sees`) land here too, seeding G3's perception. Receipts, all green with
strict clippy: same-world bit-equality + serde round-trip; identical
carves replay to identical bytes; a radius-1 carve dirties ≤8 bricks and
carving air is not an edit; burrows are roofed voids near every nest;
hills block sight and a bored tunnel grants it (the first scan version
walked into a burrow corridor, which is its own kind of receipt); and in
`mesocosm-mesh/tests/ground_projection.rs`, every matter-bearing brick
meshes through the same `mesh_volume` path bodies use, with clean bricks'
meshes untouched by a neighbour's carve. **Pending for G1 complete:**
carve as an ordered `Intent`, `Ground` inside the world snapshot/replay
hash, and genesis wiring — all in files currently in flight, adopted
with the `grown` swap.

Place-keyed brick regions over the one space; carve as an ordered intent;
dirty regions revisioned (the lens's `MapRevision`/`MapChange` discipline
generalized); the snapshot/replay hash covers bricks; `mesocosm-mesh`
consumes a world brick as a `VolumeSource` for the raster projection;
occupancy derives per brick for collision and perception queries.

**Done when:** a carved world replays to an identical hash; an unchanged
world uploads zero brick bytes across frames; a carve uploads only its
region; occupancy answers stand, burrow, and see for the near tier.

### G2. The tracer, riding the landed V1 harness

Fragment-only brick-map DDA in the lens's retained pattern: pointers and
brick atlas in 3D textures, no storage buffers, no compute; both souls;
SDF bodies composited as now. *Camera note (2026-08-06):* the DDA is
camera-neutral; with Mesocosm pulled back (ruling 4), the tracer's
long-term primary consumer is Paredros's first-person lens, and
Mesocosm's shipped projection is chosen at playfeel within the
pulled-back framing (mesh raster, side-view, or a pulled-back trace).
G2's proof value (volumetric presentation + downlevel receipts) is
unchanged.

**Done when:** the tracer renders a carved region with interiors correctly
occluded; downlevel and wgpu-GL receipts pass (the glprobe pattern,
landscape §8.8); a browser receipt lands through the same composition seam
and recording discipline V1 proved; 1080p fps is recorded on the 4060 and
on GL, and the number is reported whatever it is.

### G3. The near tier — **core LANDED 2026-08-06; drive/tier wiring LANDED 2026-08-07**

Landed additively as `places/near.rs` (kinematic `step` with slide, climb,
gravity settle, and doorway drops; `spot` sight; `Tier`/`TierLine` with
promote/demote hysteresis over hops). The former `places/hunt.rs` probe
supplied the historical chase receipt, which passes on the real seed-4242
ground: a
half-speed hunter acquires its quarry, loses it behind a hill, projects
along its heading, is forced off a cliff edge, re-acquires at a bored
den's mouth, and follows it inside, with per-tick continuity asserted
throughout (no teleports). Determinism receipt: identical runs
bit-identical. Perf receipt: **300 hunters tick in ~62µs** (release), so
the target population costs well under a millisecond. Tier receipt: the
line does not flap crossing the band, and a far hunter wakes on
promotion. Two movement laws were earned by failure, both the same
lesson: **preferences order, they never refuse** — climb-when-descending
and the cliff-edge comfort drop each deadlocked an agent until the
preference became an ordering with a forced fallback. The Hunter wiring was
deliberately not adopted. E0-E4 now wires the tier line, far-tier receipts,
and anatomy-driven drives into the world's organisms; the old chase receipt
remains probe evidence rather than an authoritative FSM.

**Superseded in destination, 2026-08-06.** The `Hunter` FSM is **a probe,
not the design**. Search/Stalk/Memory is authored behaviour with a tuned
patience constant, and it duplicates a feeding model the ecology already
owns. Its lasting contribution is proving the *queries* work: sight
through terrain, pursuit across a burrow threshold, one-voxel continuity,
and the movement law that preferences order rather than refuse. The
[general model plan](2026-08-06_general_model_plan.md) gate **E4**
replaces it with drive-and-affordance selection, where pursuit is what a
fast, large-mouthed, starving body does about a reachable meal. Do not
build further behaviour on the FSM.

Perception as sight-lines (brick DDA or parry raycast), advisor-tier
locomotion per ruling 10 (parry queries + owned move-and-slide + verlet
chain gait, quantized into integer outcomes, no persisted handles), hunt
and flee for catalogue critters; tier promotion and demotion at a hops
boundary with hysteresis.

**Done when:** a hunter acquires, pursues, loses, and re-acquires the
player across a place boundary and a burrow threshold without stutter or
teleportation; per-tick agent cost is recorded at the target population;
the far tier's aggregate outcomes stay within its existing receipts.

### G4. The burrow run

Compose G0 through G3 into §2's scenario. Its done-conditions are §2's.

## 4. Wing checks

During G1, before the brick container's shape hardens: read it against
Paredros's settlement-edit pressure (landscape V3 wording: one revisioned
snapshot, only affected products update, stale jobs rejected) and against
Isometry's bake reads. The shared organ serves three vessels or it is
Mesocosm-local and says so. Extraction to a permissive crate follows the
standing rule: after two real consumers, never declared in advance.

## 5. Stop rules

- No portals. If crossing a boundary ever reads as a scene transition, the
  design is wrong, not the tuning.
- No second authority. Caches, meshes, SDFs, occupancy planes, and DAGs
  never become world truth.
- No chunk machinery by availability; admit acceleration structures by
  trace against real mutation workloads (V4 discipline).
- No graph ships without its distinctness receipts.
- Compile-only wasm evidence is not browser support.
- A gate proven alone is not proven for the wing; the slice is the unit.
- The founding condition outranks all receipts: somebody wants another
  run.

## Findings

- 2026-08-07, **three directions adopted from the bonsai reading**
  (landscape §8.3 carries the donor row and the unverified-claim caveat):
  1. **The relief lab.** Worldgen tuning becomes an instrument: live seed
     and parameter twiddling over `grown` + `Ground`, re-rendered through
     the tracer per keystroke. Bonsai buys its "voxel shadertoy" loop by
     moving generation to the GPU; we get the same loop keeping authority
     on the CPU, because authoritative gen is deliberately coarse (65²
     relief, ~400 bricks, milliseconds). G0's distinctness receipts
     become feelable, not just assertable. Instrument for G0/G2, not a
     gate.
  2. **The presentation-amplification tier.** The terrain twin of the
     ambience tier, same constitutional line: GPU-side micro-detail
     (micro-relief within faces, scatter, texture variation) derived
     from authoritative bricks, **never entering the replay hash**,
     enhanced capability profile only. The grade already amplifies
     colour; this extends the same contract to apparent geometry.
     Bonsai's "second-stage decoration reading terrain derivatives" is
     the worked example.
  3. **Sim/render LOD unification.** Far places collapse to coarse
     representation (hulls or brick-map mips) under the *same* boundary
     concept that demotes simulation to cohorts: one `TierLine`-shaped
     line serving both, so cohort demotion and visual demotion are the
     same event over the place graph. Direction for G2+, paired with the
     general model plan's E3.


- 2026-08-06, **renderling wgpu-29 port receipt** (fork at
  `Code/crates/renderling` + `Code/crates/crabslab`): the 26→29 bump is
  mechanical. ~277 initial errors were mostly a two-wgpu split (craballoc
  0.3.1 pins wgpu 26; forked at the `v0.6.6` tag + three edits: wgpu pin,
  `PollType::Wait` fields, `pub fn len`). Renderling itself took a scripted
  pass (descriptor field renames, `Option`-wrapped depth state,
  `bind_group_layouts` Option-wrapping, `MipmapFilterMode`,
  `CurrentSurfaceTexture`, `experimental_features`) plus hand fixes. Lib and
  renderling-ui check clean; test modules pass in isolation including image
  goldens. Open: a cross-test device-teardown leak
  (VUID-vkDestroyDevice-05137, OOM accumulation over 95 sequential
  device creations) to chase before the suite is a receipt; upstream's
  suite also assumes parallel per-test devices, which our no-concurrent-
  test rule already forbids. Next gate: `Context::new` on netrender's
  `WgpuHandles` with a composed external texture and a headless golden.
- 2026-08-06, **device-unity probe PASSED**. One instance/adapter/device/
  queue (netrender `WgpuHandles`, feature union `REQUIRED_FEATURES` +
  renderling's four, intersected with the adapter). Renderling
  `Context::new(RenderTarget::from(texture), ...)` rendered a three-
  triangle ortho stage into an `Rgba8UnormSrgb` texture on the shared
  device; netrender composed it at scene-op boundary 0 under vello chrome
  via `ExternalTextureComposite` and presented a master. Headless readback:
  72 distinct colors, full coverage, spans recorded (vello_render 5.1ms,
  master_compose 48µs, tenant render outside netrender's spans). Zero
  copies, zero validation errors. Cohesion contract clauses 1 and 2 hold
  for renderling as a tenant. Probe: scratchpad `rlprobe`; receipt PNG
  captured. Renderling's mesh-tenant candidacy is now **proven at the
  seam**; remaining before the seat is confirmed: the device-teardown leak
  chase, a wing-shaped scene (voxel mesh body via `mesocosm-mesh`), and a
  browser receipt per D0 discipline.
- 2026-08-06, **leak found, fixed, suite green: 95/95.** Bisection
  (context-only survives 120 cycles; context+stage OOMs) plus wgpu's
  alive-resource report (exactly 8 buffers leaked per Stage create/drop,
  zero textures) attributed the teardown leak to a **self-referential Arc
  cycle in craballoc's `SlabBuffer`**: the allocator stores a `SlabBuffer`
  inside the `Arc<RwLock<Option<SlabBuffer>>>` slot that the stored copy's
  own `source_slab_buffer` field points back at. Fork fix: the back-pointer
  is now `Weak` (allocator owns the strong slot; handles upgrade while it
  lives; a dead allocator means no newer buffer can exist). Receipts: stage
  churn now leaks zero buffers over 24 cycles; the long-lived frame loop
  was already clean (flat counts over 600 frames); renderling's full suite
  passes 95/95 single-threaded on wgpu 29. Upstream-worthy fix; the fork
  carries it for now.
- 2026-08-06, **wing-shaped scene through renderling**. A real body grown
  by play (`World::new(2024, 80)`, the fixture's eat loop, 10 living
  parts), greedy-meshed by `mesocosm-mesh`, triangulated by
  `mesocosm-render::build_vertices`, rendered by renderling (perspective
  camera, per-face normals) on netrender's device, composed under vello
  chrome. 120 triangles, every vertex tracing to a part with provenance;
  receipt PNG captured. The pipeline holds end to end with no hand-placed
  geometry. The image also re-demonstrates the known appearance gap: solid
  same-tag volumes read as one blocky mass, which is the axial-plan and
  V2-capsule work's territory, not the tenant's. Remaining for the seat:
  the browser receipt (D0 discipline).

## Progress

- 2026-08-05: plan founded from the engine rumination. Fundamentals
  rulings recorded in §0 and amended into the founding record, `CLAUDE.md`,
  and the landscape doc the same session. V1 and V2 landed 2026-08-04
  (landscape §8.6), so G2 rides a proven browser harness and real bodies.
- 2026-08-06: rulings 9-11 added (reference targets, three-tier physics,
  renderer tenancy). Mesocosm camera amended to pulled back (ruling 4);
  G2 and G3 reworded to match. Renderling fork scout begun (wgpu 26→29
  bump feasibility).
- 2026-08-06, later: renderling tenant proven (device unity, leak fix,
  95/95, wing-shaped scene; see Findings). **G0 constructor landed**:
  `Places::grown` with full distinctness, congruence, and connectivity
  receipts; genesis adoption deferred to its owner. Worlds now differ.
- 2026-08-06, later still: **G1 container landed** (`Ground`): brick
  truth raised from the relief, roofed burrows under nests, carve with
  revision + dirty discipline, occupancy and sight queries, and the
  mesh-projection receipt through `mesh_volume`. World wiring (carve
  intent, snapshot hash, genesis) rides the same deferred swap as G0.
- 2026-08-06, end of day: **G2 probe receipt** (tracer: ~1900/~1420 fps
  Vulkan/GL, pixel-identical, lifecycle in pixels) and **G3 core landed**
  (near-tier movement, tiers with hysteresis, the Hunter, the chase
  receipt on real ground, 300 hunters at ~62µs/tick). All four gates now
  have landed cores or probe receipts; G4's burrow run is composition
  work plus the deferred world adoption.
- 2026-08-06, **G2 tracer probe receipt** (scratchpad `rlprobe`, bin
  `tracer`). Fragment-only brick-map DDA over the real seed-4242
  `Ground`: pointers in an R32Uint 3D texture (16×4×16), materials in an
  R8Uint 3D atlas (128²×64, 383 bricks), one vec4-packed uniform, no
  storage buffers, no compute, device requested at
  `downlevel_webgl2_defaults`. Numbers, reported as found: **~1900 fps on
  Vulkan, ~1420 fps on wgpu-GL at 960×540** with a 420-step budget, and
  the two backends render **pixel-identical** captures (1 px in 518,400
  beyond tolerance 2). The edit lifecycle ran in pixels: a 72-voxel bore
  carved in ~18µs, 8 dirty bricks re-uploaded (4,096 bytes) in ~10ms
  (first-call overhead included), interior then rendered with a rock
  ceiling and correct occlusion out the opening. Findings along the way:
  hit-distance-in-alpha washes PNG receipts (force opaque on capture);
  nest chambers are too small to photograph from inside, so the interior
  receipt is a carved bore, which is the better lifecycle demo anyway.
  Remaining for G2 proper: port into the lens's retained pattern
  (`MapRevision`-style uploads), grade/souls pass, SDF body compositing,
  netrender frame entry, browser receipt.
- 2026-08-07: bonsai reading adopted as three directions (relief lab,
  presentation-amplification tier, sim/render LOD unification); donor row
  and caveat in landscape §8.3.
