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

## 3. The hosts are staged (Mark, 2026-08-02)

Two host routes, ruled complementary rather than competing, with the game's
own structure picking the boundary:

- **Route A, landed same day: the HUD lane.** During an epoch you are a
  critter; the chrome that belongs there is a textless HUD. The leaf paints
  `PaintCmd`s, `paint_list_render` lowers them, netrender's vello path
  rasterizes into a transparent texture on the game's own device (the
  arrangement the workspace's wgpu-29 pin exists for), and an overlay pass
  blends it over the frame. Vello overwrites its whole target, so the
  texture-then-composite indirection is forced, not chosen.
- **Route B, deferred to its consumer: the cambium lane.** Between epochs is
  where text lives: readings, records, naming, adopt-or-branch. That screen
  is panels and fields, the catalog's home turf, and the honest trigger for
  standing up the full genet host (isometry-genet's shape) with the game
  view embedded as an external texture.
- **The guard**: route A never grows a text hack. The moment chrome wants a
  word, that is the consumer pull for route B, not a reason to teach the
  HUD lettering.

### What route A shipped

`mesocosm-render::overlay` (the premultiplied blend pass),
`Renderer::render_scene_with` (a caller pass between scene and readback, so
captures show what windows show), and `mesocosm-genet::hud` (netrender on
the shared device, refresh gated by the leaf's own retention). Verified at
`testing/mesocosm/14_minimap.png`: nine cells tinted by dominant lineage,
site dots, the player marker in its cell.

### The stack bug this shook out

`paint_list_render` copied unpremultiplied `ColorF` into netrender's
premultiplied scene fields, whose consumers divide by alpha on the way to a
brush. Every translucent fill over-brightened toward white; opaque content
passed untouched, which is why genet's mostly opaque DOM never surfaced it.
The minimap's 35%-alpha cells rendered as a white sheet. Fixed at the owning
layer (netrender, with a regression test); `hud_raster.rs` keeps a headless
probe of the exact leaf-to-texture path with a premultiply bound.

## 4. Not yet built

- **The generated backdrop.** The world rendering its own enclosure
  top-down, composited under the minimap. The overlay pass and capture seam
  are ready for it.
- **Interactivity.** Cells as hit targets wait for the route B DOM host.
- **Fields over cells.** `FieldExtent::Polygon` exists; nothing evaluates a
  field over a place yet. Consumer-pull holds.
- **Dominance-shift receipt.** Two captures of one long run showing
  different holders, once a scenario reaches far enough to flip a region.

## 5. Done conditions

- ~~The minimap visible in the windowed host~~ **done 2026-08-02**, backdrop
  still open.
- Dominance shifts on screen when the ecology shifts (verifiable with a
  scenario run: two captures, different holders).
- ~~A `testing/mesocosm/` capture showing it~~ **done**: `14_minimap.png`.
