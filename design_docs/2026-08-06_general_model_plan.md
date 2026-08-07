# The General Model (2026-08-06)

**Status: research and founding, 2026-08-06.** The ecological half is a
scheduled change with gates. The fantastical half is a **proposed shape,
not an adoption**: nothing in §6-§9 is scheduled, and F-gates exist to be
argued with. Sibling to the
[place-graph engine plan](2026-08-05_place_graph_engine_plan.md), which
owns world substrate, and the
[mesocosm founding plan](2026-07-30_mesocosm_founding_plan.md), which owns
the epoch loop.

**Wing-level notice.** §5-§9 describe machinery all three vessels would
share (Mesocosm discovers, Paredros embodies, Isometry exploits). Per
`CLAUDE.md`, wing material lives once. Siblings cite this file; they do
not copy it.

---

## 0. Why one document

Two questions arrived separately and turned out to be one question.

The first was Mark's: **shouldn't the priority be to shape the world and
its conditions so that behaviour just emerges?** The second was whether
the Madingley model, and a fantastical counterpart to it, are the right
lodestars.

They join at a single observation. Madingley's spine is *conserved biomass
flowing between pools via processes, at rates set by traits*. Strip the
word "biomass" and what remains is a **currency**, a set of **processes**
that move it, and **derivation rules** from traits to rates. A fantastical
layer is not a second engine bolted alongside that one. It is the same
machinery with different currencies, different flow constraints, and
different derivation rules. Which means the honest structure is one model
with settings, and the ecology is its first configuration.

That structure is also the answer to configurability (Mark, 2026-08-06:
the fantastical layer should be "potentially more than that and also
configurable"). A world's metaphysics is not a content pack. It is a point
in the model's own parameter space.

---

## 1. Prior-art ledger, verified 2026-08-06

Terminology first, because naming what exists tells us what does not.

| Term | Status | What it actually names |
| --- | --- | --- |
| **General Ecosystem Model (GEM)** | Established | Madingley's category. "General" = one set of ecological concepts applied to any ecosystem, terrestrial or marine, at any resolution. Madingley is described as the first process-based mechanistic GEM. |
| **Speculative evolution** | Established community term (Dougal Dixon) | The creature and ecology half, non-magical. |
| **Computational modeling of religion** | Real academic field | Agent-based modelling of belief, practice, demography. Center for Mind and Culture's Modeling Religion Project; Wildman, *Modeling Religion*. The genuine analogue on the belief side. |
| **Thaumatology** | Real word; GURPS uses it as a book title | The *study of magic systems*. The GURPS volume is explicitly a meta-book of alternative frameworks (spell-based, ceremonial, spirit-mediated, runic, freeform, material). Closest existing English word for the discipline. |
| "magical ecology" | TTRPG/worldbuilding usage, one published supplement | Content-generation advice. No formal semantics. |
| "thaumodynamics", "arcanology" | Hobbyist worldbuilding only | In-fiction disciplines invented per setting. |
| Mauss & Hubert, *A General Theory of Magic* (1902) | Real, literally titled "general theory" | Anthropological theory of magic as total social phenomenon. A theory, not a model class, and argues *against* Frazer's reduction. |

**The finding: the GEM-shaped slot is empty.** No established term names a
general, mechanistic model class for fantastical ecology or metaphysics.
Also absent, and searched for rather than skipped: any academic subfield
for procedural generation of magic systems, and any GDC talk specifically
about simulating supernatural or belief systems.

Per `CLAUDE.md`, **no name is coined here.** Recording the empty slot is
the finding. A naming round with the usual crates.io, game, studio, and
trademark checks happens separately, if ever.

### 1.1 Sources that carry weight

- **Madingley**: Purves et al. 2013; Harfoot et al. 2014, *Emergent Global
  Patterns of Ecosystem Structure and Function from a Mechanistic General
  Ecosystem Model*, PLOS Biology.
- **Effect systems**: Dwarf Fortress syndromes and spheres (wiki, verified).
- **Procedural history**: Grinblat & Bucklew, FDG 2017 PCG Workshop
  (Caves of Qud); Grinblat, "Generating Histories" in *Procedural
  Storytelling in Game Design* (2019).
- **Knowledge propagation**: James Ryan, Talk of the Town / Hennepin;
  "Simulating Character Knowledge Phenomena in Talk of the Town",
  *Game AI Pro 3* (2017). The only worked implementation found of belief
  as a *distorted propagating representation* rather than a fact.
- **Constraint design**: Sanderson's First and Second Laws (primary
  source). Second Law decomposes into three separable levers:
  *limitations* (what it cannot do), *weaknesses* (what it exposes you
  to), *costs* (what it consumes).
