# Phenotype: what a body is for

**Status: decisions and proof plan, revised 2026-08-03; downstream body-volume
boundary refreshed 2026-09-01. Anatomy descent, depth, severing, derived reach,
and the axial developmental recipe are built; the phenotype bridge described
here is not.** This document owns Mesocosm's body rules. The
cross-vessel boundary lives in the
[wing phenotype contract](2026-07-31_wing_phenotype_contract_plan.md), and
ordering remains with the
[execution waves plan](2026-07-31_execution_waves_plan.md).
The [ProcessDef plan](2026-08-01_processdef_plan.md) owns the extensible
process vocabulary, developmental expression ABI, Piccolo host, and pack
proofs. This document continues to own what those processes mean to a body.

---

## 1. The hole

Mesocosm has three competent models of a critter that do not yet explain one
another:

1. `BodyDocument` is a tree of voxel parts with mass, geometry, attachment,
   loss, and provenance.
2. `Organism` is a scalar ecology record with kingdom, mass, stage, signal,
   venom, and position.
3. `epoch::Lineage` is an abstract trait array scored against world pressures.

The played critter owns the only `BodyDocument`. Every other organism is one
`VolumeRef` and one `half_extent`. `Role` affects placement but not action.
`Trait::Jaws` answers predation without requiring a mouth. Consumers graze
because `Kingdom::Consumer` permits it, regardless of anatomy. The new
`BodyDocument::reach()` is derived correctly, but `World::within_reach()` still
uses the constant `REACH = 8`.

The governing requirement is therefore:

> **The played critter and the ecology must use one organism model, and the
> adaptation phase must change the body that model grows.**

Anything less leaves anatomy as presentation and adaptation as a second game.

---

## 2. Settled boundaries

These are inputs, not questions for this plan.

| Ruling | Consequence here |
| ------ | ---------------- |
| Bodies are part trees | Every living part but the root depends on a parent. |
| Loss cascades | Severing a part tombstones its whole living subtree. |
| Capability is derived | A stored capability score may be a cache or receipt, never authority. |
| The fold is vessel-owned | Mesocosm's biological reading does not become Paredros' or Isometry's rules. |
| Parts come from incorporation | A part retains source provenance through growth, loss, and export. |
| Placement is mostly automatic | The developmental rules choose ordinary growth; full placement remains an editor path. |
| The mind is outside the body | Paredros skills and personhood survive injury and chassis change. |
| Games couple by data | Portable schemas use primitive mirrors, not cross-repo Rust types. |

Here **fold** means any deterministic evaluation over the surviving anatomy.
It is not restricted to adding numbers. A fold may carry process sets, trace
paths, reject disconnected organs, and derive several consequences at once.

---

## 3. Working model

### Anatomy, development, phenotype

- **Anatomy** is the identity and dependency tree of one current body: stable
  part ids, parent links, loss state, and provenance.
- **Developmental rules** are heritable instructions for what may grow, which
  processes it tends to carry, and where it tends to attach. `BodyPlan` is the
  existing placement subset, not yet the whole program.
- **Phenotype** is the body actually grown in one world: anatomy plus geometry,
  material, process allocation, condition, and damage.
- **Capability** is what this vessel derives from that phenotype in its current
  environment.

The distinctions matter. A developmental rule may fail to express when the
world lacks its material. The same anatomy may read differently in water and
air. A remembered individual may retain its anatomy while a descendant grows a
new phenotype from related developmental rules.

The part tree is specifically a **structural dependency tree**. It answers who
owns a part and what is lost when one attachment fails. It is not required to
pretend that every biological connection is a tree. Circulation, signal paths,
mycelial exchange, sibling contact, and symbiotic transfer may form a separate
typed functional graph, including cycles. Tree folds collect the surviving
structure; bounded path queries evaluate the functional connections available
through it.

### Somatic and filial change

Three operations must remain distinguishable:

- incorporation changes the current body's phenotype;
- metabolized sources widen the candidate pool for later adaptation;
- adaptation changes developmental rules used to grow a descendant.

An eaten limb is therefore not automatically a heritable limb. Its geometry,
processes, and provenance may influence what becomes heritable, but the epoch
choice decides what the next body tends to grow. This is the bridge between the
embodied run and the adaptation round.

### What adaptation commits

**Ruled by Mark, 2026-08-03.** The adaptation editor may directly arrange a
candidate body, but the lineage commits to a **developmental program**, not the
candidate's literal phenotype or allocation mosaic. The editor renders one
founder preview: what that program is expected to grow from the current world,
materials, budgets, and life-stage conditions.

The preview is a prediction and an explanation receipt. A descendant grown
under the same declared inputs should reproduce it; another descendant may
realize differently under different materials, medium, injury, plasticity, or
world conditions. That variance is expression of one inherited program rather
than an implicit mutation.

Direct arrangement and auto-arrange therefore author the same kind of program
through the same validator. For a shared lineage, co-players adopt that program
together. They do not promise that every descendant will carry an identical
body. The [epoch-boundary plan](2026-08-01_epoch_boundary_plan.md) owns what
happens when one player does not adopt the proposal.

### Processes and paths

A part contributes a small set of physical or biological processes. Geometry
scales them, the anatomy connects them, and the environment supplies or denies
their inputs. Examples are illustrative, not a settled enum:

- light or chemical capture produces usable energy;
- exchange tissue admits a medium;
- digestion transforms intake;
- transport moves matter or signals through connected parts;
- contraction produces force;
- support transmits load;
- sensing receives a signal;
- secretion produces material or venom;
- reproductive tissue turns stored surplus into descendants.

A capability is a satisfied path through those processes. A biting mouth may
need intake, contraction, and digestion. A moving limb may need contraction,
support, contact with the current medium, and enough energy throughput. A
severed connector can therefore disable an intact distal part.

This is still a fold over surviving anatomy. It is simply a richer fold than a
scalar sum.


### Channels and coherence

**Recorded 2026-07-31** from a design discussion Mark relayed. It refines
D2 and it is the reason the stop rule against forcing cyclic biology into the
structural tree exists.

The proposal is that a body has layers: a structural tree (what is attached to
what, and what falls away together), a channel graph (where energy, matter,
force, and signals can travel, possibly cyclic), a control pattern (which parts
initiate or coordinate), and a capability reading (which complete paths work
here). Channel requirements come in kinds: **local**, **routed**,
**broadcast**, **redundant**, **quorum**, **centralized**.

The payoff is real, and it is that radically different organisms fall out of
one mechanism rather than needing one system each. A vertebrate concentrates
coordination and is efficient, responsive, and catastrophically vulnerable. An
octopus gives each arm local control under a central arbiter. A mycelium has
redundant cyclic channels, so damage partitions it instead of killing it. A
plant coordinates without anything brain-like. A colony acts by quorum.

**The brain is only the most important part in a plan that centralizes control
there. That fragility is a strategy, not a law**, which is exactly the shape
this game wants: a choice with a cost rather than a fact about biology.

Three corrections before any of it is built.

**The control pattern is not its own authored layer.** Each archetype above is
a *distribution of channel requirements across parts*, not a separate system: a
vertebrate is mostly `centralized`, an octopus is `local` limbs under some
`centralized` behaviour, a mycelium is `redundant` throughout, a plant is
`routed` with nothing `centralized`, a colony is `quorum`. Collapsing four
layers to three means the archetypes *emerge* instead of being enumerated.

**But keep it as a derived reading.** "Centralized", "distributed", and
"quorum-controlled" are exactly what a field journal or an inspector should say
about a creature, and they are cheap to compute from the requirements currently
being satisfied. The rule is that **requirements attach to actions and
processes, never to the organism**: a creature may be centralized for
locomotion and quorum-governed for reproduction at the same moment, and a label
stored on the organism would erase that.

**Every requirement kind needs to name the relation it is evaluated over, and
one of them does not have one yet.** `local` needs no graph. `routed`,
`redundant`, and `centralized` are path queries over the channel graph.
`quorum` is a count over a connected set. But **broadcast** is described as
diffusing through *neighbouring tissue*, and neighbouring is ambiguous: two
plates both attached to the root are spatially in contact and are not adjacent
in either the structural tree or, necessarily, the channel graph.

So broadcast either means bounded `routed`, a path within a hop limit, or it
needs a **third relation, spatial contact**, computable from `world_pivot` and
`half_extent` but quadratic and genuinely distinct from the other two.

**Cut it from the initial vocabulary rather than picking one.** Defining it as
bounded routing would still be authoring a semantic before anything needs it,
and the choice between the two readings is exactly the kind of thing a real
phenotype should settle. Mycelial resilience does not depend on it: that is
`redundant` over a cyclic channel graph, not diffusion. When some creature
genuinely needs propagation, it will show which relation it needs, and that
evidence is worth more than the guess.

**Grafting is where catalog pressure will arrive.** The graft routes are
excellent fiction: pay for awkward adapters, assimilate the process into
host-grown anatomy, let it stay partly autonomous, cultivate a symbiont that
translates, or accidentally install a parasite that hijacks the network. But
"its cut boundary may not speak your body's language" needs a *type* on the
channel endpoint, and a rich endpoint type is a compatibility matrix, which is
a catalog wearing a lab coat.

**The endpoint type is a flow, not a process.** An earlier draft here said to
reuse the process vocabulary, and that was wrong: **processes transform things,
channels carry them**, and typing an endpoint by its process conflates
"digestion" with what digestion receives and emits. The endpoint vocabulary is
therefore the smaller one — energy, matter, signal, force, medium. Matching
flows connect directly, and a mismatch is repaired by an explicit **adapter
process** that transforms one flow into another, which is a part doing work
rather than a table granting an exception. That keeps compatibility as one
comparison and makes the cost of an awkward graft a visible organ.

