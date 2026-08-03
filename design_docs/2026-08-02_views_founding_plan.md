# Views Founding: Adapter-First UI, and the Minimap as First Chrome

**Date:** 2026-08-02
**Status:** First slice landed (adapter + leaf, tested). Host embedding open.
**Companions:** mere's projection proofs plan (arrangement register; P4),
isometry's `2026-08-02_overmap_presentation_plan.md` (the second Hulls
consumer), the epoch boundary plan (places, §12).

---

## 1. The rulings (Mark, 2026-08-02)

- **Adapter-first.** `mesocosm-views` discloses world facts as a
  `sceno::Score`, `scenomise` solves placement, leaves realize the solved
  scene. A layout computed in the views crate is in the wrong crate. This
  adopts the posture isometry had to migrate to (P4 deleted its hand-rolled
  layout); mesocosm never incurs the debt.
- **Hulls first** among the reserved arrangements (Mosaic, Atlas, Hulls,
  Armature): the most versatile, with inference, metadata, and scripting
  applicable via fields. Landed in mere as `Arrangement::Hulls` plus
  `numen::FieldExtent::Polygon` (both 2026-08-02).
- **First chrome is the minimap**: which lineage dominates each region.
- **Region meaning is per-vessel.** The contract carries geometry and source
  refs; this vessel decides a cell's tint is the dominant lineage.
- **Backdrop is a DOM layer, dynamically generated** — not static, not merely
  dynamic. Mesocosm's natural backdrop is the world rendering its own
  enclosure top-down (the capture path already renders offscreen); painted-in
  effects stay available via sprigging's vello scene without committing now.
- **Dependencies**: this work is mesocosm's first coupling to genet
  (sprigging) and mere (sceno, scenomise) — git deps, branch-tracked, the
  woodshed pattern.

## 2. What landed

`crates/mesocosm-views`, two modules under the ceiling:

- **`minimap.rs`** — the adapter. `minimap_score` discloses places as
  coordinate-placed sites in a Hulls score (units are voxels, untransformed);
  `dominant_lineages` reads biomass of the living per region at projection
  time (derived, never stored — the same discipline as capability and
  temperament); `lineage_tint` gives every lineage a deterministic
  golden-angle colour.
- **`leaf.rs`** — `MinimapLeaf`, a sprigging `Leaf`: region cells filled
  translucent and stroked in the holder's tint, site dots, a player marker.
  Paints what it is handed, computes nothing, testable without a GPU.

**The congruence test is the load-bearing one**: `Places::at` and the Hulls
solver are the same nearest-site rule, so a sampled position lands in the
scene cell of exactly the place the simulation says it is in. The minimap
draws the world's own regions, not a cartographer's approximation.

## 3. Not yet built

- **Host embedding.** `mesocosm-genet` is raw winit + wgpu with no DOM or
  vello lane, so nothing on screen splices the leaf yet. Two candidate
  routes, undecided: rasterize the leaf's paint commands in the existing
  host, or stand up a cambium host lane beside it (isometry-genet's shape).
- **The generated backdrop.** Wants the top-down offscreen render on a
  cadence; belongs with host embedding.
- **Interactivity.** Cells as hit targets (the graph-canvas pattern) once a
  DOM host exists.
- **Fields over cells.** `FieldExtent::Polygon` exists; nothing in mesocosm
  evaluates a field over a place yet. Consumer-pull holds.

## 4. Done conditions

- The minimap visible in the windowed host, over a generated backdrop.
- Dominance shifts on screen when the ecology shifts (verifiable with a
  scenario run: two captures, different holders).
- A `testing/mesocosm/` capture showing it.
