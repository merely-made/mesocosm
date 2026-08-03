# Phenotype: what a body is for

**Status: decisions and proof plan, revised 2026-08-03. Anatomy descent,
depth, severing, derived reach, and the axial developmental recipe are built;
the phenotype bridge described here is not.** This document owns Mesocosm's body rules. The
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

**Cost.** For a time the adaptation lab remains explicitly provisional. This
is preferable to deleting its only working vocabulary before the replacement
can express the same questions.

**Rules: Mark at deletion.** Adding the first developmental process field does
not itself authorize removing `epoch::Trait`.

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

### P3. Branch transfer

Harvest or receive a source subtree, remap its local ids, and preserve its
source addresses and parent relations.

**Done when:** the source loses the branch, the recipient gains a functioning
or visibly incompatible branch according to the chosen route, severing the
graft cascades, and snapshot/replay agree.

### P4. Adaptation bridge

Grow several candidate developmental changes and score their phenotypes in one
authored world.

**Done when:** a chosen mutation cites a lived scarcity, produces a visibly and
mechanically different descendant, commits a developmental program rather than
a body snapshot, and reproduces its founder preview under identical declared
inputs. A changed environment may realize a legibly different phenotype from
that same program; unplayed lineages use the same evaluator; and the old trait
array has a concrete deletion receipt.

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

- `Appendage::role` maps to the parts graph, but no growth path consumes it:
  bodies are still grown by incorporation rather than developed from the
  recipe. That is the next join, and it is where `Soma` stops being renderer
  input and becomes the organism's actual anatomy.
- NPC lineages never mutate their recipes; only the player's acquires.

### Design ruling after the generator

- **2026-08-03:** the adaptation editor previews a founder phenotype but
  commits changes to the axial recipe, `BodyPlan`, process-expression rules,
  and later developmental instructions as one heritable program. This confirms
  the generator's recipe boundary rather than turning its first rendered body
  into a lineage template.
