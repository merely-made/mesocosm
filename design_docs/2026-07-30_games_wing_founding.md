# The Games Wing: Founding Record

**Status: founding record, 2026-07-30.** Wing-level. Owns the shared
architecture, laws, and vocabulary for Mesocosm, Paredros, and Isometry.
Vessel-specific design lives in each repo's own founding plan. This document
is cited by `paredros/design_docs/` and should be cited rather than copied.

Everything here was ruled in conversation on 2026-07-29 and 2026-07-30.
Claims about the existing stack were verified against code and docs on those
dates; each is marked where it matters.

---

## 1. What this is

Three games that are one wing. They share a world substrate, a lineage
model, and a trust plane; they do not share an engine, a genre, or a
schedule.

| Game | Person | Relationship | Design consequence |
| ---- | ------ | ------------ | ------------------ |
| **Mesocosm** | First | I am this body | Movement, consumption, perception, damage, feeding, and growth are experienced directly. |
| **Paredros** | Second | I live with you | Companions can be addressed, persuaded, equipped, and helped. They remain peers, and they can replace you. |
| **Isometry** | Third | They occupy the world | Characters become groups, factions, visible pieces in a shared tactical and historical account. |

**Person is agency, not camera.** Ruled 2026-07-30 after the two vocabularies
collided in a render doc. "Second person" says companions are peers you
address rather than units you command; it says nothing about where the camera
sits, and Paredros may well use a close camera. When discussing renderers, say
**camera distance**; reserve *person* for agency. Conflating them invites
exactly the drift the grammar exists to detect.

The grammar is not decoration. It is the wing's thesis (care at increasing
granularity: species, then individual, then community) and its scope-creep
detector. **Creep is person drift.** Paredros growing party micromanagement
is drifting into third person. Mesocosm growing base-tending is drifting into
second. When a feature changes the person, it is out of scope or it belongs
to a different vessel.

### The influence set

Gotcha Force, RimWorld, Phantasy Star Online, Mount & Blade, Zelda: Four
Swords, FF Crystal Chronicles, XCOM 2, Caves of Qud, Vagante, Tactics Ogre:
The Knight of Lodis, FFTA, Fire Emblem (GBA), Kenshi, Helldivers 2,
Bethesda/Obsidian TES + Fallout, Destiny.

Six through-lines run through nearly all of it, and they are the wing's
design spine: the roster is the protagonist; sortie and return, with the
loot reveal ritualized at home; the body is a build surface and damage is
subtraction; the world runs without you; co-op with friction rather than
merely shared targets; and a toybox surface over merciless systems.

---

## 2. The lifecycle

Genesis (Mesocosm) produces critters. Critters graduate into lives
(Paredros). Lives accumulate into settlements. Settlements export as
campaigns (Isometry). Worlds fork, graft, and federate.

**Spore's stages are the known failure mode**: five shallow genres sharing
nothing, so no stage could deepen another. The insurance is one rule, and it
is the same rule Isometry already runs on (`isometry/CLAUDE.md`: the
substrate/system split is load-bearing).

> **A vessel must not create a private replacement for shared world identity,
> provenance, and causal history.**

