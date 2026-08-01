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

**The control pattern is probably not its own layer.** Each archetype above is
a *distribution of channel requirements across parts*, not a separate authored
system: a vertebrate is mostly `centralized`, an octopus is `local` limbs under
some `centralized` behaviour, a mycelium is `redundant` throughout, a plant is
`routed` with nothing `centralized`, a colony is `quorum`. Collapsing four
layers to three means the archetypes *emerge* instead of being enumerated,
which is fewer authored systems and better anti-Spore discipline.

**Every requirement kind needs to name the relation it is evaluated over, and
one of them does not have one yet.** `local` needs no graph. `routed`,
`redundant`, and `centralized` are path queries over the channel graph.
`quorum` is a count over a connected set. But **broadcast** is described as
diffusing through *neighbouring tissue*, and neighbouring is ambiguous: two
plates both attached to the root are spatially in contact and are not adjacent
in either the structural tree or, necessarily, the channel graph.

So broadcast either (a) means bounded `routed`, a path within a hop limit, or
(b) needs a **third relation, spatial contact**, computable from `world_pivot`
and `half_extent` but quadratic and genuinely distinct from the other two.
**Resolve as (a).** It costs nothing, it keeps the model at two relations plus
a fold, and mycelial resilience still works because that is `redundant` over a
cyclic channel graph rather than diffusion. True spatial diffusion can arrive
later if some phenotype actually needs it, and it should have to earn its way
in.

**Grafting is where catalog pressure will arrive.** The graft routes are
excellent fiction: pay for awkward adapters, assimilate the process into
host-grown anatomy, let it stay partly autonomous, cultivate a symbiont that
translates, or accidentally install a parasite that hijacks the network. But
"its cut boundary may not speak your body's language" needs a *type* on the
channel endpoint, and a rich endpoint type is a compatibility matrix, which is
a catalog wearing a lab coat. **Keep endpoint compatibility expressed in the
same tiny process vocabulary**, so a graft check is one set-membership test
rather than a table that grows with every part ever authored.

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

> **The shared flow vocabulary is a glossary, not an engine.** Two authorities,
> one set of words, no common evaluator. The day an ecology tick and a body
> evaluation call the same solver, the substrate has grown a second engine.

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

### P1. One organism model

One authored prey and the controlled critter both use the body-bearing organism
representation.

**Done when:** changing control between them changes neither serialization nor
ecology semantics, and the scene renderer discovers both through the same path.

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
- Do not give a channel requirement a relation it cannot name; resolve
  broadcast as bounded routing rather than adding a spatial-contact graph.
- Do not let graft compatibility grow an endpoint type richer than the
  process vocabulary; a compatibility matrix is a catalog.
- Do not let the body evaluator and the ecology tick share an evaluator.
  The flow vocabulary is a glossary, not an engine.

---

## Findings

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
