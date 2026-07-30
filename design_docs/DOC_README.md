# design_docs Index

Canonical index for `design_docs/`. Per DOC_POLICY §5, this file wins over
any other index and is updated in the same session as any doc change.

## Working principles for AI assistants

- Read `../CLAUDE.md` first for repo role, terminology, and don'ts.
- Verify claims against the codebase and the sibling repos, not doc-to-doc
  consistency. This wing's founding record was corrected once already for
  trusting a stale index line over the plan it indexed.
- Plans carry done-conditions, not time estimates.
- `PROJECT_DESCRIPTION.md` is maintainer-owned; surface contradictions, do
  not edit unasked.
- The substrate/system split is load-bearing across the whole wing: one
  substrate, many rule-dressings. Keep it that way in every doc.
- The three pipeline laws (choices-not-morphology, pointable inheritance, no
  homework) govern anything that crosses between games. A violation is a
  design bug, not a preference.

## Active docs

| Doc | What it is |
| --- | ---------- |
| [DOC_POLICY.md](DOC_POLICY.md) | Documentation governance |
| [PROJECT_DESCRIPTION.md](PROJECT_DESCRIPTION.md) | Product goals and pillars (maintainer-owned) |
| [2026-07-30_games_wing_founding.md](2026-07-30_games_wing_founding.md) | **Wing-level, cited by the sibling games.** The three vessels and their care granularities (species / individuals / community), **the invariant being care granularity rather than person purity** with its three replacement guardrails and the design space that relaxation opens, the lifecycle and the restated anti-Spore law, the three pipeline laws, worlds-as-moots with fili/tulpa/gemot mapping and graduated interop, the verified state of the stack, the two-game proof pair as the next threshold, the shared vocabulary, and the open questions. |
| [2026-07-30_mesocosm_founding_plan.md](2026-07-30_mesocosm_founding_plan.md) | Vessel 1's own design and phases: the single verb (metabolize), incorporation-only parts with provenance (kleptoplasty), trait count rather than cell count as the real axis, worlds as sets of conditions (proto/ocean/ice/hothouse/gas-giant, and frankly invented ones), acquisition beyond eating, the three kingdoms (plant/animal/fungus — decomposers make the biomass economy a cycle, not a ratchet), the metabolic budget, **the epoch loop** (played epoch, then a turn-based adaptation phase where every species spends its bank in initiative order) with its checked prior art, the arena as ecological competition rather than a mode, open questions on roster size / speciation / cascades / round limits, the storyteller and where it belongs, the death model, tone, expression, and phases M0–M5. |
| [2026-07-30_engine_and_render_lane_landscape.md](2026-07-30_engine_and_render_lane_landscape.md) | Research, no decision. Rewritten same-day after review. The verified pure-Rust engine field (Bevy 0.19, Fyrox 1.0.1 and its editor, ggez 0.10, Bones as renderer-neutral missing-middle, Renderling for custom 3D; macroquad ruled out on an unpatched soundness advisory; Ambient paused). **The correction that the stack owns a host skeleton, not a game runtime**, with the missing load-bearing services enumerated. Six approaches to probe. The per-layer sharing table, the restated anti-Spore law, and why camera is not person. |
| [2026-07-30_body_pipeline_and_host_probe_plan.md](2026-07-30_body_pipeline_and_host_probe_plan.md) | Plan. Answers "a render lane for all three?" — no at that layer, but the **body pipeline** underneath is genuinely shared, because every vessel must draw the same creatures. Names the unproven assumption (runtime part attachment, which `isometry-voxel` has never demonstrated), specifies the portable artifact that becomes interchange profile v0, sets the extraction discipline for the missing runtime middle, and lays out phases R0–R4 with the host-probe receipts. |

## Archive

None yet. Retired plans go to `archive_docs/<YYYY-MM-DD>/`.