Restated 2026-07-30, at the right altitude. The earlier wording ("all stages
are rule-dressings over one substrate; if any stage grows its own engine the
wing hollows out") was directionally right and technically wrong, because it
forbade too much. What hollowed Spore was five stages that shared no *world*,
not five stages that shared no renderer.

A vessel may absolutely have its own renderer, event loop, ECS, camera, or
physics dimensionality. What it may never do is mint a second, private answer
to *who this creature is, where it came from, and what happened to it*. The
sharing rules per layer are tabulated in the
[engine and render lane landscape](2026-07-30_engine_and_render_lane_landscape.md)
§5, and the shared organ that follows from them is planned in the
[body pipeline plan](2026-07-30_body_pipeline_and_host_probe_plan.md).

Consequently: **"shared world model" is too strong** — Mesocosm's live
ecology, Paredros' settlement, and Isometry's campaign state will never be one
in-memory model. They append compatible facts to one world. The shared thing
is the **world identity and fact substrate**.

---

## 3. The three pipeline laws

These govern what crosses between games. They are load-bearing; a violation
is a design bug, not a preference.

### Law A — What travels is choices under scarcity, not morphology

A shape cannot become a faction. What crosses vessels is **meaningful choice
under scarcity**, logged as `(scarcity context, chosen, foregone, cause-link)`.
The *foregone* field is load-bearing: values are revealed by what was given
up. Repeated tradeoffs of the same shape compile into a lineage value; values
drift into ethos; ethos is what Isometry reads as a faction.

Log "chose armor over speed while starving in a cold vent," never "grew
spikes." Otherwise Isometry inherits a generated bestiary and calls it a
history.

**Boundary:** morphology still matters enormously *within* Mesocosm, where
the simulation reads the body directly. Law A governs the across-game axis
only. The two axes meet cleanly because an incorporation event is
simultaneously a morphological act and a value statement. Shapes do not
become factions; shapes become **relics** of factions, and values become the
factions themselves.

### Law B — Inheritance must be pointable

Dwarf Fortress depth that nobody notices reads as procedural noise. Each
lineage carries a **small number of very loud signatures** over unlimited
quiet drift. If a player cannot walk into a village in Isometry and say
"those are mine," the pipeline is invisible and therefore fake. If all they
see is what they made, they have no reason to explore.

The loudness selector already exists in the wing's vocabulary: **Tulpa's
attention mechanic**. Codicil holds everything; Tulpa holds the retold
subset; Isometry surfaces the retold subset as visible signatures; the untold
remainder stays quiet texture for players who dig. Endosymbiosis supplies the
loudest signature class for free, because a faction's famous incorporated
organism is a visible motif.

### Law C — No homework

Spore's early stages were a corridor to the space stage. The fix is
structural, not motivational.

> **Same seed format whether authored by play or by RNG, at every import slot
> in every game.** Player history *displaces* procedural content; it never
> gates it.

Each vessel is standalone-complete: Mesocosm's win condition references no
later game, Paredros is fully playable on RNG worlds, Isometry already
stands alone. Inheritance is enrichment. The proof pair (§6) must
demonstrate displacement explicitly: import a played critter and an RNG
critter through the same profile and confirm the consuming game cannot tell
them apart structurally. Only the player can, by pointing.

---

## 4. Worlds, moots, and lineage

**The moot is the world. A game is one participant and lens upon it.**

This lands on the existing stack without force. "Participant" is the gate
vocabulary (denizen / petition / grant); "lens" is the ortet/ramet
vocabulary. A game walks the participant gate like any other outsider and
renders the world through its own lens. A game's presence in a moot is
**pack-shaped**: schema plus scripts plus a capability manifest, which is
what Isometry's system plugins already are.

Tier mapping, using the platform ladder as-is (`mere/design_docs/TERMINOLOGY.md`):

- solo world = **a mere** (born share-ready; sharing is a membership change, never a format migration)
- shared world = **a moot**
- lineage cluster = **a moothold** when it actively federates
- ecosystem metagame = **a gemot**

**Fili and moothold are different relationships.** Fili records ancestry,
forks, and genealogy. A moothold actively federates. Related worlds form a
fili graph while remaining independent; they become a moothold when they
establish federation.

**The gemot does not own the war simulation.** A gemot provides sovereignty,
admission, shared defaults, and coordination. Ecosystem-scale war facts (the
Helldivers reading) live in a dedicated metagame moot whose projection
aggregates campaign receipts across the gemot. Game rules stay out of
constitutional machinery.

**Identity travels; trust is locally interpreted.** Tessera accrues against a
persona chain root, so standing survives persona forks, but each moot
configures scoring and admission differently. Another game verifies the same
signed evidence and assigns it different meaning. That locality is a feature.

### Provenance rule

Every world is born with stable world identity, seed and content digests, a
rules profile, and optional parent/divergence provenance. In-world
biological descent belongs to the game's own lineage schema. **Fili engages
only at world forks, campaign descent, and cross-moot grafts.** This keeps
Fili's recorded scope exact (`mere/design_docs/TERMINOLOGY.md`: moot lineage,
not event history) while making provenance impossible to lose.

