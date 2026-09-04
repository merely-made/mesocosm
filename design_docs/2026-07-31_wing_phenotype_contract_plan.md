# Wing phenotype contract: one body, sovereign readings

**Status: cross-vessel decisions and proof plan, revised 2026-08-01. No v1 wire schema
is implemented.** This plan specifies what body identity means across
Mesocosm, Paredros, and Isometry. It does not give the games one capability
system, runtime, renderer, or biological simulation.

The [games wing founding record](2026-07-30_games_wing_founding.md) remains
authority for settled laws. Mesocosm's local body rules live in the
[phenotype plan](2026-07-31_phenotype_plan.md). The
[execution waves plan](2026-07-31_execution_waves_plan.md) owns scheduling.

---

## 1. Why a wing plan is necessary

The wing has already ruled that a body is a part tree, loss cascades, the tree
is shared identity, and each vessel owns its capability fold. The v0 proof pair
predates that ruling:

- `mesocosm.body/v0` carries a flattened appearance grid, per-cell attribution,
  and flat part provenance;
- `mesocosm.chronicle/v0` carries species, flat part provenance, and deeds;
- neither carries parent links, stable subject identity, body revision, or a
  source address unique beyond `(species, part index)`.

The founding record also contains both the newer rule that topology travels and
the older finding that parent/child structure stays home. They cannot both
govern v1. This plan resolves the contradiction.

> **A current body's dependency topology is portable identity data. Its
> geometry and capabilities are projections and vessel-owned readings.**

That rule is narrower than sharing morphology and stronger than sharing a
sprite.

---

## 2. The terms

These are plain working terms, not product names.

- **Subject**: one stable critter identity. Naming it may make it a borg; a
  faction relationship may make it a character. Those additions do not mint a
  replacement subject.
- **Biological line**: Mesocosm's in-world descent relation among organisms.
  It is not Fili.
- **Body revision**: the anatomy of one subject at one causal point. Injury,
  grafting, regrowth, or chassis replacement produces a new revision for
  interchange even if a vessel mutates its live representation internally.
- **Part address**: a part id scoped by subject and body revision. A bare vector
  index is not a portable address.
- **Anatomy**: part identity, structural parent dependency, present or severed
  state, and provenance within a body revision.
- **Developmental rules**: Mesocosm's heritable instructions for growing a
  phenotype. Other vessels may carry them opaquely.
- **Phenotype**: one realized body in one environment, including geometry,
  material, functional links, processes, condition, and damage. Functional
  links may be cyclic; they do not replace the structural dependency tree.
- **Capability fold**: one vessel's derived answer to what that body permits.
- **Projection**: voxels, meshes, sprites, collision hints, summaries, and
  other derived views.
- **Chronicle**: append-only causal facts about a subject or body revision.

---

## 3. Ownership table

| Data | Authority | Portable treatment |
| ---- | --------- | ------------------ |
| Subject identity | shared subject profile | retained exactly |
| Biological line | Mesocosm critter facet | understood by Mesocosm, opaque elsewhere |
| Body revision identity | critter body facet | retained exactly |
| Part ids and parent links | critter body facet | retained as primitive topology |
| Present, severed, grafted state | ordered body history | retained or derived from accepted facts |
| Per-part source provenance | critter body facet | retained exactly |
| Developmental rules | Mesocosm critter facet | opaque unless a compatible consumer opts in |
| Functional links and process allocation | Mesocosm phenotype facet | optional and opaque unless a consumer opts in |
| Voxel geometry and material | current phenotype | optional appearance projection, never foreign rule authority |
| Reach, manipulation, armour, movement | consuming vessel | recomputed locally, never imported as verdict |
| Skills, affinities, trust, relationships | Paredros character facet | independent of body revision |
| Campaign role, allegiance, public history | Isometry facets | independent of anatomy semantics |
| Deeds and observations | chronicle/fact log | append-only with domain interpretation |

This is the substrate/system split applied to bodies. The portable substrate
states which part depends on which and where it came from. A game decides what
that arrangement means under its rules.

---

## 4. Individual continuity and biological descent

The same individual crossing vessels and a descendant founding a new life are
different operations.

### The same subject crosses

The subject id and body revision remain stable. A consumer may render the
provided projection, derive capabilities it understands, or preserve the body
facet opaquely. It must not silently mint another anatomy for the same revision.

If Paredros replaces a chassis, the character facet points to a new body
revision. Skills and relationships remain on the subject's person facet. The
old body may persist as an object, relic, corpse, or discarded revision.