- **Historical schemas as structural templates**: Frazer's two laws of
  sympathetic magic; wuxing (five nodes, two directed cycles, plus a
  correspondence join table); humoral theory (a 2-axis product space);
  Paracelsian doctrine of signatures; hermetic correspondences.

### 1.2 The cautionary source

Raph Koster's own account of Ultima Online: the simulated ecology was cut
in beta for **performance and maintainer comprehension**, after being
rewritten by an engineer who did not understand it. What died was the
behavioural half. The static resource data survived and still feeds
crafting today.

Two lessons, both binding on this document:

> **Simulated layers die to performance and to comprehension before they
> die to players.** And **the data half can survive when the dynamics half
> is throttled.**

Which argues for inspectable state (fields, syndromes, currencies) over
opaque dynamics (behaviour engines), and for building every layer so its
state stays legible and attributable even if its dynamics are cut.

---

## 2. Madingley, mechanically

- **Cohorts, not individuals.** A cohort is organisms sharing functional
  group and continuous traits within a cell, carrying three state
  variables: abundance, body mass per individual, reproductive mass. When
  cohort count exceeds a threshold, the nearest pair in trait space merges
  and biomass is conserved across the merge.
- **Autotrophs are stocks, not cohorts.** Deliberately: "individual" is
  ill-defined for a plant, and marine turnover outruns the timestep.
- **Six processes per timestep, randomised order**: metabolism, feeding,
  growth, reproduction, mortality (predation, starvation, background,
  senescence), dispersal (diffusion, starvation-driven, currents).
- **Categorical traits are minimal** (feeding mode; endo/ectotherm).
  **Continuous traits, chiefly body mass, drive every rate**, through
  allometric power laws, across 10 mg to 150,000 kg.
- **The payoff**: biomass pyramids, trophic structure along productivity
  gradients, latitudinal carnivore ratios, body-mass/density relations,
  all **emergent and unfitted**, from individual-level processes alone.

That last line is the published receipt for "shape the conditions and let
behaviour emerge."

---

## 3. Where Mesocosm already stands

Checked against `organism/ecology.rs`, 2026-08-06.

| Madingley process | Mesocosm |
| --- | --- |
| Metabolism | Present. `pay_upkeep`, budget first then body. |
| Autotrophy | Present. Producers fix, crowding shades income. |
| Herbivory | Present. Consumers graze producers in range. |
| Decomposition | Present, and explicit rather than folded into stocks. |
| Growth | Present. `gain_mass`, juvenile stage. |
| Reproduction | Present. Gestation, offspring costs a share of parent mass. |
| Mortality | Present. Starvation, senescence, predation-as-carrion. |
| **Predation** | **Absent.** `Kingdom::Consumer` eats only `Kingdom::Producer`. Nothing eats a consumer. |
| **Dispersal** | **Absent.** Birth scatter only; nothing moves. |

Mesocosm additionally has something Madingley does not: **`Signal`**, an
advertised claim that can be false. Choosing a meal is already a judgment
rather than a lookup.

So the ecological work is **two missing processes and one changed
principle**, not a rewrite.

### 3.1 The changed principle

