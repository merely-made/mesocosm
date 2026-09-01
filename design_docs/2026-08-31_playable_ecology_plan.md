# Playable Ecology Architecture Plan (2026-08-31)

**Status: plan, founded 2026-08-31 and refined 2026-09-01. Nothing in this
document claims new code.**
Mark ruled reproduction as an individual-scale micro-checkpoint and named
trophic visibility as the primary design challenge. He reaffirmed the other
load-bearing directions: body composition determines what a critter can do;
unlock conditions extend beyond a linear eating tree; world criteria shape
generated biology; and animal-, plant-, fungal-, and microbial-scale critters
belong in the playable roster.

This plan owns the **integration contract and full-game proof**: how the
existing body, ecology, history, lineage, generation, scale, and presentation
lanes compose into Mesocosm rather than remaining separate demonstrations. It
does not replace their detailed plans:

- [the played-slice plan](2026-08-28_played_slice_plan.md) owns live host
  wiring;
- [the ProcessDef plan](2026-08-01_processdef_plan.md) owns transformations,
  expression, packs, and validation;
- [the phenotype plan](2026-07-31_phenotype_plan.md) owns developmental
  programs, body realization, and capability;
- [the epoch-boundary plan](2026-08-01_epoch_boundary_plan.md) owns lineage
  review, significance, speciation, and authorship;
- [the traits and perception brief](2026-08-29_traits_and_perception_brief.md)
  owns the unresolved acquisition-cost and perception choices;
- [the forms-of-life brief](2026-08-29_forms_of_life_brief.md) owns research
  on composable forms until individual stages are ruled;
- [the elements and traits memo](2026-08-29_elements_and_traits_memo.md) owns
  the three candidate schemes for generated materials;
- [the scale plan](2026-08-29_scale_plan.md) owns cohort execution, residency,
  zoom, and stress receipts;
- [the dependency ledger](2026-08-07_dependency_ledger.md) remains the
  authority on global order and cross-plan blocking.

If this plan and an owning plan disagree about a domain detail, the owning plan
wins. If their execution order changes, the dependency ledger and this plan are
updated together.

---

## 1. The product rulings and their technical consequences

### Reproduction and the epoch are different checkpoints

**Ruled direction, 2026-08-31.** Reproduction is the checkpoint at the scale
of one critter: descent becomes concrete, a parent pays for an offspring, and
the player may continue through a descendant. The epoch boundary is the
checkpoint at the scale of a lineage and its ecology: the world is reviewed,
evidence is weighed, and a developmental program is revised for future bodies.

The two may occur near each other, but one must not silently invoke the other.
Reproduction uses the ordinary breeding transaction and a recorded control
choice. The epoch uses an explicit, deterministic world rule whose exact
trigger remains open. The reproduction checkpoint does not grow a second
lineage editor, and the lineage editor does not manufacture a special body
outside ordinary development.

Here, "checkpoint" first means a bounded decision and a pointable descent
record. Whether it also writes a durable save automatically is a separate
persistence decision.

### Unlocks are evidence, not a diet tree

Eating may supply material, a donor, and an observation. It does not map
directly from food category to reward category. A discovery condition may cite:

- a particular donor part or process;
- survival through a quantified stress;
- repeated use, failure, injury, or recovery;
- environmental exposure or a world cycle;
- a relationship, dependency, or exchange;
- a lineage or epoch achievement;
- a combination of the above under one body and one world law set.

The durable record therefore needs the evidence and route, not only the thing
unlocked. The current `Event::Learned` and `World::learn_from` are migration
inputs, not the final acquisition model.

### Generated effects use authored verbs

The generator may compose surprising biology, but it does so inside bounded,
audited mechanical languages. It can generate materials, parameters,
conditions, placements, costs, developmental bundles, and combinations. A new
simulation verb still requires an authored evaluator and a real consumer.

`ProcessDef` remains a transformation rather than a universal trait object. A
player-facing generated trait may combine process allocation, anatomy,
material properties, regulation, and lifecycle constraints, with each piece
lowered through its owning validator. This plan does not introduce a universal
`TraitDef` or a callback that runs arbitrary generated code on every tick.

### The body and world remain the authority

