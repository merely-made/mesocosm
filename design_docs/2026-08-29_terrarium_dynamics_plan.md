# Terrarium Dynamics Plan (2026-08-29)

**Status: in progress (2026-08-29); scope and rulings by Mark the same day.**
Make the terrarium compelling on its own — an ant farm worth watching — and
let the player's considerations step into that. Mark's words, ruling the
priority: "slices that shape the world and ecosystem dynamics... that's my
priority" and "let's make the terrarium/ant farm compelling on its own and
then let the player considerations step into that."

The diagnosis this plan answers (played slice plan, 2026-08-29 findings):
the first playtest's whole arc — budget gone in 3 seconds, death of old age
at 17, population 61 → 8,155 — was the ecology's tick-tuned life-history
constants driven at 60 ticks per wall second. In tick-space nothing was
broken; played wall-time mapped 60:1 onto it.

## The rulings (Mark, 2026-08-29)

- **Tempo: both, deliberately.** A canonical played tempo is picked and the
  life-history constants are retuned for feel at it. Working value proposed
  here: **10 ticks/second** (100ms input granularity; a starter lifetime
  lands at minutes once constants follow). `Runtime::new` already takes
  ticks-per-second, replay is per-intent, so pacing never touches the hash.
- **The hand: instincts under idleness.** A controlled critter obeys the
  keys while the player acts and reverts to its own drives when the player
  has been idle a while. The idle terrarium is a feature, not a failure —
  it is how the game's dynamics are observed.
- **Income: the body routes it.** A meal eaten while the budget is starved
  burns; eaten while provisioned it incorporates. Hunger decides — no menu,
  no second key. D5's burn-or-incorporate choice becomes a state you play.
- **Priority: world and ecosystem dynamics first.** Succession/epoch wiring
  (PS2) deliberately waits.
- **The bar (Mark, 2026-08-29): the RimWorld standard.** "We're trying to
  reach that rimworld bar: a composition of loops so compelling it could be
  called a story generator." Earlier the same day: "rimworld wouldn't make
  sense without the confluence of many loops and mechanics... it's hard to
  be surprised when there's nothing to learn." No single loop is the game;
  the game is loops interacting until the world produces events worth
  retelling. This bar judges every slice here: a mechanic earns its place
  by composing with the others into legible surprise, and a dynamic nobody
  can witness or remember generates nothing. The witnessing half — history
  surfaced, death seen, succession felt, the epoch retold — is where the
  loops become stories, which is why PS2 and the record-reading surfaces
  are story machinery, not plumbing.

## TD1 — the population instrument

Headless, before any tuning: run the ecology across seeds for several
lifespans and write per-kingdom population and biomass curves plus a
verdict (stabilizes within a band, oscillates, boils, collapses). The
baseline receipt documents today's boil at today's constants. Prove the
instrument before believing anything it says: a constant set known to
collapse must read as collapse.

**Done when:** a `cargo run` producible receipt (JSON curves + summary)
exists under `Code/testing/mesocosm/`, with a baseline at current constants
and at least one deliberately-broken control run proving the verdict can
fail.

## TD2 — tempo and the retune

Set the played host to the canonical tempo, then retune life-history
constants against TD1 until the terrarium breathes at watchable speed: a
starter lifetime of minutes, populations that rise and fall without boiling
or flatlining across the instrument's seeds. Headless labs keep their own
tick rates; the constants are the world's and change for everyone, which is
why TD1 comes first.

**Done when:** the instrument's receipt at the new constants shows breathing
(not boil, not collapse) over several lifespans across seeds; the played
host runs the canonical tempo; existing tests pass or are retuned with the
change documented.

## TD3 — see the ecology

The brick tracer takes exactly one pose, so the section shows one organism
in an enclosure holding dozens. An ant farm needs the ants: the lens grows
a pose roster — every organism in the slab window drawn, culled to the
section, with a documented cap chosen by uniform-size arithmetic rather
than hope. This is a mesocosm-lens change; the paredros room consumes the
same tracer, so the roster must not break the single-pose path it uses.

**Done when:** a headed capture shows multiple organisms in the section at
once; the cap and its arithmetic are documented; the replay receipt still
lands its hash; paredros-room still builds against the lens.

## TD4 — the hand and the meal

The two core-rule changes, after TD1 lands (they share files with nothing
above but sit in the same crate):

- Instincts under idleness: the world counts consecutive idle intents;
  the ecology drives a controlled critter's locomotion only past the idle
  threshold. A function of the trace, so replay is unaffected.
- Hunger-routed metabolize: the intent drops its route; the body burns when
  starved, incorporates when provisioned, and the threshold is a documented
  constant. The vitals surface already shows the state that decides.

**Done when:** holding a key moves the critter with no instinct fighting
the hand; walking away returns it to the ecology (visible in an idle run);
a starved critter's meal refills the budget on screen; replay receipts
land their hashes; the played slice plan's D5 note records that the choice
became diegetic.

## TD5 — one economy for all life (ruled 2026-08-29, Mark)

NPCs earn energy the way the player does. Today every non-played gain —
producer fixing, grazing, predation, decay — builds biomass only, and
`energy_mg` is a birth endowment that never refills (the TD2d finding), so
every hunger threshold trips early and decomposers cannot bank a corpse
against the gap to the next one. TD5 routes every organism's feeding
income through the body rule TD4 landed for the played meal: starved
(`budget_below(STARVED_UPKEEP_TICKS)`) → the gain credits the budget;
provisioned → it builds the body, as today. One predicate, one economy,
every kingdom. The `dispersal_for` zero-energy gate aligns to the same
hunger predicate while its ledger becomes real.

**Done when:** the instrument shows founded kingdoms persisting to the
horizon in a majority of seeds — `breathes` reached, zero boils, the
collapse control still collapsing; a death-cause probe shows decomposers
banking corpses and surviving the gaps; a mechanics-only receipt is
recorded before any constants follow-up, and any follow-up nudge is
documented constant by constant; fixtures re-record at the new economy.

## TD5b — the founding cohort arrives mid-life (ruled 2026-08-29, Mark)

TD5's own finding: the corpse drought is set at founding, not in `rates.rs`.
Founders stagger age over `rng.below(200)` against a lifespan in the
thousands, so the enclosure holds no real carrion until the founders
themselves start dying of old age around tick 1,800 — decomposers, banking
correctly since TD5, are all dead by tick ~430 with nothing left to bank.
Widen the stagger so a founder's age is drawn proportional to its own
`lifespan_for_mass`, so the cohort arrives distributed across its whole
life — mid-life on average, with a near-death tail that seeds carrion from
the first ticks — rather than every founder a newborn.

**Done when:** the carrion-window probe shows corpses present in the
formerly-empty tick 20-1,400 span; the instrument shows a gain (more
`breathes`, decomposers alive at the horizon) over TD5's mechanics-only
receipt; collapse control still collapses and escapees stay zero; fixtures
re-record at the new genesis.

## TD6 — the closed matter cycle and determinate growth (ruled 2026-08-29, Mark)

The mass fixed point, answered structurally. Income, upkeep and the
reproduction tax all scale as `m^0.75`, so the sign of net growth never
depends on size and no body ever arrives at an adult mass; crowding, which
counts bodies rather than mass, is the only regulator and it never bites
(TD5: producer stands reaching 40 billion mg on ~150 bodies). Mark ruled
**both** answers in one round, one hash break rather than two:

**The matter cycle.** The enclosure gets a finite matter budget held in a
per-voxel-column soil store. Producers draw matter from the soil; **light
stays the one open input** (energy arrives from outside, matter does not);
decay returns bodies to the soil where they fell; the player's `Deposit`
enriches it. Conservation is then the fixed point — mass cannot run away
because it has to be somewhere — and the same store is the **detritus
pool** Mark chose for decomposer persistence, so one system answers both
rulings.

**Determinate growth.** The body plan caps mass: a recipe-derived ceiling
per part, with income past the ceiling routed to budget or brood rather
than to more substance. With matter conserved this stops being load-bearing
stability machinery and becomes what it should be — an evolvable trait the
adaptation phase can push, gigantism as a lineage strategy.

