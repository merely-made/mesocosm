# The epoch boundary: significance, speciation, and what youth costs

**Status: design plan, 2026-08-01. Nothing here is built.** Rulings are Mark's
from the dialogue of 2026-08-01 and are marked where the reasoning is mine.

This owns what happens *between* epochs: how a run is judged, how a lineage
splits, what a player may aim at, and why a young critter is different from an
old one. The [phenotype plan](2026-07-31_phenotype_plan.md) owns bodies and
capability. The [process plan](2026-08-01_processdef_plan.md) owns the process
vocabulary. The [founding plan](2026-07-30_mesocosm_founding_plan.md) owns the
epoch loop's turn structure, which this sits on top of.

---

## 1. The finding that makes this urgent

**Speciation does not exist.** Verified 2026-08-01 in
`crates/mesocosm-core/src/organism/ecology.rs`: reproduction is
`species: parent.species`, and no `SpeciesId` is ever assigned outside world
seeding. Lineages never split. No new species is ever born in a Mesocosm world.

Three things quietly assume otherwise: the complexity frontier (which gates
switching *between* lineages), multiple-lineage play, and any notion of
distance between two creatures' ancestries. The founding plan's open question 2
named speciation, hybridisation, and drift as three tangled mechanisms needing
separate rules. None are built, and this plan rules the first.

---

## 2. Speciation is an act, not a threshold

**Ruled by Mark.** A lineage splits when something *happens*, not when a
similarity metric crosses a line.

- **For the player: naming.** You fork a line and name it. That is the act.
- **For everything else: a significant event.** An unplayed lineage splits when
  it does something the world has not seen.

Note what this rhymes with, because it is the same rule one level up: **a borg
is a named critter.** Naming promotes an individual out of being a statistic;
naming promotes a line out of being a variation. One mechanic, two scales.

The alternative most prior art uses is a divergence threshold: Thrive
auto-speciates when a population's traits drift far enough. That produces
species nobody noticed being born. Dwarf Fortress never speciates at all and its
creature types are eternal. An act-based rule gives the player a moment, gives
the world a reason, and gives both a record of *why* this line exists.

### Speciation pays

**Ruled by Mark.** A significant event is not only a trigger, it is a reward,
and subsequent ones should pay more. The ladder, roughly:

- a boon to the lineage you are currently running;
- the right to fork, name the new line, and claim an ability or upgrade that
  distinguishes it;
- and more at the top, unspecified for now.

The important structural property is that **the reward and the split are the
same event**. A player is not choosing between "take the prize" and "found a
species"; founding one *is* the prize, and it costs the run nothing except
that the new line is now a thing you can be held to.

---

## 3. Significance is abnormality against the world's record

**Ruled by Mark.** Significance is judged **retrospectively, at the end of an
epoch, relative to what that world has already seen.** Not against a difficulty
table, not against an authored list of achievements. Against the record.

Scoring must therefore happen at the epoch boundary, because the boundary is
already where you form or fork a critter, and the score is what you form it
with.

### The axes

Mark's list, recorded as given:

| Axis | Question |
| ---- | -------- |
| categorical | *what* was done |
| locality | *where*, and how confined |
| magnitude | worldwide, regional, or local |
| harmony | how much of it was cooperative |
| domination | how much of it was at somebody's expense |
| exploration | how much of the world it touched |
| biomass | percentage gained or lost, and by whom |

Harmony and domination sitting beside each other is the interesting part: they
are not opposite ends of one axis, so a run can score high on both. That is a
lineage that made itself indispensable and dangerous at once, which is a real
ecological posture and not a contradiction.

### Two consequences worth naming, and they are mine rather than Mark's

**The world gets harder to impress, and nobody has to author that.** If
significance is abnormality against the record, then the more a world has seen,
the less any given act stands out. That is a difficulty curve for free, and it
is why an old world should feel different from a young one rather than merely
having more stuff in it.

**Which gives the storyteller a much better brief than "make things happen."**
A world that has seen everything has no significance left to offer, so the
storyteller's actual job is **to keep significance achievable** by opening
possibility the world did not have before. Oxygenation permits metabolisms that
were impossible; a glaciation makes cold-tolerance newly remarkable. Pressure is
not there to hurt the player, it is there to make the record incomplete again.

That reframing also decides a question the founding plan left open: prefer
**endogenous** transitions, because they are the world making its own record
incomplete. Exogenous disturbance is the tool for when it stops doing so.

---

## 4. Goals are the half you can aim at

**Ruled by Mark.** Between epochs, the player picks from roughly five randomly
generated goals. Each is a **guaranteed special event, verified and rated
against the world record** — so a goal might be to do something no species on
that planet has managed, chosen because your lineage is suited to stretch for
it.

### The tension this creates is deliberate

Retroactive abnormality cannot be aimed at, by construction: you find out what
was remarkable after you did it. Selected goals can be aimed at, by
construction: that is what selecting one means.

These pull opposite ways and **both should stay**. They are two reward channels
with different characters: one rewards discovery and cannot be farmed, the
other rewards intention and can be planned around. A future maintainer will be
tempted to collapse them into one, and this section exists to say do not.