Capabilities, trophic roles, danger, and ecological strategies are readings of
realized bodies acting under current world conditions and costs. A generated
label cannot confer flight, photosynthesis, immunity, predation, or symbiosis.
Severing the responsible anatomy, starving its inputs, breaking its route, or
moving it into an incompatible world changes the reading without editing an
ability score.

### Different forms get different controllers over one world

Direct animal movement, plant growth allocation, fungal routing, and microbial
colony bias need not share an interaction design. They do share the same
matter, energy, spatial authority, process evaluators, history, and intent
commit path. Controllers propose bounded intentions; the world resolves them.

One representation decision is deliberately open. A connected body at one
anchor can cover many plant and animal bodies. A true mycelium, clonal stand,
or biofilm may require one played subject to own several spatial bodies or a
multi-anchor body graph. That choice changes identity, occupancy, damage,
resource routing, cohort storage, and projection, so it must be ruled before a
distributed form is implemented.

---

## 2. One authority, several derived records

The target data flow is:

```text
realized world rules + seed + ordered intentions
                         |
                deterministic World mutation
                 /             |             \
          canonical state   causal events   flow events
                 |             |             |
              bodies        History      bounded readings
                 |             |             |
               AI senses   epoch review   warnings / overlays
```

`World` stays the only live simulation authority. `Runtime` stays the
fixed-step driver and owner of replay-derived history. Rendering, UI, physics
advisors, caches, and forecasts remain projections.

### Sparse causal events and dense flow events have different jobs

The existing `history::Event` is biographical: birth, feeding, growth, death,
incorporation, learning, carving, inhabitation, and speciation form causal
lines. Trophic visibility also needs exact, frequent resource movement. Putting
every unit of upkeep, soil draw, and return into the permanent causal log would
make the wrong record carry the wrong frequency.

The working split, with names illustrative rather than ruled, is:

```text
RecordedEvent { tick, optional place, event }

FlowEvent {
    tick, place, process,
    source account, destination account,
    carrier, amount,
    organism and lineage subjects
}
```

One accepted world transaction emits the applicable records at the same commit
point. A resource mutation cannot be visible to the state while absent from
the flow record. Refused transactions emit neither resource movement nor a
false ecological consequence.

The world buffers at most one tick of both record types. Runtime drains them,
records sparse events in `History`, and reduces flows into bounded integer
windows. Replay regenerates the same event sequence and the same windows, so
the raw flow stream does not enter the world snapshot merely to serve UI.

If a future AI rule or unlock condition acts on a trend, the facts it acts on
must become deterministic state or be recomputed through the same bounded
reducer. A view-only warning never becomes hidden simulation authority.

### Realized world rules are saved facts

A durable world records its admitted process definitions, material and field
definitions, environmental schedules, generator version, and digests. A seed
alone is insufficient because generator code changes. Ordinary replay remains
seed plus ordered intentions against those realized rules; it does not rerun a
new generator and hope to recover the old world.

`WorldRules` is a working label for this immutable record, not a settled Rust
type name. The existing `process::Registry` and its digest are its first proven
component.

### Simulation detail changes; authority does not

Micro, meso, and macro are useful viewing and execution scales, not three
competing simulations. Near play may realize bodies, local interaction, and
tactile projections explicitly. Far play may execute equivalent anonymous
subjects as cohorts over coarser spatial fields. The global trophic web and
epoch nutrient graph are derived readings of those same stocks and accepted
flows. A macro summary does not own an extra population counter or settle a
resource transfer.

Use the existing terms **materialize** and **aggregate** for individual/cohort
transitions. “Hydration” already has an ecological meaning in this game and
would make moisture and simulation detail unnecessarily ambiguous. Both
representations remain canonical world state; rendering a body is a further
projection from the materialized record.

---

## 3. The ecology readings contract

Trophic visibility begins in the simulation record and ends in presentation.
It is not a health bar computed from private tuning constants.

### Facts retained in bounded windows

At minimum, by place and lineage where the data supports it:

- living, stored, soil, carrion, and detrital matter;
- captured energy, upkeep, growth, feeding, and return flows;
- births, maturation, reproductive reserve, deaths, and migration;
- matter transferred through a dependency or organism relationship;
- which processes and trophic sources supplied each flow.