Madingley's real lesson is not the process list. It is *categorical
minimal, continuous drives rates*. Mesocosm currently inverts this:
`Kingdom` is assigned at genesis and does the work, while rates are flat
constants (`MATURITY: 90`, `LIFESPAN: 600`, `GESTATION: 120`,
`GRAZES_MG: 4`, `DECAYS_MG: 3`, `FIXES_MG: 3`). A shrew and an elephant
mature on the same schedule.

### 3.2 The concrete bug

`upkeep_mg = UPKEEP_MG + biomass_mg / UPKEEP_SHARE` is **linear in mass**.
Real metabolism scales as roughly mass^0.75 (Kleiber). Linear upkeep
over-taxes large bodies at exactly the rate that makes large bodies
unviable, so the world carries a **size ceiling nobody chose**. This is
worth fixing on its own merits, independently of everything else here.

### 3.3 What emergence buys, concretely

Point the existing pieces at each other and roles stop being categories:

- **Feeding mode derives from anatomy.** The axial generator already
  produces mouths, reach, plates, speed. A body that can take living prey
  *is* a predator; nobody assigns a carnivore tag. `Kingdom` shrinks from
  a decree to a derived reading.
- **Life history derives from mass.** Maturity, lifespan, gestation,
  dispersal range, feeding rate as power laws. Large becomes slow,
  long-lived, wide-ranging, hungry but efficient, with nothing authored.
- **The `Hunter` dissolves.** Pursuit is what a fast, large-mouthed,
  starving thing does about a reachable meal. See §10.

### 3.4 Cohorts are the far tier's natural unit

The place-graph plan's two-tier simulation wants exactly Madingley's
representation: individuals near the played body, **cohorts** at graph
distance, with trait-space merging as demotion and splitting as promotion.
`TierLine` already has the shape; the cohort gives the far side the right
state.

---

## 4. Currencies as a design space

The generalisation that makes both halves one model.

A currency is a point in roughly eight dimensions:

| Axis | Values |
| --- | --- |
| **Conservation** | conserved; sourced (enters from outside); sunk (leaves); both |
| **Transfer cost** | full (giver loses what receiver gains); partial (conversion loss); none (copying) |
| **Locus** | entity; place; edge (kinship, provenance, contact); global |
| **Autonomous change** | persists; decays; regenerates; oscillates |
| **Rivalry** | rival; non-rival |
| **Transmutability** | which other currencies, at what loss |
| **Flow constraint** | which edges permit flow: trophic, contact, sight, descent, provenance |
| **Observability** | visible; inferable; hidden |

Worked points:

- **Biomass**: conserved, full-cost transfer, entity-located, decaying via
  upkeep, rival, weakly transmutable, flows along trophic and contact
  edges, partly observable through size. *The ecology is this point.*
- **A curse**: non-rival, no-cost transfer, edge-located along provenance
  or descent, persistent, hidden until it fires.
- **Ambient power**: sourced, place-located, regenerating, rival,
  observable, flows by proximity.

**Not every combination is coherent**, which is DF's spheres lesson
applied here: the generator needs an **exclusion relation** over axis
combinations, the same way a deity may not hold precluded spheres. Without
it, sampling produces incoherent worlds rather than varied ones.

**Configuration is therefore literal.** A world's metaphysics is the set
of currencies it carries and their axis values. That is a settings
surface, in the standing spirit of configurability over opinionated
defaults, and it is trackable as a design space rather than a content
list.

---

## 5. One effect system, four channels

Dwarf Fortress's syndromes are the strongest single mechanism found:
alcohol, snake venom, plague, vampirism, mummy curses, and werebeast
infection are *the same object*, applied through typed channels (contact,
ingested, inhaled, injected), carrying effect sets. Even lycanthropy's
supernatural part is a **trigger predicate over world state**, not a
special case.

The rule this implies, and it is the don't-duplicate ruling again:

> **Do not build a magic-effect type.** Build one effect-application type
> with typed channels, and let disease, venom, weather, blessing, and
> curse all emit it. Effects write into **currencies**, so a world with
> more currencies than biomass gets a richer effect space for free.

