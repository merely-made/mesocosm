# CLAUDE.md — Mesocosm Repository Role

This file defines how Claude Code should behave in this repository. Read it
first when starting any session.

---

## Project Identity

**Mesocosm** is a first-person game of lineages: you are a critter in an
enclosed mid-scale ecosystem, you grow only by incorporating other organisms,
and you try to increase your kind's share of the world's biomass. Rogue
Legacy's generational loop pointed at an ecosystem rather than a castle.

Vessel 1 of a three-game wing — Mesocosm (first person), Paredros (second
person), Isometry (third person) — that shares a world substrate, a lineage
model, and a trust plane, but no engine, genre, or schedule.

**Pre-implementation.** The repo currently holds a name reservation and
design docs. There is no game code yet.

See `design_docs/PROJECT_DESCRIPTION.md` for the product description,
`design_docs/DOC_README.md` for the doc index, and
`design_docs/2026-07-30_games_wing_founding.md` for the wing-level
architecture that Paredros and Isometry also depend on.

## Terminology

- **critter**: the plain organism word, wing-wide. Not "creature", not
  "borg" (chat shorthand, a Gotcha Force loan with an IP shadow).
- **animula**: the played soul — the little soul that guests in a body,
  across generations. In-product term only. **Never title anything Animula**;
  ANIMULA NOOK is a live Tencent mark in Class 9 game software.
- **metabolize**: the single verb. World into self, self into world.
- **kleptoplasty**: the incorporation mechanic, named for the real biology
  (an organism eats algae and keeps the working chloroplasts).
- **deme**: banked, unspent. Biology's local interbreeding population; the
  leading candidate if a unit word is wanted. Do not spend it without asking.
- **fili**: lineage across worlds (forks, campaign descent, cross-moot
  grafts). Not event history, not in-world biological descent.
- **tulpa**: the legend and memorial organ — what memory keeps of the dead.
  Proposed, **not yet inscribed in mere's lexicon**; treat as provisional.

Do not coin new names for these concepts mid-session. Naming rounds are
deliberate here: candidates get crates.io, game, studio, and trademark checks
before adoption, and the receipts are recorded.

**Collision to respect**: the bare word *flora* is spoken for platform-side
(a moot's accumulated engrams). Game vocabulary must not reuse it.

## Document Structure

All authoritative design material lives in `design_docs/`. Read
`design_docs/DOC_README.md` first.

| Path | What's there |
| ---- | ----------- |
| `design_docs/DOC_README.md` | Index and AI working principles |
| `design_docs/DOC_POLICY.md` | Documentation governance |
| `design_docs/PROJECT_DESCRIPTION.md` | Product goals, pillars (maintainer-owned) |
| `design_docs/<date>_<keyword>_plan.md` | Active plans |
| `design_docs/archive_docs/<date>/` | Retired plans |

Wing-level material lives once, here, and is cited by the sibling repos.
Do not copy it into Paredros or Isometry.

## General Guidelines

- Rust: standard idioms. No `unsafe` without documented justification.
- 600-LOC ceiling per source file. Split before adding when approaching it,
  and trim comment volume while splitting.
- Plans go in `design_docs/` per the date-keyword-plan convention with
  done-conditions, not time estimates. Never `.claude/plans/`.
- Follow `DOC_POLICY.md` for documentation changes.
- Check the Merely ecosystem before writing a new module: mere, genet,
  netrender, isometry, and the wgpu-* repos may already have the piece or the
  pattern. Name the owning layer before building anything app-local.
- Prefer runtime verification over extended static code tracing. If runtime
  diagnostics are blocked, surface that blocker early.

## Important Don'ts

- **Do not violate the three pipeline laws** (games wing founding record §3).
  What crosses between games is choices under scarcity, not morphology;
  inheritance must be pointable; player history displaces procedural content
  and never gates it.
- **Do not let a stage grow its own engine.** All stages are rule-dressings
  over one substrate. This is the anti-Spore insurance and the wing's single
  most load-bearing rule.
- **Do not describe world nouns as "chartulary-typed."** Chartulary is
  generic (containers, facets, nesting, attributed edits). Factions, places,
  characters, and history are Isometry types today. A portable profile is
  extracted after two real consumers, never declared in advance.
- **Do not build the federation platform first.** It is extracted from
  shipped games. Mesocosm is a candidate for that proof, not yet a consumer.
- Do not add rollback netcode or CRDTs speculatively. The interop model is
  additive facts plus deferred interpretation, which makes conflict
  impossible by construction.
- Do not add features beyond the active plan's current target without
  surfacing the scope change first. **The invariant is care granularity, not
  person purity** (relaxed 2026-07-30; wing founding record §1). Mesocosm is
  care for a **species**. Person may shift — the adaptation phase is
  deliberately third person — provided first person stays home, the shift is
  bounded and diegetic, and each layer could be removed with the game still
  standing. Refuse any shift that needs a second simulation or a second
  renderer: that is the multiplier that actually hollowed Spore.