**Granularity ruled per-voxel column, on measured evidence**
(`Code/testing/mesocosm/soil_granularity_probe.md`, 96 configs). At the
shipping enclosure per-voxel is *fastest* for point uptake (1.75us vs
5.66us per place — a direct index into 4KB beats a nearest-site scan) and
is the **only** grain that can express a forage radius at all: per-place
r=3 reaches 12 of 16 sites at any size, and per-crowd-cell r=3 reads every
cell that exists at today's 4x4 grid. Mark's reason for wanting the depth
is roots hunting minerals through soil, which coarse grains cannot
represent at any price. Cost is 0.015% of a tick at 10 t/s; minimum system
requirements are untouched (GPU-bound, as before).

**Done when:** total matter (soil + biomass + carrion) is conserved across
a long headless run — the load-bearing invariant, asserted as a test with a
deliberately-broken control proving it can fail; producer stands no longer
reach runaway biomass and report a bounded end mass; the instrument shows
more seeds breathing with founded kingdoms persisting, zero boils, the
collapse control still collapsing; crowding is either retired as redundant
or its remaining job is stated; fixtures re-record.

## TD7 — life priced by how it lives (ruled 2026-08-29, Mark)

TD6's two flagged residues, answered together by one principle Mark
stated: "moving has metabolic costs. locomotion doesn't come free for
animals. we consume more nutrients than plants. so for flora, roots
seeking water and nutrients and minerals from/in soil is doing so at the
speed of growth on low rent metabolism." Rent prices what a body does,
not only what it weighs.

- **Soil flow: percolation and root forage, both.** Percolation stays as
  shipped — diffusion is the medium's property. Producers additionally
  gain the forage-radius uptake the per-voxel grain was chosen for: roots
  reading neighbouring columns and drawing from the richest — slowly, at
  the speed of growth, on low-rent metabolism.
- **Locomotion-derived rent.** Upkeep gains a motility term derived from
  the body plan (the locomotion anatomy already computes), so sessile
  producers run cheap and motile consumers pay for their speed. The
  trophic asymmetry becomes physics, not an authored constant.
- **Pyramid-shaped founding.** Counts make the pyramid: many producers,
  fewer consumers, few decomposers, replacing the equal-thirds kingdom
  floor. Individual sizes stay what bodies honestly say.
- **Producers grow bigger.** Producer recipes gain the parts/volume for
  real stands — plants are big — raising their derived ceilings.