### Graduated interoperability

1. **Observe** — render or inspect a world
2. **Import** — understand creatures, places, maps, items
3. **Contribute** — add facts and artifacts under the moot's rules
4. **Participate** — control an actor or operate part of the simulation
5. **Co-simulate** — share a live runtime when rules and versions genuinely match

These are trust rungs and the test plan at once. Rung 5 is the netcode
dragon; rungs 1–4 carry the entire ecology and need no new machinery.

**Protocol keystone: additive facts, opaque preservation, deferred
interpretation.** Games append history in their own vocabulary and never
mutate foreign facts without a grant. Each game retains facts it cannot
interpret. Re-entry is *interpretation*, not merging: Mesocosm reads "lost an
arm at the ford" from Isometry's appended history and derives the
morphological consequence itself, under its own rules. Conflict is therefore
impossible by construction, which is why this needs no CRDTs and does not
disturb Isometry's standing no-rollback guardrail.

**Authority:** lenses hold revocable per-domain grants. Isometry's shape
generalizes unchanged — a client sends an *ask*, never a verdict, and the
grant holder resolves (`isometry/design_docs/2026-07-14_adjudication_and_representation_plan.md`).

**Time** is causal order; wall time is presentation. Cross-game clocks (a
colony decade against a tactics session) are unresolvable any other way, and
Isometry's per-map tick clocks already taught this at small scale.

**Space** is lens-local. Shared space interoperates at place-graph
granularity only; whether a lens draws diamonds, voxels, or hexes is its own
business. Isometry's tile-geometry seam plan already proved rules can be
geometry-independent.

---

## 5. Verified state of the stack

Checked 2026-07-29/30. Recorded because the first draft of this analysis
overstated it, and the correction matters.

**Ready, with receipts:**

- `mere.pack/v1` is **done**, not embryonic. B4 completed 2026-07-22 (typed
  `PackManifest`, personae signing in `TrustEnvelope`, `verify_pack` with
  Trusted/Unsigned/Broken, tamper cases proven Broken); B5 proved
  distribution over a real retinue link with moot curation and a refused
  mid-flight scope-widening tamper.
  (`mere/design_docs/mere_docs/implementation_strategy/2026-07-17_participant_gate_packs_plan.md`)
- Isometry has landed: typed worldgen (W0–W5: factions, places, characters,
  routes, laws, history, storylets), parties and recruitment, app-side
  adjudication, split-party per-map clocks, a pointcrawl overmap on the graph
  canvas, `isometry-voxel` recipe-not-image appearance, and signed
  multi-writer P2P sessions. C7 factions **landed 2026-07-17/18** once the
  moot/murm rebase cleared.
- Participant gate: denizen / petition / grant / pack / mod, with servitor as
  the installed-helper case.

**Not ready, stated honestly:**

- **The world-noun profile does not exist.** Chartulary is generic
  (containers, facets, nesting, attributed edits) and contains zero faction
  vocabulary; factions, places, characters, and history are Isometry types in
  `isometry-campaign`. A portable profile is **extracted after two real
  consumers**, never declared in advance. Do not describe world nouns as
  "chartulary-typed."
- **The missing arrow is adoption, not wire.** `isometry-campaign` does not
  yet lower to or recover from `mere.pack/v1`.
- **Genuinely new organs** the wing needs and the stack does not have:
  real-time netcode (Isometry is deliberately turn-based with a no-rollback
  guardrail), voxels as playable volume with a live 3D render lane (Isometry
  keeps voxels as asset substrate only), and colony/production simulation.
- **Parked deliberately:** R³ fields (numen is R² today, and numen/quint are
  a field algebra — an influence-map substrate — while seiche is rapier2d
  graph layout, so arena physics means rapier3d directly, with seiche's
  reconcile-a-physics-world-to-a-host-graph pattern as the template).

