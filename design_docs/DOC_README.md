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
| [PROJECT_DESCRIPTION.md](PROJECT_DESCRIPTION.md) | Product goals and pillars (maintainer-owned): the mesocosm names the enclosure scale, whose roster can range from one-trait cells through complex and fantastical organisms. |
| [2026-07-30_games_wing_founding.md](2026-07-30_games_wing_founding.md) | **Wing-level, cited by the sibling games.** The three vessels and their care granularities (species / individuals / community), **the invariant being care granularity rather than person purity** with its three replacement guardrails and the design space that relaxation opens, the lifecycle and the restated anti-Spore law, the three pipeline laws, worlds-as-moots with fili/tulpa/gemot mapping and graduated interop, layered critter/character/place/faction profiles carried as engrams, signed multi-writer facts with per-domain materializers rather than a universal CRDT, the MPL/permissive-library/CC-assets license split, the verified stack, and the two-game proof pair. |
| [2026-07-30_mesocosm_founding_plan.md](2026-07-30_mesocosm_founding_plan.md) | Vessel 1's own design and phases: metabolize, incorporation-only parts with provenance, trait count as the real axis, an Exocosm-informed world-condition grammar including proto/soup and impossible worlds, the three trophic strategies, metabolic costs and material flow, **the epoch loop** with complex lineages committing first and simpler lineages adapting later, complexity-frontier switching while inactive lineages keep evolving, checked prior art, ecological competition, storyteller, death, and phases M0–M5. |
| [2026-07-30_engine_and_render_lane_landscape.md](2026-07-30_engine_and_render_lane_landscape.md) | Research, no decision. Rewritten same-day after review. The verified pure-Rust engine field (Bevy 0.19, Fyrox 1.0.1 and its editor, ggez 0.10, Bones as renderer-neutral missing-middle, Renderling for custom 3D; macroquad ruled out on an unpatched soundness advisory; Ambient paused). **The correction that the stack owns a host skeleton, not a game runtime**, with the missing load-bearing services enumerated. Six approaches to probe. The per-layer sharing table, the restated anti-Spore law, and why camera is not person. |
| [2026-07-31_execution_waves_plan.md](2026-07-31_execution_waves_plan.md) | **Authority on ordering** across the wing; the governing plans own the *what*, this owns the *when*. **Wave 1 complete**: `mesocosm-core` owning state while hosts only project, the Genet host, and the Isometry projection coupling by data rather than types. The Bevy host and the Renderling probe were **dropped** once the render lane was decided. Wave 2 in progress: 2.2 (epoch and ecology lab, three authored worlds) and 2.4 (the proof pair, Law C demonstrated in the consumer) are **complete**; 2.1 playfeel and 2.3 Paredros social proof remain. Carries the finding that the adaptation phase converges by round six with zero extinctions, because income is flat and uncontested. |
| [2026-07-31_phenotype_plan.md](2026-07-31_phenotype_plan.md) | **Decisions plan, nothing built.** What a body is *for*, and the largest open hole in the project: three competent models of a creature that do not explain one another. Seven rulings in dependency order, each with options, a recommendation, its cost, and who rules it. D1 capability from processes and paths rather than folds; D2 where it computes, given that core cannot read voxel contents so surface and occlusion need box proxies; D3 deleting the trait array because `BodyPlan` already is the growth-parameter system; **D4 splitting the meal, which depends on nothing and should go first**; D5 contested income; D6 the chronicle tree at v1; D7 the field journal. |
| [2026-07-30_body_pipeline_and_host_probe_plan.md](2026-07-30_body_pipeline_and_host_probe_plan.md) | Plan. The shared organ is the **body pipeline**, not one renderer. `.vox` is the first authoring input; an explicit parts graph over content-addressed volumes is portable truth; greedy meshes, rigid transforms, deformation, and sprite bakes are projections and caches. Names runtime attachment as the unproven assumption, specifies interchange profile v0, and lays out R0–R4 with Genet/engine/Bones receipts. |

## Archive

None yet. Retired plans go to `archive_docs/<YYYY-MM-DD>/`.