Alongside, not inside, this round: Mark opened the composable-forms
question — microbe / virus / parasite / symbiote as inhabitable classes
at animal scale ("watching an animal you inhabit do its thing under your
influence"), forms of life as "conditionally composable classes." That
gets its own research brief (traits, prior-art frameworks, and which
combinations the substrate can honestly carry), not a slot in this round.

**Done when:** the instrument reaches breathes in a majority of seeds with
zero boils, founded kingdoms at the horizon, the collapse control still
collapsing, matter still conserved to the milligram (the TD6 test is the
gate every change must pass); the rent asymmetry is derived from body-plan
numbers, not tuned; fixtures re-record.

## Findings

- **2026-08-29 (TD7, and it wants a ruling): consumers and decomposers are
  recruitment-limited, not mortality-limited, and reproduction never learned
  about determinate growth.** Measured over 3,000 idle ticks, seeds 1 and 5,
  counting `Born` and `Died` events by kingdom and sampling every living body
  against its own `mass_ceiling_mg`:

  | seed 1 / seed 5 | births | deaths | mean body / adult mass |
  | --- | ---: | ---: | ---: |
  | producers | 252 / 253 | 132 / 104 | 0.32 / 0.37 |
  | consumers | **13 / 6** | 26 / 21 | **0.23 / 0.17** |
  | decomposers | **0 / 2** | 5 / 7 | **0.00 / 0.03** |

  Producers replace themselves twice over; consumers breed at half their death
  rate and decomposers essentially never breed at all. They are not being
  killed faster than producers — they are failing to recruit, because they live
  at a fifth of the adult mass their own body plan describes and a decomposer
  lives at its starvation floor. TD6 derived an adult mass from the body plan
  and made growth determinate; `Organism::can_reproduce` still gates on an
  absolute `STARVATION_MG * OFFSPRING_COST` floor of 80 mg and a gestation
  clock, so it has no idea what "adult" means, and a body stalled at 17-23% of
  its ceiling sheds a quarter of itself per brood into a child born smaller
  still. **Mark's call:** whether breeding should be gated on the plan's own
  adult mass (life history's own answer, and the missing half of TD6), on the
  reserve rather than the body, or left alone with the shortfall fixed
  elsewhere. No `rates.rs` constant reaches it — the sweep above is the
  evidence.

- **2026-08-29 (TD7): decomposers still starve inside a full larder, which is
  TD2c's finding re-measured with the food actually present.** The same probe
  read a mean of **12-15 carrion bodies standing in the enclosure** in seeds 1
  and 5, while the decomposers dying in those runs had gone 26-106 ticks since
  their last meal and were dying at 15-20 mg. Five founding decomposers took
  602 mg of scavenging in 3,000 ticks with a dozen corpses on the ground. The
  binding constraint is `DECOMPOSE_RANGE` and the search, not the yield: TD6
  said raising `DECAYS_BASE_MG` would not rescue them, and quadrupling it in
  this round's sweep did not. It is the one kingdom whose failure is a
  *movement* problem, and it is the reason `breathes` needs a reach ruling
  rather than a rate.

- **2026-08-29 (TD7): a body with no actuator still walks, and now walks for
  free.** `axis::seed` gives a limbed line a chance of drawing no `Limb` tagma
  at all, so a seed can found consumers or decomposers with `actuator_span` 0 —
  across the ten measured seeds at the shipped founding, 22 of 160 consumers
  and 20 of 50 decomposers.
  Those bodies pay a plant's rent, and `dispersal_for` floors `locomotion()` at
  1, so they move anyway. Seed 2 is the visible consequence: its consumer
  species draws an unlimbed recipe, grazes at a producer's price, and is the
  one collapse in the receipt. TD7 priced the machinery honestly; nothing yet
  says a body without the machinery cannot travel. **Mark's call**, since it is
  a rule about what a body plan is allowed to do rather than a number.

- **2026-08-29 (TD6, and it wants a ruling): point uptake at per-voxel grain
  starves the enclosure, because a producer can reach 1 column of 1,089.** The
  granularity ruling was made on performance and expressiveness evidence, which
  it still passes; this is new evidence of a different kind and it invalidates
  nothing the probe measured. Measured, with a sealed store: a producer drains
  the column it stands on within tens of ticks and thereafter earns **exactly
  the rent it just paid** — net zero, permanently, at any constants, because
  the only matter returning to its column is its own. A probe at soil 300
  mg/column read 17 producers standing on **0 mg** with **340,000 mg** lying in
  the enclosure around them; consumers dead by tick 1,100, decomposers by 800,
  every seed collapsing. Raising the seed does not fix it — it converts the
  collapse into producers *mining outward* column by column (610-825 bodies,
  consumers still dead), because reachability rather than supply is what binds.
  **What shipped is soil percolation**, added under protest and flagged here:
  once per tick every column sheds `1/PERCOLATION_DIVISOR` of what it holds
  into its four neighbours, exact in integers, so a column's loss is its
  neighbours' gain and conservation is untouched. It is the medium's own
  property (dissolved minerals move through soil, which is *why* a root that
  searches a radius finds more) rather than the foraging behaviour this round
  deliberately left unbuilt, and it is what makes "the enclosure gets a finite
  matter budget" mean *one* budget. It is decisively load-bearing: with it off
  and everything else identical, every baseline seed ends at **~1 organism**;
  with it on at divisor 8, 46-100. **Mark's call:** whether the real answer is
  the forage radius the grain was chosen for (TD7), percolation as shipped, a
  coarser grain for uptake only, or producers that relocate off spent ground.
  This is the one decision in TD6 taken without him, and it was taken because
  the alternative was shipping a dead terrarium.
- **2026-08-29 (TD6): consumers, not producers, are what still fails.** With
  the cycle closed and percolation supplying the stand, the chain now dies at
  the same place it died in TD5 and TD5b: the founding cohort's ~20 consumers
  over-graze ~17 producers in the first 200 ticks, grow to 700-1,500 mg on the
  proceeds (their body plans allow 2,000-2,500), and then starve in a thinned
  pasture; decomposers follow. The consumer:producer *ceiling* ratio is ~4:1
  because a consumer recipe realizes ~24 parts against a producer's ~5, so
  determinate growth as derived lets a grazer outgrow its own food's adult size
  by four times — an inverted pyramid written into the body plans, not into the
  constants. That is the next round's target, and it is the third round running
  that `breathes` has been out of reach for this reason.
- **2026-08-29 (TD1):** the ecology at 61 founders is **bimodal, not
  reliably explosive**: across seeds 1-5, three boil and two collapse. The
  boils are pure producer blooms — producers 60 → 600+, consumers flat at
  1-3 (in one seed the played lineage dies while plants overrun the
  enclosure), decomposers gone early — so crowding's self-thinning is not
  enough to cap a bloom. The collapses are the mirror: a founder pool one
  bad species draw from zero producers starves the whole chain (seed 3
  founded with none). The retune target is therefore trophic balance —
  producer throttle, consumer viability, founder composition — not "less
  growth". Verdicts early-exit (boil and collapse are terminal; only
  breathes needs the full horizon), which took the receipt from unbounded
  to under two seconds.
- **2026-08-29 (TD2 consequence, noted in advance):** any constant change
  alters world dynamics from genesis, so every recorded trace's replay
  hash — including the 2026-08-28 playtest fixture — stops matching the
  moment TD2 lands. The old files stay as history with their claim
  retired, and a fresh demo trace is recorded at the new constants.

## Progress

- **2026-08-29 (later still): TD7 landed — rent is priced by how a body lives,
  and the base of the pyramid is fixed. The tiers above it are not, and the
  round measured exactly why.** All four ruled changes are in, conservation is
  still milligram-exact, and the verdict tally is unmoved at **0 breathes / 9
  thins / 0 boil / 1 collapse**. What moved is everything under that number:
  the producer stand went from a thinned 3,267-27,616 mg to 14,192-65,541 mg of
  standing biomass, and the two failing kingdoms turn out to be
  **recruitment-limited, not mortality-limited**.

  **1. Root forage** (`places/soil.rs`, 363 lines). `Soil::draw_richest_within`
  reads `columns_within` and then takes the ordinary income out of the richest
  column it found: **wide reach, one tick's draw**, never the radius' worth of
  columns at once. `FORAGE_RADIUS = 3` — the reach the per-voxel grain was ruled
  for, 49 of 1,089 columns here, where every coarser grain's r=3 already covered
  the whole world. Ties go to the lowest column index so a stand on flat ground
  forages the same way in every replay. Percolation is exactly as TD6 shipped
  it. Conservation is untouched because this is the same `draw`: the addressing
  and the transfer were kept apart for precisely this, and the forage read was
  an addition rather than a redesign, as the TD6 entry predicted.

  **2. Locomotion-derived rent, and the formula is the body plan's own.**
  `Organism::locomotion()` was a sum of each living contractile part's longest
  half-extent, floored at 1 for the drive selector. The floor is now split off
  as `Organism::actuator_span()`, which reads **0** for a body with no actuator
  — what a sessile plan honestly says — and `locomotion()` is that `.max(1)`.
  `ecology::upkeep_for_body` prices it:

  ```text
  rent = UPKEEP_BASE_MG + m^0.75 * (ceiling + span * REFERENCE_SEGMENT_MG)
                        / (UPKEEP_SCALE * ceiling)
  ```

  Three numbers, all already here: `biomass_mg`, `actuator_span`, and TD6's
  `mass_ceiling_mg`. The motile multiple is `span * 100 / ceiling` — **the
  actuator swing a body carries per reference segment of body it carries it
  on**. Dividing by the plan's own adult mass is what makes it scale-free: both
  halves grow with the body, so the term reads a body's *build* and not its
  size, and a long plant does not pay for being long. **No new constant**, and
  `REFERENCE_SEGMENT_MG` is `REFERENCE_MASS_MG` renamed for the job.

  The consequences are readings. A sessile body reads span 0 and the formula
  collapses to `m^0.75 / UPKEEP_SCALE` **exactly**, to the milligram — a test
  asserts it. A palette limb is half-extent `[4,1,1]`, so it swings 4 and holds
  a 64 mg ceiling against an axial segment's 100 mg, which bounds the whole
  surcharge by construction at `1 + 4*100/64` = 7.25x for a body of nothing but
  limbs. Measured at genesis across seeds 1-10, mean rent per founder:

  | kingdom | before TD7 | after TD7 |
  | --- | --- | --- |
  | producers (span 0, always) | 1.4-1.9 mg/tick | **1.4-1.9, unchanged** |
  | consumers, limbed recipe (span 16-108) | 1.5-1.8 | **2.3-6.4** |
  | decomposers, limbed recipe (span 32-64) | 1.4-1.7 | **2.6-4.6** |
  | either, unlimbed recipe (span 0) | 1.5-1.8 | **1.2-1.8, unchanged** |

  Before this round every kingdom paid the same 1.4-1.9 mg/tick for the same
  mass. That is the trophic asymmetry TD7 was for, and it is anatomy.

  **3. Pyramid founding** (`world/genesis.rs`). `pyramid(count)` composes the
  tiers exactly rather than drawing them: `PRODUCER_SHARE` 2/3, `CONSUMER_SHARE`
  1/4, the rest decomposers, then a Fisher-Yates shuffle over the seeded stream
  so the pyramid is the world's shape rather than a distribution it usually
  lands near. At the shipping 60 non-played founders that is **40 / 15 / 5**,
  and 40/16/5 with the played consumer — in every seed, asserted. TD2b's kingdom
  floor is kept: a tier that rounds to nothing takes one founder from the widest
  rather than leaving a rung out, tested over foundings of 3 to 12. The species
  draw inverted with it — the tier a founder drew now names its species, since a
  species has one inherited silhouette.

  **4. Producers grow bigger** (`axis.rs`). Only the unlimbed branch of
  `axis::seed` changed, and only its two existing draws: stretches
  `1 + below(2)` -> `1 + below(3)`, segments per stretch `1 + below(6)` ->
  `4 + below(8)`. No new part type, no new template, no palette change — the
  same axial rules asked for a larger stand. Measured at genesis, seeds 1-10:

  | | parts per body | derived ceiling | mean ceiling per seed |
  | --- | --- | --- | --- |
  | producers before | 2-16 | 200-1,600 mg | 477-1,069 mg |
  | producers after | **5-35** | **500-3,500 mg** | **747-3,175 mg** |
  | consumers (unchanged) | 3-48 | 300-3,576 mg | 1,392-2,799 mg |

  TD6's inverted pyramid — a grazer able to outgrow its own food's adult size by
  four times — is gone from the body plans.

  **Verdicts. The tally is unmoved and the world underneath it is not.**
  Baseline reproduced once before touching anything and it matched TD6's table
  milligram for milligram. Receipt: `td7_priced.json` (mechanics only; no
  `rates.rs` constant was changed, so there is no `td7_priced_tuned.json`).

  | seed | verdict | start | end | P/C/D start | P/C/D end | end biomass | soil end | total matter |
  | ---: | --- | ---: | ---: | --- | --- | ---: | ---: | ---: |
  | 1 | thins | 61 | 117 | 40/16/5 | 117/0/0 | 63,473 mg | 33,305 | 148,390 |
  | 2 | collapse | 61 | 0 | 40/16/5 | 0/0/0 | 0 mg | 141,677 | 144,646 |
  | 3 | thins | 61 | 89 | 40/16/5 | 89/0/0 | 58,974 mg | 52,557 | 148,386 |
  | 4 | thins | 61 | 120 | 40/16/5 | 120/0/0 | 53,932 mg | 43,626 | 143,412 |
  | 5 | thins | 61 | 119 | 40/16/5 | 119/0/0 | 65,541 mg | 35,990 | 147,150 |
  | 6 | thins | 61 | 114 | 40/16/5 | 114/0/0 | 24,234 mg | 100,840 | 147,810 |
  | 7 | thins | 61 | 155 | 40/16/5 | 155/0/0 | 43,670 mg | 62,493 | 144,338 |
  | 8 | thins | 61 | 83 | 40/16/5 | 83/0/0 | 26,450 mg | 97,742 | 147,702 |
  | 9 | thins | 61 | 76 | 40/16/5 | 76/0/0 | 14,192 mg | 121,018 | 149,070 |
  | 10 | thins | 61 | 91 | 40/16/5 | 91/0/0 | 32,528 mg | 88,288 | 145,330 |

  Control all collapse, max escapees 0, `total_matter_mg` identical across every
  sample of every run. Against TD6: end biomass 3,267-27,616 -> 14,192-65,541
  mg, and the soil is drawn down to 33,305-121,018 mg from a flat
  108,900 — the stand is now eating the enclosure the way a stand should. The
  one collapse moved from seed 10 to seed 2 (whose consumer species draws an
  unlimbed recipe, so it grazes at a plant's rent).

  **The bounded `rates.rs` pass was run and is recorded here rather than
  shipped** — the same call TD5 and TD6 made, for the same reason: nothing
  reached `breathes`. Six configurations, ten seeds each:

  | GRAZES / DECAYS / COMFORT / HUNGRY | breathes | thins | boil | collapse |
  | --- | ---: | ---: | ---: | ---: |
  | 3 / 4 / 1 / 8 (shipped) | 0 | 9 | 0 | 1 |
  | 6 / 4 / 1 / 8 | 0 | 9 | 0 | 1 |
  | 6 / 4 / 2 / 8 | 0 | 9 | 0 | 1 |
  | 6 / 8 / 2 / 8 | 0 | 8 | 0 | 2 |
  | 6 / 8 / 2 / 24 | 0 | 8 | 0 | 2 |
  | 9 / 16 / 2 / 24 | 0 | 9 | 0 | 1 |
  | 12 / 16 / 3 / 24 | 0 | 9 | 0 | 1 |

  Every one of them ends every seed at `P/0/0`. Raising the graze and decay
  rates and loosening crowding buys **bigger producer stands** (up to 315 at the
  horizon) and nothing else, which is itself the evidence that the binding
  constraint is not a rate.

  **Fixtures.** Demo re-recorded: 120 intents, hash **`33ffc5b46789be9d`** (was
  `c8713ce9a82f5d6f`), `--replay` headed landing it exactly, exit 0, 30 frames
  on the RTX 4060 (Vulkan) — 55 body parts, 18 roster members, ground revision
  2. Instrument proven: one bit flipped in the recorded hash exits 1 with
  `MISMATCH`. Default paths (`ps1_played.trace.json` / `.json` / `.png`). The
  capture reads a section that is visibly a different ecology from TD6's: a
  chain of green producer segments strung along the whole surface with salmon
  and red bodies among them, the minimap, and vitals at `energy 1364 mg` — with
  a **burn** notice this time, which is TD7's rent showing on the played body.

  **Tests.** `cargo test --workspace`: green (`mesocosm-lens` run separately at
  `--test-threads=1`, 38 passed, per the standing environment residue).
  `cargo clippy --workspace --all-targets -- -D warnings`: clean.
  `cargo fmt --all --check`: clean. `cargo check -p paredros-room --features
  r1-proof`: builds (the same one pre-existing `dead_code` warning in that repo
  TD6 recorded, not this round's). `axis.rs` went past the ceiling, so its
  test module moved to `axis/tests.rs` (428 + 202) — the same split the ecology
  already uses. Six tests were retuned, one line of why on each:
  `allometric_rates_cross_three_orders_without_flat_steps` now asks for the
  sessile rent explicitly; `serialization_does_not_distinguish_the_played_critter`
  compares every field of an organism that a tick does not move rather than
  bytes, because a wider forage read lets a different hand reach a plant's
  mouthful inside one tick; `a_held_critter_still_ages_pays_rent_and_can_die`
  measures rent against what the critter ate, because forty producers mean a
  held grazer now out-earns its rent; both venom tests subtract the ecology's
  own bite from the played meal's receipt; and mesocosm-mesh's two attachment
  tests provision the critter, because TD7's rent moved the starved horizon
  further from empty than the walk to a neighbour leaves it.

- **2026-08-29 (later): TD6 landed — the matter cycle closes, growth becomes
  determinate, and the enclosure stops being able to make mass.** Conservation
  is exact and proven; the runaway is gone; the verdict tally regressed, and
  the reason is a structural finding that wants a ruling.

  **The store.** `places::soil` (`crates/mesocosm-core/src/places/soil.rs`,
  306 lines) holds one `u64` of milligrams per voxel column, row-major in z
  then x over `-extent..=extent`, sized from `ENCLOSURE` — 33x33 = 1,089
  columns, 8.7 KB. World state: a `soil` field on `World`, serialized, inside
  `state_hash`, round-tripped by a test. A `Column` is an opaque index only
  `Soil` mints, and `column_at` **clamps** rather than refusing, so nothing
  deposited can fall off the edge of the ledger. Addressing (`column_at`,
  `columns_within`) is deliberately separate from transfer (`matter_mg`,
  `draw`, `deposit`), so the ruled forage radius is a read over
  `columns_within` and then the same `draw` — a test asserts r=3 reaches 49 of
  1,089 columns here, which is the reach the granularity ruling was for. The
  foraging *behaviour* is not built, as scoped. `Soil` is not `Ground`: bricks
  are what is solid, soil is what can be eaten out of the floor, and a carve
  moves none of the second.

  **The cycle, closed at nine seams.** Producers `draw` from their own column
  and can take nothing more; rent goes into the column the body stands on
  (both halves — budget-paid and the self-consumption when the budget is
  empty); carrion decay puts its milligram in the column it is lying on; a
  death releases the reserve the body was still carrying; travel is paid in
  substance into the ground it was covered over, for NPCs and for the played
  `Intent::Move` alike; `Intent::Deposit` now enriches the column instead of
  minting a carcass; the played meal returns what the eater did not keep —
  the meal's own reserve, an odd milligram a mirrored split could not halve,
  the half of a pair that would not attach, and what a bite of venom cost;
  and **a birth is provisioned rather than conjured** — until now a parent
  paid `cost` once in body mass and the child was handed a body worth `cost`
  *and* a budget worth `cost`, so every birth in the enclosure minted matter.
  The child's opening budget now comes out of the parent's own reserve.

  Genesis seeds `SOIL_SEED_MG_PER_COLUMN = 100` per column — the ecology's own
  `REFERENCE_MASS_MG`, so the rule reads *one reference body's worth of
  substance under every voxel column*. At the shipping enclosure that is
  108,900 mg, about three times what a 61-founder cohort carries in bodies and
  reserves. Sized from `ENCLOSURE`, never hardcoded.

  **The tick had to be restructured, and this is why.** Crediting an eater its
  full mouthful and reconciling afterwards conjures matter, because two grazers
  reach the same small producer and the second finds less than it bit for —
  and there is no reconciliation that is always payable (the eater may have
  died and released its reserve before the drain pass ran). The conservation
  test caught it at **tick 28 of seed 4, 1 mg**. So the tick is now three
  passes: rent and the bite each body *reaches for*; the meals settled one at
  a time with both bodies in hand, crediting exactly what came out of the prey;
  then dispersal, maturity, decay and death. The birth pass moved to
  `ecology/breeding.rs` at the 600-line ceiling, following the crate's own
  sibling-file precedent.

  **Determinate growth, derived, no new number.** `Organism::mass_ceiling_mg`
  sums a per-part ceiling over the living body, and a part's ceiling is its own
  voxel volume priced so that a reference segment holds `REFERENCE_MASS_MG` —
  both numbers were already here. `gain_mass` caps at it and returns what would
  not fit; `earn` routes the overflow to the budget (capped the same way) and
  hands the remainder back to the world; a feeding body clamps its bite to
  `intake_room_mg`, so **a full body does not feed**. Measured at genesis: mean
  producer ceiling 477-870 mg against a mean drawn mass of ~290, mean consumer
  ceiling 2,044-2,456, the played critter 1,568-2,522; 0-3 founders of 61 open
  at or over their ceiling. The way past the ceiling is the game's own verb —
  eating adds *parts*, and every part brings its ceiling with it.

  **Crowding is not redundant, and the round measured why.** The soil bounds a
  stand's **mass**; nothing in a closed matter budget bounds its **number**. A
  run with crowding removed answered a finite enclosure by subdividing: 620
  producers and still climbing at the horizon at every soil seed swept (100,
  300, 500, 600, 1,000 mg/column) and every minimum-body size swept
  (`STARVATION_MG` 20/60/120). `CROWD_CELL`/`CROWD_COMFORT` are therefore kept
  at TD2c's 8/1, unchanged, and the rent floor under them is kept too — but it
  is no longer a hand-out: it is a *request*, and the column answers it or does
  not. Density is the job crowding still does; it is simply no longer the only
  regulator.

  **Conservation, and the proof it can fail.** `tests/matter.rs`: four seeds x
  4,000 idle ticks, checked **milligram-exact every tick** (births, deaths,
  grazing, predation, scavenging, decay, dispersal and the founding cohort
  dying of old age all fall inside that window), plus a played-verb run
  covering deposit, movement, carve and metabolize. **Zero exceptions** — no
  sink, no source, no tolerance. Two things are outside the account because
  they are not matter: light (the ruled open input, which powers uptake and
  never enters the ledger) and ground bricks. The same `conserved()` the long
  runs use is handed a conjured milligram and a leaked one and must report
  both; the conjured control replays the pre-TD6 producer income exactly.
  Flipping that control's assertion to prove it really trips:

  ```text
  thread 'the_check_catches_income_conjured_the_way_it_used_to_be' panicked at
  crates\mesocosm-core\tests\matter.rs:146:10:
  PROOF RUN: the deliberately-broken control must trip the check: "matter is not
  conserved after conjuring a producer's old income: 145549 mg against 145538 mg
  at genesis (11 mg conjured); soil 109002 mg, 61 bodies holding 36547 mg"
  ```

  The instrument carries it too: every seed's receipt records `total_matter_mg`
  per sample, and it is identical across every sample of every run.

  **Verdicts, reported honestly: this is a regression.** Baseline was 2
  breathes / 7 thins / 0 boil / 1 collapse (TD5b). TD6 reads **0 breathes / 9
  thins / 0 boil / 1 collapse**, control all collapse, max escapees 0. Receipt:
  `td6_matter.json`.

  | seed | verdict | start | end | P/C/D start | P/C/D end | end biomass | soil end | total matter |
  | ---: | --- | ---: | ---: | --- | --- | ---: | ---: | ---: |
  | 1 | thins | 61 | 36 | 22/25/14 | 36/0/0 | 3,267 mg | 139,784 | 145,538 |
  | 2 | thins | 61 | 60 | 20/21/20 | 60/0/0 | 27,616 mg | 96,778 | 146,602 |
  | 3 | thins | 61 | 40 | 17/27/17 | 40/0/0 | 7,400 mg | 130,604 | 145,180 |
  | 4 | thins | 61 | 83 | 17/21/23 | 83/0/0 | 9,522 mg | 125,069 | 141,652 |
  | 5 | thins | 61 | 100 | 16/25/20 | 100/0/0 | 9,106 mg | 129,782 | 147,248 |
  | 6 | thins | 61 | 69 | 16/23/22 | 69/0/0 | 7,346 mg | 133,943 | 146,890 |
  | 7 | thins | 61 | 78 | 22/20/19 | 78/0/0 | 8,425 mg | 131,345 | 147,070 |
  | 8 | thins | 61 | 56 | 24/19/18 | 56/0/0 | 7,720 mg | 132,195 | 146,032 |
  | 9 | thins | 61 | 64 | 24/20/17 | 64/0/0 | 21,052 mg | 109,033 | 145,148 |
  | 10 | collapse | 61 | 0 | 23/20/18 | 0/0/0 | 0 mg | 147,375 | 147,904 |

  **The runaway is gone and the numbers say so.** TD5's producer stands reached
  ~4 x 10^10 mg on ~150 bodies. End biomass is now **3,267-27,616 mg**, six
  orders of magnitude down, and it is bounded by construction rather than by
  tuning: the rest of the world's matter is in the ground, where it can be
  pointed at.

  **No `rates.rs` constant was changed and no `td6_matter_tuned.json` was
  written** — the same call TD5 made, for the same reason. The bounded pass was
  run and is recorded here rather than shipped: soil seed 100/300/500/600/1,000
  mg per column, percolation divisor 8/16/32/64, `CROWD_COMFORT` 1/2/3,
  `STARVATION_MG` 20/60/120. None reached `breathes`. Below ~300 mg/column
  without percolation everything collapses; at 300-1,000 producers boil in
  *count* (610-825 bodies, still rising) while consumers and decomposers are
  already dead; `CROWD_COMFORT` 3 buys bigger stands (237-299 producers) and
  one seed each keeping consumers or decomposers, but no seed keeping both.

  **Fixtures.** Demo re-recorded: 120 intents, hash **`c8713ce9a82f5d6f`** (was
  `e8ba7206ac96834f`), `--replay` headed landing it exactly, exit 0, 30 frames
  on the RTX 4060 (Vulkan) — 61 body parts, 12 roster members, ground revision
  2. Instrument proven: one bit flipped in the recorded hash exits 1 with
  `MISMATCH`. Default paths (`ps1_played.trace.json` / `.json` / `.png`). The
  capture reads the section with the played critter's wide-armed silhouette at
  centre, two green roster bodies to its left and three lavender ones to its
  right, the minimap, and vitals at `energy 615 mg` with no burn notice.

  **Tests.** `cargo test --workspace`: green (`mesocosm-lens` run separately at
  `--test-threads=1`, 38 passed, per the standing environment residue).
  `cargo clippy --workspace --all-targets -- -D warnings`: clean.
  `cargo fmt --all --check`: clean. `cargo check -p paredros-room --features
  r1-proof`: builds (one pre-existing `dead_code` warning in that repo, not
  this round's). Four tests were retuned with a one-line why on each:
  `ecology::tests`'s fixture half-extent went `[1,1,1]` to `[5,5,5]` (27 voxels
  carrying 300 mg was invisible until a body plan had an adult mass);
  `a_contractile_consumer_can_take_live_consumer_prey` widened its long-and-thin
  shapes to `[8,2,2]`/`[4,4,4]` so a 300 mg body sits under its own ceiling and
  can still bite; and `deposit_returns_matter_to_the_enclosure` now asserts the
  ground got richer and no carcass was minted.

- **2026-08-29 (later): TD5b landed — founders arrive mid-life, and the
  corpse drought closes.** `genesis.rs`'s age draw was `rng.below(200)`
  regardless of the founder's own mass; it is now
  `rng.below(lifespan_for_mass(mass))` — proportional to the individual's
  own life-history rate, uniform across its **whole** life. Mean age lands at
  the founder's own midpoint (mid-life, the title's word, not a knob picked
  to sound right), and the draw's near-death tail seeds carrion from the
  first ticks without any founder's age exceeding its own lifespan. The
  played critter (organism 0) keeps `age: 0` — its own line, separate from
  the loop, untouched — because the player's life should start near its
  beginning, not drawn from the same whole-life distribution as the ecology
  around it. One-line why lives on the draw itself; genesis is otherwise
  unchanged, `since_offspring`'s own stagger included.

  **The carrion window (probe, seeds 1 and 7 — TD5's own probe seeds,
  extended to 2,000 ticks, sampled every 10).** Before: the [20,1400] span
  held *some* corpse on 360/1381 and 316/1381 sampled ticks respectively,
  but every one was a trickle — 1 to ~30 mg, a starvation death decaying
  at 1 mg/tick before the next one replaced it — so TD5's "no real carrion"
  reading holds even though the window was not literally always at zero.
  Real accumulation (hundreds of thousands of mg, from old-age deaths)
  only began at tick 1,267-1,330. After: seed 1 holds a corpse on
  **1,381/1,381** sampled ticks in the window — full coverage — at
  hundreds to low thousands of mg from tick 30 on; seed 7 holds one on
  1,092/1,381, with the last empty tick at 409 and the window continuously
  occupied after. The drought's far edge is gone; its near edge (the first
  ~20-30 ticks, before any founder can plausibly have died) is structural
  and untouched by this change.

  **Verdicts.** Baseline 0 breathes / 9 thins / 0 boil / 1 collapse (of 10,
  reproduced before touching genesis, matching TD5's own receipt exactly)
  → **2 breathes / 7 thins / 0 boil / 1 collapse**. Decomposers alive at the
  10,000-tick horizon: 0 of 10 seeds before, **3 of 10 after** (seed 3: 0 →
  37; seed 7: 0 → 14; seed 9: 0 → 31). Seeds 3 and 7 convert fully —
  founded P/C/D end at 90/3/37 and 54/70/14, every kingdom the world
  started with still standing. Collapse control still all-collapse (the
  control founds no extra founders, so it never touches the changed draw);
  max escapees still 0 across every sample. Receipt: `td5b_midlife.json`,
  both batches, same schema as every earlier round's.

  **Reported honestly: this is not an unmixed win.** Total end-population
  fell in most seeds that were already thinning (seed 1: 364 → 143; seed 4:
  369 → 195; seed 6: 125 → 48; seed 8: 308 → 90; seed 10: 168 → 83) — the
  wider stagger's near-death tail is real attrition, not just useful
  corpses, so a founding cohort that arrives partway through dying trades
  some peak population for the carrion that population's death provides.
  Seed 9 is the sharpest trade: consumers held 35 at the horizon before and
  **0 after** — the predicted risk (an early die-off wave thinning live
  prey) landed in this one seed, even as its decomposers went 0 → 31. Seed
  5's consumers fell 2 → 0, already marginal either side. TD5's own
  majority-breathing bar is still unmet (2 of 10, not a majority) — TD5b
  closes the structural finding TD5 could not reach from `rates.rs`, it
  does not finish TD5's own done-condition.

  **Fixtures.** Demo re-recorded: 120 intents, hash **`e8ba7206ac96834f`**
  (was `02b072a5cfe7b10b`), `--replay` headed landing it exactly, exit 0, 30
  frames on the RTX 4060 (Vulkan) — 61 body parts, 14 roster members, ground
  revision 2. Instrument re-proven: one bit flipped in the recorded hash
  exits 1 with `MISMATCH`. Written to the default paths (`ps1_played.trace.json`
  / `.json` / `.png`).

  **Tests.** `genesis.rs` gained one test, `ages_are_staggered_across_the_founders_own_lifespan`
  (max founder age exceeds the old flat 200 cap; the played critter stays
  `age: 0`), with a one-line why on both the test and the draw it checks.
  The two existing genesis tests (`every_seed_founds_all_three_kingdoms`,
  `since_offspring_is_staggered_like_age`) needed no change — neither reads
  age's distribution — and pass unmodified. `cargo test --workspace`: green,
  with one environment residue below. `cargo clippy --workspace --all-targets
  -- -D warnings`: clean. `cargo fmt --check`: clean.

  **No constant in `rates.rs` was touched.** The corpse drought was
  genuinely a genesis question, as TD5's finding said; nothing here argues
  for a retune, though seed 9's consumer trade-off is worth watching if a
  future round touches predation pressure.

  Two residues. **`mesocosm-lens`'s GPU-adapter tests deadlock under the
  default parallel test harness in this environment** — reproduced on the
  unmodified crate (nothing this round touched it), confirmed by isolating:
  `cargo test -p mesocosm-lens` hangs indefinitely with several
  `tracer_tests` cases stuck past 60 seconds regardless of thread count
  contention, but `cargo test -p mesocosm-lens -- --test-threads=1` passes
  all 38 clean in 14 seconds. Pre-existing, out of this round's scope
  (rendering, not genesis), and not touched; `cargo test --workspace` here
  means that plus the rest of the workspace at default threading, both
  green. Second, the wider stagger widens the founding population's own
  variance — a seed's early die-off is now part of its founder draw the way
  its kingdom composition already was, so a future retune reading "seed 9
  collapsed" should check whether that seed's draw changed before assuming
  a constant did.

- **2026-08-29 (later): TD5's mechanics landed; its done-condition did not,
  and the reason is structural.** The economy is one rule now, and the
  instrument says so without saying `breathes`.

  **The routing.** `ecology::step` gained one private `earn(organism, mg)`,
  and all four NPC income paths call it instead of `gain_mass`: producer
  fixing (the crowd-divided, rent-floored share), grazing and predation (the
  meal's biomass, one shared call), and decay. Inside it is TD4's question
  verbatim — `budget_below(STARVED_UPKEEP_TICKS)` credits `energy_mg`,
  otherwise the body grows as before. The played critter's path in
  `world::act` is untouched and unchanged in behavior; it was already this
  rule, which is the point. `dispersal_for`'s extra step moved off literal
  `energy_mg == 0` onto `is_hungry`, and `HUNGRY_UPKEEP_TICKS` plus the
  predicate moved from `movement` into `rates` so the tick's one hunger
  horizon sits with the tick's numbers. **Tick order is unchanged and had to
  be**: rent is paid before income, so `earn` reads this tick's post-rent
  reserve — a body is asked whether it is starving *after* the day has cost
  it something, which is the only reading that means anything.

  **Verdicts (mechanics only, no constant touched).** 0 breathes / 9 thins /
  0 boil / 1 collapse, control all collapse, escapees 0 — the TD2d tally to
  the seed. The tally is unmoved; underneath it is not. Consumers alive at
  the horizon went from 3 seeds to 5, and far larger where they survive
  (seed 7: 39 → 108, seed 8: 35 → 42, seed 9: 12 → 35); births and deaths
  roughly doubled across the board (seed 1: 205/138 → 899/542), which is a
  chain turning over rather than a stand sitting still. Seed 2's lone
  collapse is the same founder draw that survived three constant regimes.
  Receipt: `td5_economy.json`. **Decomposers are still zero at the horizon
  in every seed**, so `thins` is honest and `breathes` is unreached.

  **The probe (seeds 1, 7, 9).** Decomposers do bank now — 1,320-2,908
  organism-ticks holding a reserve — and mean life rose where it could
  (seed 7: 215 → 257 ticks). But the last decomposer in every seed dies by
  tick 365-430, and the probe's carrion curve says why: **the enclosure holds
  no corpses at all between roughly tick 20 and tick 1,400.** Founders are
  age-staggered over `rng.below(200)` against a 2,000-3,000-tick lifespan, so
  nothing dies of old age until ~1,800; early starvation corpses are ≤ 20 mg
  (`STARVATION_MG`) and decay at 1 mg/tick, so they are gone in twenty ticks.
  After 1,400 carrion is abundant — 20+ bodies, tens of millions of mg — and
  there is no decomposer left to eat it. That is locked matter, and it is a
  *timing* failure, not a yield or a search one.

  **The bounded constants pass found nothing worth shipping, and that is the
  finding.** `UPKEEP_SCALE` 62 → 186 (rent to a third, stretching the birth
  endowment) moved last-decomposer-death 365/385/430 → 487/705/646 and made
  the producer stand's biomass ~5x worse; `STARVATION_MG` 20 → 100 (bigger
  corpses from starvation deaths) put real bodies in the early enclosure and
  moved the deaths to 656/534/719. Neither comes within 1,000 ticks of the
  drought's far edge, because no rates.rs constant sets when the founders
  die. Both were reverted; **no constant changed and no
  `td5_economy_tuned.json` was written**, so the mechanics-only picture is
  the whole picture.

  **Fixtures.** Demo re-recorded: 120 intents, hash **`02b072a5cfe7b10b`**
  (was `18615c6b8309f821`), `--replay` headed landing it exactly, exit 0,
  30 frames on the RTX 4060 (Vulkan) — 61 body parts, 17 roster members,
  ground revision 2. Instrument proven: one bit flipped in the recorded hash
  exits 1 with `MISMATCH`. Default paths (`ps1_played.trace.json` / `.json` /
  `.png`). The capture reads roster silhouettes across the section, the
  minimap, and vitals at `energy 586 mg` with no burn notice — the last meal
  was more than the 25-tick notice window before the final frame.

  One test was retired by the change and says so: `instinct.rs`'s
  `walking_away_gives_the_body_back_to_the_ecology` set the played critter's
  budget to zero and watched it wander, but an NPC now eats its way out of
  hunger on the first tick and correctly stands still, so the wander claim
  moved to a `stranded()` fixture — empty budget, empty enclosure.

- **2026-08-29 (later): TD4 landed, with the tempo and fresh fixtures.**
  Both core rules, the canonical 10 t/s host, and a re-recorded receipt set.

  **The hand.** `World` counts consecutive `Intent::Idle` applications in a
  plain hashed field; any other intent resets it. Below
  `INSTINCT_IDLE_TICKS = 30` (three seconds at the tempo) `World::held()`
  names the controlled critter and the ecology skips **only its dispersal** —
  it still ages, pays rent, feeds, breeds and dies. Past the threshold `held`
  is `None` and its own drives resume, with control itself never moving: the
  next keypress lands mid-stride, nothing is handed over or reclaimed. The
  count is a function of the trace, so every host and every replay reaches the
  same answer at every frame rate; a host-side wall clock would have made the
  ecology depend on how fast the machine drew.

  **The meal.** `Intent::Metabolize` dropped its `Route` and carries only
  `Placement` — which was always the other question (*where a kept part goes*,
  a growth policy an editor needs) rather than the one D5 asked. Burn-or-build
  is now the body's, decided in `World::apply` at
  `STARVED_UPKEEP_TICKS = 100`: ten seconds of standing still, about a third of
  a 1,000 mg starter's 333-tick budget. Wide on purpose — the ecology's own
  hunger horizon is eight ticks, the point a body starts eating itself, and
  routing there would have meant every meal grew you until the tick before you
  died. Both thresholds ask the same question through one predicate,
  `Organism::budget_below(ticks)`: hunger is never a milligram count, because
  a large body burns through the same number faster. `Route` survives as what
  the body concluded. The F key is gone; E/Space is the whole verb.

  **Tempo.** Host `ticks_per_second` 60 → 10. Two presentation constants were
  ticks-at-60 and were retimed with it: the vitals notice window (150 → 25,
  the same 2.5 s) and the minimap backdrop cadence (10 → 2, restoring both the
  wall cadence and the per-frame cost). Nothing else in the host counts ticks.

  **Receipts.** Demo re-recorded at the new world and tempo: 120 intents, hash
  **`18615c6b8309f821`**, `--replay` headed landing it exactly, exit 0, over 30
  frames on the RTX 4060 (Vulkan) — 61 body parts, 16 roster members, ground
  revision 2. Instrument proven once: a trace with one bit flipped in its
  recorded hash exits 1 with `MISMATCH`. Written to the default paths
  (`ps1_played.trace.json` / `.json` / `.png`), superseding the 2026-08-28
  fixtures. The capture shows the section with a dozen-plus bodies in it, the
  minimap, and the vitals panel reading `energy 793 mg` with a live **burned**
  notice — the meal's destination, said on screen, which is the whole of the
  feedback now that the player is not the one choosing it. The trace's own
  verbs now include a forty-tick hands-off stretch, so the fixture covers both
  halves of TD4.

  **The idle run, by hand (109 steps, no keys touched).** Every recorded
  intent was `Idle`. The played critter held station for exactly the
  documented window and **first moved on step 29** — the apply at which the
  count reaches 30 — then wandered [0,21,0] → [3,21,1] on its own drives. By
  the end 27 of the surviving founders had moved, the population had run
  61 → 41, the section roster was 24, and the budget had fallen 941 → 488 mg
  paying rent with nobody earning it. The trace replays to its hash exactly.

  Three residues. **A held critter cannot flee, and a fat one is the best meal
  in the enclosure**: the 200-step demo died every time around tick 134 with
  up to six predators feeding on it at once, which is why `DEMO_STEPS` is 120.
  That is the movement-economy seam from the playtest findings, flipped —
  before TD4 the ecology fled for you. Second, growth raises rent, so a critter
  that grows fast crosses `STARVED_UPKEEP_TICKS` on its own success and its
  next meals burn; whether that negative feedback is the design or an accident
  of the m^0.75 scaling is Mark's (it touches the "individual mass has no fixed
  point" finding below). Third, `population_instrument.rs` drives worlds with
  `Intent::Idle`, so its controlled critter is now held for the first 30 of
  ~10,000 ticks — negligible, and the TD1-TD2d receipts were not re-run.

- **2026-08-29 (later): TD2d's scavenger sight landed.** Scavengers seek
  carrion out to sight range and bite at `DECOMPOSE_RANGE`, mirroring the
  grazer split; hunger wanders below eight upkeep-ticks of budget instead
  of at literal zero. The death-cause probe proves the mechanism:
  starvations beside in-range carrion fell to ~0. The verdict tally did
  not move (9 thins / 1 collapse) — decomposers now die traveling, or
  waiting for anything nearby to die: the binding constraint is corpse
  throughput, not search. Receipt: `td2d_scavengers.json`.
- **2026-08-29 (TD2d finding): NPC energy never refills.** `energy_mg` is
  endowed at birth and topped up only for the controlled critter (the
  metabolize gain in `world/act.rs`); every other organism's feeding
  builds biomass only, so all NPCs cross any hunger threshold early and
  live off their bodies thereafter. The NPC economy runs on biomass;
  energy is effectively a played-only ledger. Whether that is the design
  (and decomposer persistence instead wants, e.g., continuous detritus
  income or carrion-timing constants) or NPCs should earn energy like the
  player does is a Mark-level fork, recorded here unruled. Related loose
  end: `dispersal_for`'s bonus still gates on literal zero energy,
  inconsistent with the new hunger threshold.
- **2026-08-29 (later): TD2c's persistence retune landed.** The verdict
  gained `thins` — count held but a founded kingdom is gone at the
  horizon — which honestly re-read TD2b's six "breathes" as producer-only
  stands. Producer supply got headroom over grazing demand (FIXES 2 → 5,
  GRAZES 2 → 3), crowding re-sized for the walled enclosure (CELL back to
  8, COMFORT 1: TD2's doubling only ever compensated for escapees on an
  unbounded plain). Verdicts: 0 breathes / 6 thins / 4 collapse → 0
  breathes / **9 thins / 1 collapse**, zero boils throughout, consumers
  alive at the horizon in 3 seeds (10-23 individuals) where before none.
  Control still collapses; escapees still zero. Receipt:
  `td2c_persistence.json`. The lone collapse (seed 2, consumer-heavy
  draw stripping its base before the first brood) survived every variant
  under three constant regimes — likely legitimate founder variance
  rather than a tuning failure.
- **2026-08-29 (TD2c finding, proven by death-cause probe): decomposers
  cannot search.** Across probed seeds, most decomposer starvations
  happened with carrion lying in the enclosure but outside
  `DECOMPOSE_RANGE 6` — the only radius a scavenger can act on, because
  `preferred_target` caps scavenger *seeking* at bite range while grazers
  and predators seek out to `sight_range`, and `disperse` only wanders at
  exactly zero energy, so a scavenger stands still until half dead.
  `DECAYS_BASE_MG` swept to 6 with no effect — yield is not the binding
  constraint. Fix (TD2d): mirror the grazer's seek/reach split for
  carrion, and let a hungry-but-not-empty body wander. No decomposer
  survives to the horizon in any seed until then, so `breathes` stays
  unreachable by constants.
- **2026-08-29 (later): TD2b's walls and founding floor landed.** The
  enclosure edge refuses a step (wall, not cliff) for every walker — and
  the fix needed three doors closed, not one: `step_for`'s candidates,
  nest routes (burrow mouths and depth drifted up to 12 voxels past the
  bound — a half-unreachable trap once walls exist), and **birth scatter**,
  which threw offspring through the wall with no bound check and was the
  instrument's actual escapee source. Proof across every sample of every
  run: occupied span exactly 16, zero bodies outside; seed 1's end biomass
  fell 4,375x (4.4e9 → 1.0e6 mg) with crowding finally engaged. Genesis
  now founds all three kingdoms every seed (Fisher-Yates floor over the
  three non-played species) and staggers `since_offspring` like `age`.
  Verdicts: 6 breathes / 0 boil / 4 collapse; control still collapses.
  Receipt: `td2b_walls.json`.
- **2026-08-29 (TD2b finding, the next retune's target):** balanced
  founding changed the problem the constants face. In every breathing run
  consumers and decomposers now die to zero by tick 10,000 (pure producer
  stands), and two formerly-breathing seeds collapse from healthy
  ~20/20/18 starts within ~1,000 ticks: a balanced consumer cohort exerts
  predation pressure the TD2 constants — tuned against mostly-producer or
  mostly-empty draws — never saw. TD2c: retune against the new genesis,
  targeting all-kingdom persistence.
- **2026-08-29 (later): TD3's roster landed** — the section shows the
  ants. The tracer gains a second uniform (`@binding(3)`) of up to **40
  roster members at 10 capsules each** (352 B/member, 14,096 B — 86% of
  the 16 KiB downlevel-WebGL2 binding limit the lens's own GL probe holds
  it to; a storage buffer or the 64 KiB desktop limit buys ~4x if that
  reach is ever dropped). Members are silhouettes — no eyes; the
  controlled critter keeps the full single-pose path, which is additive-
  compatible and pixel-identical when the roster is empty (asserted in a
  test). Host feeds every alive organism in the slab window through the
  shared projection. Capture `td3_roster.png`: ~22 visible bodies plus
  the controlled critter (37 submitted; the rest behind terrain). Replay
  hash proven at the pre-retune commit in an isolated worktree
  (`49c7a47f`, exit 0); the live fixture is stale from TD2 by design.
  Measured occupancy that sized the cap: 26-34 of 60 genesis organisms
  in-slab; median body 8-19 living parts. Residues: cap headroom is thin
  (37 of 40 at genesis — resize when TD2b changes occupancy);
  `BodyLensProjection::project` hashes more than a pose needs
  (`project_pose` variant would trim it); `tracer_tests.rs` sits over the
  600-line ceiling from before this work.
- **2026-08-29 (later): TD2's constants landed** — ten seeds go from 0
  breathes / 7 boils to **7 breathes / 0 boils** (remaining 3 collapses are
  founder draws, see the structural findings). Lifespan base 600 → 1800 (a
  1,000mg starter lives 3,000 ticks — five minutes at 10 t/s), gestation
  4x against lifespan's 3x (the knob that decided boil vs breathe),
  producer/grazer income halved-ish, upkeep scale halved (budget 166 → 333
  ticks), crowding cells widened and comfort halved. Collapse control
  still reads collapse. Receipt: `td2_retune.json` beside TD1's
  before-history. Every prior replay hash is now broken by design, per the
  Findings note.
- **2026-08-29:** founded; tempo (both-deliberately, working value 10 t/s),
  instincts-under-idleness, body-routed income, and dynamics-first priority
  all ruled by Mark the same day.

## Findings (structural, from TD2 — constants cannot reach these)

- **The terrarium has no walls.** `Ground::grow` lays bricks over ±16 but
  `Ground::solid` calls everything below y=0 floor forever, so
  `near::step_for`'s doorway/forced-drop branches walk bodies off the edge
  onto an infinite plain (seed 1: 71 escapees by tick 10,000; occupied
  span 16 → ~220). Off the map, crowding — the ecology's only density
  regulator — never engages: producers grow to 10^7 mg, consumers and
  decomposers cannot find food within range. The enclosure is the vessel's
  ruled identity; the edge should refuse a step, wall not cliff. (TD2b)
- **Founding has no kingdom floor.** Each of the 3 non-played species
  draws its kingdom at 4/6 producer, 1/6 consumer, 1/6 decomposer: over
  seeds 1-10 that yields 2 worlds with zero producer species (both
  collapse under any constants), 5 with no consumer species beyond the
  played lineage, 7 with no decomposer. An ant farm is stocked with a
  working web: genesis should guarantee each kingdom at least one species.
  Founders also all start `since_offspring: 0`, gating the first brood in
  any world behind a full gestation; staggering it as `age` already is
  converts at least one collapse seed. (TD2b)
- **The enclosure has a corpse drought, and it is set at founding.** (TD5)
  Founders are staggered over `rng.below(200)` ticks of age against a
  2,000-3,000-tick lifespan, so no founder dies of old age until ~1,800;
  starvation corpses before then hold ≤ `STARVATION_MG` (20 mg) and decay at
  1 mg/tick, so they last twenty ticks. The measured result is an enclosure
  holding **zero carrion from about tick 20 to tick 1,400** and then
  20+ bodies and tens of millions of milligrams forever after — a larder
  that opens long after its only customers are dead (last decomposer: tick
  365-430 across probed seeds). Decomposers cannot be rescued from the
  income side, whatever the routing or the yield, because there is nothing
  to route: `DECAYS_BASE_MG` was swept in TD2c, search was fixed in TD2d,
  and TD5's banking is real but has nothing to bank. Confirmed by
  diagnostic, in a throwaway worktree, deliberately not shipped: widening
  the founding stagger to `rng.below(2000)` puts **42 decomposers alive at
  the 10,000-tick horizon** in seed 7 and multiplies mean decomposer life
  by ~5 in the other two probed seeds. Whether the fix is the stagger, a
  slower carrion decay, a detritus pool that exists before anything dies,
  or a founding cohort that arrives already mid-life is Mark's — it is a
  genesis and matter-cycle question, not a rates.rs one, and against the
  RimWorld bar it is the loop that currently fails to compose: decomposers
  are a kingdom nobody ever sees do their job.
  **Ruled 2026-08-29: the founding cohort arrives mid-life (TD5b).** Age now
  draws proportional to the founder's own `lifespan_for_mass`, not a flat
  200; see TD5b's Progress entry for the carrion-window and verdict
  receipts. Not a full close — seed 9 traded its consumers for its
  decomposers, and the window's first ~20-30 ticks are still structurally
  empty — but the far edge (tick 20-1,400 holding no real carrion) is gone.
- **Individual mass has no fixed point.** Income, upkeep, and the
  reproduction tax all scale as m^0.75, so net growth's sign is
  mass-independent: bodies either grow without bound or shrink to stall;
  no constant choice creates an adult size — only crowding (which counts
  bodies, not mass) can bite. A real fix is a body-plan-derived mass
  ceiling or substrate-limited income — a mechanics design conversation
  for Mark, deliberately not attempted this round.
  **Closed 2026-08-29 (TD6): both fixes landed, and both were needed.**
  Substrate-limited income makes the enclosure's matter a fixed total (end
  biomass 3,267-27,616 mg against TD5's ~4 x 10^10), and the body plan's own
  voxel volume gives every body an adult mass. Crowding survives with its job
  restated: the soil bounds a stand's mass, crowding bounds its number, and
  removing it let a stand answer a finite budget by subdividing into 620
  ever-smaller plants. See TD6's Progress entry.
