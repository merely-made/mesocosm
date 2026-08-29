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

- **2026-08-29:** founded; tempo (both-deliberately, working value 10 t/s),
  instincts-under-idleness, body-routed income, and dynamics-first priority
  all ruled by Mark the same day.
