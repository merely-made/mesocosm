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

## Findings

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
- **Individual mass has no fixed point.** Income, upkeep, and the
  reproduction tax all scale as m^0.75, so net growth's sign is
  mass-independent: bodies either grow without bound or shrink to stall;
  no constant choice creates an adult size — only crowding (which counts
  bodies, not mass) can bite. A real fix is a body-plan-derived mass
  ceiling or substrate-limited income — a mechanics design conversation
  for Mark, deliberately not attempted this round.