**Discipline:** the platform is extracted from shipped games, never built
platform-first. Mesocosm is a *candidate* for that proof, not yet a real
second consumer.

---

## 6. The next architectural threshold

Not a federation platform. **A portable world profile proven by two actual
games.**

The proof pair:

1. Mesocosm mints an organism.
2. Isometry imports it as a token, preserving the opaque body plan and fili
   ancestry, rendering appearance through the voxel recipe pipeline.
3. Isometry appends a played history.
4. Mesocosm reads the descendant back without either side losing facts.
5. The same slot accepts an RNG-authored organism indistinguishably (Law C).

This forces **interchange profile v0** into existence, which is the founding
artifact of the wing. Its fields, sharpened 2026-07-30 and detailed in the
[body pipeline plan](2026-07-30_body_pipeline_and_host_probe_plan.md) §3:
body topology (parts, attachment frames, parent/child structure), **per-part
provenance** (what each part used to be — the keystone, and Law A's raw
material), mass and collision hints, fili ancestry, the Law A tradeoff
record, Law B's loud inherited signatures, and **optional** projection
recipes.

Projection recipes are optional on purpose. `isometry-voxel` recipes are an
excellent first projection codec and probably Isometry's, but making them
canonical would let today's renderer leak into the substrate. Each vessel
derives its own presentation from topology.

It rides `mere.pack/v1`, so no new wire format is invented. Keep schema
negotiation microscopic in v0: profile strings, versions, required and
optional capability sets. Resist building a negotiation framework before two
consumers exist.

---

## 7. Vocabulary

Names verified on crates.io and against games, studios, and trademarks on
2026-07-30 unless noted. Stub publication and any trademark filing are the
maintainer's manual step.

| Word | Role |
| ---- | ---- |
| **Mesocosm** | Vessel 1. Ecology's mid-scale enclosed experimental ecosystem. Critters are not cells, so *meso-* is the accurate scale, and each generational run is one experiment in the same enclosure. |
| **Paredros** | Vessel 2. "The one who sits beside": the Greek Magical Papyri's acquired companion, and in classical civic use an assessor seated beside a magistrate. A colleague, not a servant. |
| **critter** | The plain organism word, wing-wide. |
| **animula** | The played soul in Mesocosm: Hadrian's *animula vagula blandula, hospes comesque corporis*, the little soul that guests in a body. In-product term only — **ANIMULA NOOK** is a live Tencent mark in Class 9 game software, so the word must never title a game. |
| **kleptoplasty** | The incorporation mechanic. Real biology: an organism eats algae and retains the functional chloroplasts. |
| **metabolize** | Mesocosm's single verb: world into self, self into world. |
| **fili** | Lineage across worlds. Reserved in `mere/design_docs/TERMINOLOGY.md` for moot ancestry, forks, and genealogy. Not event history, not content descent. |
| **tulpa** | The legend and memorial organ: what memory makes of history, sustained by continued attention. Proposed 2026-07-30, crates.io free, **lexicon inscription still pending the maintainer's ruling.** |
| **deme** | Banked, unspent. Biology's local interbreeding population; the leading candidate for Mesocosm's unit word if one is wanted. |

Also banked clean and unspent: **coppice**, **diaspore**, **holobiont**.
Killed with receipts: *zoophyte* (rejected on taste), *imago*,
*palingenesis*, *ecotone*, *loam* (all occupied), *kleptocosm*, *idiocosm*,
*ipsocosm* (argued down on tone, meaning, and register).

