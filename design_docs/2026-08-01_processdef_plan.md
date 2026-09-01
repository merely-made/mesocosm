# ProcessDef: authored biology without ability flags

**Status: in progress, refreshed 2026-09-01. PD1b is COMPLETE. Its identity
slice landed 2026-08-08 (native `ProcessDef` records, per-definition digests,
a registry) and its allocation half landed 2026-09-01: a private
`BodyPhenotype` wrapper, per-part authoritative cell-graph mosaics seeded from
geometry, one proposal and one validator for direct and automatic arrangement,
and an intentional snapshot format bump. PD2's one additional embodied process
is next. No pack loader or Mesocosm Piccolo host exists yet. PD0 and the PD1a
allocation design pass are complete.**

This plan owns Mesocosm's extensible process vocabulary, developmental
expression boundary, content-pack shape, and Piccolo proof. The
[phenotype plan](2026-07-31_phenotype_plan.md) continues to own body and
capability semantics. The
[dependency ledger](2026-08-07_dependency_ledger.md) owns scheduling, and the
[playable ecology plan](2026-08-31_playable_ecology_plan.md) owns the product
integration proof that consumes PD1b and PD2.
The [wing phenotype contract](2026-07-31_wing_phenotype_contract_plan.md)
owns what can cross into Paredros and Isometry.

---

## 1. The ruling

`ProcessDef` is the right name for one versioned, namespaced biological or
physical transformation that a phenotype may perform. It is not a gene, a
body part, a capability, or a script callback.

The full causal ladder is:

> acquisition and inheritance -> developmental instruction -> expression in
> parts and channels -> satisfied process path -> vessel-owned capability ->
> ecological consequence

That distinction keeps the system broad without making it vague. A process
definition can say that exposed tissue transforms light into usable energy.
A developmental instruction can say where that tissue tends to grow and when
it expresses. The current phenotype records which sites actually expressed it.
Mesocosm then decides whether those sites, their connections, the available
light, and the body's costs produce a working capability.

The process definition never says `can_photosynthesize = true`. Piccolo never
writes energy into the world. Capability remains a reading of embodied facts.

### The P2 rule changes, but its principle survives

P2 currently says "processes are read, not stored" because geometry alone
derives its three native processes. That was the correct first proof: a long
part contracts and buys reach, while a plate does not.

Geometry cannot remain the only process authority. Two equally flat plates
may be armour and photosynthetic tissue. A bulky part may be stomach, storage,
lung, or inert ballast. The mature rule is therefore:

> **Process allocation is phenotype data produced by validated development;
> capability is always read from allocation, anatomy, channels, cost, and the
> current environment.**

An expressed process is not a stored ability score. It is closer to a working
organ: located on a particular part, paid for, connected or disconnected,
conditioned by the environment, and pointable to its developmental and source
provenance. Reshaping, starving, severing, or moving worlds can still make it
stop working without editing a capability number.

**Phenotypic plasticity is intended.** `BodyPlan` constrains structural growth;
it does not uniquely determine the realized critter. The same body plan, and
even the same inherited developmental program, may produce different process
allocation when the environment, acquired sources, life history, or recorded
entropy differs. Identical inputs and entropy still produce an identical
phenotype. A lineage is therefore heritable without being a mold that stamps
the same organism every time.

---

## 2. What is and is not a process

The broad gene-expression system needs several kinds of consequence. Treating
all of them as `ProcessDef` would recreate the old trait array with strings.

| Consequence | Working representation | Example |
| --- | --- | --- |
| transformation | `ProcessDef` | light capture, digestion, contraction, secretion |
| shape and attachment tendency | developmental anatomy rule | bilateral limbs, branching hyphae, a thick shell |
| material property | material or tissue definition | antifreeze, stiffness, translucency, insulation |
| internal connection | channel-development rule | routed energy, redundant signals, centralized control |
| regulation | expression rule | active in cold, suppressed in darkness, induced after injury |
| lifecycle | developmental trigger | spores after drought, metamorphosis, budding at surplus |
| appearance and signalling | phenotype presentation fact | warning colour, mimicry, scent |
| relationship | ecological or social fact | pollinator dependence, gut symbiont, colony quorum |

Only the first row is `ProcessDef`. The first implementation adds process
expression. Other developmental operations join the typed proposal vocabulary
only when a played phenotype needs them. There is no universal `TraitDef` in
this plan, and the provisional `epoch::Trait` array remains until the
phenotype plan's deletion gate is met.

### A small vocabulary can still produce a large pool

The intended breadth comes from composition, conditions, placement, and cost,
not hundreds of bespoke booleans. Illustrative families include:

- capture of light, heat gradients, chemicals, radiation, pressure, or magic;
- exchange and respiration in air, water, methane, ammonia, or stranger media;
- digestion, fermentation, detoxification, concentration, and storage;
- contraction, support, adhesion, buoyancy, jetting, burrowing, and flight;
- sensing light, vibration, chemistry, heat, fields, time, or nearby signals;
- secretion of venom, glue, ink, spores, silk, minerals, lures, or shelters;
- repair, regeneration, dormancy, shedding, budding, and metamorphosis;
- communication and control through local, routed, redundant, centralized, or
  quorum requirements;
- symbiotic exchange across the skin and ecological engineering outside it.

A new process is useful only when a game rule consumes its inputs and outputs.
Names in a catalog do not create mechanics.

### One displayed trait, three compiled programs

The epoch UI may present a generated trait as one coherent biological idea.
That presentation does not imply one runtime representation. A candidate
lowers into the mechanisms that own its consequences:

1. A **condition program** evaluates newly admitted evidence or a bounded
   reading when a relevant event arrives. It governs discovery eligibility.
2. A **development program** runs at founding, filial growth, paid remodeling,
   repair, or another named discrete trigger. It proposes anatomy, allocation,
   channels, regulation, or lifecycle instructions through the existing atomic
   validator.
3. A **`ProcessDef`** is consumed by an authored native evaluator during the
   repeated world work that actually transforms matter, energy, signal, force,
   or medium.

The first two are not variants of `ProcessDef`, and none is a universal
per-tick trait AST. All three use versioned typed data, integer or fixed-point
values, explicit input facts, and hard bounds on node count, edge count, depth,
evaluation work, and output size. Same-tick dependencies form an acyclic graph.
Feedback requires named state and a later tick, so a generated cycle cannot
recurse inside one evaluation.

Combination semantics belong to native consumers. A consumer declares whether
multiple contributions gate, add with saturation, take a minimum or maximum,
or compete for one exclusive slot. The generator chooses admitted definitions,
parameters, placements, and combination laws; it cannot coin an operator that
the validator and explanation path do not understand.

A realized generated candidate persists the exact component definition
references, parameters, placements, condition, generator provenance, and
digest. Its player-facing name is presentation. Save, replay, and peers do not
reconstruct mechanics by rerunning the generator from that name or a seed.