Windows are fixed-size and deterministic. Several resolutions may coexist,
such as recent ticks, the current life stage, and the current epoch, but every
retention length is explicit and tested. Cohort materialization and aggregation
must preserve the sufficient totals these readings need. Near/far tier
promotion may trigger that transition, but it is not the transition itself.

### Interpretable leading indicators

The first indicators are ratios and trends whose evidence can be shown:

- **support ratio:** prey or producer growth against consumer withdrawal;
- **replacement ratio:** maturation against mortality;
- **resource runway:** available stock against net depletion;
- **return ratio:** decomposition and soil return against detritus production;
- **recruitment failure:** births that do not reach reproductive mass;
- **dependency concentration:** reliance on one lineage, carrier, or return
  path.

A warning says what moved and over what window, for example: "grazer demand
has exceeded producer regrowth for 240 ticks." It does not present an
unexplained collapse percentage.

### Four presentation distances

1. **World cues:** thinning stands, missing juveniles, accumulating remains,
   altered coloration, motion, or behaviour.
2. **Immediate reading:** a compact trend or refusal near the played critter.
3. **Inspect view:** stocks, flows, dependency edges, place, and uncertainty.
4. **Epoch review:** the causal chain from choices through intermediate
   lineages to collapse, adaptation, or recovery.

The authoritative reading may be omniscient internally. What the player sees
is filtered by the played body's senses, learned evidence, attention budget,
and the review phase's ruled access. Exact postmortem knowledge does not leak
into live play by accident.

The existing population instrument's `breathes`, `thins`, `boils`, and
`collapses` verdicts remain test classifications. They do not become player
language without a separate interaction ruling.

### Forecasts come after measured warnings

The first implementation uses current stocks, observed flows, and
interpretable leading indicators. A later forecast may fork deterministic
snapshots over a bounded horizon and label its assumptions. It is a projection
and never a cheaper simulation authority.