**Collision to respect:** the bare word *flora* is already spoken for
platform-side (a moot's accumulated engrams and geist). Game vocabulary must
not reuse it.

---

## 8. Open questions

Carried forward deliberately. Each needs a ruling before the work it gates.

1. **Licensing — direction set 2026-07-30, exact license still open.**
   Mark's call: **MIT/Apache for the reusable parts, and copyleft
   guarantees for the game roughly equivalent to Cataclysm: Dark Days
   Ahead**, with MPL floated as the alternative. That matches the split the
   radio business already runs (crates MIT/Apache, firmware GPLv3).

   One correction to carry into the decision: **CDDA is CC BY-SA 3.0 for
   code and content both**, which is an unusual choice and not a good model
   to copy literally. Creative Commons themselves advise against CC licenses
   for software — they carry no patent grant and do not address
   source-versus-object distribution, both of which matter for a Rust
   project with dependencies. The equivalent *guarantees* are better reached
   as:

   - **Extracted libraries**: MIT OR Apache-2.0 (the standing convention).
   - **Game code**: **GPLv3** for CDDA-strength copyleft, or **MPL-2.0** for
     weaker file-level copyleft. MPL is already in the stack's vocabulary
     (retinue is MPL-2.0, and the Servo-derived lanes are), so it is the
     lower-friction choice; GPLv3 is the stronger guarantee and matches the
     firmware precedent.
   - **Assets and content**: **CC BY-SA 4.0**, which is what CC licenses are
     actually for, and which delivers the CDDA feel where it belongs.

   Decide before real game code lands, since relicensing after outside
   contributions is painful.
2. **Render and engine stack.** Given its own research doc:
   [`2026-07-30_engine_and_render_lane_landscape.md`](2026-07-30_engine_and_render_lane_landscape.md).
   The earlier framing here (Bevy versus custom wgpu) was a false binary and
   is superseded. Short version: Fyrox is the forkable engine and the only
   Rust one with an editor; "custom" is an assembly in which **eleven of
   thirteen components are already owned** (winit, netrender/vello,
   isometry-voxel, Firewheel, armillary, codicil/muniment, numen/quint,
   cambium, parley, rapier via seiche); the real gap is a 3D renderer and a
   mesher. Renderers need not be shared across vessels — the one-substrate
   law binds the world model, not the pixels — so splitting the bet is live.
   Also carries Mark's proposed dimensionality (Mesocosm 2/2.5D, Paredros 3D
   first person, Isometry 3D third person) **and the flag that the Isometry
   half contradicts a standing ruling in that repo** and needs its own plan
   there.
3. **Paredros' unit word.** "Borg" is chat shorthand — a Gotcha Force loan
   with an IP shadow. The battle-frame noun is unnamed.
4. **Tulpa's inscription and shape**, including the attention mechanic that
   Law B depends on.
5. **Fili v0**: home (beside chartulary and codicil in mere's eidetic
   family), data model, and relationship to `chartulary::stemma` — build
   beside it, not on it.
6. **Attachment-creep levers.** Relationship drift the player cannot
   influence reads as random punishment (the Darkest Dungeon 2 lesson).
7. **Co-op**, deferred to last in every vessel; the shared-burden designs
   (the Crystal Chronicles chalice, Four Swords friction) are unexplored.
8. **The constellation boundary.** Two vessels are named. A colony game or
   strategy game as further vessels is not ruled in; vessels earn existence
   by shipping.
9. ~~Where Gotcha Force lands.~~ **Resolved 2026-07-30**, and my proposed
   answer was wrong in an instructive way. The arena is not a mode and was
   never missing: **the arena is ecological competition itself** — a lineage
   competing for resources and a niche in a strange world. Collection also
   runs the other way round from how I first put it: the *world* collects
   lineages, and the player's displace generated ones, which is Law C seen
   from inside the fiction. Gotcha Force's point budget stays in Mesocosm as
   the **metabolic budget**, not as a roster cap. What genuinely belongs to
   Isometry is **taming** — collecting critters and characters into a faction
   roster, which is befriending a goblin or taming a dog, and which Isometry
   already implements as `convince`. Detail in the Mesocosm founding plan.
10. **Run rhythm across the wing** (2026-07-30). Mesocosm runs generations,
    Paredros runs expeditions against a settlement that keeps (the Heroes of
    Hammerwatch shape), Isometry runs campaigns. One rhythm, three scales,
    and it delivers the sortie-and-return through-line under half the
    influence set. It also produced the wing's first concrete co-op design:
    your character visits another player's settlement, which is
    "bring your own hero" and, at world scale, the graft the lineage model
    already describes.