Crossing may preserve or reinterpret phenotype, by explicit choice:

- **Carry this body** preserves current process allocation as faithfully as
  the destination permits. World-required accommodations are explicit
  adaptations and mint a causally linked body revision when they change the
  body.
- **Regrow here** preserves subject identity, genotype, developmental rules,
  and provenance while realizing a phenotype under destination conditions. It
  expects a new body revision and may look or function quite differently.

Neither route imports a capability verdict. The receiver derives its own
reading. The prior phenotype remains pointable, and an opaque consumer preserves
the Mesocosm phenotype facet if the route promises lossless continuation.
The destination declares compatibility and the cost of available
accommodations; the traveler chooses among feasible routes. An incompatible
carry is refused or redirected to regrowth rather than silently rewritten.

### A descendant is founded

Mesocosm consumes a biological-line record, ancestral anatomy and provenance,
accepted causal facts, and its developmental rules. It then mints a new subject
and a new body revision. Ancestral topology may inform inherited motifs, but it
does not make the descendant the same body.

This is where `Chronicle::found` currently collapses two concepts. Its star
body is scaffolding, and its reused species-plus-part indexing is not yet a
complete identity model.

### A memory outlives both

Isometry may retain a sprite, history, relic, or legend after the subject and
biological line are gone. That persistence is not biological descent and is not
Fili. The wing's existing tulpa proposal covers remembered survival if that
term is eventually inscribed. **(2026-09-02 note: this organ is now called
hagiograph; "tulpa" has been renamed to gemot's federated adapter-training
lane; see repo `CLAUDE.md`.)**

---

## 5. Part provenance and branch transfer

`from_species + from_part` cannot distinguish two organisms of the same species
that both have `PartId(3)`. V1 provenance therefore needs a source address:

```text
source subject
source body revision
source part
biological line, when known
acquisition event
```

This is a conceptual field list, not a compile-ready schema.

When a subtree is grafted:

1. the source operation identifies a living subtree under one source revision;
2. destination-local part ids are freshly allocated;
3. internal parent relations are remapped to the new ids;
4. the graft root attaches to one destination part;
5. every destination part retains its source address;
6. internal functional links may cross with the branch, while cut boundary
   links must be re-established under the destination phenotype rules;
7. the source's loss and destination's acquisition are causally linked facts;
8. severing the graft root later removes the imported dependency subtree.

Assimilation is different. It may preserve process provenance and a cause link
while the destination developmental rules produce a different topology. A
consumer must be able to tell which operation occurred.

---

## 6. V1 artifact split

### `mesocosm.body/v1`

The body profile is the home of a current body revision. V1 should add primitive
identity and topology beside its optional appearance projection:

- subject id;
- body revision id and causal predecessor, when any;
- biological-line reference;
- root part id;
- per-part id, optional parent id, state, and source provenance;
- optional flattened cells, attribution, collision hints, or projection recipe.

Offsets, pivots, exact voxel volumes, functional links, process allocation, and
developmental rules are included only when the profile explicitly declares the
relevant optional capability. A weak consumer can preserve the fields it cannot
interpret or use only the baked projection.

The reader still mirrors primitives locally. Carrying a `parent: Option<u32>`
does not require linking `mesocosm-core`.

### `mesocosm.chronicle/v1`

The chronicle should stop duplicating a flat body snapshot. It is an event
stream scoped to a subject and, where relevant, a body revision. A part-affecting
deed names a stable part address and the revision against which the claim was
made.

The body profile and chronicle therefore travel as related engrams or related
parts of a bundle:

- body profile: what anatomy this revision claims;
- chronicle: what happened and what other vessels claim happened;
- projection: what a weak renderer may show.

This separation prevents two independent snapshots from drifting inside the
same package.

### V0 handling

There is no production save population to migrate. When v1 is built, the two
repositories update their fixtures together, retain explicit version refusal,
and remove obsolete v0-only tests rather than carrying a permanent migration
layer. A fixture converter is permitted if it helps prove the change, but it is
not a public compatibility promise.

---

## 7. Authority and concurrent facts

A live body is timing-sensitive state. One simulation authority or ordered
session accepts severing, grafting, regrowth, and body replacement. The anatomy
tree is not a CRDT.

Other vessels append facts and proposals. An Isometry fact that narrates “lost
an arm” is history. A granted operation that names a body revision and part
address may also petition the anatomy authority to recognize a loss. Mesocosm
interprets that claim under its own rules and may accept it, reject it, or
branch the subject's history. The foreign fact remains either way.

