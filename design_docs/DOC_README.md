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
| [2026-07-30_games_wing_founding.md](2026-07-30_games_wing_founding.md) | **Wing-level, cited by the sibling games.** The three-vessel person grammar, the lifecycle and its anti-Spore rule, the three pipeline laws, worlds-as-moots with fili/tulpa/gemot mapping and graduated interop, the verified state of the stack (what is ready, what is not), the two-game proof pair as the next threshold, the shared vocabulary, and the open questions. |
| [2026-07-30_mesocosm_founding_plan.md](2026-07-30_mesocosm_founding_plan.md) | Vessel 1's own design and phases: the single verb (metabolize), incorporation-only parts with provenance (kleptoplasty), trait count rather than cell count as the real axis, worlds as sets of conditions (proto/ocean/ice/hothouse/gas-giant, and frankly invented ones), acquisition beyond eating, the three kingdoms (plant/animal/fungus — decomposers make the biomass economy a cycle, not a ratchet), the metabolic budget, **the epoch loop** (played epoch, then a turn-based adaptation phase where every species spends its bank in initiative order) with its checked prior art, the arena as ecological competition rather than a mode, open questions on roster size / speciation / cascades / round limits, the storyteller and where it belongs, the death model, tone, expression, and phases M0–M5. |
| [2026-07-30_engine_and_render_lane_landscape.md](2026-07-30_engine_and_render_lane_landscape.md) | Research, no decision: the pure-Rust engine field (Bevy, Fyrox and its editor, Macroquad, ggez, and what is dead), why "custom wgpu" is an assembly with eleven of thirteen components already owned, five approaches worth probing including splitting the bet per vessel, how to decide by probing M0 more than once, what must stay shared regardless of renderer, and the flag that the proposed 3D Isometry contradicts a standing ruling in that repo. |

## Archive

None yet. Retired plans go to `archive_docs/<YYYY-MM-DD>/`.
