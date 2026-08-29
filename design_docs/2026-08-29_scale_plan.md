# Scale Plan (2026-08-29)

**Status: founded 2026-08-29; ruled by Mark the same day. Execution begins
after TD6 (matter cycle) lands — every rung below touches the same crate.**

The ruling, in Mark's words: "scale is remarkable to behold, a feature in
its own right! a large scale intelligible terrarium-esque world generator,
that scales from the immediate scale to huge scale through the magic of
zoom... would be super cool to have a big, dynamic local ecology. it is
also a good systems test, a fantastic one! all scales matter and this one
does too. we should definitely expand on those arbitrary limits to stress
the bounds of what is possible with this stack."

This supersedes the standing caution in the landscape doc's stop rules for
this repo: chunk/streaming machinery was deferred until real scale
pressure existed, and this ruling creates that pressure deliberately —
admission is still by trace (each rung lands with a measured receipt), but
the demand is no longer hypothetical. The 2026-08-05 rule that sim tier
and render LOD share facts, never mechanism, stands untouched.

## Where the arbitrary limits actually are (measured 2026-08-29)

- **Terrain is O(area), never O(volume):** `SURFACE_BAND = 24` caps height,
  bricks are 8³ at 1 byte/voxel. Today ±16 → ~100 bricks, ~50 KiB.
- **±87 is the unwindowed wall:** `modulus::MAX_BRICKS = 2047` (the 1 MiB
  atlas) with the whole enclosure resident. The windowed mode
  (`with_capacity` + `retarget`) is already proven at ±256 — by Paredros.
- **The place graph does not scale at all:** `PLACE_SIDE = 3` is fixed at
  nine regions whatever the enclosure; raising it reorders RNG draws and
  breaks every existing seed's replay (accepted — hashes break by design
  every round this week).
- **The far tier is a pessimization today:** far organisms skip physics but
  scan the whole living/carrion roster with no distance cap — O(far × N).
  Harmless at diameter 2; quadratic the moment the graph grows.
- **`Cohort` exists and is unwired:** conserved-total bucketing with
  deterministic split, currently used only for tally reporting. Every far
  organism still runs the full individual pass, so tiering changes
  fidelity, not cost.
- **Population is the real ceiling:** measured N^1.6 organism loop — 300
  bodies ≈ 1.23 ms/tick, saturation ≈ 4,700 at the 10 t/s budget on the
  reference machine (Ryzen 9 7940HS).
- **The snapshot is monolithic:** `state_hash` re-serializes and re-scans
  the whole world every call; fine per receipt, O(E²) growth.

## The ladder

Each rung pushes one limit, lands with a receipt, and stops if the trace
says the stack breaks — finding the break IS a deliverable ("a good
systems test"). And per Mark, "with optimization, of course": when a
rung's receipt shows a binding constraint, the rung optimizes it rather
than merely recording it — the already-named candidates being the
far-tier O(far × N) scan, `Places::at`'s linear lookup, the N^1.6
organism loop's bucketing, and (only once proven binding) the monolithic
snapshot hash. Optimization is admitted the same way everything else is:
by a before/after trace, never by suspicion. Population scales with area
at every rung, or the world gets diffuse instead of big.

**S1 — the wide terrarium (constants only).** `ENCLOSURE` 16 → 64, founders
scaled with area, the instrument re-run at size. No new machinery; proves
the whole existing stack at 16× the area. Receipt: instrument verdicts and
tick/hash/memory costs at ±64 vs ±16; a headed capture; the played host
still holding frame at 10 t/s.

**S2 — resident is not visible (adopt the windowed atlas).** Mesocosm
adopts `BrickMap::with_capacity`/`retarget` — the Paredros-proven paging —
keyed to the section slab, so the enclosure can exceed the 1 MiB atlas.
Receipt: ±128 or beyond playable; retargets without texture churn; replay
hash untouched by camera movement (residency is presentation).

**S3 — the region tier becomes real (the big one).** `PLACE_SIDE` grows
with the enclosure; `Places::at` gets a spatial index; far-tier perception
gets a distance cap and bucketing (kill the O(far × N) scan); and `Cohort`
becomes an actual execution path — far organisms simulated as cohorts with
conserved totals, split back deterministically on promotion. This is what
buys a big population instead of a big empty map. Receipt: the instrument
at thousands of organisms; per-tick cost curves near vs far; conservation
(TD6's invariant) holding across cohort merge/split — the matter cycle
must survive compression.

**S4 — the magic of zoom.** Continuous pull-back from critter scale to
whole-world scale: the section's orthographic extent as a zoom axis,
bodies handing off to silhouettes to region tint as the roster cap and
legibility demand — sim tier and render LOD sharing hop/region facts,
never mechanism, per the standing rule. Presentation only; never in the
trace. Receipt: captures across the zoom range; the far view legible
(territory, flows) rather than a smear.

**S5 — the stress receipt.** The systems test named as such: a matrix of
enclosure × population across the rungs — tick cost, hash cost, memory,
atlas behavior, where each ceiling actually stands after S1-S4 — written
down like the soil probe was, as the stack's measured envelope. Receipt:
the document itself, with every number reproducible from a cargo run.

## Not this ladder

The world tier above the enclosure — many regions as a world of records,
the overmap/chartulary shape — stays deferred: it is a graph-and-records
problem, not a voxel problem, and it should be designed against the wing's
fidelity contract with Mark rather than grown out of a stress test.
Incremental/merkle snapshot hashing is admitted only when a rung's receipt
shows the monolithic hash is the binding constraint, not before.

## Findings

- **2026-08-29: the creatures are too big for the world, and S1 is the fix**
  (Mark, from the captures: "don't the creatures in the game seem super big
  to you?"). The measured ratio, from `PartPalette::primitive` against
  `ENCLOSURE = 16` (a 33-voxel span): a mass segment is `[2,2,2]` = 5 voxels
  (15% of the world's width), a limb `[4,1,1]` = 9 voxels long (**27%**), a
  plate `[4,4,1]` = 9x9. A played body carries ~61 parts, and 60 organisms
  at that size share a 33-voxel box — hence the overlapping pile in
  `td3_roster.png`. A creature is also larger than a terrain brick
  (`BRICK = 8`), so body features resolve finer than the world does, which
  is backwards.
  **Fix the world, not the bodies.** `part_ceiling_mg` prices voxel volume
  at 0.8 mg/voxel, so shrinking half-extents would cut mass ceilings by
  roughly eightfold and drag the economy TD6/TD7 just tuned; it would also
  spend shape legibility a 5-voxel body cannot spare. S1 costs neither. At
  `ENCLOSURE = 64` (129 voxels) a limb falls from 27% to 7%.
  Two consequences that argue the same way:
  - **Per-voxel soil granularity is currently moot.** The grain was chosen
    (2026-08-29, on measured evidence) so roots could hunt across
    neighbouring columns — but a 9-voxel producer already straddles nine.
    Its forage radius sits inside its own footprint until the world grows.
  - **The world is crossable in about two seconds** (1-2 voxel moves, 33
    voxels, 10 t/s ≈ twenty ticks). Scale is a felt property, not only a
    systems-test one.

## Progress

- **2026-08-29:** founded on the day's scaling sweep; ruled by Mark
  (scale as a feature, stress the bounds). Execution queued behind TD6.