### What "verified" costs

A generator that guarantees a goal is achievable, novel against the record, and
suited to the current lineage is performing a search over world state, not
drawing from a table. That is the expensive part of this plan and it wants
bounding before it is built: a goal that turns out to be impossible is worse
than no goal at all, and a goal that is trivially achievable is worse than
either.

---

## 5. Temperament: plasticity is a life stage, and youth pays for it

**Ruled by Mark**, resolving a question I had posed as either/or. Creatures
change most at the beginning and keep changing throughout, so temperament is
neither fixed at imprinting nor uniformly fluid. It is a **life-stage
conditional: more ability to develop, at the cost of the debuffs and
vulnerabilities of youth.**

That is a real trade rather than a curve someone tuned. A juvenile is malleable
and weak; a mature creature is capable and increasingly set. Which means:

- **Plasticity is a resource that depletes**, and spending an epoch young is
  spending capability to buy change.
- **It gives the life stages mechanical teeth** they did not have. `Stage` is
  currently `Juvenile | Mature | Carrion | Spent` and juvenile means little
  beyond not yet reproducing.
- **It interacts with population-as-lives.** Choosing to inhabit a juvenile is
  choosing a fragile body that can still become something; choosing a mature one
  is choosing a capable body that mostly cannot.

Temperament itself stays as ruled on 2026-08-01: **biographical, formed from a
heritable substrate, shaped by environment.** Genetics give you the gland; the
gland does not give you the temper; what happened to you, given that gland,
does. It is therefore a fold over substrate and history, exactly as capability
is a fold over anatomy, and it belongs on the borg layer rather than the
biological one.

---

## 6. What this needs that does not exist

All four verified absent on 2026-08-01.

| Missing | Needed by | Note |
| ------- | --------- | ---- |
| **An event log** | significance, the world record, world events | `apply_all` returns outcomes and drops them; the runtime keeps one tick. |
| **A species tree** | speciation, ancestral distance | Reproduction copies `species` verbatim; there is no parentage record anywhere. |
| **A world record** | abnormality, goal verification | A derived index over the event log: what has been done, by whom, at what scale. |
| **A place graph** | locality and magnitude | `CROWD_CELL` is an 8-voxel grid for density, not named regions. Worldwide, regional, and local need somewhere to be. |

Two of these have homes already suggested elsewhere. **The event log's growth
problem is what tulpa was invented for**: codicil holds everything, tulpa holds
the retold subset, and "significant world events" and "what memory keeps" turn
out to be one selector at two scales. And **the place graph is the same
place-graph granularity the wing already uses for shared space**, so locality
scoring and cross-vessel space want the same structure.

In-world biological descent is explicitly **not** fili, which is world-lineage
across forks and grafts. The species tree needs its own home and does not have
one. Build it beside `chartulary::stemma` rather than on it, per the standing
rule.

---

## 7. Stop rules

- Do not speciate on a similarity threshold. Splitting is an act with a record
  of why.
- Do not author a significance table. Significance is measured against the
  world's own record or it is not significance.
- Do not collapse retroactive abnormality and selected goals into one system.
- Do not let the storyteller pick outcomes. It picks the insult; the world
  decides the consequence.
- Do not offer a goal that has not been verified achievable and novel.
- Do not make plasticity free. Youth buys change with capability.
- Do not store a temperament. It is a fold over substrate and history.
- Do not grow the event log without a selector; that is the tulpa mechanic and
  it is load-bearing rather than decorative.

---

## 8. Open questions

1. **What is the smallest world record that supports abnormality?** A full
   event log is the expensive answer. A per-axis set of high-water marks might
   be enough, and would make "no species has done this" a cheap comparison.
2. **Does an unplayed lineage's speciation get a name?** If naming is the act,
   something must name NPC forks, and a procedural name is a different kind of
   fact from a player's.
3. **Can a goal be failed, or only unmet?** Failure implies stakes and a
   record; unmet implies it simply expires.
4. **Does plasticity apply to anatomy, temperament, or both?** The life-stage
   ruling was made about temperament, but the same trade would work for
   developmental change, and P4's adaptation bridge is where that decides.
5. **How does ancestral distance survive a world boundary?** A chronicle
   carries part provenance by species id; two worlds' species ids mean nothing
   to each other, so cross-world distance needs the wing contract's subject
   identity rather than a local id.

---

## Findings

- **2026-08-01:** speciation does not exist; `species: parent.species` and no
  other assignment outside seeding. Lineages cannot split, so ancestral
  distance is zero for every pair and no new species is ever born.
- **2026-08-01:** there is no accumulating event log. `World::apply_all`
  returns `Vec<Outcome>` and drops it; `mesocosm-runtime` keeps one tick's
  worth in `last`. Every proposal in this plan that reads history needs one.
- **2026-08-01:** worlds have no places. `CROWD_CELL` is a density grid, so
  locality and magnitude have nothing to be measured against yet.

## Progress

- **2026-08-01:** speciation-by-act, significance-as-abnormality, the scoring
  axes, the reward ladder, goal selection, and life-stage plasticity recorded
  from dialogue. No implementation added.