### The same question at widening boundaries

The discussion's strongest move, and it belongs in this plan because it
*generates* D5's destination list rather than restating it.

A part survives by participating in a body. A body survives by keeping its
internal flows coherent. A species survives by occupying and modifying a niche.
An ecosystem survives through overlapping cycles and tensions. Centralization
is efficient and vulnerable at every one of those scales; redundancy is
resilient and expensive at every one.

**Channels can cross the skin.** Gut symbionts digest for you, fungi connect
roots, a particular tree completes your reproductive cycle. So instead of
incorporating a useful organism, you may cultivate it as an external organ.
That is the extended phenotype, made playable, and it turns the recurring
decision into:

> Do I internalize this capability, form a relationship that supplies it, or
> reshape the environment so it arises reliably?

Each route has its own cost. Internalizing is portable and controllable, and
adds upkeep and fragility. Symbiosis is efficient specialization, and creates
dependency. Niche construction supports whole lineages, and competitors exploit
it while disturbances destroy it.

That is why D5's destinations are the destinations. Burn, incorporate,
provision, deposit or build, and cultivate are not an arbitrary list; they are
the answers to *where does this capability live: in me, in a relationship, or
in the world?*

**One risk to guard.** The observation that a body and an ecology share a small
flow vocabulary (sources, transformations, storage, channels, sinks, signals,
constraints) is elegant and is the single most likely place for this project to
over-generalize into a physics of everything. The discussion already says the
two remain separate authorities using a common grammar. Make that a rule rather
than an intention:

> **Do not share an evaluator before two sovereign rule systems have
> independently proven the same mechanism.** Two authorities, one set of words,
> and no common solver until each has earned its own.

Stated as a gate rather than a prohibition, because the absolute form was too
strong to keep. If a body evaluation and an ecology tick later grow two genuine
implementations of the same bounded flow primitive, extraction should be
available; that is the wing's standing rule everywhere else, where a shared
thing is extracted after two real consumers rather than declared in advance.
What must not happen is the reverse order: one solver written first and two
domains bent to fit it.

---

## 4. Decisions

### D1. Who owns a body?

**Question.** Is the player a special body beside scalar organisms, or is a
body part of the organism model?

**Recommendation: every organism owns or references a body, and control names
an `OrganismId`.** `World` should stop storing a separate player body, position,
and energy as the long-term model. Storage may intern shared immutable volumes
or developmental data, but rules must see the same organism shape for played
and unplayed critters.

This is a rule contract, not a demand to keep every distant organism fully
realized. Nearby, played, injured, named, or otherwise consequential subjects
carry individual body revisions. Distant abundance may be aggregated into
cohorts with conserved mass, energy, developmental distributions, and causal
seeds. Materializing an individual consumes cohort state deterministically;
aggregating it returns state without erasing a named or played subject's
chronicle. Simulation resolution must not become a second set of biology rules.

This unblocks four things at once:

- predators and prey can damage or lose actual parts;
- incorporation can take a source subtree rather than minting a generic blob;
- NPC actions can be granted and constrained by phenotype;
- switching the played lineage changes control, not representation.

**Cost.** This is a state-model migration touching snapshots, fixtures,
render-scene assembly, ecology stepping, and intent targeting. It must preserve
the integer-only deterministic boundary.

**Rules: Mark.** This is the largest architectural ruling in this plan.

### D2. What produces capability?

**Question.** How does an anatomy become an action?

**Options.**

1. Flat constants keyed by kingdom or species.
2. Additive part scores.
3. A deterministic process network evaluated by collecting the surviving tree
   and running bounded path queries over its typed functional links.

**Recommendation: option 3.** Additive scores remain useful outputs for UI and
AI, but cannot be authority. They cannot express an intact jaw disconnected
from its actuator, a photosynthetic plate hidden behind another part, or a
respiratory surface in the wrong medium.

The first implementation should prove a tiny vocabulary, not design biology in
advance. Reach is the first scalar fold already built. One connected process
path and one path broken by severing are enough for the next proof.

**Cost.** Process identity becomes core game data. The vocabulary needs version
discipline and authored examples, and the evaluator needs useful explanations
for why a path did or did not work.

**Rules: Mark.** The evaluator shape is an implementation decision after the
process vocabulary is accepted.

`ProcessDef` is now accepted as the working name for one namespaced
transformation. The schema and authoring path are specified in the
[ProcessDef plan](2026-08-01_processdef_plan.md). It is explicitly not the
universal gene type: anatomy, material, regulation, lifecycle, signalling, and
relationships remain distinct developmental consequences.

### D3. Where does the fold compute?

`mesocosm-core` owns rules, but it deliberately cannot read voxel contents. A
part currently exposes mass, half-extents, pivot, orientation, attachment, and
provenance. Exact exposed voxel surface and visibility live downstream.

**Recommendation: compute authoritative capability in core using those exact
core facts and bounded geometry summaries.** Begin with box-derived areas,
lengths, depth, adjacency, and coarse coverage. If real voxel analysis becomes
necessary, a resolver may write a deterministic summary into the part when the
volume is admitted. `mesocosm-mesh` may calculate that summary, but it must not
own the rule or be queried during simulation.

The summary is validated input, comparable to collision hints. It is not a
projection result feeding arbitrary floats back into the core.

**Cost.** Coarse exposure is deliberately less accurate than rendering. Any
cached summary becomes versioned phenotype data and must be reproducible from
its source volume.

**Rules: implementation boundary.** Reopen only with a phenotype the box model
cannot distinguish.

### D3a. When do voxel cells become body state?

Today a body is already composed from voxel parts without putting cell bytes in
the core. `BodyDocument` owns the dependency tree, mass, extent, attachment,
loss, and provenance; each part cites an immutable content address through
`VolumeRef`. `mesocosm-mesh::Volume` resolves that address to a one-byte
material grid, greedily meshes each distinct volume once, and reuses it for
every placement. Attachment adds a placement and severing tombstones a subtree,
so neither operation requires a mutable micro-grid per organism.

**Rule: whole-part damage remains the incumbent.** Cell-level mutation enters
authoritative phenotype state only when a played bite, injury, repair, or
development case cannot be expressed honestly as part loss, mass change, or a
new part. That proof chooses between:

1. a newly admitted immutable volume with a new content address; or
2. a deterministic per-body patch over an immutable base volume.

The world transaction updates the patch/reference, part mass and integrity,
validated geometry summaries, and body revision atomically. It cannot mutate a
shared content-addressed volume behind every body that cites it.