Generic early-warning research makes the same sequencing argument. Critical
slowing, variance, and autocorrelation can precede some transitions, but their
false-alarm and missed-alarm rates can remain severe even under favourable
conditions. Mesocosm therefore starts with mechanistic stocks and flows, keeps
an induced-stress arm and a neutral control, and labels uncertainty before it
experiments with generic tipping-point indicators. See
[Scheffer et al. 2009](https://doi.org/10.1038/nature08227) and
[Boettiger and Hastings 2012](https://doi.org/10.1098/rsif.2012.0125).

---

## 4. Integration order and done-conditions

The dependency shape is:

```text
PE0 flow record and first reading -> PE1 reproduction and succession

PD1b allocation -> PD2 one embodied process -> PE2 embodied discovery

PE0 + PE1 + PE2
  -> P3 branch transfer -> PD3/PD4 authoring parity
  -> PE3 lineage review + P4/PD5 filial expression
  -> PE4 world-generated biology
  -> PE5 another form of life and durable relationships
  -> PE6 cohort scale and ecological zoom
  -> PE7 collapse-and-recovery proof
```

The dependency ledger decides actual dispatch. Each phase below must deliver a
visible game change as well as a technical receipt.

### PE0: one flow record, one useful warning

Add the smallest general flow event and bounded reducer that can reconcile the
current matter cycle. Cover current soil draw and return, feeding, body growth,
upkeep, birth, death, and carrion return. Extend sparse events with tick and
place through an envelope rather than copying those fields into every variant.

The first view should expose one useful, evidenced trend. Replacement ratio is
the smallest current candidate because births, maturation, and deaths already
exist as events; support ratio follows once production and withdrawal are flow
facts.

**Done when:** the sum of recorded transfers reconciles with compartment
changes in a controlled run; accepted and refused transactions cannot disagree
with the stream; replay produces byte-identical readings; draining readings
does not change the world hash; a headed capture shows the first trend; and a
neutral control does not raise the warning that an induced stress arm raises.

### PE1: reproduction and succession as the individual checkpoint

Use the existing adult-mass gate, filial realization, matter debit, parent
link, and `Event::Born`. Add the host experience around a birth involving the
controlled critter, together with the already-planned death, witnessing, and
`Intent::TakeControl` path. The exact choice among continuing the parent,
taking the offspring, or applying a world default is an open interaction
ruling. Whichever is chosen enters the trace.

The lineage program does not change here. A descendant realizes the current
program under its own seed and conditions. Siblings remain in the ecology.

**Done when:** reproduction opens a bounded individual checkpoint; parent,
offspring, cost, and descent are pointable; one recorded choice resumes play;
death can continue through an eligible descendant; siblings persist without
becoming menu inventory; birth flows reconcile to the milligram; replay lands
the same body, control holder, history, readings, and state hash.

### PE2: discovery becomes an embodied option

Complete PD1b's allocation half and PD2's one native, visible process. Replace
the current all-appendages meal lesson with a condition evaluator and an
evidence-bearing discovery record. Distinguish observation, somatic
incorporation, developmental availability, expression, and inheritance.

Part-level eating starts with one bounded proof: consuming a severed or corpse
part settles that part's exact matter and donor evidence and cannot teach
unrelated parts from the donor's recipe. It does not attach a functioning
source branch; live subtree transfer remains phenotype P3. Full live
dismemberment is not a prerequisite for this proof.

**Done when:** one condition unrelated to food unlocks a candidate; one meal
supplies evidence without unlocking an incompatible candidate; one consumed
part settles only its own matter and provenance; the PD2 process is located
on anatomy, paid for, useful under one condition, dormant under another, and
lost when its dependency is severed; direct and automatic fixtures use the
same validator even if NPC acquisition itself remains an open ruling.

### PE3: the lineage checkpoint turns discovery into a descendant

Follow the owning phenotype and ProcessDef order through P3 branch transfer,
PD3 static pack admission, PD4 authoring parity, P4's adaptation bridge, and
PD5 filial expression. Give `Runtime::end_epoch` a production caller and open
the route-B review over the same world. Replace the provisional scalar
adaptation result with a validated developmental-program revision. The player
reviews ecology readings and discovery evidence, spends a finite lineage
budget, previews a founder, and commits a program for future descendants. At
least one unplayed lineage takes a turn through the same proposal and validator
path.

The epoch trigger is a versioned world rule and remains to be chosen. PE3 adds
that minimal realized rule before PE4 generalizes world-law generation. It is
not implicitly every reproduction.

**Done when:** a player finishes an epoch, can explain why each offered change
is available and what it costs, commits one body-program revision, watches one
rival lineage respond, and returns through ordinary development to the live
terrarium in a descendant that can express the admitted option; somatic
incorporation, discovery, lineage commitment, and filial expression remain
distinct records; the old scalar trait array is either removed under its
existing deletion gate or marked explicitly as non-authoritative; replay and
the world record agree.

### PE4: world criteria generate mechanically distinct biology

Choose one material scheme from the elements memo only after PE2 proves the
consumer. Generate an immutable world-law record first, then admitted material
and process parameters, viable founding programs, and candidate weights. Run
reachability and headless ecology checks before exposing the world.

Every generated candidate states its causes, inputs, outputs, costs, counters,
observable cues, and inheritance path. Mechanical fingerprint and rank tests
reject vocabularies whose extra nouns do not reach independent formulas.

**Done when:** one ordinary world and one impossible world use the same
evaluators but demand visibly different strategies; each has a reachable
energy-capture path, matter-return path, reproductive route, and counterplay;
the generated definitions and digests survive save and replay; changing a
rule-bearing byte changes the digest; material ids resolve through the saved
world-local definition table; every admitted scalar field names its consumer,
dimension, cadence, and conservation rule; and a deliberately mechanically
duplicate vocabulary is rejected by the anti-affix receipt.

### PE5: prove a second form before expanding the roster

Choose one non-animal control model with a real ecological consequence. A
plant growth-allocation proof may fit the current connected body. A fungal or
microbial proof first rules whether its played subject is connected,
multi-anchor, or a colony projection. Add a durable organism relationship only
when the chosen form consumes it; parasite, mutualist, and commensal remain
readings of net flow rather than stored classes.

**Done when:** two forms play differently through different controllers while
using the same world transactions; generated processes remain usable by their
NPC policies; any relationship routes exact matter and survives replay; its
net-flow reading can change as conditions change; and neither controller owns
a private movement, matter, or history engine.

### PE6: make the roster and readings survive scale

Resume the scale ladder against the product load. S3's spatial index,
distance-capped far tier, and cohort execution are the load-bearing simulation
work; S2's windowed atlas remains the presentation path beyond the current
resident wall. Which runs first remains a scale-plan ruling. S4 zoom adds
silhouettes, region tint, and flow presentation over the same place and cohort
facts.

**Done when:** near individuals and far cohorts conserve matter and the
sufficient ecology-reading totals across materialization and aggregation;
thousands of critters hold the configured tick budget; generated processes and
relationships have explicit far-tier reductions or keep their subjects near
for a measured reason; zoom remains out of the trace; and the far view reveals
territories and flows rather than a clipped roster. A materialized subject does
not reroll when it crosses the boundary twice; an aggregate/materialize round
trip with no intervening tick restores the same cohort bytes; and named, played,
relationship-bearing, injured, or chronicled subjects retain pointable
identity. Each far-tier evaluator passes its declared all-individual comparison
envelope, and repeated boundary crossing produces no biological consequence.

### PE7: the collapse-and-recovery proof

Compose the preceding phases in one seeded scenario. The control arm remains
viable. A stress arm deliberately overdraws one trophic support path. The game
surfaces a concrete warning before the chosen recovery point is lost, the
player changes strategy, and a recovery arm restores the flow.

**Done when:** identical rules, seed, and intentions reproduce state, history,
flows, warnings, and lineage revisions; severing or adding anatomy changes a
real capability and cost; a non-feeding discovery and an incompatible meal are
both visible; reproduction and the epoch boundary each perform their distinct
checkpoint job; an ordinary and an impossible world produce different viable
strategies; induced collapse is warned, causally explained, and recoverable in
the control; and the configured population, tick, memory, and render budgets
have measured receipts.

---

## 5. Stop rules

- Do not add a warning counter beside the simulation when a transition can
  emit the fact it needs.
- Do not put dense per-tick flow in permanent causal history or world snapshots
  merely for presentation.
- Do not let a view, forecast, controller, script, renderer, or physics advisor
  mutate authoritative ecology.
- Do not treat every generated trait component as `ProcessDef`; lower each
  component through its sovereign evaluator.
- Do not run unbounded authored or generated callbacks on ecology ticks.
- Do not save only a seed when realized generated definitions affect the
  world's meaning.
- Do not grant a capability, trophic role, or relationship by label when it can
  be read from anatomy, transactions, and current conditions.
- Do not make reproduction a duplicate epoch editor or make the epoch boundary
  a special reproduction route.
- Do not hide omniscient ecological truth behind a body that has not perceived
  or learned it.
- Do not compress a critter into a cohort unless matter, lineage, lifecycle,
  process, relationship, and reading totals have named reductions.

---

## 6. Open rulings

1. At the reproduction checkpoint, does the player continue the parent, take
   the offspring, or choose between them? Is there a world-configurable
   default?
2. What deterministic condition ends an epoch? Timer, world condition,
   lineage event, configurable rule, or a composition of them?
3. Are fungal networks, clonal stands, and microbial colonies genuine
   multi-anchor subjects or connected local critters at the first proof?
4. Which generated-material scheme from the elements memo is the first one
   built?
5. Do NPC lineages acquire new developmental vocabulary through the same
   evidence rules, or only evaluate inherited candidates?
6. How much ecology truth is available during live play, and how much becomes
   available only during epoch review or postmortem?
7. What exact recoverability condition makes a trophic collapse terminal in
   the game, as distinct from the population instrument's test verdict?
8. When the scale lane resumes, does S3's correctness and cohort work precede
   S2's wider resident window?

None blocks founding PE0. Questions 1 and 2 block the full PE1/PE3 interaction;
3 blocks a distributed PE5 form; 4 blocks PE4; 5 blocks world-wide acquisition;
6 blocks final warning presentation; 7 blocks the terminal run condition; and
8 blocks scale dispatch.

---

## 7. Downstream architecture gates

These are ownership assignments and integration checks, not extra phases. The
detailed requirements and research live in the plan that owns each mechanism.

| Concern | Owning record | First integration gate | Required before admission |
| --- | --- | --- | --- |
| Individual/cohort execution | [Scale](2026-08-29_scale_plan.md) and [place graph](2026-08-05_place_graph_engine_plan.md) | PE6 | Exact zero-tick aggregate/materialize round trip; persistent pointable subjects; named reductions; per-evaluator all-individual comparison envelope; unsupported-process fallback. |
| Generated material vocabulary | [Elements and traits](2026-08-29_elements_and_traits_memo.md#storage-shape-shared-by-all-three) | PE4 | Saved world-local definitions and compact ids; exact mass reconciliation; measured local-palette versus wider-cell decision only when the one-byte baseline binds. |
| Sub-part body mutation | [Phenotype D3a](2026-07-31_phenotype_plan.md#d3a-when-do-voxel-cells-become-body-state) | First played case in PE2 or PE3 that cannot use whole-part loss | New immutable volume or explicit body patch; atomic body revision; bounded revision-safe mesh/collider work; truthful fallback. |
| Generated trait execution | [ProcessDef](2026-08-01_processdef_plan.md#one-displayed-trait-three-compiled-programs) and [acquisition](2026-08-29_traits_and_perception_brief.md) | PE2 then PE3 | Event-driven condition, discrete development program, and native repeated process remain separate; each is typed and bounded; the exact realized candidate and digest persist. |
| Environmental fields | [Resident views](2026-08-14_resident_views_composition_plan.md#field-admission-boundary-2026-09-01) and [elements](2026-08-29_elements_and_traits_memo.md#field-dimensionality-is-part-of-admission) | First PE4 world rule that needs a new field | Named consumer, honest domain, cadence, sources/sinks, boundaries, units/range, conservation, scale rule, cost, and control. |

Three cross-lane rules stay here. Representation changes are world
transactions and cannot create events or resources. A derived mesh, collider,
field view, warning, or trophic graph never becomes authority through
convenience. New machinery enters through the first played consumer and its
receipt; a platform-shaped possibility does not reorder PE0-PE7.

---

## 8. Findings

- **2026-08-31, history seam:** `crates/mesocosm-core/src/history.rs` records
  causal subjects but carries no general tick, place, process, or resource-delta
  envelope. It is suited to sparse biography and insufficient by itself for
  trophic flow.
- **2026-08-31, runtime seam:** `crates/mesocosm-runtime/src/runtime.rs` already
  drains one tick of events beside the ordered trace and rebuilds history during
  replay. It is the existing owner for a replay-derived bounded flow reducer.
- **2026-08-31, reproduction seam:** `organism::can_reproduce` already reads
  adult mass and gestation; `ecology::breeding` debits parent body and reserve,
  realizes a filial body through the lineage recipe, records the parent, and
  conserves matter. The missing checkpoint is host, control, and presentation
  composition rather than a second breeding system.
- **2026-08-31, process seam:** native `ProcessDef` identity, definition
  digests, and a registry have landed. PD1b's `BodyPhenotype` allocation half
  and PD2's first additional played process remain open.
- **2026-08-31, acquisition seam:** `World::learn_from` still runs after both
  burn and incorporation, teaches every non-innate appendage in the donor
  lineage recipe rather than the consumed part, and returns early for an
  unplayed eater.
- **2026-08-31, reading seam:** `score::readings` computes retrospective epoch
  feats from `World` and `History`; vitals and minimap expose immediate
  individual and place facts. There is no bounded stock-and-flow reading or
  early trophic warning in the live views.
- **2026-08-31, scale seam:** the measured scale plan finds a fixed nine-place
  graph, an unbounded far-tier target scan, unwired cohort execution, and a
  population ceiling before the target roster. Flow and relationship designs
  must state their cohort reductions rather than assume individual storage.
- **2026-09-01, downstream ownership audit:** each proposed subsystem already
  has an owning plan and an incumbent seam. Section 7 routes their admission
  gates without adding an engine layer or changing the product order.

---

## 9. Progress

- **2026-09-01:** refined the downstream architecture from technical prior art,
  routed multi-scale execution, materials, body mutation, generated-trait
  execution, and fields into their owning plans, and summarized their admission
  checks in §7. PE0-PE7 order is unchanged; documentation only.

- **2026-08-31:** founded from Mark's first-principles Mesocosm review. The
  architecture, integration order, trophic-readings contract, done-conditions,
  stop rules, live-code findings, and eight remaining rulings were recorded.
  Documentation only; no implementation dispatched.
