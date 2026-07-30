# Mesocosm: Founding Plan

**Status: plan, 2026-07-30. Nothing implemented.** Vessel 1 of the games
wing. Shared architecture, the three pipeline laws, and the wing vocabulary
live in [the games wing founding record](2026-07-30_games_wing_founding.md)
and are not repeated here.

---

## 1. The game

**First person. You are this body.** You are a critter in an enclosed
mid-scale ecosystem. You grow by incorporating other organisms. You die, and
your line continues or does not. Your goal is to increase your kind's share
of the world's biomass, which cannot be achieved by predation alone.

Structurally it is Rogue Legacy's generational run loop pointed at an
ecosystem instead of a castle: **morph yourself, morph the world.** A
successful run establishes the lineage in the persistent world. The world
keeps what your generations did to it.

### The single verb: metabolize

World into self, self into world. One input-level verb covering combat,
growth, building, terraforming, and reef-laying, read at different scales.
Movement, consumption, perception, damage, feeding, and growth are all
experienced directly, never through a menu.

Reference points for verb density: flOw (drift and eat), Vampire Survivors
(decision density over input complexity), Spore's cell stage (the good part).

### Parts come only from incorporation

**This is the keystone.** Spore's cell stage failed as evolution because
parts came from a catalog, so parts were stat sticks and evolution was
shopping. Here:

> You gain parts only by incorporating other organisms. Every part carries
> provenance, because it used to be somebody.

The real biology is endosymbiosis, which is the literal history of the
eukaryotic cell, and its sharpest form is **kleptoplasty**: an organism eats
algae and retains the functional chloroplasts.

The consequence reaches the whole wing. Items and magic in later vessels
**derive** from this rather than being bolted on: a faction's signature
weapon in Isometry is an organism its ancestors ate. An "item" is a creature
record — opaque body plan, fili ancestry, projection profile — so the
interchange profile already carries everything an item needs.

### The flora/fauna loop

The defining cycle. The two kingdoms are the two lineage **strategies**:

- **Sessile producers** (algae, coral, forest): slow, structural,
  high-biomass, terraforming. They make the world habitable.
- **Mobile consumers**: fast, agile, low-biomass. They shape the producers
  by grazing and predation.

Biomass share is therefore unwinnable by consumers alone — not as a tuning
knob but structurally, because someone has to make the biomass. This is what
makes the goal a balancing act rather than a power curve, and it is niche
construction rather than growth: you engineer the world to favor your line.

**Kleptoplasty is the boundary-crossing move.** A consumer line that steals
photosynthesis becomes partly a producer. The incorporation mechanic is also
the bridge between kingdoms, not merely a parts pipe.

*Trophic cascade* is the candidate name for world-morphing-by-eating (remove
a predator and the vegetation changes). Unruled.

**Naming constraint:** the bare word *flora* is spoken for platform-side.
Game vocabulary must not reuse it.

### Multiple lineages

You may switch between lineages. Lines can advantage or disadvantage one
another, so a mature deme may run a producer line and a consumer line that
support each other. Favoritism is real: some lines are outcompeted entirely
and leave only traces, or mutate into other critters. Lineage favoritism is
the first identified attachment-creep lever.

### Death

Dead is dead; lineage persists. Descent requires more than a rebuild — a
line must actually be carried, by offspring or by a tended continuation.
Impact without anyone carrying the line is not descent at all; it is
persistence as **tulpa**. The two organs partition the afterlife cleanly:
**fili records continuity of line, tulpa records what memory keeps.**

### Tone

RimWorld vanilla. Sincere, affectionate, mortal. Dark events may happen;
the endless organ-theft and incest-joke register is out. This matters more
than usual here, because incorporation carries real ethical weight once
critters are not necessarily unintelligent — play it with ritual
seriousness (Qud's water ritual), never as a loot economy.

### Expression

Nonverbal, or Tomodachi Life-style quirk vignettes. Gibberish voice is fine.
Personality renders as movement, habits, and emotes — Rain World is the
proof that per-individual movement personality produces attachment with zero
dialogue.

### Not a chore

The soup is a **place**, not a character creator. It runs autonomously and
produces feral critters on its own; you descend into it to midwife one you
want to be yours. If the loop is good enough to play for its own sake, the
tax inverts into a draw. This is Law C applied locally.

---

## 2. What is genuinely new here

Stated plainly so the cost is visible:

- **Voxels as playable volume.** Isometry keeps voxels as asset substrate and
  explicitly gates 3D lenses behind their own plan and render lane. Mesocosm
  inverts that: the volume is the level and the critter. This is a new
  wgpu-native render lane, and Mesocosm is its pressure vessel.
- **Physics-legible bodies.** Spore shipped 228 parts that resolved to stat
  icons, so the simulation never read form. The fix is that form *is* the
  dynamics input: voxel mass is inertia, part placement is balance and reach
  and hitbox, armor distribution decides where subtraction hurts. rapier3d
  reads the body directly, so form cannot be ignored.
- **Procedural animation over grown morphologies.** Hecker's real-time motion
  retargeting is the prior art for animating bodies nobody has seen.

Available rather than new: `numen`/`quint` is a portable field algebra
(scalar and vector fields, couplings, closed-form evaluation with
finite-difference gradients, Rhai authoring, a Burn lowering with a wgpu
backend) — an influence-map substrate suited to threat fields, flee
gradients, hazard falloff, and drift. `seiche` is rapier2d graph layout, and
its real contribution is the pattern: a physics world continuously reconciled
to a host graph the physics never owns.

---

## 3. Phases

Done-conditions, not estimates. Each phase must be worth playing on its own.

### M0 — The verb
One critter, one enclosure, metabolize. Eat, incorporate, deposit, in 3D
voxel space on rapier3d.

**Done when** a playtester will re-enter the soup voluntarily, without a
goal being offered.

### M1 — Incorporation with provenance
Parts arrive only by incorporation. Each part records what it used to be.
Bodies are physics-legible: mass, balance, and reach follow from placement.

**Done when** two runs that ate differently *play* differently, without a
stat screen being consulted.

### M2 — The deed log
Runs write `(scarcity context, chosen, foregone, cause-link)` entries.
Append-only, codicil-shaped.

**Done when** a run's log reads back as a legible story of what that critter
valued, to someone who did not watch the run.

### M3 — Lineage and the world that remembers
Death, descent, and traces. Successful runs establish a lineage in the
persistent world; the world keeps the marks.

**Done when** a player recognizes their own earlier line's handiwork in a
world they did not expect to.

### M4 — The two kingdoms
Producer and consumer strategies, lineage switching, biomass share as the
win condition.

**Done when** a player deliberately grows a producer line to support a
consumer line, unprompted.

### M5 — Export
Interchange profile v0 over `mere.pack/v1`, and the proof pair with Isometry
(wing founding record §6).

**Done when** a played critter and an RNG critter enter Isometry through the
same slot indistinguishably, and Mesocosm reads the descendant back with
nothing lost.

---

## 4. Findings

*Verified facts discovered during the work, dated, with references. Empty at
founding.*

- **2026-07-30**: `numen`/`quint`/`seiche` live at
  `mere/crates/conatus/{numen,quint,seiche}`; there is no `repos/conatus`
  (absorbed by the 2026-07-23 consolidation). `seiche` wraps `rapier2d`
  0.33 and is graph layout, not a game physics engine. Confirms rapier3d as
  a direct dependency rather than a conatus consumer.

---

## 5. Progress

- **2026-07-30**: repo founded, name reserved, design recorded. No code.