Meshes, capsule poses, colliders, and GPU buffers remain revision-tagged
projections. Async work names the exact body and volume revision it consumed;
the host discards a stale result. The queue is bounded and deduplicated by
subject, projection kind, and revision; configurable priority favors the
played and visible bodies, while backpressure keeps a truthful fallback rather
than stalling the ecology tick. Collision or tactile proposals carry the same
revision and the world refuses one computed from an older body. While detailed
work is pending, the host uses a truthful lower-detail projection and a
conservative current-revision collider, such as the existing capsule path or
direct voxels. The Sapling’s
[lazy organism-model strategy](https://thesaplinggame.com/devlogs/optimization.html)
shows the value of rate-limited projection work, but substituting an ancestor’s
appearance would conflict with Mesocosm’s rule that visible anatomy explains
capability.

Greedy meshing is the measured incumbent. Instanced voxels, direct raymarching,
Surface Nets, and Dual Contouring are candidates only after a body binds their
tradeoff. [Dual Contouring](https://doi.org/10.1145/566570.566586) starts from
signed-grid Hermite samples and emits a surface, which may suit smooth tissue
and may also erase the categorical block character of a voxel body. A fixed
`16^3` edit tile is a benchmark candidate rather than a phenotype law.

The structural body graph also remains the right animation source. Spore’s
[morphology-independent retargeting](https://www.chrishecker.com/Real-time_Motion_Retargeting_to_Highly_Varied_User-Created_Morphologies)
demonstrates that authored motion can preserve structural relationships while
being applied to previously unseen skeletons. Mesocosm can derive pose goals
from its part graph without making the mesh or a fixed skeleton authoritative.
A related developmental precedent is Karl Sims’
[directed graph genotype](https://doi.org/10.1145/192161.192167), which grows
body and controller together; Mesocosm retains stronger matter, replay,
descent, and legibility constraints rather than its scalar fitness target.
A pose used only for animation remains a projection. If joint motion changes
occupied space, reach, damage, or collision, its quantized frame state is part
of `BodyPhenotype` and advances through the deterministic world transaction;
physics may propose it but cannot keep the consequential pose privately.

### D4. What happens to the trait array?

`BodyPlan` currently describes symmetry, role-facing preferences, and
tolerance. It does not describe photosynthesis, contraction, digestion,
respiration, or process allocation. It is therefore not yet a replacement for
`Lineage::traits`.

**Recommendation: keep the trait array as provisional adaptation scaffolding
until a phenotype-derived scorer replaces every responsibility it currently
has.** Extend the developmental representation beside `BodyPlan` only as the
first process proof demands. Rename or consolidate the types after two real
growth rules exist.

Retire the trait array when all of these are true:

1. an adaptation candidate changes developmental rules rather than a score;
2. growing that candidate produces a body;
3. the phenotype evaluator scores the grown body in a world;
4. played and unplayed lineages use the same path;
5. the old array can be deleted with its tests rather than maintained as a
   compatibility layer.

**Receipt, 2026-09-02 (P4a/PD5, then P4b/PE3a): four of the five are met, and
the array stays.**

- **1. Met.** `program::Revision` states *declared sites* — a part role, an
  admitted `ProcessRef`, a bounded cell count — and nothing scalar. Nothing in
  the commit path reads or writes a number that stands for fitness;
  `World::revise` builds the revision out of the discovery's own `Candidate`
  and refuses rather than substituting.
- **2. Met.** `program::preview` realizes the recipe and develops the declared
  sites into a `BodyPhenotype`, and a birth under a revision produces the same
  body through the same `program::express`. Both are receipted:
  `a_founder_preview_is_the_same_body_twice` and
  `a_birth_expresses_its_lines_revision_and_pays_for_it`.
- **3. Met (P4b).** `World::score` grows a candidate in a copy of the world for
  a bounded run and reads the flow record — income against rent for that line's
  bodies — which is Mark's ruling of 2026-09-01. It is the phenotype evaluator
  the condition asks for in the strongest sense available: it does not read the
  grown body at all, it reads what the grown body *earned in a world*. There is
  still no fitness term and no static formula, and the ordering is one function
  (`Score::beats`) over net income.
- **4. Met.** `World::revise` is one transaction and `World::express_filially`
  reads no `controlled`; `an_unplayed_lineage_takes_the_same_path` commits on an
  NPC line and watches its next birth arrive expressing it, by the identical
  code.
- **5. Not met, and it is now the only one.** `epoch::Trait`, `epoch::adapt`'s
  `fitness`, `epoch::standing` and `epoch::worlds` are untouched. What replaced
  their *job* is `world::adapt`: the ordering idea (descending complexity,
  commits landing immediately) was kept and brought across; the trait array and
  the squared-deficit fitness were not used. So the array is a compatibility
  layer with no consumer in the world model — but deleting it with its tests is
  Mark's call, and it was explicitly out of scope here.

So the array **remains explicitly provisional**, and the wait is now on the
deletion itself rather than on a missing replacement.

**Cost.** For a time the adaptation lab remains explicitly provisional. This
is preferable to deleting its only working vocabulary before the replacement
can express the same questions.

**Rules: Mark at deletion.** Adding the first developmental process field does
not itself authorize removing `epoch::Trait`, and neither does the scorer
landing: condition 5 is Mark's to call, and PE3a deliberately did not touch the
module or its tests.

### D5. Where does a meal go?

The default incorporation path currently grants a part and half the meal's mass
as energy. It also bypasses the venom subtraction used by the explicit editor
path. The central verb therefore collapses its tradeoff and its advertised
danger.

**Recommendation: keep one verb, but route the result.** The smallest playable
proof offers two destinations:

- **burn:** gain immediate usable energy and retain no part;
- **incorporate:** commit material to growth, pay the relevant risk and upkeep,
  and gain little or no immediate energy.

Further destinations arrive only when their receiving systems exist:

- provision reproduction;
- deposit or build a niche;
- assimilate a process while regrowing form under the host plan;
- graft a source subtree with its topology intact.

Assimilation and grafting are distinct. Assimilation preserves biological
function and provenance while allowing host-shaped growth. Grafting preserves
the source dependency structure and should carry compatibility and upkeep
costs. Neither is an ordinary inventory equip action.

**Why first in implementation.** Burn versus incorporate can be tested before
D1 through D4 and asks whether metabolize contains a worthwhile repeated
choice. The full branch operation waits on D1 because scalar prey have no
subtree to transfer.

**Cost.** This changes intent encoding, input, receipts, replay fixtures,
resource accounting, and the headed interaction. It is not only a one-function
edit.

**Rules: Mark.** This remains the recommended first playable proof.

### D6. Where does adaptation income come from?

The epoch lab converges because every lineage receives flat income and every
pressure has a direct answer. A single global pool weighted by fitness would
create a rich-get-richer collapse, but it would not yet model an ecology.

**Recommendation: bank is reproductive surplus earned from finite local
flows.** Worlds provide spatially and seasonally bounded sources. Phenotypes
gain access to some sources, compete where their paths overlap, and may support
one another where outputs become inputs. Distinct niches can coexist; crowded
lineages can fail locally; migration can matter.

The epoch layer may aggregate those lived results, but it must not manufacture
them from a global fitness ranking. Evented disturbances then move or disrupt
flows as seasoning rather than acting as the only source of loss.

**Cost.** This couples the played ecology to the adaptation phase and requires
the moment-to-epoch lift that the current lab deliberately avoids.

**Rules: Mark.** Defer implementation until the meal choice and one phenotype
path are playable.

### D7. What crosses the wing?

This is owned by the
[wing phenotype contract](2026-07-31_wing_phenotype_contract_plan.md).
Mesocosm's local consequence is simple: do not implement chronicle v1 as
another flat anatomy snapshot. A body profile carries a body revision and its
topology; a chronicle carries causal facts addressed to that revision. Geometry
may travel as an optional appearance projection, never as another game's rule
authority.

`Chronicle::found` currently says it regrows a descendant but attaches every
surviving part directly to the root. That star is acknowledged scaffolding. It
must be replaced before depth or path connectivity affects a returning line.

**Rules: wing plan.** Schema work waits until stable subject, body-revision,
and part addresses are settled.

### D8. What makes witnessing rewarding?

The adaptation transcript is a changelog. A useful field journal would contain
observed behaviour, uncertain hypotheses, remembered signals, dependency
diagrams, lineage changes, and before-and-after epoch summaries. Knowledge may
persist across descendants without becoming permanent stat power.

**Recommendation: defer the interface, retain observation material now.** Core
events should preserve what was observable, by whom, and under which sensory
conditions. They should not record hidden truth in a player-facing knowledge
stream. The journal can later materialize those observations into claims and
confidence.

**Cost.** This needs a UI surface and a distinction between world truth and
observer knowledge. Mesocosm has not selected its full UI lane.

**Rules: Mark, later.** It is not a Wave 2 implementation target.

---

## 5. Proof order

This is dependency order for phenotype work. The execution waves plan decides
when each proof runs.

### P0. Meal choice — **mechanically done 2026-07-31; the judgment is open**

Burn and incorporate produce mutually exclusive receipts from the same meal.

**Done when:** both paths replay identically; venom applies consistently; and
the headed choice feels tense rather than clerical on the existing body.

**Landed.** `Intent` now has one eating verb. `Intent::Incorporate` and the
old explicit `Intent::Metabolize` collapsed into
`Metabolize { organism, route }` over `Route::{Burn, Incorporate, Place}`,
so the editor path became a route rather than a second verb. `Route::Place`
holds the parent, offset, and yaw the editor arm used to carry inline.

Three behaviours changed, and each was a defect rather than a tuning choice:

- **Incorporating yields no immediate energy.** It used to grant
  `eaten.mass_mg / 2` *and* a part, which is why the central verb asked
  nothing. Burning yields the full mass and no part. A meal cannot be both
  meals.
- **Venom is charged on every route.** The explicit path subtracted it and the
  automatic path did not, so the safe-looking verb was the dangerous one and a
  warning signal was worth reading only in the editor.
- **Placement resolves before the meal is consumed.** A refused incorporation
  used to remove the organism and then re-insert it on failure; it now
  reserves nothing until the placement is known, so a refusal cannot lose a
  meal.

Seven tests in `tests/meal.rs`, including that both routes replay identically
across a snapshot boundary and that two worlds identical up to one meal diverge
in exactly the claimed way. The chronicle fixtures are byte-identical
afterwards, so the Isometry interchange is undisturbed.

Host: `E` grows, `F` burns, space grows, and an unattended capture run grows so
it still produces a body to look at.

**Still open, and it is the part that matters.** Whether the choice is tense or
clerical is Mark's judgment at the keyboard, and no test supplies it. Two
things are known to be missing from a fair reading: burning has no use yet
beyond deferring starvation, because energy only pays for movement, and there
is no visible pressure that makes hoarding mass feel costly.

### P1. One organism model — **LANDED 2026-08-01**

One authored prey and the controlled critter both use the body-bearing organism
representation.

**Done when:** changing control between them changes neither serialization nor
ecology semantics, and the scene renderer discovers both through the same path.

**Landed.** `Organism` carries a `BodyDocument` and an `energy_mg`; `volume` and
`half_extent` became readings of the root part, so a body and the shape the
world sees cannot disagree. `World` lost `position`, `body`, and `energy_mg`,
and gained `controlled: OrganismId`. The played critter is organism zero, built
by the same constructor as everything else.

Ten tests in `tests/control.rs`. The load-bearing ones: taking control leaves
the roster byte-identical, two worlds differing only in who is inhabited
serialize to the same length, and forty ticks of ecology run identically
whichever organism is pointed at.

**Three things the migration surfaced that the design pass did not predict.**

1. **A critter could eat itself.** Putting the player in the roster made
   self-consumption expressible for the first time. Refused as
   `Rejection::Itself`, and every prey search in the codebase now filters the
   controlled organism out. This was found by a panic, not by review.
2. **The played critter can now die.** The ecology retains on `stage != Spent`
   and nothing exempts the player, which is correct: an exemption would be a
   rule branching on who is playing. So `controlled()` returns `Option`,
   acting while disembodied is `Rejection::Disembodied` rather than a panic,
   and a world can outlive whoever was in it. That is what "leave the world
   running" has to mean.
3. **`mass_mg` and `body.total_mass_mg()` are now two truths.** Kept
   deliberately: merging them means routing grazing and upkeep through parts,
   which is the "bigger bodies cost more" work that belongs to P2. Recorded as
   a finding rather than left silent.

**Replay hashes moved, as predicted.** `World::new` now seeds one more organism
and ids shift by one, so the played fixture's hunt finds different prey and
grows to 36 parts where it grew to 47. Fixtures regenerated in the same commit,
the full Mesocosm-to-Isometry-and-back loop reproven, and the property asserted
in place of hash stability is that **the same intent trace moves whoever is
inhabited by the same amount**.

### P2. One embodied consequence — **LANDED 2026-08-01**

Wire `BodyDocument::reach()` into actual interaction, then add one connected
process path.

**Done when:** two bodies have different reachable actions because of anatomy;
severing a dependency removes the action; no capability number was edited; and
the receipt and headed view state which embodied requirement failed.

**Landed.** `const REACH: i32 = 8` is gone from the core, and gone again from
the host, which had been keeping its own copy of the rule for presentation.

`mesocosm-core::process` holds the whole vocabulary: three processes
(`Contract`, `Intake`, `Sense`) and one capability (`Reach`). **Processes are
read from geometry** through `classify`, never stored, so a part cannot be
given an ability its shape does not imply. A long thin part is an actuator, a
bulky one admits material, a small one senses, and a plate does neither while
still being armour.

Reach is a **satisfied path, not a measurement**: the distance to the furthest
living part that contracts, plus that part's extent, floored at the body's own
bulk. So a bare critter touches what is against it, a limb extends that, a
longer limb extends it further, a plate buys nothing, and severing the limb
takes the reach with it. Seven tests in `process.rs` and seven in
`tests/embodied.rs`.

**Refusals now name the unmet requirement.** `Rejection::OutOfReach(Unmet)`
distinguishes *you have no actuator at all* from *your actuator does not extend
that far*, because those are different problems and a player deserves to be
told which.

**What running it headed found, which no test did.** An unattended capture grew
**one part in nine hundred frames**. Reach fell from 8 to about 3 for a
starting critter, and the auto-eat driver stood still waiting for food to
arrive. Fixed by making it hunt. Every fixture in the workspace had the same
assumption baked in as a literal `<= 8`, including the shared replay trace,
which now records itself by *driving* a scratch world rather than guessing
which organisms are close enough.

**The silhouette started showing capability**, which was predicted and is
pleasant to see land: the limbs a critter grew for reach are the parts visibly
sticking out of it. Colour already showed history; shape now shows what a body
can do. Receipt: `Code/testing/mesocosm/10_derived_reach.png`, a 97-part
critter after 452 steps.

**The ledger reconciliation followed on the same day**, and it is what P0's
open judgment was actually waiting on. `Organism::mass_mg` is gone, upkeep
scales with the body, and upkeep takes the budget before it takes flesh. See
the ledger section above. `organism.rs` was split first, into types and
`organism/ecology.rs`, because it had passed this repo's line ceiling and the
rule is to split before adding.

### P3. Branch transfer — **LANDED 2026-09-01**

Harvest or receive a source subtree, remap its local ids, and preserve its
source addresses and parent relations.

**Done when:** the source loses the branch, the recipient gains a functioning
or visibly incompatible branch according to the chosen route, severing the
graft cascades, and snapshot/replay agree.

**All met.** `Intent::Graft { organism, part, crossing }` takes a living
subtree off a carcass and sets it on the played body. Four steps, and each of
them is a named operation rather than an assembled call site:

- **harvest.** `BodyPhenotype::harvest` lifts the subtree without changing
  anything, as a `Branch` of `Cutting`s: each carries its donor-local id, its
  donor-local parent, its joint, and what the donor had allocated on it.
  Reading and taking are separate on purpose — the world resolves where the
  branch would land, and whether the recipient can hold it, before the donor
  loses a milligram.
- **remap.** `BodyPhenotype::receive` attaches every part through the ordinary
  allocator, rewriting each internal parent onto the freshly allocated id and
  preserving the joint exactly. The branch arrives shaped the way it grew.
- **graft.** One `AllocationProposal` over exactly the arriving parts, lowered
  by `BodyPhenotype::develop`. **There is no second attachment authority and no
  second developmental authority**: a carried arrangement this body's rules
  would refuse is refused by the same validator that would refuse it from the
  editor, and it is never substituted.
- **the terms.** The world's affinity table returns the verdict, before
  anything moves.

**Where the route comes from, and why there are two of them.** The wing
contract separates *carry this body* from *regrow here* for an individual
crossing vessels; a branch crossing between bodies is that question one scale
down, and `Crossing::{Carry, Regrow}` is it. The ProcessDef plan's three graft
verdicts then decide which routes are feasible, exactly as the contract
requires — *the destination declares compatibility; an incompatible carry is
refused or redirected to regrowth rather than silently rewritten*:

| Verdict | `Carry` | `Regrow` |
| --- | --- | --- |
| `Native` (same domain) | the donor's arrangement, cell for cell | this body's own seeding |
| `Adapter` (a favoured edge) | attached, and **expressing nothing** until an adapter is grown | this body's own seeding |
| `Refused` (a disfavoured edge) | `Rejection::Incompatible { from, into }` | still feasible |

That answers the ProcessDef plan's open question — *is a disfavoured edge a
hard gate or an expensive recoverable graft* — only for **carrying**: it is a
hard gate, and regrowth is what remains. Whether regrowth should itself be
gated or priced across a disfavoured edge is still Mark's.

**Visibly incompatible is a mechanical state, not a label.** An adapted branch
is on the body, weighs what it weighed, and pays rent for that weight, and
every cell of it is free: `explain` reports `free == capacity` with no sites,
`expresses_on` is false, and PD2's gland does not come across, so the same
branch that makes a body sting on a native carry makes nothing on an adapted
one. Growing the adapter afterwards is an ordinary `Intent::Rearrange` on
ordinary free tissue.

**Affinity is a table, not an enum.** `mesocosm-core::graft` holds `Domain`,
`Verdict`, `Crossing` and `Affinity`; a world holds one `Affinity` and each
lineage carries a `Domain` drawn from its own salted stream and inherited by a
fork. The default is the ruling's three-domain favoured cycle. The domains are
**numbers**, deliberately: the ruling's animal-like/fungal-like/plant-like is
English for a default world, and naming them is a naming round rather than an
implementation decision. `Affinity::digest` is over the rule-bearing bytes, so
two worlds that agree about a domain number and disagree about its edges cannot
agree about a graft.

**Provenance is per part.** Every arriving part's `Origin::Incorporated`
names *its own* donor id — not the branch root's and not the donor's root,
which is the correction PE2 made one part at a time and this one makes for a
whole subtree. `Origin` itself is unchanged, so the body-only bytes mesh, Lens
and Isometry consume did not move; the richer source address the wing contract
V1 wants (subject, body revision, acquisition event) is still V1's, and the
transaction-level half of it is on the world's `Graft` record.

**Two things the implementation surfaced that the design pass did not
predict.**

1. **The plan has to be asked about the branch, not about its root.** The first
   cut resolved a site for the graft root's own half-extent, which is the wrong
   question: a branch keeps its internal joints, so a site with room for the
   part at the top of it is not a site with room for the thing. Every graft in
   the recorded demo was refused `NoRoom`. `Branch::bounds` now reports the box
   the whole branch occupies in its own frame, the plan sites *that*, and the
   root is placed by the offset between the two. `Yaw::rotate` and
   `Yaw::compose` became public for it, because working the box out before
   anything is attached is the same arithmetic `world_pivot` does and must not
   be a second copy of it.
2. **A world outlives a body, and the record has to know that.** The world
   keeps one `Graft` — the `last_observation` arrangement, and for the same
   reason — but the branch belongs to the creature that took it, and the demo
   succeeds into a descendant. A panel reading the record against whoever is
   played *now* would name parts of somebody else's anatomy. The record carries
   its `recipient`, and `World::carried_branch` is the reading that respects it.

**The price is PD2's, applied per part.** `Instruction::cost_cells` is one
count across every part a proposal names, and a cell is worth what its own
part's tissue is worth, so a multi-part development cannot be priced at one
part's rate without inventing one. The graft asks the same question the
validator asks — which cells end up expressing something other than what they
express now — one part at a time. No new number, and the ordering falls out
rather than being chosen: a regrowth costs nothing, a native carry costs the
difference, and an adapted carry costs every occupied cell, so the awkward
graft is the expensive one. Mark's acquisition-cost formula, which PE2 recorded
as still open, is untouched by this.

**Receipts.** Eight tests in `tests/embodied/graft.rs`, one per clause: the
source loses the branch and the recipient gains every part of it; every
transferred part names the part it came off and keeps its joint; a native carry
lands a functioning branch; a cross-domain carry lands a visibly incompatible
one and it can be repaired; a disfavoured carry is refused and regrowth is the
route that remains; severing the graft takes the whole imported branch and
still explains it; a transfer survives a snapshot and replays to the same hash;
and the whole of a body is not a branch. Seven more in `graft.rs` for the
table, and one in `species.rs` for a fork inheriting what its parent is made
of. `tests/matter.rs` conserves through both routes **and** through a refused
one; `tests/flows/transfers.rs` reconciles the transfer as one
`Process::Graft` record naming both subjects, beside PE0's whole-tick
reconciliation. Two in `mesocosm-views` for the panel's two rows in the exact
words a player reads, and for the two refusals. Splits at the ceiling:
`tests/flows/transfers.rs` out of `flows.rs`, and
`mesocosm-genet/src/played/tests.rs` out of `played.rs`.

**The recorded demo gains one.** At tick 340, the step after the endurance
window closes, the demo takes a two-part branch off a carcass of line 2 — a
`Carry` over a favoured cross-domain edge, so it lands **needing an adapter**
and doing nothing, which is the more interesting of the two landings to have in
the loop. It was tried at step 40 first and the recording came out with no heir
at the death checkpoint: three extra parts early enough changes what a critter
eats for three thousand ticks. The trace still comes through the PE2 discovery
at 219, continues at three births, and succeeds into a descendant at 3000.
Hash `652c5bcfdc6013c1`.

**The instrument did not move, and it could not have.** It drives
`World::apply` with nothing but `Intent::Idle`, and a graft is a played verb,
so no run of it can reach one. Nor did founding move underneath it: a lineage's
tissue domain is drawn from its own `GRAFT_SALT` stream, exactly as its recipe
is, so no ecology draw shifted. Re-run anyway against the drawn baseline, and
all ten seeds came back **identical** to what DC4 recorded — verdict, start,
peak, peak tick, end, decided tick, cumulative births and deaths, end kingdom
counts and end biomass, seed for seed and to the milligram. **0 breathes /
10 thins / 0 boil / 0 collapse** stands unchanged. Stopped after the baseline
batch, for PE2's reason: the isolating mechanism is structural rather than a
per-seed coincidence, and finishing the other five batches would have
overwritten `dc4_roster.json` with new timing on an unmoved result. That file
is byte-identical to what DC4 recorded.

**Capture:** `Code/testing/mesocosm/p3_graft.png`, written by
`mesocosm-genet/examples/p3_receipt.rs`. Two real pipelines on one sheet — the
anatomy through `mesocosm-render`, the panel through the cambium chrome —
because a mid-run host frame buries a digging critter in its own burrow and
shows the transfer to nobody. The body wears a pink frond and a cream limb the
recipient's own palette does not use; underneath, *branch: 2 parts from part 2
of line 5* and *terms: carried on part 2 — needs an adapter, doing nothing
yet*.

**Residues, and what PD3/PD4 and PE3 inherit.**

- **The affinity table has no door yet.** It is native data with a digest, the
  way `discovery::conditions()` is, and PD3's pack admission is where a world
  gets to hold a different one. The shape is already right for it — a domain
  count and a list of favoured directed edges — and `Affinity::digest` is what
  stops a packed table's meaning drifting under its own label. Nothing here
  reruns a generator from a name.
- **The domains are unnamed on purpose**, so a panel says what the *verdict*
  was and never what the tissue is called. Naming them is a naming round and
  Mark's; until then a receipt that wanted to say "fungal-like" cannot.
- **The adapter is free tissue, not an organ.** The ProcessDef ruling describes
  an adapter that occupies cells near the graft boundary, consumes upkeep, or
  reduces throughput, and that a learned compatibility process can shrink. What
  landed is the first of those and only the first: the branch arrives with its
  tissue free, and what the player grows on it is an ordinary process, not an
  adapter with its own identity. A real adapter definition is a `ProcessDef`,
  which is PD3's door.
- **Nothing carries channel links across, because there are none.** The wing
  contract's clause about internal functional links crossing with a branch and
  cut-boundary links being re-established is inert until PD6 builds the channel
  graph. The structural half is done and the functional half has nothing to do.
- **`Rearranged` still does not know it came from a discovery**, and now
  neither does it know it came from a graft: a graft's development is an
  ordinary `Instruction` and the only thing tying it to the transfer is the
  revision on the world's `Graft` record. PE2 flagged the first half of this;
  PE3's review is where a join would earn its keep.
- **A graft is not a feat.** `score.rs` reads meals and ignores
  `Event::Grafted`, which is right for now — wearing somebody is not eating
  them — but PE3's lineage review is the first place a reading might want to
  say a line is building itself out of its neighbours.
- **NPC lineages never graft.** The verb is the played body's, exactly as the
  ecology's own behaviour never proposes a development. That is PE2's open NPC
  acquisition ruling, unchanged and now one verb wider.

**What live transfer would still need**, since this proof deliberately harvests
from a corpse. Three things, and only the first is small. A donor-side
consequence model, because taking a branch off something alive is an injury and
the ecology has no notion of one: today `sever` is the whole vocabulary and
nothing decides whether the donor dies, flees, or fights. A refusal or a
contest, because a living donor is an agent and helping yourself to its arm
cannot be an unopposed reach check. And **the milligrams of the cut itself** —
phenotype D3a's gate — because the boundary between two parts is where a live
cut lands, and whole-part loss cannot express "half of this segment came away"
without either creating or destroying matter. PE2 refused to eat a severed part
for exactly that reason and the refusal still stands: a severed branch's mass
has already left the account. None of that is P3's, and none of it is needed
for what P3 claims.

### P4. Adaptation bridge — **PARTIAL 2026-09-02** (six of seven clauses)

Grow several candidate developmental changes and score their phenotypes in one
authored world.

**Done when:** a chosen mutation cites a lived scarcity, produces a visibly and
mechanically different descendant, commits a developmental program rather than
a body snapshot, and reproduces its founder preview under identical declared
inputs. A changed environment may realize a legibly different phenotype from
that same program; unplayed lineages use the same evaluator; and the old trait
array has a concrete deletion receipt.

**What landed (P4a, the lineage program).** A lineage carries a versioned,
append-only development program; a commit is a recorded intent
(`Intent::Revise`) over a world transaction (`World::revise`); a descendant is
born expressing it through the one validator, or is born anyway with the
record naming the revision it could not express and why; and a founder preview
realizes the same program from declared inputs alone. So five of the seven
clauses stand: the mutation cites a lived scarcity (`program::Citation` carries
the condition and the discovery digest), the descendant is mechanically
different (it secretes, and the bite is what eats it pays), a program is
committed rather than a snapshot (declared sites, never cells), the preview
reproduces under identical declared inputs, and rich versus lean ground grows
one program into two legibly different phenotypes. Unplayed lineages used the
same *path*, and the trait array held three of its five deletion conditions
(§D4) at that point.

**What landed second (P4b, the scorer), 2026-09-02.** `World::score` grows one
candidate in a **copy** of the world — the revision committed on that line in
the copy, the copy driven for a bounded window with nobody at the keyboard — and
reads the flow record: income, rent, everything else that left, and how many
bodies of the line were born inside the window. That is Mark's ruling of
2026-09-01 exactly: no static formula over body readings, no fitness term, and
no scoring vocabulary that did not already exist. `Score` is those figures plus
the run length; the only place it becomes a single number is `Score::beats`,
which is net income and a strict comparison, so a tie leaves the status quo.
The copy is discarded and the real world's hash is unmoved, which is asserted
rather than argued.

`World::candidates` puts the founding revision — no change — first on every
line's list, so *the status quo beat every candidate* is a reading rather than
an absence, and `World::adapt_round` runs the turn for every unplayed living
line in initiative order, committing immediately through `World::revise`. So the
gate's own sentence is answered — several candidate developmental changes are
grown and their phenotypes scored in a world — and with it the sixth clause:
unplayed lineages use the same **evaluator** as well as the same path. There is
only one evaluator, and nothing in it reads `controlled` except to skip the
played line, whose turn is the review.

**The window is one brood interval, and it is load-bearing.** A revision only
ever shows up in descendants, so a window with no birth in it scores the
candidate and the status quo *identically*. Measured on seed 4,242: at one
judgement window (60 ticks) every line scored the gland about a tenth of a
percent **worse**, which is its development cost and nothing else; the sign
turns between 240 and 600. `DEFAULT_SCORE_TICKS` is therefore
`rates::GESTATION_BASE` (480), the shortest span with a name that reaches past
that, and it is a world rule (`WorldRules::score_ticks`) because what an
unplayed line commits is the world.

**What remains, and it is one clause.** *The old trait array has a concrete
deletion receipt* — §D4 now holds **four of five** conditions, and the fifth is
that the array can be deleted with its tests rather than maintained. It still
cannot: `epoch::Trait` and the four modules around it are untouched, and
deleting them is Mark's call and explicitly not this slice's. So P4 stays
partial on that clause and nothing else, and it is a ruling rather than work.

**Not P4's, and not blocking it.** Reviewing several candidates *on screen*,
pricing the choice and previewing a founder is the playable ecology plan's PE3b.
Revision cost stays flat, ruled 2026-09-01; life-stage pricing (epoch-boundary
plan §8 q4) is still open and would multiply the descendant's price in one
place, `program::express`.

### P5. Contested flow

Lift finite local resource results into epoch surplus.

**Done when:** niche overlap can cause a lineage to fail, distinct resource
paths can coexist, and changing spatial access changes the adaptation bank
without a global fitness-share rule. Crossing the local simulation-resolution
boundary conserves biomass and lineage state rather than rerolling the ecology.

### P6. Cross-vessel body revision

Execute the wing contract against Isometry, then Paredros when it becomes a
real consumer.

**Done when:** see the wing plan's acceptance receipts.

---

## 6. Stop rules

- Do not add a broad process catalog before one path is played.
- Do not delete the trait lab before phenotype evaluation replaces it.
- Do not let mesh or render output become simulation authority.
- Do not make the player the only organism with anatomy.
- Do not confuse one organism rule contract with one permanently realized voxel
  tree for every distant life form.
- Do not store an adaptation preview as the lineage's body template. The
  heritable authority is the developmental program; phenotype is a realization.
- Do not force circulation, exchange, or other cyclic biological networks into
  the structural dependency tree.
- Do not use a global fitness-weighted pool as a substitute for resource flow.
- Do not version the wire before identity and part addressing are settled.
- Do not build the journal before observation events distinguish appearance
  from hidden truth.
- Do not give a channel requirement a relation it cannot name. Broadcast is
  cut from the initial vocabulary until a phenotype proves which relation
  it needs.
- Do not store a control archetype on an organism. Requirements attach to
  actions and processes; centralized or quorum is a derived reading.
- Do not type a channel endpoint by its process. Endpoints are flows;
  processes transform them, and a mismatch is an adapter part rather than
  a compatibility table.
- Do not share an evaluator between the body and the ecology before each
  has independently proven the same mechanism.
- Do not mutate bytes behind a shared content-addressed `VolumeRef`; body-local
  cell change produces a new immutable reference or an explicit body patch.
- Do not let an async mesh or collider result publish after its source body or
  volume revision has changed.
- Do not let projection work queue without a cap, deduplication, or a truthful
  fallback under backpressure.
- Do not commit a collision or tactile proposal against a different body
  revision than the one it queried.
- Do not use a visually different ancestor as the fallback for a body whose
  current anatomy changes capability; lower fidelity stays truthful.
- Do not let a renderer- or physics-private joint pose change reach, occupancy,
  damage, or another capability; consequential pose is quantized phenotype
  state.

---

---

## 7. P1 design pass: one organism model

**Written 2026-08-01, before implementation, because P1 is a state migration
rather than a feature.** Five things must be settled first. Nothing here is
built; each entry states the question, the answer, and why the alternatives
lose.

Current shape, verified: `World` holds `position`, `energy_mg`, and one
`BodyDocument` for the played critter, beside `organisms: Vec<Organism>` where
each organism is a scalar record with `volume`, `half_extent`, `position`,
`mass_mg`, `stage`, `signal`, `venom_mg`, and `guise`. The played critter is
not in that vector.

### 7.1 What does control name?

**`controlled: OrganismId` on `World`.** Control is a pointer, not a shape.

Everything else follows from this one line. Switching lineage becomes moving a
pointer rather than reconstructing state, an unplayed critter and a played one
serialize identically, and the wing's third law holds at the level of the
simulation rather than only the file format. It also makes "leave the world
running" expressible: nothing about a world requires anyone to be in it.

The alternative, a `played: bool` on the organism, reintroduces exactly the
marker Law C forbids and would let rules branch on it.

### 7.2 Where do position, energy, and body live?

**All three move onto `Organism`.** `World::position` and `World::energy_mg`
become fields every organism has, and `World::body` becomes a body every
organism may have.

Position is uncontroversial: organisms already carry one, and the played
critter's separate copy is the anomaly. Energy is the load-bearing one, because
**an unplayed critter that cannot run out of energy cannot starve**, and the
ecology's existing `mass_mg` drain is a different quantity doing a similar job.
Those two want reconciling during the migration rather than after it, and the
honest reading is that `mass_mg` is what a body weighs while `energy_mg` is
what it can spend.

### 7.3 Realized individuals versus cohorts

**The rule contract is one model; the storage may split when scale demands
it.** Rules must never ask whether a subject is realized.

The split is a future optimisation with a gate, not a shape to build now.
Nothing today needs it, and an aggregation path with no consumer would be the
speculative generality this plan's own stop rules forbid. What is settled in
advance is the *conservation and no-reroll* discipline below, so that when the
split arrives it cannot quietly lose creatures.

A subject is **realized** when it is played, nearby, named, injured, or
otherwise consequential, and carries its own body revision. Everything else is
a **cohort**: a species, a count, conserved mass and energy, a developmental
distribution, and a causal seed.

Two operations, and they must be exact inverses on the conserved quantities:

- **Materialize**: draw an individual out of a cohort, deterministically from
  the seed, subtracting its mass and energy from the cohort's totals.
- **Aggregate**: return an individual to its cohort, adding those quantities
  back. **Refuses** for any subject with a chronicle, a name, or player
  history, because folding those back into a distribution is the fact loss the
  wing's keystone forbids.

The failure mode to design against is rerolling: crossing the boundary twice
must not resample a creature into a different one. That is why the seed is part
of cohort state rather than drawn at materialization.

### 7.4 How are bodies referenced?

**By value on realized subjects, interned by developmental identity for
cohorts.** A cohort does not hold N bodies; it holds the rules that would grow
them and the statistics of what did grow.

Interning shared *immutable* volume data is already the pattern (`VolumeRef` is
a content address and the core never reads through it). Bodies are mutable and
individual, so they are not interned; what can be shared is the developmental
program behind them.

**Do not intern bodies by structural equality.** Two critters that currently
look identical have different futures and different provenance, and sharing
their anatomy would make one's injury the other's.

### 7.5 How does the current player migrate without changing replay?

**It does not migrate silently.** The existing replay fixtures encode a world
whose player is not in the organism vector, so any faithful migration changes
the state hash. Pretending otherwise would be the more expensive mistake.

The plan is therefore:

1. Land the migration with the fixtures **regenerated in the same commit**, and
   state plainly in that commit that hashes moved and why.
2. Keep the pre-migration fixture bytes alongside, so the old trace can still be
   decoded and compared structurally even though it no longer hashes equal.
3. Assert the property that actually matters, which is not hash stability across
   a schema change but **that the same intent trace produces the same result
   before and after control moves between two subjects**.

The determinism guarantee is about a build replaying its own traces, not about
a schema never changing. The refusal machinery built for the interchange is the
precedent: a version boundary is honest, a silent reinterpretation is not.

### Done when

- `World` has no player-specific `position`, `energy_mg`, or `body`.
- Control is an `OrganismId`, and moving it changes neither serialization nor
  ecology semantics.
- The scene renderer discovers the played critter through the same path as
  everything else.
- Replay holds within the new schema, with regenerated fixtures and a commit
  that says so.

**The movement property expires at P2.** "The same trace moves whoever is
inhabited by the same amount" is a P1 receipt and nothing more. P2 exists to
make different bodies move and spend differently, so that assertion is
*supposed* to stop being true. The lasting guarantees are narrower and should
be the ones cited later: **deterministic replay within one schema**, and
**changing control rebuilds neither organism**.

**Corrected 2026-08-01, before implementing.** An earlier draft of this list
also required a cohort round-trip. That is P5's done-condition, duplicated here
by mistake. §7.3 settles the cohort *rule contract*, which P1 satisfies by
having nothing branch on realized-ness; the second storage arrives when
scale demands it, and building an aggregation path with no consumer would be
the speculative generality this plan's own stop rules forbid.

### Rulings taken during P1

**Self-prey, ruled 2026-08-01.** A subject cannot target itself through
`Metabolize`, and `Rejection::Itself` is correct. This forbids treating
yourself as *prey*; it deliberately does **not** forbid future
self-resorption, which is consuming one of your own parts under starvation or
metamorphosis. That is a different, **part-addressed** operation and it stays
open.

**Mortality, ruled 2026-08-01.** The played organism receives no ecological
immunity, and its death does not end the world. **Disembodiment is a seam**:
it is where witnessing, world examination, adaptation, and choosing another
eligible critter happen. `World::control_lost()` names whose body was lost on
the tick it happened, so a host has something to react to rather than a state
it must infer.

Two consequences landed with it. Control now ends when the subject stops being
*alive* rather than when its row leaves the roster: natural death makes an
organism carrion, which lingers until it is spent, and an earlier cut that
checked only for the id left a decomposing critter walking and eating.
And `controlled` is `Option<OrganismId>` with no ghost body, because *nobody is
embodied* and *this id no longer names anything* are different facts and a
caller has to be able to tell them apart.

**Control is a recorded intent, ruled 2026-08-01.** `Intent::TakeControl`
replaces the public mutator P1 first shipped. Ordered intents are the only path
by which world state changes, and a control change made outside that path would
replay every fact about a run except who was living it. Lineage switching is
gameplay, so it belongs in the trace.

### Population is lives

**Ruled 2026-08-01, and it falls out of the mortality ruling rather than being
added to it.** If death costs you a body and you then pick another, then **a
lineage with twenty surviving individuals is a lineage with twenty
perspectives**. Nobody had to design that; it is what the previous ruling
already meant.

Three consequences, and the second is the one this plan has been missing.

**Reproduction becomes a survival resource.** Provisioning offspring is not a
score, it is buying lives. That is the destination D5 needed: burning currently
has no downside because nothing makes hoarding costly, and *provision* is the
first route with real teeth, because spending a meal on a descendant is
spending it on your own continuity.

**Keeping the niche that supports your kin matters more than collecting
upgrades.** A lineage whose food supply collapses loses lives, not points.

**Extinction removes an embodiment option even when the lineage survives in the
record.** The archive keeps what a form *was*; it does not keep somewhere to
stand.

#### The pools, and which of them are real

The design names six pools. **Two exist, one is a query, three are new state,
and one is not a pool at all** — worth separating before anyone builds six
systems.

| Pool | Status |
| ---- | ------ |
| Living population | **Exists**: `World::organisms`. |
| Embodiment pool | **A query, not storage.** Living individuals the player may inhabit, which is a filter over the above. Storing it would let it drift. |
| Known roster | **New state.** Lineages encountered, studied, or inhabited. This is observer knowledge, which D8 already says to start retaining before the journal that displays it exists. |
| Dormant pool | **New state.** Spores, eggs, seeds, cysts, symbionts: forms that can return. |
| Lineage archive | **New state**, partly derivable from chronicles. What an extinct form was and what it did. |
| Possibility space | **Not a pool.** See below. |

**Possibility space must be computed, never stored.** It is what current world
conditions could generate, support, or admit, and the entire value of the idea
is that world change *edits* it: oxygenation permits unfamiliar metabolisms,
cooling closes tropical niches and opens cryophilic ones, a decomposer's
extinction locks nutrients away. That only works if it is derived from world
state on demand. Stored, it becomes a spawn table that drifts out of step with
the world it claims to describe, and the world-change mechanic quietly stops
working.

This is the same rule as capability: **derived from what is actually there,
never a number somebody maintains.** It is worth stating twice because it will
be tempting twice.

#### Two recovery routes

- **Inhabit** an existing eligible organism. Its body, condition, relationships,
  and location are already real, and you take them as they are.
- **Found** a new simple organism from dormant or environmental material, below
  the achieved complexity frontier.

**Founding consumes actual world possibility**, which is what keeps "start
something simpler" from being an infinite-life menu. An ocean with viable
microbial life offers humble restarts indefinitely; a sterilised world offers
none. That makes sterilisation a real loss condition without anyone authoring a
game-over.

#### The hazard to design against

**More offspring means more lives, which makes fecundity strictly better at the
meta level regardless of in-world fitness.** A lineage of two hundred algae has
two hundred lives; a lineage of three apex predators has three. If lives are
the currency, the dominant strategy is to play the simplest, most fecund thing
in the world and never build anything.

The counterweight has to be that **a simple critter's run is genuinely less
capable** — fewer reachable actions, less it can eat, less it can survive.
That is exactly what phenotype-derived capability provides, which makes this
hazard another argument for P2 rather than a separate problem. Recorded now so
that when someone tunes fecundity, they know which tension they are tuning.

#### Death as the examination screen

Disembodiment was already ruled a seam. This says what is *on* it: which bodies
remain, which lineages are declining, which are dormant, why something can no
longer survive here, and which environmental threshold has just been crossed.

Choosing the next critter is respawning, reading the simulation, and deciding
which part of the biosphere deserves care, in one act. **That is a better answer
to the witnessing gap than a field journal**, because it arrives diegetically at
a moment the player already has to stop, rather than as a screen they must
remember to open. D8's interface question is not answered, but its *material*
now has an obvious home.

#### Event vocabulary

- **Endogenous transitions** arise from the ecology: oxygenation, trophic
  cascades, soil creation, reef building, invasive competition.
- **Exogenous disturbances** arrive from outside: impacts, eruptions, orbital
  changes, abrupt climate shifts.
- **Storyteller pressure** chooses when and where a disturbance enters. **World
  state determines its consequences.**

That last split is the load-bearing one. Volcanism does not delete thirty
percent of the roster; it changes light, temperature, acidity, and substrate,
and the ecology decides what survives. A storyteller that picks the *outcome*
is authoring a calamity; one that picks the *insult* is applying pressure to a
world that answers for itself.

### The three ledgers

**Named 2026-08-01, because "two truths" was the wrong framing and would have
hardened into a permanent excuse.** There are three accounts, and until P2
reconciles them each rule must say which one it reads:

**Reconciled 2026-08-01. There are two accounts now, not three.**

| Account | What it is | Authoritative for |
| ------- | ---------- | ----------------- |
| `Organism::biomass_mg()` | **Body mass**, read off the surviving parts. | Everything about substance: growth, feeding, starvation, reproduction, upkeep, and anything derived from the phenotype. |
| `Organism::energy_mg` | **Budget.** What a subject can spend. | Acting: movement, the cost side of metabolizing, and the first call on upkeep. |

~~`Organism::mass_mg`~~ is **gone**. The scalar the ecology moved is now the
body itself: `gain_mass` and `spend_mass` write through to the root part, so
grazing, upkeep, death, and reproduction all move the same quantity anatomy
reads. There is no longer a place where the two can disagree.

**Upkeep scales with the body**, which is the whole point:
`UPKEEP_MG + biomass / UPKEEP_SHARE`. Flat upkeep is why a forty-part critter
cost exactly what a single cell cost, and why growing had no downside.

**Upkeep takes the budget first and the body second.** That ordering is what
makes burning a meal worth something: banked energy buys survival directly, and
a creature with nothing left consumes itself. Autophagy, and it falls out of
the ordering rather than being a mechanic anybody added.

So **burn or grow is finally a question**: burning fills the tank, growing
enlarges the engine and raises the rent forever.

**One defect the naming exposed before the reconciliation, fixed 2026-08-01.** Reproduction cloned the
parent's whole anatomy while charging a quarter of its reserve, so a forty-part
parent produced a forty-part child and paid for a fraction of one. Read
literally across two ledgers, that manufactured structural mass out of nothing.
An offspring now starts as a single root part sized to exactly what was paid.
Inheriting a developmental program and regrowing a phenotype from it remains
P4; this is the honest placeholder, and unlike the clone it conserves mass.

**Done.** Grazing and upkeep no longer move a scalar anatomy cannot see.

### Not in P1

Damage to specific parts, prey that lose subtrees, phenotype-granted actions,
and branch transfer. Those are P2 and P3, and they are the *reason* for P1
rather than part of it.

## Findings

- **2026-09-01:** a graft's placement is decided by the branch, not by the part
  at the top of it. Siting the graft root alone refused every graft in the
  recorded demo with `NoRoom`, because a branch keeps its own joints and a site
  with room for one part is not a site with room for a subtree. This is the
  same corner-versus-pivot family of mistake the body pipeline plan records:
  measuring the wrong extent of the right thing.
- **2026-09-01:** the world's record of a transfer outlives the body that took
  it. `last_graft` is world state and survives succession; the branch does not,
  because it was on the creature that died. A reading that spoke about "the
  branch" without checking whose it was would name parts of a descendant's
  anatomy. `World::carried_branch` is the scoped reading, and the record
  carries its `recipient` so the scoping is a fact rather than a convention.
- **2026-09-01:** the existing body geometry path already has the useful
  granularity the downstream proposal was seeking: immutable micro-voxel
  volumes per distinct part, a content address in the body, greedy mesh reuse,
  and a separate revision-tagged capsule projection. What it lacks is
  body-local cell mutation and the atomic invalidation contract that mutation
  would require. A fixed local-grid size or new mesher is not yet the missing
  authority.

- **2026-08-01:** making upkeep scale with the body changed what play can
  reach. A hunting critter grew to 36 parts under flat upkeep and to 14 once
  growth cost something, which **re-created the Law C size tell in the other
  direction**: the generator still produced up to 50 parts, so a consumer could
  again guess origin from a part count. Isometry's
  `size_does_not_give_the_played_one_away` caught it, which is twice that test
  has earned its keep. The generator's ceiling is now coupled to the world's
  economics as a **standing maintenance obligation**: past some size a creature
  cannot pay its own rent, and a generated creature larger than play can
  sustain is not one the world could have produced.
- **2026-08-01:** the ecology's tick-count tests became untunable once survival
  depended on both body mass and a banked budget. Rewritten to run until an
  outcome occurs rather than for a fixed number of ticks, which is what they
  were always asserting.
- **2026-08-01:** a well-fed mimic now holds its weight rather than visibly
  wasting, because upkeep drains its budget before its flesh. The tell became
  the *absence of growth beside a real plant* rather than visible decline,
  which takes patience to read. That is a better tell, and it arrived as a
  consequence rather than a design.

- **2026-08-01:** every fixture in the workspace had `REACH = 8` baked in as a
  literal `<= 8`, including the shared replay trace, the host's presentation
  dimming, and three test helpers. Deriving reach broke all of them at once,
  which is the honest cost of replacing a constant that several modules had
  quietly agreed on. The replay trace now records itself by driving.
- **2026-08-01:** the headed run caught what the suite could not. With reach
  derived, an unattended capture grew one part in nine hundred frames, because
  the auto-eat driver waited for food instead of hunting. Every test passed
  throughout.

- **2026-08-01:** there are **two eligibility rules that never meet**.
  `World::is_eligible` gates `Intent::TakeControl` on being alive, while
  `epoch::can_switch_to` implements the ruled complexity frontier and is called
  by nothing outside its own tests. So the frontier is not enforced where
  control actually moves: today you could inhabit anything alive, however
  elaborate. It cannot be fixed by wiring one to the other, because
  `can_switch_to` reasons over `epoch::Lineage` and its trait array while
  control reasons over `Organism`. **It is the P1 problem again at the lineage
  level: two creature models that do not share a notion of complexity.** The
  frontier becomes enforceable when the trait array retires (D4), and until
  then embodiment eligibility is deliberately weaker than the design says.

- **2026-08-01:** P1's first cut let a dead critter keep playing. `controlled()`
  checked only that the id was in the roster, but natural death leaves an
  organism as carrion until it is spent, so a decomposing subject could still
  move and eat. Control now requires the subject to be alive, and the test that
  covered this removed the row by hand instead of killing it through the
  ecology, which is why it passed.
- **2026-08-01:** P1's `take_control` mutated the world outside `apply`,
  contradicting the core's own boundary that ordered intents are the only
  mutation path. A trace could reproduce everything about a run except who was
  living it. Replaced by `Intent::TakeControl`.
- **2026-08-01:** reproduction manufactured structural mass, cloning a parent's
  whole anatomy for a quarter of its reserve. Found by naming the ledgers
  rather than by a test, which is the argument for naming them.

- **2026-08-01:** P1 surfaced that the played critter could target itself once
  it joined the organism roster. Refused, and every prey search now excludes
  the controlled organism. Found by a panic in `controlled_body_mut`, which is
  the kind of defect a state migration produces and a review does not.
- **2026-08-01:** nothing exempts the played critter from the ecology's
  `stage != Spent` retention, so it can die. Correct rather than a bug, since
  an exemption would branch on who is playing, but it means embodiment is now
  an `Option` and every acting intent needs a disembodied refusal.
- **2026-08-01:** `Organism::mass_mg` and `Organism::body.total_mass_mg()` are
  two accounts of the same quantity. P1 kept both on purpose; reconciling them
  means routing grazing and upkeep through parts, which is P2's work and is
  also what makes a large body cost something.

- **2026-08-01:** P0's first cut typed the editor path as `Route::Place`, a
  destination beside `Burn`. That conflated *where a meal goes* with *how a
  part finds its site*. Corrected to `Route::Incorporate { placement }`, so
  burn, grow, and the later destinations stay one axis and placement is a
  policy of growing.
- **2026-08-01:** P0 subtracted venom before adding burned mass, so a nearly
  starved critter lost part of a toxin to the zero floor and then collected the
  full meal. Approaching death was a discount on poison, and the original test
  enshrined the ordering. Gains now land before costs. The floor itself
  remains: energy is unsigned, so venom beyond what a critter holds is forgiven
  rather than owed, and a debt or damage model is a later decision.
- **2026-08-01:** P0 removed the organism and charged its venom before the
  attachment was known to succeed, and dropped the roster restore the previous
  code had. Unreachable with today's `attach`, and reachable as soon as graft
  compatibility exists. Now one transaction: everything that can refuse is
  checked first, a failed landing puts the meal back, and the ledger moves only
  once the meal has landed.

- **2026-07-31:** the automatic incorporation path skipped the venom
  subtraction the explicit path paid, so the default verb was strictly safer
  than the editor one. Fixed in P0; it had been the reverse of the intent.
- **2026-07-31:** incorporation removed the organism *before* resolving
  placement and re-inserted it on failure. Correct in outcome, but it meant a
  refusal briefly mutated the roster. P0 resolves placement first.

- **2026-07-31:** `BodyDocument` can descend, measure depth, tombstone a
  severed subtree, and derive reach. Gameplay still uses fixed reach.
- **2026-07-31:** `Origin::Incorporated { from_species, from_part }` does not
  uniquely identify a source individual or body revision. It is sufficient for
  the v0 proof and insufficient for branch provenance.
- **2026-07-31:** the body projection and chronicle duplicate a flat list of
  part origins. Neither carries the current anatomy tree.

## Progress

- **2026-09-02, P4 partial (P4a, the lineage program): a line commits a
  program, and its descendants are born under it.** `mesocosm-core::program`
  holds a versioned, append-only development program on each `Species`:
  immutable `Revision`s that declare *sites* (part role, admitted
  `ProcessRef`, bounded cells) and cite the discovery they were committed
  against. `Intent::Revise` is the played door over one `World::revise`
  transaction; `World::express_filially` develops each newborn under its line's
  current revision through the one validator, charging the child's own reserve
  into the ground under it; `Species::preview` realizes a founder from declared
  inputs alone. Rich versus lean ground grows one program (same digest) into
  two phenotypes (different digests), which is 2026-08-03's ruling made
  executable. **What is not built is what needs a ruling**: no scorer, no epoch
  trigger, no plasticity multiplier, no review screen — see the P4 gate for
  which clause each blocks and §D4 for the deletion receipt (conditions 1, 2
  and 4 met; `epoch::Trait` untouched). The ProcessDef plan's 2026-09-02 PD5
  entry carries the shapes and the full receipts.

- **2026-09-01, P3 landed: a branch changes bodies.** `Intent::Graft` harvests
  a living subtree off a carcass, remaps its ids, preserves its joints and its
  per-part source addresses, and lowers its allocation through the one
  validator. The crossing is the wing contract's carry-or-regrow; the world's
  affinity table returns the ProcessDef plan's three verdicts and they decide
  which crossings are feasible. See the P3 gate above for the landed shape, the
  two findings, the price derivation and the receipts. `crates/mesocosm-core`
  lib **354** green (+8: the affinity table's own claims, and a fork
  inheriting what its parent is made of), `tests/embodied`
  **49** (+8), the standing gates `matter/flows/succession/embodied`
  **6 + 11 + 7 + 49**, `mesocosm-views` **22** (+2), `mesocosm-genet` lib
  **18** (+1), workspace green, clippy and fmt clean.

- **2026-09-01:** recorded the admission gate for sub-part voxel mutation,
  immutable-volume versus per-body-patch alternatives, revision-safe async
  projections, truthful fallback requirement, and morphology-independent
  animation boundary. Proof order is unchanged; documentation only.

- **2026-08-03:** confirmed that direct and automatic adaptation arrange a
  founder preview while committing a heritable developmental program. The
  axial recipe and `BodyPlan` are the first implemented pieces of that program;
  world-conditioned phenotype realization remains P4.
- **2026-08-01:** `ProcessDef` accepted for one namespaced transformation. The
  linked ProcessDef plan now owns extensible definitions, expression, packs,
  and Piccolo while this plan retains body and capability semantics. No
  implementation added by that planning pass.
- **2026-07-31:** plan revised after the anatomy implementation and wing-scope
  audit. No phenotype implementation was added.

---

## The axial generator (built 2026-08-03)

Bodies were being sculpted, one creature at a time with no relatives. Ruled
and built: body plans come from an **axial recipe** read head to tail, in
`mesocosm-core::axis`.

Four rules, which are biology's own: **segments** repeated along one axis
(metamerism); **serial homology**, every segment carrying the same appendage
machinery; **tagmata**, segments grouped into named stretches; and **regional
identity**, a stretch deciding what its segments bear or suppressing them.
Symmetry, already in `BodyPlan`, is the fifth.

**It composes rather than replaces** (Mark, 2026-08-03). `BodyPlan` stays the
placement policy that growth consumes and adaptation edits; the axial recipe
is the scaffold above it. The scaffold says a thoracic segment bears a limb;
the policy says which way a limb points and whether it mirrors.

### The catalogue is the test

`axis::catalogue` builds centipede, millipede, insect, spider, tetrapod, and
snake from the same four rules, and the tests assert the relationships rather
than the shapes: a millipede is a centipede with `per_segment` set to two; a
snake is a tetrapod with its girdles suppressed and its trunk lengthened, same
number of stretches; dividing a one-stretch worm is how tagmatisation arises.
**Homeosis is one field**: `assign` is *antennapedia*, the Hox mutant that
grows legs where feelers belong.

Rendered proof at `testing/mesocosm/18_plan_*.png`, five plans through one
renderer with nothing per-creature in it: the centipede walks legs the whole
way down, the insect clusters them on its thorax and trails a bare abdomen,
the snake has none.

### Acquisition is Hox-like (the kleptoplasty ruling)

A lineage carries a **lexicon** of appendage kinds and cannot express one it
has never eaten. `acquire` returns whether the kind was new, so a meal is a
discovery rather than a calorie, and `assign` refuses anything unspeakable.
Only `None` and `Mouth` are innate.

This makes incorporation *developmental*: eating something teaches your line a
word, and the plan decides where to say it. A grafted part is not bolted onto
a body; it enters the recipe.

### Complexity counts kinds, not length

The frontier's new axis (agreed 2026-08-03): **repetition is cheap,
vocabulary and regionalisation are expensive.** Serial homology means each
extra identical segment adds almost no information, so segments carry a
repetition discount, stretches carry more, and distinct appendage kinds carry
most. A hundred-segment worm is long; a short creature expressing five kinds
across five stretches is intricate. Tested: an insect out-scores both a
longer centipede and a much longer snake.

### Wired into the world (same day)

All four gaps closed, and one of them changed a rule:

- **Worldgen seeds a recipe per founding lineage**, from its own salted
  stream so the ecology's draws are untouched. Producers stay simple;
  anything that moves gets stretches and limbs. A seeded world now holds
  several body plans rather than one shape at several sizes.
- **Eating teaches a word.** `World::learn_from` runs after a meal lands: the
  eater's line acquires every non-innate appendage kind the eaten line could
  grow, and a *new* kind records an `Event::Learned`. A word already known is
  just food, which is what makes the first one a discovery.
- **A fork inherits its parent's recipe**, vocabulary included. A founder
  does not forget what its line had learned.
- **The frontier reads recipe intricacy**, not part count.

**The rule that changed**: growing a body no longer raises the frontier,
because bulk is not intricacy. That threatened to disconnect the frontier
from play entirely, so the lexicon counts toward complexity at half weight:
a line that *could* grow five kinds has come further than one that could grow
one. The loop is now **eat something new, learn a word, the ceiling lifts**,
which ties acquisition directly to the thing it should gate.

One ordering bug found on the way: a lesson was landing in history before the
meal that taught it, because resolution records its own consequences before
`apply` records the act. The act is now inserted at the boundary where
resolution began, so an act always precedes what it caused.

### Still open

- **The live authority join landed 2026-08-05.**
  `mesocosm-core::development` consumes `Appendage::role` and develops
  `Recipe + Soma` into a mass-conserving `BodyDocument`. `World::new`, ecology
  offspring, `Chronicle::found`, and the founder-preview seam now use that same
  developer. The world snapshots its local `PartPalette`; each organism stores
  its realization seed. Under-provisioned births wait rather than dropping
  anatomy or manufacturing mass. ProcessDef allocation is now the next body
  migration.
- NPC lineages never mutate their recipes; only the player's acquires.

### Design ruling after the generator

- **2026-08-03:** the adaptation editor previews a founder phenotype but
  commits changes to the axial recipe, `BodyPlan`, process-expression rules,
  and later developmental instructions as one heritable program. This confirms
  the generator's recipe boundary rather than turning its first rendered body
  into a lineage template.
- **2026-08-05:** `Recipe + Soma` now has one core path to authoritative
  anatomy, and the V2 menagerie consumes it. This closes the parallel renderer
  body without pretending the world lifecycle migration is already done.
- **2026-08-05:** the world lifecycle now consumes that path too. Founders and
  offspring realize their lineage recipe under the world's snapshotted
  palette, returned chronicles regrow local topology through it, and founder
  previews call the same `Species::realize` function. The migration also made
  multi-part mass spending real by folding costs across every living part.
