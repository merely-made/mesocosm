# Phenotype: what a body is for

**Status: decisions and proof plan, revised 2026-07-31. Anatomy descent,
depth, severing, and a first derived reach fold are built; the phenotype
bridge described here is not.** This document owns Mesocosm's body rules. The
cross-vessel boundary lives in the
[wing phenotype contract](2026-07-31_wing_phenotype_contract_plan.md), and
ordering remains with the
[execution waves plan](2026-07-31_execution_waves_plan.md).

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

### P2. One embodied consequence

Wire `BodyDocument::reach()` into actual interaction, then add one connected
process path.

**Done when:** two bodies have different reachable actions because of anatomy;
severing a dependency removes the action; no capability number was edited; and
the receipt and headed view state which embodied requirement failed.

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
mechanically different descendant, unplayed lineages use the same evaluator,
and the old trait array has a concrete deletion receipt.

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

**The rule contract is one model; the storage is two.** Rules must never ask
whether a subject is realized.

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

**Corrected 2026-08-01, before implementing.** An earlier draft of this list
also required a cohort round-trip. That is P5's done-condition, duplicated here
by mistake. §7.3 settles the cohort *rule contract*, which P1 satisfies by
having nothing branch on realized-ness; the second storage arrives when
scale demands it, and building an aggregation path with no consumer would be
the speculative generality this plan's own stop rules forbid.

### Not in P1

Damage to specific parts, prey that lose subtrees, phenotype-granted actions,
and branch transfer. Those are P2 and P3, and they are the *reason* for P1
rather than part of it.

## Findings

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

- **2026-07-31:** plan revised after the anatomy implementation and wing-scope
  audit. No phenotype implementation was added.