[Kappa’s rule language and tools](https://tools.kappalanguage.org/docs/KaSim-manual-master/KaSim_manual.htm)
are useful prior art for handling combinatorial structured state through local
typed patterns, signatures, static reachability and influence analysis, and
causal traces. Mesocosm can borrow those admission questions while retaining
its deterministic fixed-step transaction path. It does not adopt Kappa’s
stochastic event scheduler or turn anatomy, lifecycle, relationships, and
presentation into one graph-rewrite engine.

This split changes no phase order. PD2 first proves one repeated process; PE2
then composes it with one event-driven discovery condition. PE3 proves a
discrete development program that expresses the admitted option in a
descendant.

---

## 3. Core vocabulary

These are working Rust shapes, **illustrative rather than compile-ready**.
The implementation begins with only the fields demanded by the first proofs.

```rust
pub struct ProcessRef {
    pub id: QualifiedProcessId,   // for example, "mesocosm:contract"
    pub definition: DefinitionDigest, // exact admitted definition
}

pub struct ProcessDef {
    pub id: QualifiedProcessId,
    pub abi: u16,
    pub inputs: Vec<FlowPortDef>,
    pub outputs: Vec<FlowPortDef>,
    pub site: SiteRequirements,
    pub basal_cost: CostDef,
    pub active_cost: CostDef,
}

pub struct ExpressedProcess {
    pub site: ProcessSiteId,
    pub process: ProcessRef,
    pub part: PartId,
    pub cells: Vec<CapacityCellId>,
    pub cause: ExpressionCause,
    pub source: Option<Provenance>,
}

pub struct CapacityCell {
    pub id: CapacityCellId,
    pub neighbours: Vec<CapacityCellId>,
}

pub struct PartAllocation {
    pub part: PartId,
    pub cells: Vec<CapacityCell>,
    pub sites: Vec<ExpressedProcess>,
}
```

This sketch is illustrative rather than compile-ready. PD1a removed the two
stored capacity scalars from the earlier sketch. Structural capacity is the
number of living cells in the graph and a site's allocation is the cells it
occupies. One source of truth makes conservation countable instead of asking
two integers to agree.

`QualifiedProcessId` is pack-qualified at the authoring and wire boundaries.
`DefinitionDigest` prevents the same friendly id from silently changing its
meaning without making every imported process depend on the source world's
whole ruleset. A world separately records the `RulesetDigest` of the complete
set it admitted. The runtime may intern a process reference to a compact
integer inside one simulation, but that integer never becomes portable
identity.

`FlowKind` remains distinct from process identity. Its first host-owned set is
small: matter, energy, signal, force, and medium. A process transforms or
exposes flows; a channel carries them. A pack can qualify a substance or medium
without adding a new transport law. A mismatch requires an embodied adapter
process, not a compatibility table.

`ProcessDef` data is deterministic and integer-only. It contains no Lua value,
closure, filesystem path, clock, random generator, or rendering fact. It can
therefore be admitted into a world's ruleset and evaluated by
`mesocosm-core` without running a script in the simulation loop.

### The allocation mosaic

**Ruled 2026-08-01.** Each part has finite process capacity. Several processes
may share it, and every expressed site consumes some of the same budget. A leaf
may capture light, signal with pigment, and repair itself, but emphasizing one
leaves less tissue for the others. This is the correct competition boundary:
the tradeoff lives inside the organ rather than in a flat organism-wide score.

The working UI is a Diablo-like inventory for each part: process sites occupy a
small **allocation mosaic**, and their relations can be inspected and edited at
developmental moments. Adjacency matters because processes that remain near
one another across turns and epochs may cooperate, interfere, or become
candidates for hybridization. The mosaic is phenotype state keyed to a stable
`PartId`; it is not a capability verdict and it does not make renderer voxels
simulation authority.

**Ruled 2026-08-01: the mosaic is an authoritative graph of capacity cells.**
A process site occupies a connected subgraph. The evaluator may inspect each
cell's neighbours, a site's boundary, connected regions, and the entire
part-mosaic. A Diablo-like 2D layout is a useful editor and inspector, but its
screen coordinates are not authority; radically different parts do not have to
pretend they are rectangles. Allocation still conserves capacity: occupied
plus free cells equal the part's current capacity, and damage or shrinkage
cannot leave a site occupying cells that no longer exist.

### Hybridization across epochs

**Ruled 2026-08-01.** Adjacency may produce a somatic compound during an
embodied epoch: neighbouring sites can cooperate, interfere, or expose a new
transformation without rewriting either parent. Use, survival, and repetition
make that compound eligible for the adaptation bank. The adaptation phase may
then stabilize it as a heritable form, citing both parent processes and the
lived compound history.

This gives the two game phases different jobs. The first-person epoch discovers
the combination by living with it; the adaptation turn decides whether the
lineage commits to it. A mid-epoch adjacency never silently rewrites genotype.
Process hybridization is distinct from combining two biological lines, though
a stabilized process may later participate in lineage hybridization. Still
open is the stabilized form's identity: a newly admitted derived `ProcessDef`,
or a heritable compound recipe evaluated through its parents.

### Directed graft affinity

The proposed balancing model is a directed affinity graph over tissue domains,
not the existing `Kingdom::{Producer, Consumer, Decomposer}` trophic role. A
default world might use animal-like -> fungal-like -> plant-like -> animal-like
as the favoured cross-graft cycle, with the reverse edges disfavoured. The bare
word `flora` remains reserved by the platform and is not a game-data label.

The graph belongs to world or pack data rather than a universal three-value
enum. Soup organisms, colonies, mineral life, and magical worlds may define
different domains and edges. Same-domain grafts are ordinarily native. A
cross-domain edge determines whether the boundary connects directly, requires
an adapter process, or refuses.

The adapter is embodied: it occupies cells near the graft boundary, consumes
upkeep, or reduces channel throughput. A learned compatibility process can
shrink that footprint or penalty, analogous to buying off a dual-wielding
penalty. The remaining ruling is whether a disfavoured edge is normally a hard
gate or an expensive but recoverable graft.

### What a definition may control

A definition may declare:

- typed inputs and outputs;
- structural and environmental site requirements;
- basal and active costs using bounded integer curves;
- the relations its ports require, once those relations exist;
- plain author-facing labels and explanation text kept outside rule authority.

A definition may not declare:

- a capability verdict such as flight, reach, or immunity;
- arbitrary changes to an organism or world;
- an unbounded callback on every ecology tick;
- a new channel relation by naming it;
- renderer, camera, mesh, or sprite behavior;
- Paredros or Isometry consequences.

New verbs require a typed Mesocosm consumer. Piccolo can author data for a
recognized consumer; it cannot smuggle a new state mutation through a string.

---

## 4. Developmental expression ABI

Piccolo is an authoring engine at discrete developmental moments. It is not the
simulation engine.

The host invokes expression at bounded triggers:

- founding or filial regrowth;
- a chosen adaptation;
- assimilation or grafting;
- growth, paid remodeling, injury repair, or regeneration;
- lifecycle stage change such as metamorphosis.

It does not invoke Lua once per organism per ecology tick. Accepted output is
lowered to native process allocations and channel-development instructions,
which the core can evaluate repeatedly without the script.

**Ruled 2026-08-01.** A changing environment does not itself rewrite the
allocation mosaic. Existing processes may become active, dormant, efficient,
or starved as their inputs change. Reallocating tissue requires a discrete
developmental event with a cost and a causal record. That keeps a whole roster
and several co-op players tractable: the simulation evaluates activity, while
expression runs only when somebody actually changes.

### Request

`ExpressionRequest` is a frozen view containing only declared facts:

- the trigger and subject/body revision;
- stable part addresses, topology, geometry summaries, condition, and damage;
- existing expressed processes and channels;
- acquired process candidates with source provenance;
- the heritable developmental instructions being evaluated;
- integer material and metabolic budgets;
- relevant quantized world conditions;
- host-supplied deterministic entropy.

The request does not expose mutable Rust objects. Scripts cannot inspect hidden
world state that the host did not put in the request.

### Proposal

The smallest proposal says which admitted process should express on which
existing or proposed part, at what bounded capacity. Later proposal variants
may request a typed channel connection or anatomy-development operation only
after the corresponding native validator exists.

The proposal does not choose its own cost. It references an admitted
`ProcessDef`; the host calculates cost, validates the site and source, checks
the budget, and either lowers the whole proposal or refuses it. Partial
acceptance would make fixture and player explanations ambiguous, so v1 is
atomic.

Every refusal names the boundary that failed: unknown process, stale ruleset,
missing source, invalid part, site mismatch, insufficient material, cycle or
graph limit, channel mismatch, output limit, or exhausted fuel.

### Sandbox

Reuse Isometry's proven pattern:

- `piccolo::Lua::core()` rather than an ambient standard library;
- host policy for fuel, output bytes, nesting, and collection lengths;
- host-owned entropy with the draw trace recorded by fixtures;
- structured Rust-to-Lua input and typed Lua-to-Rust output;
- no network, filesystem, environment, wall clock, threads, or host mutation;
- a declared entrypoint, `express(request, entropy) -> proposal`;
- exact fixtures for proposal, entropy draws, validation, and refusal.

Mesocosm does not depend on `isometry-system`. Its biological request,
proposal, validator, and lowering are sovereign game code. If both games end
up with materially identical sandbox, fuel, entropy, and tagged-value code,
that small runner becomes an extraction candidate. It is not extracted first.

Piccolo is selected for this lane because Isometry already proves it in the
stack. This does not make Lua the mandated backend for every future extension.
The durable boundary is typed request, typed proposal, validation, and commit.

---

## 5. Pack and ruleset shape

The first pack is deliberately plain and inspectable:

```text
mesocosm-pack.json
processes/
  light_capture.json
expression/
  light_capture.lua
fixtures/
  light_capture_bright.json
  light_capture_dark.json
assets/
  ... optional presentation assets ...
ATTRIBUTION.md
```

The manifest declares the pack id, version, format ABI, every definition,
script, and fixture, dependencies, and SPDX license metadata. All declared
paths are canonicalized and must remain inside the pack root. Duplicate
qualified ids, dependency cycles, unknown flow kinds, and undeclared files are
rejected before Lua loads.

Admission produces a deterministic `RulesetDigest` over the manifest and the
bytes of every rule-bearing declared file in sorted path order. A world records
the exact digest, not merely a friendly pack version. Definitions admitted
into one ruleset are immutable for that world. Editing a pack creates a new
ruleset and an explicit world revision rather than changing living bodies
underfoot.

Each lowered definition also receives its own `DefinitionDigest`. That is what
an expressed process and a portable body cite. The ruleset digest answers
"which complete biology was this world running?" The definition digest answers
"which exact transformation did this organ express?"

### License gate

- Mesocosm definitions, Lua scripts, pack loaders, validators, fixtures, and
  game-specific schemas are MPL-2.0.
- A generic sandbox crate may become `MIT OR Apache-2.0` only after Isometry
  and Mesocosm prove the same extracted boundary.
- Original visual, audio, and other game assets are CC BY-SA 4.0 and require an
  attribution entry.
- Imported assets and third-party packs retain their own licenses; the manifest
  records them without implying that Mesocosm relicenses them.

---

## 6. Persistence, replay, and co-op authority

Lua makes a proposal. The validated result is the fact.

An expression commit records at least the trigger, body revision, ruleset
digest, accepted `ExpressedProcess` changes, exact costs, provenance, and
entropy used. Replaying applies that record. It does not rerun Lua and hope the
same version happens to be installed.

The world's admitted definitions are native deterministic data. A participant
can continue ordinary simulation from the lowered ruleset without the authoring
script. Re-running expression, revising the pack, or creating new developmental
proposals requires the source pack and an authority allowed to do so.

For co-op, the live simulation orders body and resource changes. Multiple
writers may sign proposals, but two proposals spending the same material or
claiming the same developmental transition do not commute. The world's
materializer accepts, orders, or refuses them and commits one exact result.
This is not a universal CRDT problem.

The first network proof is commit-result:

1. one authority runs Piccolo against a frozen request;
2. native validation lowers the proposal;
3. the accepted expression record enters the ordered world history;
4. peers apply the record without executing Lua;
5. snapshot plus history converges to the same body and state hash.

Seed replay may later be offered for vetted packs, but it is an optimization
and authoring convenience. It is not the convergence contract.

### Missing packs

A foreign body may preserve unknown `ProcessRef`s, allocation, and provenance
opaquely. Mesocosm must not silently substitute a similar local process. If the
lowered definition required to simulate that body is absent, authoritative
continuation refuses with a missing-ruleset diagnostic. Projection and
archival may still work.

---

## 7. Player legibility

Moddability earns its place only when it creates a choice someone can read.
Every expressed process therefore needs an explanation path usable by the
field journal, body inspector, AI, and failure receipts:

- what the process does in plain language;
- which part expresses it and where it came from;
- what it consumes, produces, and costs;
- which environmental conditions enable or suppress it;
- which channel or anatomical requirement is broken;
- which actions and ecological effects currently depend on it.

The first authored process proof should create a visible tradeoff, not only a
successful fixture. The leading candidate is light capture: expression on an
exposed plate can earn energy in a bright world, costs tissue and upkeep, goes
dormant in darkness, and disappears when the responsible part is severed. It
crosses body, world, ecology, and explanation with one small definition.

The exact first process is ruled at gate PD2. If light capture requires a flow
system larger than the proof, choose an existing mechanic such as venom
secretion. Do not invent a dummy capability solely to make the pack pass.

### Direct and automatic arrangement

**Ruled 2026-08-01, clarified 2026-08-03.** During adaptation, a player may
arrange a candidate founder phenotype directly or ask the game to auto-arrange
it. Unplayed lineages use the automatic path. These are two proposal sources
over one developmental validator, not player biology and simulation biology:

- both consume the same acquired candidates, capacity, material, and
  plasticity budgets;
- both must satisfy the same site, adjacency, connectivity, graft, channel,
  and cost rules;
- both lower an accepted candidate into the same kind of heritable
  developmental instruction and causal record;
- automatic arrangement may optimize a declared aim, but cannot invoke a
  privileged mutation or skip a refusal the direct editor would receive.

The editor exposes authorship, not authority. Its screen arrangement lowers to
an authoritative candidate cell graph, and the validator decides whether it
can exist under the preview's declared conditions. The lineage commits the
developmental program that produced that candidate, not the literal mosaic.
Later bodies realize their own phenotype mosaics from that program and their
actual conditions.

The [epoch-boundary plan](2026-08-01_epoch_boundary_plan.md) owns the
multi-writer result: players on one lineage may adopt one validated program
together, while disagreement preserves the proposal as a branch rather than
merging preview mosaics cell by cell.

---

## 8. Wing boundary

`ProcessRef`, expressed allocation, and source provenance may eventually ride
the optional Mesocosm phenotype facet of body v1. The definition and Lua source
travel as an engram or pack, not inline in every body.

Paredros and Isometry may preserve those facts, inspect them, project them, or
map a known process into their own rules. They do not inherit Mesocosm's
capability verdict. A photosynthetic critter arriving in Paredros remains the
same subject and body; whether sunlight affects settlement work, combat, or
social trust is Paredros' decision.

This keeps the layered reading intact: a character may wrap a named critter's
body and history, while body processes remain one composable facet rather than
becoming the character schema.

**Ruled 2026-08-01: crossing offers two phenotype routes.**

1. **Carry this body:** preserve the current allocation mosaic as faithfully as
   the destination permits. If the destination world requires gills, wings, or
   another accommodation, each change is an explicit adaptation with a cause;
   it does not silently overwrite the arriving phenotype.
2. **Regrow here:** preserve genotype, developmental program, provenance, and
   subject continuity, then realize a phenotype under destination conditions.
   This intentionally may produce a different body from the same inherited
   program.

Either route may create a new body revision while retaining the same subject.
The prior revision remains pointable. A receiving vessel still derives its own
capabilities, and a weak consumer may preserve the Mesocosm phenotype facet
opaquely. **The destination declares compatibility, available accommodations,
and their costs; the traveler chooses among the feasible routes.** An
incompatible body is refused or offered regrowth. It is never silently
rewritten by either side.

Do not revise body v1 for this lane before the wing contract's subject,
body-revision, and part-address gates are stable. Local proof comes first.

---

## 9. Proof gates and file seams

### PD0. Close P2's account: **LANDED 2026-08-01**

`d9af641` removed `Organism::mass_mg`, routed feeding, upkeep, starvation,
death, carrion, and reproduction through body mass, and made upkeep scale with
what a critter carries. Burning pays the budget; growing raises the standing
cost.

**Done:** body mass is the ecology's only mass account, larger bodies cost more
to carry, the headed host shows budget and upkeep, and the next state migration
has a clean landed base. P0's repeat-play judgment remains open because only a
player can close it.

### PD1a. Allocation design pass: **COMPLETE 2026-08-05**

Treat this as a state migration before touching the enum. Today `Process` is
derived on every call and stored nowhere. Adding allocation creates state that
must be constructed, updated with growth, tombstoned with loss, serialized,
replayed, explained, and projected.

One ownership ruling is already settled by the wing contract: functional links
and process allocation are an **optional Mesocosm phenotype facet**, not
required primitive topology in `mesocosm.body/v1`. Local Rust layout does not
get to change that portable authority boundary.

The design pass must compare at least:

1. allocation fields carried locally beside each `Part` inside
   `BodyDocument`, then projected into a separate phenotype facet;
2. a `PhenotypeState` beside `BodyDocument`, keyed by stable `PartId`;
3. a wrapper that owns anatomy and phenotype and makes mutations transactional.

"Smallest diff" is not the criterion. The choice must prevent orphaned
allocation when a part is attached or severed, keep structural topology usable
without Mesocosm semantics, and let capability evaluation receive anatomy,
phenotype, and environment explicitly.

The state owner must hold one finite allocation mosaic per participating
`PartId`. Process sites compete for its capacity; all rearrangement is an
ordered developmental event; and mosaic adjacency must survive snapshot,
replay, branch transfer, and the optional phenotype projection. The mosaic is
an authoritative cell graph; any 2D inventory layout is a projection of it.

**Done when:** one state owner is ruled with constructor, attach, sever,
snapshot, replay, explanation, and body-v1 projection paths drawn; failure
atomicity and capacity conservation are named; carry-body and regrow-here
projection paths are distinguished; fixture changes are bounded; and the
implementation gate has exact file seams and tests. The pass also names one
proposal shape and validator used by both direct and automatic arrangement,
draws the boundary between a candidate phenotype and its committed
developmental program, and leaves lineage revision and adoption to the epoch
boundary. No state migration lands in PD1a.

#### Ownership ruling: one transactional wrapper

The local owner will be a private `BodyPhenotype` wrapper containing a
`BodyDocument` and an `AllocationState`. This selects the third candidate.

- Allocation fields inside `Part` are rejected. They would make the structural
  anatomy document depend on Mesocosm process semantics and would change the
  body-only bytes consumed by mesh, Lens, and other vessels.
- A freely mutable `PhenotypeState` beside a freely mutable `BodyDocument` is
  rejected. It permits exactly the split account PD0 removed: a part can be
  attached, severed, or restored without the allocation state following it.
- The wrapper keeps both representations independently readable while making
  every mutation that affects both representations one operation. A caller may
  construct and project a plain `BodyDocument`; a live `Organism` owns a
  `BodyPhenotype` and exposes immutable `body()` and `allocations()` readings.

The wrapper fields are private. World mutation does not receive `&mut
BodyDocument` or `&mut AllocationState` separately. Body-only authoring tests
may continue to use `BodyDocument` directly, which keeps primitive topology
usable without loading the biology system.

#### Capacity ruling: structure and current availability are different

Each living part owns one stored, connected graph of capacity cells. There is
no second `capacity` scalar: structural capacity is the count of living cells,
and a process site's allocation is the disjoint set of cells it occupies.
Occupied plus free cells therefore equals structural capacity by construction.

PD1b seeds the graph deterministically from the part's integer geometry and
the admitted allocation rules at a developmental event. The initial generator
is a coarse orthogonal tissue lattice whose axes follow `half_extent`; it stores
cell ids and adjacency, not coordinates or renderer voxels. Different UI
layouts may draw the same graph. A later developmental program may choose a
different admitted topology, but changing topology is itself a paid, ordered
developmental event.

Current availability is evaluated separately from structural capacity. Mass,
condition, starvation, environment, or a missing input may make allocated
cells dormant or ineffective without rewriting the mosaic. Ordinary grazing
and upkeep therefore do not shuffle organs every tick. Irreversible injury,
shrinkage, or remodeling explicitly tombstones cells and deterministically
deactivates any site that no longer owns a valid connected subgraph. Recovery
can reactivate an intact allocation. The hard cell ceiling is core safety
policy; the lower active capacity and cell quantum belong to world rules and
are configurable.

#### Lifecycle and atomicity

| Operation | Anatomy and allocation consequence |
| --- | --- |
| Construct | Build anatomy and one mosaic for every living part in temporary state, validate all invariants, then publish the wrapper. |
| Develop from a recipe | `Recipe + Soma` first creates a real `BodyDocument`; allocation is seeded against those actual parts in the same unpublished candidate. |
| Attach or graft | Preflight the new part, source provenance, mosaic, initial sites, and all costs. A planned mirrored pair validates together before either side commits. |
| Sever | Compute the anatomy subtree once, then tombstone the same `PartId`s and their mosaics in one commit. Historical cells and sites remain explainable but cannot contribute. |
| Remodel | Apply one complete validated allocation proposal to temporary state. A refusal leaves body, allocation, budget, and causal record byte-identical. |
| Snapshot | `BodyPhenotype` derives deterministic serialization inside each `Organism`; whole-world capture needs no hand-written field list. |
| Replay | Existing growth intents seed the same mosaics; a new developmental intent carries the complete proposal and is validated again against replayed state. |
| Explain | A capability trace cites process definition, part, site, cells, satisfied or missing inputs, cost, and the current environmental reading. |

Attach and sever on a standalone `BodyDocument` remain anatomy operations.
They cannot create a live organism with orphaned phenotype because only the
wrapper is accepted by `Organism` after PD1b.

#### One proposal and validator

Direct arrangement and auto-arrange both produce one `AllocationProposal`.
It carries the expected digest of the current `BodyPhenotype` and the complete
desired sites for every part it touches, rather than an order-dependent series
of drag operations. Each proposed site names its admitted `ProcessRef`, source
and expression cause, and a sorted set of existing cell ids.

One native validator checks the expected digest, living part addresses,
definition digests, source provenance, cell existence, disjoint occupancy,
connected site subgraphs, site requirements, budget and graph limits. It
either returns one `ValidatedDevelopment` with cost and explanation or one
specific refusal. The runtime accepts the proposal, not a host-prevalidated
result, so replay and co-op do not trust UI code. Proposal source is diagnostic
metadata at most; it cannot alter validation.

The exact mosaic is a candidate or somatic realization, not the heritable
artifact. At an epoch boundary, adaptation may translate its accepted intent
and lived result into a `DevelopmentalProgramDelta`: process preferences,
target part motifs, adjacency constraints, triggers, provenance, and paid
tradeoffs. It does not copy cell ids into the lineage. Re-realizing that program
under identical declared inputs must reproduce the founder preview; changed
conditions may produce another valid phenotype. Immutable lineage revision,
co-signing, adoption, and branching remain owned by the epoch boundary.

#### Carry this body and regrow here

`BodyPhenotype` projects anatomy and phenotype separately. The body-v1 path
receives primitive topology and stable addresses once W1 supplies them. The
optional Mesocosm phenotype facet receives the allocation graph, sites,
definition digests, condition, and provenance. Neither projection carries a
capability verdict.

- **Carry this body** sends the current anatomy revision plus the exact
  phenotype facet. A destination either admits the definitions and preserves
  the allocation, offers an explicit adaptation that creates a causally linked
  revision, or refuses carry.
- **Regrow here** sends the developmental program, acquisition provenance, and
  the prior revision reference. The destination realizes a new body and
  allocation under its conditions. The old mosaic is history, not a body
  template.

The local wrapper does not settle the v1 subject or revision schema and is not
serialized wholesale as the portable body profile.

#### Developmental anatomy prerequisite

The allocation audit exposed an earlier join that must precede PD1b. V2 itself
did not harden incorporation-grown bodies because Lens projects any
`BodyDocument`. The axial menagerie did, however, still use a renderer-only
`critter::Body::from_plan`, while `Appendage::role` reached no authoritative
part.

The first half landed 2026-08-05 in `mesocosm-core::development`:
`Recipe + Soma + PartPalette` now produces one mass-conserving
`BodyDocument`. Axial segments form the dependency spine, appendages attach to
the segment that expressed them, and every supplied part template is refused
unless its geometry classifies as the promised role. The Lens menagerie now
uses `BodyLensProjection` over that document and the parallel recipe-specific
renderer constructor is gone.

The live join landed 2026-08-05. `Species::realize` is the one unpublished-body
path used by world founders, offspring, and future adaptation previews. Every
organism stores the entropy that realized it. `World` stores its admitted
`PartPalette`, because the recipe is lineage-local while the materials and
geometry in which it grows are world-local. Both are snapshotted.

Filial provisioning is binding. A birth whose paid mass cannot keep every
expressed part positive waits without spending mass, allocating an id, or
advancing ecology entropy. Genesis has no parent ledger, so a rare undersized
founder begins at the recipe's exact structural mass floor. Incorporation
remains a somatic attach during an epoch.

`Chronicle::found` now also consumes a local recipe, developmental seed, mass,
and palette. It grows local topology, maps historical origins where sites
exist, and tombstones locally addressed loss subtrees. Origins without a site
remain in the chronicle rather than forcing another game's geometry into this
one.

The migration exposed one genuine multi-part ledger defect: upkeep and
reproduction still debited only the root. `Organism::spend_mass` now folds over
all living parts in stable order. Without that correction, non-root mass was
unspendable and consumer-only populations could reproduce their way around
starvation.

#### Exact PD1b seams and receipts

- `crates/mesocosm-core/src/phenotype.rs`: private `BodyPhenotype`, lifecycle,
  digest, allocation readings, and explicit anatomy access;
- `crates/mesocosm-core/src/phenotype/allocation.rs`: cell graph, sites,
  conservation and activity;
- `crates/mesocosm-core/src/phenotype/develop.rs`: proposal, shared validator,
  atomic commit and explanations;
- `crates/mesocosm-core/src/process.rs`: qualified ids, definition digests,
  the three native definitions, registry, and explicit capability evaluation
  over anatomy, allocation and environment;
- `crates/mesocosm-core/src/organism.rs`: replace public `body` storage with a
  private `BodyPhenotype`; keep body, mass and projection readings;
- `crates/mesocosm-core/src/world/act.rs`: route incorporation and the new
  developmental intent through wrapper transactions;
- `crates/mesocosm-core/src/organism/ecology.rs`: retain the landed
  recipe-developed offspring path while routing mass and condition changes
  through the phenotype wrapper without rewriting mosaics;
- `crates/mesocosm-core/tests/embodied.rs` plus a focused allocation test:
  native parity, exact refusal explanations, attach/sever lifecycle, stale
  proposal refusal, direct/automatic parity and conservation;
- snapshot and replay fixtures: one intentional current-schema bump. No
  compatibility shim for unreleased world bytes. Standalone `BodyDocument`
  bytes and the V2 Lens/mesh/Isometry projection contract do not change.

The implementation receipts are: every living part has exactly one mosaic;
every living cell is occupied once or free; an invalid multi-part development
leaves the complete wrapper and budget unchanged; severing removes the same
subtree from capability evaluation and allocation activity; mass loss can make
a site dormant without moving it; restoration and replay agree; the same valid
proposal from direct and automatic arrangement produces byte-identical state;
and a body-only projection still decodes without Mesocosm phenotype semantics.

### PD1b. Native ProcessDef migration: **LANDED 2026-09-01**

Execute PD1a's ruling. Replace the closed `Process` enum as identity authority
with registry-backed `ProcessDef` records for the **four** existing native
processes. Existing geometry deterministically seeds their initial allocation,
and `Reach` keeps exactly its P2 semantics.

**Correction, recorded 2026-09-01.** This gate was written when there were
three natives. DC1.5 added a fourth — `Process::Fix`, expressed by
`Role::Plate`, the fixing/producer reading — together with `Process::ALL` and
the registry parity receipt. Every "three" in this gate means four; the
implementation covers all four and `the_registry_and_the_native_view_agree`
is what makes a fifth impossible to add silently.

Expected seams, subject to PD1a:

- `crates/mesocosm-core/src/process.rs`: ids, definitions, registry, allocation;
- the ruled phenotype owner: allocation lifecycle and part addressing;
- `crates/mesocosm-core/tests/embodied.rs`: parity, severing, explanations;
- snapshot and replay fixtures: one intentional format bump, with no legacy
  compatibility shim for unreleased data.

**Done when:** the four built-ins are ordinary definitions, every living
allocation names a living part, attach and sever cannot split anatomy from
phenotype, every part mosaic conserves capacity, rearrangement occurs only
through recorded events, existing reach outcomes and refusals hold, and a part
still cannot acquire a capability by editing a number. The same valid
candidate submitted through direct and automatic proposal sources lowers to
the same developmental instruction, and the same invalid candidate receives
the same refusal. Re-realization under changed declared conditions may produce
a different valid phenotype without changing process identity or provenance.

**All met.** See the 2026-09-01 Progress entry for the landed shape, the
receipts, and the residues PD2 inherits.

### PD2. One native played process

Hand-author one additional `ProcessDef` in Rust and play it before building the
pack or Lua machinery. Light capture leads; venom secretion is the bounded
fallback if light capture would require the whole channel evaluator.

This is the mechanic proof. It may use a native developmental fixture or an
explicit editor operation to allocate the process. That temporary authoring
path is deleted when the pack/Piccolo path replaces it; it does not become a
second permanent system.

**Done when:** acquiring or expressing the process creates a readable choice;
allocation locates it on a part and charges its cost; world conditions can make
it useful or dormant; severing its dependency removes the consequence; and a
headed receipt explains all four states.

Only this gate chooses the first process. It does not open a catalog pass.

### PD3. Static pack admission

Encode the already-played PD2 definition in a data-only pack, validate it,
lower it into the core ruleset, and remove the native authoring duplicate. Lua
is not involved yet.

Likely seam: a new MPL-2.0 `mesocosm-phenotype` crate for pack discovery,
admission, and later Piccolo hosting. It may depend on `mesocosm-core`; core may
not depend on it.

**Done when:** the packed definition lowers to the same `ProcessDef` and game
outcome as the native proof; namespaced ids do not collide; path escape and
malformed schema are refused; changing one rule-bearing byte changes the
ruleset digest; and a snapshot identifies the exact admitted ruleset.

### PD4. Piccolo authoring parity

Add the typed request/proposal bridge, native validator, bounded runner, and one
declared fixture pair for the process already proven at PD2 and packed at PD3.

**Done when:** Piccolo can propose the same accepted allocation the native
fixture produced; the same context and entropy produce the same proposal and
draw trace; contrasting developmental contexts can produce different
phenotypes from the same body plan; unknown ids, invalid parts, excessive
output, and exhausted fuel refuse cleanly; Lua has no direct world mutation
path; and the temporary native authoring path is gone.

### PD5. Filial expression

Connect the authored process to P4's adaptation bridge. Metabolized source
material widens the candidate bank, a chosen developmental change references
that lived source, and the descendant regrows a phenotype that may express it.

**Done when:** somatic incorporation, dormant acquisition, and filial expression
are distinct records; unplayed lineages use the same expression path; the
source provenance survives; and the old trait array has either met every
deletion condition or remains explicitly provisional.

### PD6. Channels under pressure

Add only the flow ports and relation needed by the first process path that
cannot be represented by local anatomy. Start with local or routed. Redundant,
centralized, and quorum arrive when a body proves them. Broadcast remains cut.

**Done when:** a working path survives through valid typed connections, an
incompatible graft needs a visible adapter or refuses, severing a connector
breaks the path, and the explanation names the missing flow.

### PD7. Persistence and authority

Commit a validated expression result, restore it without Lua, and apply it on
two simulation participants.

**Done when:** snapshot plus ordered record converges; a peer does not rerun
Lua; stale rulesets and missing lowered definitions refuse explicitly; and two
concurrent proposals spending the same budget resolve through the named world
materializer rather than last-writer-wins.

### PD8. Extraction audit

Compare Mesocosm's runner with Isometry's actual generator runtime after PD4
through PD7 are proven.

**Done when:** either a small sandbox/entropy/tagged-value crate has two real
consumers and moves under `MIT OR Apache-2.0`, or the audit records why the two
hosts are still meaningfully different. No extraction is also a valid receipt.

---

## 10. Scheduling

This lane interleaves with the phenotype plan rather than replacing it:

1. PD0 is complete;
2. PD1a is complete and the core recipe-to-body authority proof is landed;
3. live founding, offspring, local regrowth, and the founder-preview seam now
   consume the shared developer;
4. migrate native process identity and allocation in PD1b;
5. play one native process at PD2 before building authoring infrastructure;
6. execute P3 branch transfer with stable process identity and allocation;
7. encode the proven mechanic as a static pack at PD3;
8. prove Piccolo authoring parity at PD4 before P4 adaptation;
9. use PD5 as P4's filial phenotype replacement proof;
10. add PD6 only when the played process needs a non-local path;
11. let P5 contested flow consume the proven process and ecology vocabularies;
12. run PD7 and the wing projection gate before PD8 extraction.

P0's playfeel judgment remains a user test throughout. A technically correct
process system does not answer whether burn or grow is worth choosing again.

---

## 11. Stop rules

- Do not use `ProcessDef` as a universal gene or trait record.
- Do not let Lua run the ecology loop or mutate world state directly.
- Do not store capability verdicts on parts or organisms.
- Do not accept a process without a native consumer for its flows.
- Do not give direct arrangement or auto-arrange separate validation rules.
  They are proposal sources over one developmental authority.
- Do not build pack or Lua infrastructure before PD2 proves a process worth
  authoring.
- Do not add a broad process catalog after PD2; one played process authorizes
  an authoring path, not a biology encyclopedia.
- Do not let scripts choose their own cost or bypass acquisition provenance.
- Do not add a channel relation because an authored string names it.
- Do not silently substitute a definition when a ruleset is missing.
- Do not make pack version text stand in for a content digest.
- Do not make peers rerun Lua to converge.
- Do not turn ordered resource conflicts into a universal CRDT.
- Do not extract the Piccolo host before Isometry and Mesocosm are demonstrably
  the same consumer at that boundary.
- Do not project Mesocosm capability authority into another vessel.
- Do not retire `epoch::Trait` until the phenotype plan's five deletion
  conditions pass.
- Do not evaluate discovery conditions by polling every organism every tick;
  route relevant accepted evidence to bounded condition evaluators.
- Do not admit an instantaneous dependency cycle. Biological feedback crosses
  an explicit state boundary and a later tick.
- Do not leave stacking to iteration order. Every repeated evaluator declares
  and tests its combination law.

---

## 12. Open rulings

These are intentionally deferred to the gate with evidence:

1. The first non-native played process at PD2: light capture is recommended;
   venom secretion is the bounded fallback.
2. The exact portable shape of a lowered `ProcessDef`: settle after local
   snapshot and branch-transfer proofs, before body v1.
3. Whether a world embeds lowered rule definitions or content-addresses them
   beside the snapshot: PD3 and PD7 must prove missing-pack behavior before
   choosing.
4. Whether a second scripting backend is ever useful: no abstraction or ruling
   until another real consumer asks.
5. Whether the first played process needs surface, interior, or junction cell
   kinds. PD1a ruled structural graph capacity and separate current
   availability; PD2 must supply a real consumer before cells gain kinds.
6. Stabilized hybrid identity: a newly admitted derived `ProcessDef` versus a
   heritable compound recipe that retains its parents at evaluation time.
7. Disfavoured graft semantics: hard refusal by default versus an adapter tax
   that a learned compatibility process can reduce.

---

## 13. Findings

- **2026-09-01, generated-trait execution:** the existing plan already has the
  correct representation split: `ProcessDef` for transformations, discrete
  expression triggers for development, and acquisition evidence outside both.
  A universal trait AST would erase that ownership. Kappa supports typed local
  rules, reachability, influence, and causal analysis as validation precedent;
  its stochastic scheduler is not compatible with Mesocosm's authoritative
  fixed-step transaction path.

- **2026-08-01, Mesocosm P2.** `crates/mesocosm-core/src/process.rs` has a
  closed three-variant native `Process` enum. Geometry classifies each process;
  `Reach` is the only capability, and severing removes it. This is a sound
  parity fixture for PD1, not yet an extensible definition system.
- **2026-08-01, phenotype boundary.** The phenotype plan already distinguishes
  process identity from channel flow, cuts broadcast pending evidence, and
  forbids a shared body/ecology evaluator before two sovereign proofs. This
  plan preserves those rulings.
- **2026-08-01, Isometry donor.** `isometry-system` uses Piccolo 0.3 with
  `Lua::core()`, host-owned entropy, finite fuel, bounded tagged output,
  path-contained declared pack assets, exact fixtures, and commit-result
  generation. The game-specific request and result types remain inside
  Isometry, which is the correct reuse boundary for Mesocosm too.
- **2026-08-01, P2 closeout.** `d9af641` removed the scalar mass account,
  split organism ecology into its own module, routed substance through body
  parts, and made upkeep scale with biomass. PD0 is complete.
- **2026-08-01, implementation review.** Registry, pack loading, and Piccolo
  would otherwise put three infrastructure gates before the first new process
  was played. PD2 now proves one hand-authored native mechanic first; PD3 and
  PD4 then replace its authoring path with pack data and Piccolo.
- **2026-08-01, maintainer design answers.** Processes share finite per-part
  capacity; an authoritative cell graph makes local and whole-mosaic evaluation
  available; somatic compounds may stabilize into heritable forms at an epoch;
  allocation changes only through discrete developmental events; and the
  destination declares feasible crossing routes while the traveler chooses.
- **2026-08-01, graft-affinity proposal.** A default world's directed
  animal-like, fungal-like, and plant-like affinity cycle can balance grafting
  without becoming the trophic `Kingdom` enum. World data owns the graph; a
  disfavoured boundary may demand a capacity-consuming adapter whose penalty
  can be reduced by an evolved compatibility process.
- **2026-08-01, arrangement authorship.** Direct editing and auto-arrange are
  two sources of the same validated developmental proposal. Shared-lineage
  agreement adopts one child revision; disagreement branches at the epoch
  boundary rather than merging allocation cells.
- **2026-08-03, co-signing target.** The concrete mosaic is a founder preview.
  Direct and automatic arrangement author a developmental program, and that
  program is what shared-lineage players adopt or branch from.

---

## 14. Progress

- **2026-09-01, PD1b complete: allocation gets its owner.**

  **The four-process correction, first.** This gate was written against "the
  three existing native processes". DC1.5 added a fourth — `Process::Fix`,
  expressed by `Role::Plate`, the fixing/producer reading — together with
  `Process::ALL` and a registry parity receipt. §9 is corrected rather than
  quietly contradicted, the implementation covers four, and
  `the_registry_and_the_native_view_agree` is what makes a fifth impossible to
  add in silence.

  **What allocation is.** `crates/mesocosm-core/src/phenotype.rs` holds
  `BodyPhenotype { body, mosaics, revision }`, every field private.
  `phenotype/mosaic.rs` holds one `Mosaic` per part — lattice `dims`, a sorted
  `lost` list, and its `Site`s — and the mosaics are **index-aligned with
  `BodyDocument::parts`**. That alignment is the whole invariant: an anatomy
  tombstone *is* an allocation tombstone, and the two cannot disagree because
  there is nothing to synchronise. A `Site` names a `ProcessRef`, the sorted
  connected set of `CellId`s it occupies, and an `Expressed` cause —
  `Geometry` for what a shape grew, `Arranged { revision }` for what a
  development placed. There is no capacity scalar: structural capacity is the
  count of living cells, and occupied plus free equals it by construction.

  **Identity moved off the enum.** A site cites
  `ProcessRef { definition: DefinitionDigest }` — the content address of the
  exact admitted definition — and resolves it through `Registry::resolve`,
  which answers `None` rather than substituting a similar local process.
  `Process` survives as the native binding for engine fast paths and
  `Role::processes` survives as the **seeding rule**; neither is what a
  phenotype stores. Only the digest travels, because `ProcessId`'s
  `&'static str` cannot be deserialized into an owned world; PD3 widens the
  record when packs mint owned ids.

  **The seed.** A part's lattice is `half_extent / 2 + 1` cells per axis,
  clamped to four — so `MAX_CELLS` is 64, derived from the axis bound rather
  than typed twice. A `[4,1,1]` limb is a chain of three, a `[4,4,1]` plate a
  3x3 sheet, a `[1,1,1]` sensor one cell. Adjacency is derived from the dims
  inside `Mosaic::neighbours`, which is the one seam a later admitted topology
  arrives through: every reader asks the mosaic rather than doing lattice
  arithmetic of its own. **A seeded site takes the whole part.** That is the
  honest lowering of "this shape does this thing" — nothing is pre-donated, so
  the first development that wants a second process has to take tissue off the
  first, which is the tradeoff PD1a deliberately put inside the organ. Any
  other share would have been a number invented to make it painless.

  **One validator, and the equivalence is structural.**
  `phenotype/develop.rs` holds one `AllocationProposal` — expected digest,
  the sorted parts it rewrites, and the complete desired sites for each — and
  one `validate`. `Arrangement::{Direct, Automatic}` rides along and the
  validator never reads it, so the parity receipt is not a coincidence to be
  re-checked but a property of the shape: `develop` returns
  `Development { instruction, source }`, and `Instruction` (revision, parts,
  sites, `cost_cells`, resulting digest) contains no author. `arrange(&p, Aim)`
  builds the automatic side and asks `Mosaic::seed` for the seeding rule
  rather than reimplementing it, so auto-arrange cannot drift from what
  development would have grown. Fifteen named refusals, checked in a fixed
  order that is part of the contract.

  **Where the anti-Spore property now lives.** `ProcessDef::admits(role)` gates
  every proposed site by the part's classified shape, so `SiteMismatch` refuses
  contraction on a plate. To make a frond an actuator you must make it a limb,
  which is a different shape and a different part. There is still no capability
  field anywhere.

  **Cost is counted, not invented.** `Instruction::cost_cells` is the number of
  cells whose expression changed. PD2 prices that in milligrams when a played
  process gives the price a consumer; this slice refused to guess one, and no
  ecology number moved as a result.

  **Ownership, and one deliberate refinement of the seam.** `Organism.body:
  BodyDocument` became `Organism.phenotype: BodyPhenotype` with an
  `Organism::body()` reading. §9's seam said "private `BodyPhenotype`"; the
  field is `pub` and the **wrapper's** fields are private, which is the
  invariant the ruling actually protects — no caller can obtain
  `&mut BodyDocument` from a live organism, and `BodyPhenotype::seed` is the
  only constructor. Keeping the field public is what lets a test or a host
  sever through the transactional API without an `Organism` passthrough for
  every wrapper verb. `world/act.rs`'s incorporation now attaches through the
  wrapper, and its no-room rollback restores the **whole phenotype** rather
  than the anatomy — a body put back with grown mosaics would have been
  exactly the split account this wrapper exists to prevent.

  **The format bump, and what it cost.** One intentional break, no shim:
  `Organism` now serialises a phenotype where it serialised a body. Measured on
  the two worlds that matter — `World::new(4242, 24)` grows 5.3% (21 KB of
  399 KB, 871 living parts, 5,416 cells) and the demo's 916-founder world grows
  **28.0%** (773 KB of 2.76 MB, 31,762 living parts, 198,193 cells). Roughly
  ten of the thirteen bytes per site are the 64-bit definition digest as a
  postcard varint. §3 already permits interning a process reference to a
  compact integer inside one simulation; doing it needs a world-owned admitted
  ruleset, which is PD3's, so the digest stays whole here and the lever is
  recorded rather than pulled. Standalone `BodyDocument` bytes and the V2
  Lens/mesh/Isometry projection contract are untouched.

  **Receipts.**
  - `mesocosm-core` lib: **337** green. `process/tests.rs` (10) gains
    `a_reference_resolves_to_the_definition_it_addresses`: every native's
    stored reference resolves back to exactly its own definition, and an
    address this ruleset does not hold answers `None` rather than the nearest
    local process. `phenotype/tests.rs` (21):
    geometry seeds what it used to only answer; a seeded part arrives fully
    committed; every living allocation names a living part; capacity conserves;
    the lattice follows the shape; attach seeds in the same operation; sever
    removes the allocation and its consequence together and still explains the
    branch; a stale proposal moves nothing; **direct and automatic lower the
    same candidate to the same instruction and byte-identical state**; the same
    invalid candidate earns the same refusal; a part cannot acquire a
    capability by editing a number; every refusal names its boundary; an
    invalid multi-part development leaves the wrapper byte-identical; an
    unknown definition refuses rather than substitutes; a severed part cannot
    be rearranged; rearrangement is ordered and on the record; the phenotype
    round-trips; the mosaic and the geometry reading agree; irreversible loss
    takes capacity and the site with it; an explanation names the definition.
  - `tests/embodied.rs` **16** green, split at the ceiling into
    `tests/embodied/allocation.rs`: every founder carries one mosaic per living
    part; the allocation and the anatomy reading agree across a whole roster
    and all four processes; incorporating a part seeds its allocation in the
    same meal; one validator serves the player and the game over a live body;
    re-realizing the same program under a different world's admitted materials
    grows a phenotype with a genuinely different amount of tissue that
    expresses the same definitions for the same reason, while identical inputs
    still reproduce the phenotype exactly.
  - `cargo test -p mesocosm-core --test matter --test flows --test succession
    --release`: **5 + 9 + 7** green. Conservation and PE0's reconciliation are
    unmoved.
  - **The instrument is unmoved.** Drawn baseline, all ten seeds re-run and
    compared against the DC4/PE1 receipt **seed for seed** — verdict, start,
    peak, peak tick, end, cumulative births and cumulative deaths each
    identical. **0 breathes / 10 thins / 0 boil / 0 collapse**, exactly as
    recorded. The run was stopped after the baseline rather than left to
    overwrite `dc4_roster.json`, which is byte-identical to what DC4 wrote.
    Wall times are not compared: the sweep shared the machine with the debug
    test suite.
  - **Fixtures re-recorded.** The demo trace's **intent stream is
    byte-identical** — the same three births continued, the same
    `TakeControl { organism: 2205 }` at step 3000, the same fourth birth at
    3040 — and only the hash moved, `f90123db6f2a5ac5` to
    **`0ebe0655317a7392`**, which is the format bump and nothing else. Headed
    `--replay` runs 3,100 steps over 775 frames and matches, exit 0; a hash
    falsified by one bit exits 1.
  - `cargo test --workspace` green — **639** tests, and the lens crate's 45
    passed in the ordinary parallel run rather than needing
    `--test-threads=1`. Clippy `-D warnings` clean, `cargo fmt --all --check`
    clean, `cargo check -p paredros-room --features r1-proof` builds (its one
    `dead_code` warning on `brick::retarget_from_ground` predates this work;
    the paredros tree is untouched and clean).

  **Splits at the ceiling**, per the workspace rule: `process/tests.rs` out of
  `process.rs`, `organism/ledger.rs` out of `organism.rs` (mass ceiling,
  intake room, gain and spend, complexity, upkeep and the rent payment), and
  `tests/embodied/allocation.rs` out of `tests/embodied.rs`. An integration
  test's crate root resolves `mod` against `tests/`, so the last one carries an
  explicit `#[path]`; a bare `tests/allocation.rs` would have become a second
  test binary.

  **Residues, and what PD2 and PE2 inherit.**
  - **`performs` could be more honest, and was deliberately left alone.**
    `BodyDocument::performs`, `reach`, `canopy`, `mouth` and `feeding_mode`
    still read geometry. The truer reading is "some living site on a living
    part expresses this definition", which is `BodyPhenotype::expresses` —
    already implemented, already receipted as agreeing with `performs` over a
    whole roster and every native process. Rewriting the anatomy readings onto
    it is PD2's, because it is only a *different* answer once something makes
    a site dormant or a development moves one, and doing it here would have
    made a representation change into an ecology change. One reading did get
    strictly more honest and is already in: **an injury can now be explained.**
    `BodyPhenotype::explain` answers for a severed part — "this branch fixed,
    on this much tissue, because its shape did, and it is gone" — which no
    process reading could say before, since severing erased the fact from the
    process view entirely and left only `Part::severed`. `canopy` is the next
    candidate: its first clause asks `performs(Process::Fix)`, and the truer
    question is whether the plate's tissue is actually allocated to fixing,
    which only becomes a different question when a development can take that
    tissue away.
  - **Dormancy is unbuilt.** PD1a's structural-capacity/current-availability
    split is honoured by construction — mass changes move no organ — but
    nothing yet evaluates availability. `Mosaic::tombstone` carries the
    irreversible half of the rule and has no live caller; sub-part injury is
    phenotype D3a's gate.
  - **No world intent, by choice.** `BodyPhenotype::develop` is the only
    mutation path and returns its record; the phenotype stores the ordering
    (`revision`) and each site stores the revision that placed it, so
    rearrangement is ordered and readable in the snapshot. An
    `Intent::Rearrange` and a `History` event belong with PD2's editor
    operation, which is the first thing that will actually rearrange anything.
  - **The registry is still `Registry::native()`, reached internally.** Seeding
    and validation call it rather than receiving it. When PD3 gives a world its
    own admitted ruleset it becomes a parameter, and `World` starts recording
    the ruleset digest a snapshot cites.
  - **`World` exposes no phenotype reading.** `World::body()` still answers a
    `BodyDocument`; PE2's inspector will want `World::phenotype()` and
    `BodyPhenotype::explain` behind it. Not added ahead of its consumer.
  - **Multi-process roles are unexercised.** Every role expresses exactly one
    process today, so `Mosaic::seed`'s even-share splitter has never had to
    divide a part. The validator is the authority on connectivity and will
    catch a bad split; the splitter itself wants a receipt the first time a
    role expresses two.

- **2026-09-01:** recorded the condition/development/process compilation split,
  hard graph and work bounds, delayed-cycle rule, explicit stacking laws, and
  the Kappa research boundary. Existing PD and PE ordering is unchanged;
  documentation only.

- **2026-08-31:** reconciled the status with the landed PD1b identity slice.
  The playable ecology plan consumes PD1b allocation and PD2 in PE2, then
  preserves P3 and PD3/PD4 before PE3 invokes P4/PD5 for filial expression.
  This changes no ProcessDef gate and dispatches no code.

- **2026-08-01:** `ProcessDef` accepted as the working name. Architecture,
  Piccolo boundary, pack and license gate, authority model, proof gates, stop
  rules, and execution interleave recorded. No implementation added.
- **2026-08-01:** revised after implementing-agent review. PD0 marked landed;
  PD1 split into design and migration; native play moved before pack and Lua;
  phenotype-facet portability ruled separately from local Rust storage; and
  phenotypic plasticity made explicit.
- **2026-08-01:** first PD1a question pass recorded finite per-part mosaics,
  event-driven remodeling, and the carry-body/regrow-here crossing choice.
- **2026-08-01:** second question pass ruled authoritative cell-graph mosaics,
  somatic compounds stabilized through adaptation, and destination-declared
  crossing options chosen by the traveler. Directed graft affinity recorded
  for the next pass.
- **2026-08-01:** third question pass ruled direct and automatic arrangement
  through one validator, with shared-lineage disagreement resolved by immutable
  descent rather than mutation in place or cell-wise merge.
- **2026-08-03:** clarified that arrangement previews a phenotype while the
  accepted, co-signed artifact is its developmental program. Changed world and
  body conditions may realize that program differently.
- **2026-08-05:** PD1a complete. Ruled a private transactional
  `BodyPhenotype` wrapper, one authoritative cell graph with no duplicate
  capacity scalar, separate structural capacity and current availability, one
  complete allocation proposal and validator for direct and automatic
  arrangement, and distinct carry-body and regrow-here projections. Exact
  PD1b seams and receipts recorded.
- **2026-08-05:** the prerequisite recipe-to-anatomy authority proof landed in
  `mesocosm-core::development`. `Recipe + Soma + PartPalette` now produces a
  real mass-conserving `BodyDocument`, and the Lens menagerie projects that
  document through V2 instead of constructing a renderer-only recipe body.
- **2026-08-05:** the live constructor join landed. World founders and ecology
  offspring call `Species::realize`; the world snapshots its palette and every
  organism its developmental seed; under-provisioned births wait atomically;
  `Chronicle::found` regrows through the same recipe developer; and the
  migration corrected body-mass spending across every living part. PD1b is now
  the next gate.
- **2026-08-06:** host/palette drift caught and closed. `PartPalette::primitive`
  grew a `sensor` template (tag 4) that `mesocosm-genet`'s fixture volume table
  did not carry, so `mesh_body` failed with `MissingVolume { part: PartId(3) }`
  as soon as a developed body used one. The fixture now **enumerates
  `Role`** and sizes each volume from `palette.template(role).half_extent`
  rather than listing tags literally, so a palette that grows a template can no
  longer silently outrun the host that draws it. Standing note for PD1b and the
  pack loader: any admitted-vocabulary change must be reachable by enumeration
  from the palette, never by a hand-maintained mirror on the presentation side.
- **2026-08-07 (audit):** two contract corrections. **Replay direction:**
  the live runtime is right and parts of this plan's persistence prose were
  wrong; replay is seed plus ordered intents with history *derived*, never
  history as replay input. Preserve that direction everywhere. **Provenance
  is three separable kinds:** acquired developmental vocabulary, somatic
  graft provenance, and filial material/lineage provenance. The founding
  slogan "every part used to be somebody" conflicts with `Origin::Founding`
  and recipe regrowth unless these are kept distinct.
- **2026-08-08, PD1b slice 1 landed (identity layer).** `ProcessId`
  (namespaced, static until PD3 admits owned strings), `ProcessDef`
  records for the three natives with per-definition digests over their
  rule-bearing bytes, a deterministic `Registry` with a ruleset digest,
  and `Process::id()` resolving the native binding through it. `Role`'s
  fast path is now receipted against the registry (`the_registry_and_
  the_native_view_agree`), so expression is defined by data and the enum
  may not drift; `a_rule_bearing_byte_changes_the_digest` proves the PD3
  digest property early. Reach semantics untouched, all existing
  receipts green. **Open for PD1b complete:** the `BodyPhenotype`
  wrapper, `phenotype/allocation.rs` and `develop.rs`, geometry-seeded
  allocation, `organism.body` privatization, and the wrapper-routed
  intents — the survey confirmed none of the PD1a seam files exist yet,
  so that half is a full surgery scheduled as its own session.