---

## 6. Derivation rules (proposed)

How generated bodies and places acquire fantastical properties without
anyone authoring them. All three have historical schemas behind them and,
usefully, existing hooks here.

- **Signatures** (Paracelsus): form indicates function. A rule for reading
  properties *off* a generated morphology. Hook: the axial generator.
- **Similarity** (Frazer's first law): like produces like. Formally, a
  distance metric over trait vectors. Hook: recipes and somas are already
  trait vectors.
- **Contagion** (Frazer's second law): things once in contact continue to
  act at a distance. Formally, an edge in a provenance graph. Hook:
  **every incorporated part already carries `Provenance`.**

The alignment is lucky rather than clever, and it means sympathetic magic
is nearly free here.

### 6.1 Kleptoplasty past biology

The wing's acquisition metaphor already reaches: kleptoplasty is keeping
working machinery from what you eat. Extending it past biological traits
needs **no new intent**; metabolize stays the one verb.

It needs **criteria**, which is Sanderson's Second Law made mechanical:

- a part structurally capable of hosting the property (capability, not
  inventory);
- a source in a state that releases it;
- finite capacity, so acquisition trades against acquisition.

This is `ProcessDef` and phenotype work, not new machinery.

### 6.2 The skeleton: identity, facts, derivation, projection (2026-08-07)

The entity-model question ("classic ECS, or something else?") resolves
into four layers the stack has been converging on separately. The prompt
that named it was [PolyCSS](https://github.com/LayoutitStudio/polycss), a
CSS 3D engine rendering VOX/glTF meshes as real DOM elements, one per
polygon, each individually addressable and styled by rules. Its world-lane
technique does not transfer (per-polygon DOM at simulation scale is
cardinality death, and the tracer lane already exists). Its *architecture*
names our middle layer.

| Layer | What it is | Already standing |
| --- | --- | --- |
| **Identity** | Stable ids | `OrganismId`, `PartId`, `PlaceId`; chartulary Container platform-side |
| **Facts** | Facets as plain serialized state: mass, traits, currencies. Ordered, hashed, replayable | the core |
| **Derivation** | A small, strictly ordered, inspectable rule cascade computing everything downstream | grade blocks; E0 allometry; §6 rules; V2's dependency digests as the invalidation |
| **Projection** | Per-vessel lenses reading computed values, every emitted element carrying source identity | genet-probe doctrine; `BodyLensProjection` sidecars |

The correspondence that makes "derivation" a *styling* layer, closing a
ruling made earlier ("the soul question is a styling matter"):

- signatures and similarity are **selectors over trait vectors**;
- world-default, then kingdom, then species, then individual override is
  **the cascade**;
- allometric rates, derived feeding modes, and fantastical properties are
  **computed values**;
- the grade is literally the stylesheet's visual half.

Prior art the stack already owns: **livery**, whose enumerable
TOML-property-database discipline is the tamed version of this. The
binding constraint comes from §1.2 and from CSS's own failure mode
(specificity wars): the cascade stays small, strictly ordered, and
attributable. "Why is this critter fast" must answer with a rule chain.

Not chosen: archetypal ECS. At thousands of near-tier individuals with
cohorts above (E3), determinism and snapshot-hashing are worth more than
iteration throughput, and facts-as-facets is what the replay contract
already is.

Two adjacencies recorded while here: for **Isometry**, PolyCSS is
near-literal prior art (Foundry-class scenes as identity-bearing DOM
elements styled by rules, with a VOX import path rhyming with the bake
pipeline). And the **sprite-stacking deferral has expired**: it was
parked pending a pulled-back camera, which Mesocosm now has (place-graph
plan §0.4); PolyCSS is structurally sprite stacking in the DOM.

---

## 7. Composition grammar (proposed)

Ars Magica's formulation is the canonical generative magic grammar: 5
Techniques x 10 Forms, plus requisites, plus per-combination magnitude
guidelines that let any point be *priced* without being authored.

The adaptation, sharper than adopting it whole:

> **Fix the Technique axis; generate the Form axis from the world's own
> ontology.** Create, perceive, transform, control, destroy are near
> universal verbs over anything. Forms should come from what the world
> actually contains: its materials, its kingdoms, its currencies.

Technique x Form is then a matrix, and per the precluded-pairs rule **a
given world fills only some cells**. Which cells exist is that world's
magical character. A closed-form cost function over the parameter vector
(Morrowind's spellmaker is the worked example; Angband's power budget is
the learned-distribution variant) keeps a generated space balanced without
hand tuning.

---

## 8. Discovery as the delivery vehicle (proposed)

If the laws differ per world, **learning them is the content**. NetHack
shuffles appearance-to-identity per game and deliberately supplies more
appearances than items, so elimination stays imperfect. Morrowind's
alchemy hides most ingredient effects and combines by intersection.
Ultima Ratio Regum makes religious identity *inferable from observable
behaviour* rather than readable from a panel.

This distributes across the wing without any vessel converting anything:
**Mesocosm discovers, Paredros embodies** (transformations, pacts, curses
as syndromes), **Isometry exploits** (auras and fields as tactical
terrain).

The cheapest first version is worth building before anything ambitious: a
small fixed-width effect vector per organism, combination by intersection.
It turns the existing procedural ecology into a magic-materials economy at
nearly no cost.

---

## 9. Fields (proposed)

The most implementable "magic as simulated quantity" model found is,
unexpectedly, Genshin's elemental gauge theory: application in gauge
units, an aura tax on creation, linear decay inversely proportional to
base duration, consumption on reaction, per-source internal cooldowns. It
is deterministic and integer-friendly, which matters here more than it
matters there. Divinity: Original Sin 2 is the same idea expressed
spatially, as persistent surface state with combination rules.

For a place graph over voxel ground the natural form is a **small vector
of channels per place**, with units, decay rate, and reactions, read by
spawn rules, terrain, and the ecology's mortality terms. Vintage Story's
temporal stability and Black & White's belief-generated influence radius
are the same shape at world scale.

Breath of the Wild supplies the law that keeps such a table from
exploding: elements act on materials, materials do not act on materials.
The asymmetry is what makes a small rule set multiply rather than square.

---

## 10. Gates

### Ecological, scheduled

**E0. Allometry, and the size ceiling.** Replace flat rate constants with
mass-derived rates; fix linear upkeep to a ^0.75-shaped law in integer
arithmetic.
**Done when:** a world sustains bodies across at least three orders of
magnitude of mass; maturity, lifespan, gestation, and feeding rate all
vary with mass; existing ecology receipts are re-greened rather than
deleted; no rate constant remains that should have been a function of mass.

**E1. Predation, and feeding mode from anatomy.** Consumers may take
living consumers. `Kingdom` becomes a derived reading of anatomy rather
than a genesis assignment.
**Done when:** a lineage whose bodies acquire the relevant parts begins
taking live prey with nothing in the code naming it a predator; trophic
levels are countable in a run; and the reckoning can distinguish predation
from scavenging (it already can, by event order).

**E2. Dispersal.** Movement as a far-tier process: diffusion, plus
starvation-driven migration.
**Done when:** populations track productivity across the place graph over
an epoch, and a locally exhausted place is left rather than starved in.

**E3. Cohorts.** Far-tier state becomes cohorts with trait-space merging;
near tier stays individual. Promotion and demotion are cohort split and
merge.
**Done when:** biomass is conserved across every merge and split; far-tier
outcomes stay within existing receipts; and the population the far tier
can carry rises by an order of magnitude.

**E4. Drives replace the `Hunter`.** Behaviour selection scores reachable
affordances against need and body. The FSM demotes to a probe (see
place-graph plan G3) and leaves the tree.
**Done when:** the chase receipt still passes with no type named `Hunter`
in the path; a slow armoured starving body does something *different* from
a fast large-mouthed one under identical conditions; and a predator that
picks badly starves.

### Fantastical, proposed only

**Not scheduled. No adoption decision. Listed so the shape can be argued
with.**

- **F0. Currency registry.** Currencies as first-class configuration with
  the §4 axes, an exclusion relation, and biomass expressed as one point
  in it rather than a special case.
- **F1. One effect application type.** §5, with existing metabolic and
  mortality terms as its first targets.
- **F2. Derivation.** §6: signatures, similarity, contagion, over existing
  trait vectors and provenance edges. Kleptoplasty criteria.
- **F3. Discovery.** §8, cheapest version first.
- **F4. Fields.** §9, per place, integer.
- **F5. Grammar.** §7, only after F0-F2 exist to generate Forms *from*.

---

## 11. Stop rules

- **No second engine.** The fantastical layer is currencies, effects, and
  derivation rules over the existing machinery. If it grows its own
  simulation loop, it has become the thing the anti-Spore law forbids.
- **State legible, dynamics throttleable** (§1.2). Every layer must leave
  inspectable, attributable state behind if its dynamics get cut.
- **No name coined without a naming round.** §1's empty slot is a finding,
  not an invitation.
- **Do not model a plan on an unshipped system.** Dwarf Fortress's full
  procedural magic does not ship; spheres, secrets, syndromes, and
  primordial remnants do. Cite what exists.
- **Sample constraints, not powers.** Sanderson's Second Law: a generator
  over powers produces noise, a generator over limitations, weaknesses,
  and costs produces character.
- **Generated rules must be discoverable** or generation has bought
  nothing a content pack would not have.
- Do not let effects, fields, or currencies become world truth in place of
  the integer authority. They are state *in* the world, not a second
  authority over it.

---

## Findings

- **2026-08-06:** the GEM-shaped slot for a general model class of
  fantastical ecology is **empty** (§1), verified rather than assumed.
  Adjacent named things exist: GEM, speculative evolution, computational
  modeling of religion, thaumatology.
- **2026-08-06:** Mesocosm's ecology already implements six of Madingley's
  eight process slots (§3). The gap is predation and dispersal.
- **2026-08-06:** `upkeep_mg` is linear in body mass where allometry says
  ^0.75, imposing an unchosen size ceiling (§3.2).
- **2026-08-06:** the wing already carries hooks for all three classical
  derivation rules: trait vectors for similarity, morphology for
  signatures, and `Provenance` edges for contagion (§6).
- **2026-08-07:** the entity skeleton is four layers, not an ECS:
  identity, facts, derivation-as-cascade, projection-with-identity
  (§6.2). PolyCSS supplied the naming prompt; livery supplies the tamed
  prior art; sprite stacking's pulled-back-camera deferral has expired.

### Verification debts

Carried honestly from the research sweep, to be settled before anything
in §6-§9 is built on the details:

- The Caves of Qud state-machine-plus-replacement-grammar description and
  the "rationalise cause and effect after the fact" framing come from
  search summaries and paper metadata; the FDG 2017 PDF itself was not
  extracted. Re-verify directly.
- **Dominions' province-level belief propagation was not verified** and
  looks like one of the stronger missing examples. Worth a dedicated look.
- Spore's part-capability property claim is forum-sourced only.
- Sanderson's Third Law was not read at the primary source.
- Bay 12 dev pages were only partially captured verbatim.
- Not investigated for budget: Populous, Dungeon Crawl Stone Soup's
  schools, Niche, Cataclysm: DDA's Magiclysm spell schema, Potion Craft's
  navigable 2D effect space.

## Progress

- **2026-08-06:** founded. Ecological half scheduled as E0-E4; fantastical
  half proposed as F0-F5 pending a ruling. Prior-art ledger and
  verification debts recorded. Research sweep covered ~20 games, the
  design-theory literature, and historical schemas; the Ultima Online
  postmortem supplied the binding caution.
- **2026-08-07:** §6.2 added: the four-layer skeleton and the
  styling-as-derivation correspondence, from the PolyCSS reading session.