Two concurrent claims against the same revision remain visible until the body
domain's materializer sequences or branches them. Set union preserves the
claims; it does not merge two anatomies.

---

## 8. Vessel readings

### Mesocosm

- owns biological development, metabolism, process paths, injury, regrowth,
  and descent;
- treats phenotype as moment-to-moment game state;
- mints new subjects for descendants;
- exports body revisions and causal records.

### Paredros

- refers to a current body revision from the stable subject;
- derives mobility, manipulation, equipment affordances, and embodied combat
  consequences under its own rules;
- keeps skills, personality, trust, and relationships outside the body;
- may replace a chassis without replacing the person.

### Isometry

- can render the optional projection and preserve anatomy opaquely;
- mints no biological part types in its substrate;
- lets a system plugin interpret anatomy only when that campaign wants it;
- appends campaign history and granted body-affecting proposals without
  becoming body authority.

---

## 9. Proof gates

### W0. Contract reconciliation

**Done when:** the wing founding record, phenotype plan, body-profile comments,
and doc index agree that topology is portable, geometry is an optional
projection, and capability is vessel-owned.

### W1. Stable addresses in Mesocosm

**Done when:** two organisms of one species can each have `PartId(3)` and a
provenance record still identifies the exact source; body revision changes are
causally ordered; snapshot and replay preserve those addresses.

### W2. Local branch proof

**Done when:** Mesocosm transfers or assimilates a source subtree according to
the local phenotype plan, loss cascades on both sides, and provenance identifies
the source revision and part after destination ids are remapped.

### W3. Mesocosm to Isometry v1

**Done when:** Isometry reads a v1 body using local primitive mirror structs,
renders its projection, preserves unknown anatomy data, appends a deed addressed
to the body revision, and returns it without linking Mesocosm.

### W4. Isometry to Mesocosm interpretation

**Done when:** Mesocosm distinguishes narration from a granted body-affecting
claim, applies an accepted loss to the correct revision, rejects a stale or
wrong-subject address, and retains every foreign fact.

### W5. Paredros second reading

**Done when:** a character keeps skills and relationships while changing body
revision; losing a relevant subtree changes Paredros-derived capability; and a
generated body uses the same slot as a played one.

### W6. Descendant distinction

**Done when:** returning the same subject preserves body revision identity,
while founding a descendant mints a new subject and phenotype with pointable
ancestry; neither operation is implemented as a flat star rebuild.

---

## 10. Stop rules

- Do not share a wing-wide capability enum or evaluator.
- Do not treat a sprite, flattened voxel grid, collision box, or cached score as
  body authority.
- Do not use species plus local part index as unique provenance.
- Do not duplicate current anatomy in both body and chronicle v1.
- Do not let a foreign vessel mutate anatomy merely by narrating an injury.
- Do not make exact ancestral geometry mandatory for a descendant.
- Do not implement v1 before stable subject, revision, and part addresses are
  proven locally.
- Do not extract a generic body library until Paredros supplies the second
  capability reading. The wire contract may precede that extraction.

---

## Findings

- **2026-07-31:** the v0 body profile is an appearance projection with flat
  provenance, and the v0 chronicle separately duplicates flat provenance.
- **2026-07-31:** the newer wing anatomy ruling contradicts the v0 finding that
  parent/child structure stays home. Primitive topology can cross without a
  Rust type dependency.
- **2026-07-31:** Paredros already separates chassis from skills and personhood,
  which is the real second pressure on body revision identity even though that
  repository is pre-implementation.
- **2026-07-31:** Isometry already proves opaque local-mirror decoding and
  additive history, but it does not yet consume topology.
- **2026-08-01:** cross-world phenotype handling is a choice between carrying
  the current body with explicit destination adaptations and regrowing a body
  from genotype under destination conditions. Both preserve subject continuity
  and point to the prior revision; neither imports capability verdicts. The
  destination declares feasibility and cost while the traveler chooses.

## Progress

- **2026-07-31:** founding contract written; no schema or code change made.
- **2026-08-01:** carry-this-body and regrow-here routes added after the first
  ProcessDef allocation design questions. No schema or code change made.
- **2026-08-01:** crossing authority refined: destination offers feasible,
  costed routes and the traveler chooses. No schema or code change made.
- **2026-09-02, terminology note (doc only):** Mark authorized renaming the
  memorial organ to **hagiograph** and reassigning **tulpa** to gemot's
  federated adapter-training lane. A dated note was added at this doc's
  tulpa mention rather than rewriting the historical text; see repo
  `CLAUDE.md`. No code changed.
