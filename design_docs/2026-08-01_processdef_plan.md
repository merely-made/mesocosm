# ProcessDef: authored biology without ability flags

**Status: plan, 2026-08-01. No ProcessDef schema, pack loader, or Mesocosm
Piccolo host exists yet. P2's native `Contract`, `Intake`, `Sense`, and derived
`Reach` are the starting proof. P2's biomass and upkeep reconciliation landed
in `d9af641`; the next work is PD1's allocation design pass, not a registry
refactor.**

This plan owns Mesocosm's extensible process vocabulary, developmental
expression boundary, content-pack shape, and Piccolo proof. The
[phenotype plan](2026-07-31_phenotype_plan.md) continues to own body and
capability semantics. The
[execution waves plan](2026-07-31_execution_waves_plan.md) owns scheduling.
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
    pub process: ProcessRef,
    pub part: PartId,
    pub capacity: u32,
    pub cause: ExpressionCause,
    pub source: Option<Provenance>,
}

pub struct PartAllocation {
    pub part: PartId,
    pub capacity: u32,
    pub sites: Vec<ExpressedProcess>,
    pub topology: AllocationTopology, // ruled by PD1a
}
```

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

### PD1a. Allocation design pass

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

### PD1b. Native ProcessDef migration

Execute PD1a's ruling. Replace the closed `Process` enum as identity authority
with registry-backed `ProcessDef` records for the three existing native
processes. Existing geometry deterministically seeds their initial allocation,
and `Reach` keeps exactly its P2 semantics.

Expected seams, subject to PD1a:

- `crates/mesocosm-core/src/process.rs`: ids, definitions, registry, allocation;
- the ruled phenotype owner: allocation lifecycle and part addressing;
- `crates/mesocosm-core/tests/embodied.rs`: parity, severing, explanations;
- snapshot and replay fixtures: one intentional format bump, with no legacy
  compatibility shim for unreleased data.

**Done when:** the three built-ins are ordinary definitions, every living
allocation names a living part, attach and sever cannot split anatomy from
phenotype, every part mosaic conserves capacity, rearrangement occurs only
through recorded events, existing reach outcomes and refusals hold, and a part
still cannot acquire a capability by editing a number. The same valid
candidate submitted through direct and automatic proposal sources lowers to
the same developmental instruction, and the same invalid candidate receives
the same refusal. Re-realization under changed declared conditions may produce
a different valid phenotype without changing process identity or provenance.

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
2. rule allocation ownership and mutation in PD1a, then migrate in PD1b;
3. play one native process at PD2 before building authoring infrastructure;
4. execute P3 branch transfer with stable process identity and allocation;
5. encode the proven mechanic as a static pack at PD3;
6. prove Piccolo authoring parity at PD4 before P4 adaptation;
7. use PD5 as P4's filial phenotype replacement proof;
8. add PD6 only when the played process needs a non-local path;
9. let P5 contested flow consume the proven process and ecology vocabularies;
10. run PD7 and the wing projection gate before PD8 extraction.

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

---

## 12. Open rulings

These are intentionally deferred to the gate with evidence:

1. The first non-native played process at PD2: light capture is recommended;
   venom secretion is the bounded fallback.
2. The exact portable shape of a lowered `ProcessDef`: settle after local
   snapshot and branch-transfer proofs, before body v1.
3. Local allocation layout: PD1a compares storage within `BodyDocument`, a
   sibling `PhenotypeState`, and a transactional wrapper. Portable ownership is
   already settled: allocation projects as an optional Mesocosm phenotype
   facet, regardless of the local Rust layout.
4. Whether a world embeds lowered rule definitions or content-addresses them
   beside the snapshot: PD3 and PD7 must prove missing-pack behavior before
   choosing.
5. Whether a second scripting backend is ever useful: no abstraction or ruling
   until another real consumer asks.
6. Capacity source: how living mass, geometry, growth, and damage add or remove
   graph cells, and whether cells need surface, interior, or junction kinds.
7. Stabilized hybrid identity: a newly admitted derived `ProcessDef` versus a
   heritable compound recipe that retains its parents at evaluation time.
8. Disfavoured graft semantics: hard refusal by default versus an adapter tax
   that a learned compatibility process can reduce.

---

## 13. Findings

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
