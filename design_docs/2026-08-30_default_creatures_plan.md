# Default Creatures Plan (2026-08-30)

**Status: in progress (2026-08-31). DC1, DC1.5 and DC2 landed; DC3 next, and
§7's first question now has evidence behind it.** Founded on the ruling that closed
the TD series in
[`2026-08-29_terrarium_dynamics_plan.md`](2026-08-29_terrarium_dynamics_plan.md)
§"The series closes here". This plan owns the *body* half of that ruling; the
update stage is the other half and is not this plan.

---

## 0. The ruling

Mark, 2026-08-30, quoted in the terrarium dynamics plan:

> "making creatures just sorta voxel chunks is pretty weird. let's start from
> the presumption that things have senses, limbs, and set roles. let's start
> with default, well tuned creatures with bodies, yes, made of voxels, but a
> bunch of small ones, recognizably flora and fauna like how the isometry
> tokens aren't just sorta cylinders... we're shooting for imagined creatures
> made of voxels here. the game is not beginning at the start of evolution.
> we're not dealing with single cells inventing life's basics. there should be
> some built in expectations of capacities."

And, owning the second playtest's finding
([`2026-08-28_played_slice_plan.md`](2026-08-28_played_slice_plan.md), Findings
2026-08-29): bodies read as "abstract voxel shapes", and the direction was
"shrink those down and put a few together in the shape of a body plan, maybe
you have something that looks more like a critter or flora then."

Two demands, and they are not the same demand:

1. **Founding starts from authored, capable archetypes** rather than a per-tier
   random draw. Senses, limbs and roles are presumed rather than rolled for.
2. **A body is many small voxel parts arranged into a body plan**, at the
   Isometry-token bar, rather than a handful of large blocks.

## 1. What this replaces, and what it keeps

**Replaced: the founding draw.** `axis::seed` (`crates/mesocosm-core/src/axis.rs:386`)
draws one recipe per founding species from a salted stream — a head, one to four
stretches, and an appendage lottery (`rng.below(6)`: 2/6 limb, 1/6 feeler on the
first stretch, 1/6 plate on the last, else bare). That draw is the direct cause
of TD11's seventh finding: sense organs land per *tier*, so five of ten seeds
found a world where **not one body has a sensing part**, and 307 of 3,070
founding fauna can see. Archetypes retire the lottery. `axis::seed` stays in the
tree as the generator it always was — it is what a *soup* world would still use
(founding plan §"Where new lineages come from": any world may begin anywhere on
the trait ladder) — but it stops being how the shipping enclosure founds.

**Kept: the representation.** Archetypes are **data in the existing machinery**,
not a parallel system:

- A `Recipe` (tagmata, appendages, lexicon, variance) is still what a lineage
  carries and what `Species::realize` reads.
- `Soma::develop` still varies an individual off it, so kin still resemble each
  other without being clones.
- `develop_body` is still the one path from recipe to `BodyDocument`, still
  conserving `mass_mg` exactly, still refusing an under-provisioned birth.
- `plan::classify` still reads a role off geometry. Nothing gets a stat flag.

The precedent is already in the tree: `axis::catalogue`
(`crates/mesocosm-core/src/axis/catalogue.rs`) hand-writes centipede,
millipede, insect, spider, tetrapod and snake as in-code `Recipe` constants. Its
own doc comment says "these are reference points, not content"; this plan adds a
**second, sibling set that is content**, and does not repurpose the catalogue —
the catalogue's job is to prove the four axial rules reach real animals, and
these are imagined creatures.

**The enabling ruling already exists.** Founding plan
([`2026-07-30_mesocosm_founding_plan.md`](2026-07-30_mesocosm_founding_plan.md),
§"The authoring caution"): *"author the organisms, generate the arrangements.
That is the wave 2.2 ruling (three authored worlds, not procedural generation)
holding one level further down, at the bestiary."* This plan is that sentence
executed. The Vesta reading it cites — "roughly a dozen organisms each carrying
one strange mechanism followed all the way through: complexity from specificity,
not quantity" — sizes the roster below.

**Two standing rules bind and are not negotiable here.** *Do not let a stage
grow its own engine* and *refuse any shift that needs a second simulation
authority* (`CLAUDE.md`, Important Don'ts). An archetype file that produced
bodies by a route other than `Recipe → Soma → develop_body` would be exactly the
second authority, and would re-open the split account PD0 closed. Every proposal
below stays inside the one path.

**And the phenotype plan's boundary is untouched.** 2026-08-03 ruling: "the
adaptation editor previews a founder phenotype but commits changes to the axial
recipe, `BodyPlan`, process-expression rules... as one heritable program. This
confirms the generator's recipe boundary rather than turning its first rendered
body into a lineage template." An archetype is a *starting* recipe. It is not a
template the epoch boundary must respect, and a lineage that mutates away from
its archetype is the point rather than a defect.

---

## 2. The part-scale arithmetic

**This is the load-bearing section.** S1 ruled "fix the world, not the bodies"
for the *world size* question and gave its reason — `part_ceiling_mg` prices
voxel volume, so shrinking half-extents would drag the economy TD6 and TD7 had
just tuned (`2026-08-29_scale_plan.md`, Findings 2026-08-29). This plan is the
deliberate body-side change S1 deferred, so it owes that arithmetic in full.

All figures below are derived from the formulas in
`crates/mesocosm-core/src/organism/ecology/rates.rs` and
`crates/mesocosm-core/src/organism.rs`, not measured. Measuring them against the
instrument is a done-condition of slice DC2.

### 2.1 The four numbers a body's economy reads

| reading | source | formula |
| --- | --- | --- |
| `part_ceiling_mg` | `rates.rs:119` | `voxels × 100 / 125`, floored, min 1 — **0.8 mg per voxel** |
| `mass_ceiling_mg` | `organism.rs:360` | sum of `part_ceiling_mg` over living parts |
| `actuator_span` | `organism.rs:300` | sum of longest half-extent over parts performing `Contract` |
| `sensor_span` | `organism.rs:327` | the same, over parts performing `Sense` |

and every rate is one of these two shapes (`rates.rs:200`, `build_multiple`):

```text
build_multiple = (ceiling + span × 100) / ceiling
rent   = 1 + m^0.75 × build_multiple / 62          (TD7)
bite   = 3 × m^0.75 × build_multiple / m_ref^0.75  (TD9)
decay  = 4 × m^0.75 × build_multiple / m_ref^0.75  (TD9)
sight  = 8 × build_multiple(sensor_span)           (TD11)
breeding gate = 33% of ceiling                     (TD8)
```

`Role` is read from geometry (`plan.rs:108`) and `Role → Process` is fixed
(`process.rs:54`): **Limb → Contract, Mass → Intake, Sensor → Sense, Plate →
nothing.** So of the four roles, only Limb and Sensor can move an economy
number at all.

### 2.2 The three findings

**Finding 1 — part *count* appears nowhere in the economy.** Not in the ceiling,
not in the build multiple, not in any rate. Ceiling is total voxel volume; span
is a sum over a subset of parts. **A body re-carved into more, smaller parts at
constant total voxel volume has an identical adult mass, an identical breeding
gate, and identical allometric terms.** The only per-part cost is integer
flooring — each part floors its own ceiling independently, so N parts lose up to
N−1 mg against one. On a 1,284 mg body at 33 parts that is at most 2.5%.

**Finding 2 — Mass and Plate detail is free; Limb and Sensor detail is priced.**
Mass has no span term and Plate has no process, so an archetype may spend as
many Mass and Plate parts on its silhouette as the *render* budget allows, at
exactly zero economic cost. Limbs and sensors are counted by their span, and
span is what the whole TD series is normalized against.

**Finding 3 — the economy is scale-free in size and is NOT scale-free in
slenderness.** This is the trap, and it is the one number an archetype palette
must respect. Define a template's **build price**:

```text
build price = 100 × longest_half_extent / part_ceiling_mg
            = 125 × longest_half_extent / voxel_volume
```

It is what one part of that shape contributes to a body's build multiple, per
milligram of the body it hangs on. Ceiling is cubic in half-extent; span is
linear; so **build price scales as 1 / cross-sectional area.**

| template | voxels | ceiling | span | build price |
| --- | ---: | ---: | ---: | ---: |
| limb `[4,1,1]` (the primitive palette) | 81 | 64 | 4 | **6.25** |
| limb `[5,1,1]` | 99 | 79 | 5 | 6.33 |
| limb `[3,1,1]` | 63 | 50 | 3 | 6.00 |
| limb `[5,2,2]` | 275 | 220 | 5 | 2.27 |
| limb `[3,1,0]` (thinned to a 3×1 cross-section) | 21 | 16 | 3 | **18.75** |
| limb `[3,0,0]` (thinned to a 1×1 rod) | 7 | 5 | 3 | **60.0** |
| sensor `[1,1,1]` (the primitive palette) | 27 | 21 | 1 | **4.76** |
| sensor `[1,1,0]` | 9 | 7 | 1 | 14.29 |
| sensor `[1,0,0]` | 3 | 2 | 1 | 50.0 |
| sensor `[0,0,0]` | 1 | 1 | **0** | 0 |

TD7's stated bound — "a body made of nothing but limbs reads `4 × 100 / 64` =
6.25, so no anatomy can price itself past ~7x" (`rates.rs:317`) — is **exactly
the primitive limb's build price**. TD11's "no anatomy may see the enclosure,
46 voxels" (`rates.rs:281`) is `8 × (1 + 4.76)`. Both bounds are properties of
the palette, asserted in doc comments as if they were properties of the game. A
finer palette that thins its limbs raises the first bound from 7× to 20× or 60×
in silence.

Two smaller traps in the same table. `classify` returns `Sensor` only when every
half-extent is ≤ 1, and `Limb` only when one axis is more than twice the others
— so **the smallest legal limb is `[3,1,1]`, seven voxels long.** Limbs cannot be
made much shorter; they can only be made thinner, which is the expensive
direction. And a `[0,0,0]` sensor is legal, one voxel, and contributes span
**zero** (`unsigned_abs().max()` of `[0,0,0]` is 0), so a body of one-voxel eyes
is blind. An archetype's *decorative* eye voxels and its *functional* sense
organs are therefore not the same parts.

### 2.3 The rule this gives archetype authoring

> **Conserve total voxel volume per tier, and keep every Limb template at a 3×3
> cross-section and every Sensor at `[1,1,1]`. Then no `REFERENCE_*` constant
> needs re-deriving, and every TD-series number lands in the band it was tuned
> in.**

The formulas being ceiling-normalized *is* enough — but only because those two
invariants hold. Break the volume invariant and the whole ecology moves in
absolute mass. Break the cross-section invariant and the build multiple moves
while the ceiling does not, which is the drag S1 warned about arriving through a
door S1 did not name.

The invariant should be **asserted, not commented**: `PartPalette::validate`
(`development.rs:75`) already refuses a template whose geometry does not
classify as its role. It should also refuse a Limb or Sensor template whose
build price exceeds the primitive palette's. That is the diagnostics-assert-
invariants pattern, and it turns a doc comment into a test.

### 2.4 The worked comparison

Two carvings of one grazer, matched exactly on total voxel volume. **Carving A**
is a representative body the current `axis::seed` produces; **carving B** is a
proposed archetype. Illustrative, not compile-ready.

**A — today (15 parts).** Recipe `[Tagma(1, Mouth), Tagma(3, Limb),
Tagma(4, None)]`: 9 mass `[2,2,2]` + 6 limb `[4,1,1]`.
**B — archetype (33 parts).** 8 trunk `[2,2,1]`, 1 head `[2,1,1]`, 1 snout
`[2,1,0]`, 2 eyes `[1,1,1]`, 6 legs `[3,1,1]`, 6 feet `[2,1,0]`, 6 dorsal
plates `[3,3,0]`, 3 tail `[2,1,1]`.

| | A (15 parts) | B (33 parts) |
| --- | ---: | ---: |
| total voxel volume | 1,611 | **1,611** |
| `mass_ceiling_mg` | 1,284 | **1,284** |
| distinct part shapes | 2 | 6 |
| `actuator_span` | 24 | 18 |
| `sensor_span` | 0 | 2 |
| build multiple | 2.87 | 2.40 |
| rent at adult mass | 10 mg/tick | 9 mg/tick |
| bite at adult mass | 58 mg | 49 mg |
| sight horizon | **8** voxels | **9** voxels |
| breeding gate (33% of ceiling) | 423 mg | 423 mg |
| birth floor (`biomass/4 ≥ parts`) | parent ≥ 60 mg | parent ≥ 132 mg |
| ticks of reserve at full | 128 | 142 |

Read across: **doubling the part count moved nothing.** The two columns differ
only where B chose a `[3,1,1]` leg over a `[4,1,1]` one and grew two eyes, and
both differences are inside the band TD7 measured for seeded fauna ("roughly 2
to 4 swing per reference segment"). B's build multiple is *lower* than A's, so
if anything the archetype is gentler on the economy than the draw it replaces.

The producer case is stronger still: a producer is all Mass parts and reads
`actuator_span` 0, so its build multiple is exactly 1 and every rate reduces to
the plain `allometric_rate` it has always been. **A voxel fern of 60 small parts
is bit-identical in the ecology to today's 22-block stalk of the same volume.**
Since the pyramid founds 610 of 916 organisms as producers, that is most of what
the eye sees, obtained for nothing.

### 2.5 Where the part count *does* bind

| constraint | threshold | today | at a 33-part archetype |
| --- | --- | --- | --- |
| `MAX_ROSTER_CAPSULES` = 10 — every non-played body | 10 parts | 15 (33% dropped) | 33 (**70% dropped**) |
| `MAX_CAPSULES` = 96 — the played body is not posed at all past it | 96 living parts | ~15 at genesis; **61 measured after ten meals** | 33 at genesis |
| birth at TD8's gate rather than later: `ceiling ≥ 12.1 × parts` | mean part ≥ 15.2 voxels | 107 voxels | 49 voxels |
| fertility at all: `ceiling ≥ 4 × parts` | mean part ≥ 5 voxels | ✓ | ✓ |
| genesis draw `100 + rng.below(400)` ≥ part count | ≤ 100 parts | ✓ | ✓ |
| TD7/TD11 bounds | build price ≤ 6.25 / 4.76 | 6.25 / 4.76 | must be held (§2.3) |

The fertility rows are worth stating because they are silent failures. A birth
realizes the recipe at `cost = parent.biomass_mg() / OFFSPRING_COST`
(`breeding.rs:49`) and `continue`s if `realize` returns `InsufficientMass`
(`breeding.rs:65`) — no matter is created and no entropy spent, so the world
stays sound, but the lineage simply stops breeding. A body whose mean part falls
below **5 voxels** is permanently sterile; below **15.2 voxels** it breeds later
than the gate TD8 tuned. Both are far away at 33 parts. Neither is far away at
300.

**The conclusion of this section: the render budget binds first, by a wide
margin, and the economy does not need re-deriving.**

---

## 3. The render budget

### 3.1 The arithmetic that exists

`mesocosm-lens` traces fragment-only for downlevel reach, so poses live in a
**uniform**, and the binding it must fit is
`Limits::downlevel_webgl2_defaults().max_uniform_buffer_binding_size` =
**16,384 bytes** (`lib.rs:42-80`; `g2_glprobe` requests those limits, so it is a
live ceiling). Two structures share the budget
(`crates/mesocosm-lens/src/tracer/params.rs`):

- `TraceParams` = camera 64 + space + fog 16 + look 16 + `clip_from_world` 64 +
  `CritterParams`. `CritterParams` is bounds 16 + tint 16 + eyes 32 + `32 ×
  MAX_CAPSULES` = **3,136 B at 96**. Header and pose together ≈ 3.4 KiB, about
  **21% of the downlevel limit**.
- the roster buffer = 16 + `MAX_ROSTER × RosterPose`, `RosterPose` = 16 + 16 +
  `32 × MAX_ROSTER_CAPSULES` = **352 B at 10**; total `16 + 40 × 352` =
  **14,096 B, 86.0%** of the downlevel limit.

So with `M` members at `C` capsules each the roster costs `16 + M(32 + 32C)`,
and the budget is:

```text
downlevel (16,384 B):  M × (C + 1) ≤ 511
desktop   (65,536 B):  M × (C + 1) ≤ 2,047
```

| configuration | M × (C+1) | downlevel | desktop |
| --- | ---: | --- | --- |
| 40 members × 10 capsules (today) | 440 | 86% ✓ | 21% ✓ |
| 40 × 11 | 480 | 94% ✓ | 23% ✓ |
| **40 × 33 (an archetype roster)** | **1,360** | **266% ✗** | **66% ✓** |
| 24 × 33 | 816 | 160% ✗ | 40% ✓ |
| 15 × 33 | 510 | 100% ✓ | 25% ✓ |
| 40 × 50 | 2,040 | ✗ | 100% ✓ |

### 3.2 Three facts this exposes

**The played pose's 96 is arbitrary.** `TraceParams` spends only 21% of the
downlevel limit at `MAX_CAPSULES = 96`; the headroom allows roughly **500**
capsules before the downlevel binding is reached. 96 is not a hardware number.

**Past 96 living parts a body is silently invisible.** `BodyLensProjection::project`
returns `TooManyCapsules` (`lens/src/body.rs:74`), and `pose_at` in the host
swallows it — `BodyLensProjection::project(...).ok()` (`genet/src/section.rs:475`),
documented as "a body past the lens's capsule limit yields `None` and the
section traces without it rather than refusing the frame". The first playtest's
body went **52 → 61 parts in ten meals**; five parts a meal reaches 96 in seven
more. This is a code-verified mechanism for the second playtest's finding, *"the
followed critter disappeared while he was not dead"*, whose cause that plan
records as unknown. It is a candidate, not a confirmation: proving it means
replaying `mark_playtest2` and reading the part count on the frame the body
vanished.

**Roster truncation drops the wrong ten parts.** `RosterPose::from_pose` takes
`pose.capsules.iter().take(10)` in document order (`params.rs:121`), and document
order is `BodyDocument::living()`, which is the axial chain from the root
outward. A truncated body is therefore its head end, not its silhouette. At 15
parts that loses a third; at 33 it loses 70% and every non-played critter in the
terrarium reads as a stump — the exact opposite of what the ruling asks for.

### 3.3 What is proposed

Staged, cheapest first. The last stage is Mark's.

- **DC-R1 (free, no budget change).** Order a roster member's capsules by
  descending radius before truncating, so ten capsules spend themselves on the
  silhouette. Surface `capsules_dropped` beside the existing
  `BrickDiagnostics::roster_dropped`, and make `pose_at`'s `None` loud in the
  receipt rather than silent on the frame.
- **DC-R2 (cheap, stays downlevel-safe).** Raise `MAX_CAPSULES` from 96 to
  **256**. Cost: `TraceParams` goes ≈3.4 KiB → ≈8.5 KiB, **52% of the downlevel
  limit**, still fits. This retires the vanish for a played body carrying an
  archetype plus a long career of meals. Raise `MAX_ROSTER_CAPSULES` 10 → 11,
  which is all the downlevel roster budget has (94%).
- **DC-R3 (the ruling).** The downlevel roster budget cannot carry
  many-small-voxel bodies at forty members — it is short by 2.7×, and no
  rearrangement of 511 fixes that. Two answers:
  - **(a) Profile the roster.** Make roster capacity a construction parameter
    with a downlevel-safe default and a desktop profile; Mesocosm's genet host
    requests desktop and gets **40 × 33 at 66% of the desktop limit**, with room
    to reach 40 × 50. Cheap, reversible, and it keeps `mesocosm-lens` honest for
    a future browser tenant instead of quietly dropping the WebGL2 reach the
    tracer was built for. **This is my recommendation.**
  - **(b) Take bodies off the uniform path.** Bodies are voxel volumes and the
    tracer already marches a brick atlas for terrain; voxelizing bodies into a
    second small atlas removes the uniform limit entirely and is what "made of
    many small voxels" literally asks for. It needs per-frame paging for moving
    bodies, whose nearest existing pattern is S2's Paredros-proven
    `BrickMap::with_capacity`/`retarget`. This is the honest long answer and it
    belongs on the scale ladder beside S2/S4, not inside this plan.

  Recommendation: **(a) now, (b) recorded as scale-ladder work.**
- **Colour is a separate question and is flagged, not answered.** A pose carries
  **one tint** (`BodyPlacement::tint`, `CritterParams::tint_count`), so a body is
  one colour whatever its parts. The Isometry bar is a **seven-entry palette on
  one figure**. Three options: a four-entry per-pose palette indexed by the
  part's `Role` (+64 B per pose, derived, no authoring); a full per-capsule tint
  (+16 B per capsule, +50% on the roster — 40 × 33 then needs 2,020 of the
  desktop 2,047, i.e. exactly at the wall); or leave it single-tinted. The first
  is cheap and gets legs, plates and eyes reading differently from the trunk.
  Mark's.

**Note on the existing open ruling.** The rulings register §10 ("Decide how to
lift the 40-body roster cap") asks whether to raise `MAX_ROSTER` or hand far
bodies to silhouettes. That question is about *how many bodies*; this one is
about *how much of each body*. They share the same 511/2,047 budget and should be
ruled together.

---

## 4. The archetype roster

### 4.1 The machinery change archetypes require

**`PartPalette` admits exactly four shapes per world** — one template per `Role`
(`development.rs:32`), and `develop_body` reads `palette.template(role)` for
every part it makes. **A body today cannot contain more than four distinct
shapes**, which is why bodies read as repeated blocks and why no amount of extra
segments reaches the bar. Isometry's reference figure (`isometry-voxel::demo::hero`,
a 10×24×8 humanoid) is built from **fourteen axis-aligned box fills of eight
distinct extents**, plus two one-voxel eyes and a seven-colour palette. Shape
vocabulary, not part count, is the gap.

The minimal change that keeps one authority:

- **`PartPalette` widens to a small fixed array per role** — e.g.
  `mass: [Option<PartTemplate>; 4]` and likewise for limb, plate and sensor —
  staying `Copy`, staying bounded, staying snapshotted with the world. It
  remains world state, which is its stated purpose ("keeping the palette as
  world state means another world can admit different materials without changing
  a lineage's recipe", `development.rs:41`), and it keeps `plan.rs`'s intent
  honest: *if nothing long and thin lives in your world, you cannot grow limbs*.
- **`Tagma` gains two shape selectors** — which template its segments are made
  of and which its appendages are — as small indices into that palette. This is
  the same kind of field `per_segment` already is, it stays serde-stable, and it
  keeps `Recipe` the single heritable program.
- **`PartPalette::validate` gains the build-price check** from §2.3.

Rejected: an archetype format that builds a `BodyDocument` directly. That is the
second authority, and it re-opens the split account PD0 closed.

**This reorders the RNG stream and every existing hash breaks.** Accepted, as it
has been every round this week (scale plan: "hashes break by design"). Fixtures
re-record.

### 4.2 How many, and which

Sized by the founding plan's own Vesta reading — *complexity from specificity,
not quantity*, roughly a dozen organisms each carrying one idea followed
through — and by two structural pressures. Genesis founds exactly **three**
non-played species today (`genesis.rs:91`, one per kingdom), which is why TD10
found kinship's discount cancels: each tier is one interbreeding species. And
the pyramid founds **610 / 229 / 77** producers / consumers / decomposers, so
producer variety is most of what a capture shows.

Proposed: **eight archetypes**, and the number is Mark's.

| tier | count | what each is for |
| --- | ---: | --- |
| Producer (`Symmetry::Radial`) | 3 | a low ground mat; a mid many-fronded shrub; a tall single-stalk with a crown. All Mass parts, so all three are economically free (§2.4) and they carry the terrarium's whole visual field. |
| Consumer (`Symmetry::Bilateral`) | 3 | a browsing hexapod; a low fast pursuit form; a small armoured opportunist. See §4.3 — the grazer/predator distinction is currently blocked. |
| Decomposer (`Symmetry::None`) | 2 | a spreading crust; a mobile detritivore that walks to carrion. |

**No names are proposed.** Naming rounds in this repo are deliberate and carry
crates.io / game / studio / trademark checks (`CLAUDE.md`, Terminology). The
slices below use role-descriptive identifiers (`producer_mat`,
`consumer_browser`, `decomposer_crust`) and **the naming round is flagged as
Mark's** — including whether these want in-world names at all, given that a
founding lineage is deliberately unnamed (`species.rs:66`: "they were there
before anybody arrived to name them").

### 4.3 Two blockers the roster runs straight into

**Every limbed consumer is a predator.** `Organism::feeding_mode`
(`organism.rs:284`) reads `Kingdom::Consumer if self.body.performs(Process::Contract)
=> FeedingMode::Predator`, else `Grazer`. Grazer-vs-predator is today *exactly*
"has a limb or not". The ruling says every archetype presumes limbs — so an
authored grazer becomes a predator and the grazer disappears from the world.
This must be ruled before the consumer archetypes are authored. Options, with
the legwork done:

- Read a different anatomy fact: a mouth's *geometry*. `Appendage::Mouth` maps
  to `Role::Mass` today (`axis.rs:95`); a jaw-shaped mouth (a Limb-classified
  part at the head) versus a cropping mouth (a Mass one) would make the
  distinction geometric, which is the anti-Spore posture the whole role system
  takes. Costs: a mouth that classifies as Limb also adds actuator span, so a
  predator would pay and earn more than a grazer — arguably correct.
- Read the lexicon: a line that can express a jaw kind. Ties feeding mode to
  acquisition, which is the kleptoplasty loop.
- Declare it: split `Kingdom::Consumer`, or carry feeding mode on the `Species`.
  Cheapest, and the one that most looks like a stat flag.

**Symmetry *is* kingdom.** `Kingdom::from_symmetry` (`organism.rs:73`) maps
Radial → Producer, Bilateral → Consumer, None → Decomposer, and `Organism::kingdom`
reads it off `body.plan.symmetry`. So an archetype's silhouette symmetry is not
free: a radial predator or a bilateral plant is currently unrepresentable, and
`BodyPlan::mirrors` only pairs appendages for `Bilateral`, so producers and
decomposers get no mirrored growth at all. Authoring will hit this immediately —
a radial frond wants its fronds around the stalk, which `BodyPlan` cannot
express. Whether symmetry stays the kingdom reading is Mark's; it is a
one-reading change with a wide blast radius (`rates.rs` tests, `genesis.rs`, the
minimap's lineage colouring).

---

## 5. What founding becomes

The shape of `World::with_development_palette` (`world/genesis.rs`) is kept, and
so is every guarantee it currently makes:

- **The pyramid survives exactly.** `pyramid(count)` still composes 2/3 : 1/4 :
  rest, still asserted per seed at 610 / 229 / 77.
- **The kingdom floor survives.** Every tier is still founded.
- **The age and gestation stagger survive**, still proportional to each
  founder's own `lifespan_for_mass` / `gestation_for_mass`.
- **The seeded-stream discipline survives.** Recipes still draw from their own
  salted stream (`RECIPE_SALT`) so body generation never advances the ecology
  stream, and the founder's development seed still comes from `DEVELOPMENT_SALT`.

What changes is one step. Where genesis today maps one species per kingdom and
calls `axis::seed`, it instead **draws an archetype from that tier's archetype
list** off the same salted stream, and installs the archetype's recipe. Species
ids run `2..=(1 + archetype_count)` instead of `2..=4`, so the world founds eight
lineages rather than three — which is also what TD10's structural finding wanted.

**Individual variation still comes from `Soma::develop`**, which is the existing
answer and needs no new mechanism: each founder varies its per-tagma segment
count by the recipe's `variance` and may lose an appendage to developmental
absence. An archetype should set `variance` deliberately rather than inheriting
`axis::seed`'s `1 + rng.below(2)` — a hexapod with a varying leg count is a
different creature, while a frond with a varying frond count is the same plant.

The played critter (`OrganismId(0)`, `SpeciesId(1)`) currently founds with
`Recipe::default_founding()` — one bare four-segment stretch, the simplest body
there is. Under this plan it founds from a consumer archetype like everything
else. Whether the player should start from the *same* archetype every seed, or
draw one, is Mark's.

---

## 6. Slices, with done-conditions

Each slice lands with a receipt in `Code/testing/mesocosm/` and re-recorded
fixtures. Conservation (`cargo test -p mesocosm-core --test matter`) is the gate
on every one of them, at any part scale.

### DC1 — the palette learns more than four shapes

`PartPalette` widens to a fixed array per role; `Tagma` carries shape selectors;
`validate` gains the build-price assertion from §2.3. No archetype yet: the
primitive palette is expressed in the new shape and every existing recipe reads
the same templates it read before.

**Done when:** the primitive palette in the new form produces bodies
**bit-identical** to today's for every seed 1–10 at genesis (the null change is
the receipt); `validate` refuses a Limb template whose build price exceeds 6.25
and a Sensor above 4.76, with a test for each; conservation exact; workspace
tests and `clippy -D warnings` clean.

### DC1.5 — kingdom unbinds from symmetry

Added by §6.5's second ruling, and ordered before any archetype because every
archetype's recipe depends on which anatomy makes it what it is. A fourth
`Process` expressed by `Role::Plate`; `Kingdom` and `FeedingMode` read off
feeding organs; `Kingdom::from_symmetry` retired and `Symmetry` left as pure
body-plan geometry; `axis::seed` drawing a body that reads as the tier it was
asked for, since the pyramid no longer authors a `Kingdom` field.

**Done when:** every founding body in seeds 1–10 reads the kingdom its tier
drew, asserted as a test rather than probed; both consumer readings are
reachable at founding; conservation exact; the ten-seed instrument verdicts no
worse than DC1's (0 breathes / 10 thins / 0 boil / 0 collapse), with movement in
either direction reported rather than absorbed; fixtures re-recorded (hashes
move by design) and `--replay` exiting 0 against a re-recorded trace and 1
against a falsified one; workspace tests, `clippy -D warnings` and `fmt` clean;
and the decomposer's anatomical reading either found and stated, or returned to
Mark unbuilt rather than shipped as a stat flag.

### DC2 — one archetype, measured against the economy

The `consumer_browser` archetype of §2.4's carving B, authored as a `Recipe`
plus its palette entries, installed for the consumer tier only. Everything else
founds as it does today, so the arm is isolable.

**Done when:** the instrument (`population_instrument.rs`) runs its ten seeds
with the archetype arm on and off; the measured `mass_ceiling_mg`,
`actuator_span`, `sensor_span`, build multiple, rent, bite and sight of a
founded archetype body **match §2.4's table**, with any divergence explained
rather than absorbed; the ten-seed verdicts are no worse than TD11's (thins is
the standing baseline, not breathes — the TD ruling retired breathes as a gate);
total matter identical across every sample; **no `REFERENCE_*` or `*_BASE`
constant is touched**, and if one has to be, the slice stops and returns to Mark.

### DC3 — the render budget

DC-R1 and DC-R2 from §3.3; DC-R3 waits on the ruling.

**Done when:** a roster member truncated to its capsule budget is truncated by
descending radius, proven by a capture pair at the same seed and frame;
`MAX_CAPSULES` is 256 and a headed run confirms the uniform binding is accepted
at the downlevel limits `g2_glprobe` requests; a body pushed past the cap is
**reported** in the host receipt rather than silently unposed; the played-slice
demo trace replays to its hash unchanged (this is all presentation and cannot
reach a trace).

### DC4 — the eight archetypes, and the Isometry bar

The full roster, genesis founding one lineage per archetype, the pyramid and
floor asserted unchanged.

**Done when:** every seed 1–10 founds all eight lineages with the pyramid intact
(610/229/77) and the kingdom floor holding; **every founded fauna body has at
least one part performing `Sense` and at least one performing `Contract`** —
TD11's census, run again, reading 3,070 of 3,070 instead of 307; the ten-seed
instrument verdicts and matter conservation hold; and the receipt is a set of
headed captures at the ruled framing that **Mark looks at and calls critters**.
That last condition is a judgment and no test supplies it, exactly as P0's was.

### DC5 — colour, if ruled

Whichever of §3.3's three colour options Mark takes.

**Done when:** the ruled option is implemented, the budget arithmetic in
`lib.rs`'s cap comment is updated to match, and a capture shows a body whose
legs, plates and sense organs read differently from its trunk.

---

## 6.5 The two blockers, ruled (2026-08-30, Mark)

Both §4.3 blockers were ruled the day the plan landed:

- **Grazer vs predator: mouth geometry.** A jaw — a Limb-classified mouth
  part at the head — makes a predator; a cropping, Mass-classified mouth
  makes a grazer. Geometric, the same anti-Spore posture the role system
  takes, and a jaw adds actuator span so a predator pays and earns more,
  which is correct.
- **Kingdom unbinds from symmetry.** The deeper fix, taken deliberately:
  kingdom becomes a reading of feeding anatomy rather than of
  `body.plan.symmetry`, and symmetry becomes pure body-plan geometry. This
  pulls the forms brief's Stage 1 in as a dependency — a producer must be
  readable from *fixing anatomy*, and `Role::Plate` ("Fins, plates,
  leaves") is the already-named geometry for it. The decomposer's anatomy
  reading is the open design question the implementing slice must surface
  if no honest reading exists. Known consequences, priced in the forms
  brief's Stage 1 cost list and accepted: replay hashes move
  (`feeding_mode` reaches `state_hash` via the fauna decision trace),
  `CohortKey` semantics, `betrays_itself`, and guise's referent. The
  mixotroph-as-prey question (register §13) stays deferred — archetypes
  are single-mode, so edibility keeps reading the derived kingdom until a
  body can honestly do both.

The slice order gains a step: the unbinding is DC1.5 — after the palette
widens and before any archetype is authored, since every archetype's
recipe depends on which anatomy makes it what it is.

## 7. Open questions — Mark's

1. **How many archetypes, and are eight the right eight?** §4.2 proposes 3/3/2.
2. **What separates a grazer from a predator once every consumer has limbs?**
   §4.3, first blocker. This one blocks DC4 and has no defensible default.
3. **Does symmetry stay the kingdom reading?** §4.3, second blocker. It
   constrains every archetype's silhouette and it is not obviously worth its
   cost.
4. **The render ruling: profile the roster for desktop, or move bodies off the
   uniform path?** §3.3 DC-R3. Recommendation on record; the choice is not mine.
   Rule it together with the rulings register's §10.
5. **Colour: per-role palette, per-capsule tint, or stay single-tinted?** §3.3.
6. **Does the played critter start from a fixed archetype or a drawn one?** §5.
7. **Do the archetypes get in-world names, or stay unnamed the way founding
   lineages deliberately are?** §4.2. Either way the naming round is yours; no
   names are proposed here.
8. **Does `axis::seed` stay in the tree as the soup world's generator, or go?**
   §1 assumes it stays. It has no other caller once genesis stops using it.

---

## 8. What this plan does not do

- **The update stage.** The other half of the closing ruling — each lineage
  evaluating and refining its bodyplan and trajectory at the epoch boundary — is
  PS2 and the epoch boundary plan's, not this one. This plan gives that stage
  something worth refining; it does not build it.
- **Evolution mechanics.** NPC lineages still never mutate their recipes
  (`World::learn_from` returns early for anything but the controlled critter).
  That is the traits brief's incorporation half and stays open.
- **New part types.** No new `Role`, no new `Appendage`, no new `Process`. The
  archetypes are built from the four roles and six appendages that exist. If an
  archetype genuinely cannot be expressed, that is a finding to bring back, not
  a licence to add a role.
- **World size, framing, or the scale ladder.** S2/S3/S4 are untouched. The
  traced-body option in §3.3(b) is *recorded* for that ladder, not scheduled
  here.
- **The economy's constants.** DC2's done-condition is explicitly that no
  `REFERENCE_*` or `*_BASE` constant moves. If the measurement says one must,
  the slice stops.

---

## Findings

- **2026-08-31 (DC3): the vanish is confirmed, on a live body, by
  counterfactual.** §3.2 called it a candidate. A headed run grew the played
  critter to **349 living parts** and the trace of it replays exactly
  (`dc3_vanish.trace.json`, `97bf29f3ca1fd587`). At `df2a741` that replay draws
  **no body at all** while the receipt says `body_parts: 349` and the minimap
  shows the critter alive; after DC3 the same replay draws it truncated and
  reports the 93 parts it could not carry. Cause and cure both proven on the
  same trace. `dc3_vanish_before.png`, `dc3_vanish_after.png`.

- **2026-08-31 (DC3): the `mark_playtest2` trace no longer grows the body it
  grew.** DC1, DC1.5 and DC2 all moved founding, so the preserved trace
  diverges from its recorded hash immediately; its critter reaches 58 parts by
  frame 10 and is dead by frame 30, ending `body_parts: 0`. It is still a good
  world driver — the roster capture pair uses it — but it is no longer evidence
  about *that* body, and the vanish had to be re-grown to be shown. Playtest
  traces are worth re-recording when founding moves.

- **2026-08-31 (DC3): descending radius keeps the bulk and drops the fronds,
  and on a stand of producers that is not obviously prettier.** DC-R1 landed
  exactly as §3.3 specifies and the change is unambiguous — at one seed and
  frame the visible stalks go from about 25 to about 13 — but the parts it
  drops are the tall thin `[4,4,1]` fronds and the parts it keeps are the round
  root masses. Silhouette *area* is what the argument asks for and what it
  delivers; silhouette *reading* may want **extent** (`radius² × length`) as
  the key instead, which is the same four lines. Ordering key returned to Mark
  rather than substituted. `dc3_roster_before.png`, `dc3_roster_after.png`.

- **2026-08-31 (DC3): one capsule cannot carry the primitive `Plate`.** The
  radius rule fixed the archetype's six shapes but `[4,4,1]` is 4:1
  anisotropic, and the mean takes it from radius 1 to **2.5** — the standing
  crop goes from thin blades to thick columns. A geometric mean gives 2.0 and
  is no better; there is no single radius that represents a 9×9×3 slab. The
  honest answer is a second capsule for plate-shaped parts, which is a
  projection redesign and belongs with DC5's colour question rather than in
  DC3. `lens/src/body.rs::capsule_for`.

- **2026-08-31 (DC2): carving B's exactness came from its six dorsal plates,
  and post-DC1.5 a consumer cannot wear them.** `Role::Plate` performs
  `Process::Fix` now, so §2.4's "6 dorsal plates `[3,3,0]`" would make the
  browser read **Producer** rather than Consumer — the armour collision DC1.5
  named as a standing constraint, hit on the very first archetype exactly as it
  predicted. Replacing them is not free arithmetic either. Holding *both* of
  carving B's totals (1,611 voxels **and** 1,284 mg) requires some part whose
  voxel count is not a multiple of five, because `part_ceiling_mg` is
  `voxels * 4 / 5` floored and 4 x 1,611 / 5 is not an integer; the plates were
  the only fine-grained parts doing that. **The smallest `Mass` shape whose
  volume is not a multiple of five is `[3,3,3]` — 343 voxels, 274 mg**, a
  seventh of the whole body in one block, because every smaller candidate
  (`[3,1,1]`, `[3,3,1]`, `[1,1,1]`, `[3,3,0]`) classifies as Limb, Plate or
  Sensor rather than Mass. So the choice was 1,284 mg with a chunky body or a
  fine-grained body one or two voxels off the volume, and the slice took the
  milligram: every *rate* in §2.4's column reads off the ceiling and the spans,
  and none reads off the voxel count directly. `axis/archetype.rs`, §2.2.

- **2026-08-31 (DC2): a decorative sense voxel is also an arithmetic tool.**
  §2.2 named `[0,0,0]` as a legal one-voxel `Sensor` contributing span **zero**,
  and it turns out to be the one part in the vocabulary that can move a body's
  ceiling by a single milligram without moving any span. Two of them are what
  land the browser on 1,284 mg exactly at 33 parts; without them the same
  ceiling forces a `[3,3,3]` block and the part count falls to 15, which is
  carving A's. `archetype::SPECK`.

- **2026-08-31 (DC2): the authored grazer is not worse than the naive one; the
  *tier* is.** Ten seeds with the consumer tier founding the browser:
  **1 breathes / 5 thins / 0 boil / 4 collapse**, against DC1.5's naive mobile
  grazer at 1/6/0/3 and the standing baseline's 0/10/0/0 — the same regime, one
  seed either way, and it is a regime the body cannot leave. Measured beside it,
  the reason: **DC1.5's transitional draw founds a *predator* tier in nine of
  ten seeds** (ceiling 1,420-3,148, actuator span 20-128, sensor span **zero**
  — the drawn consumer is blind), and the tenth (seed 2) is a *sessile* grazer,
  which is the one baseline seed where consumers survive to the horizon. The
  archetype arm founds 230 mobile grazers in *every* seed, at a breeding gate of
  423 mg against the drawn consumer's 468-1,038, with a working eye (sight 9
  against 8) and the whole 610-strong stand as food. A tier that is one
  interbreeding species (TD10) therefore cannot hold both readings, so DC2's arm
  is an all-grazer world by construction. **This is a roster question, not a
  body question**, and it is DC4's to answer: §4.2's consumer tier is three
  archetypes, and a founding that installs the browser without the pursuit form
  beside it is measuring half a roster. `dc2_browser.json`.

- **2026-08-31 (DC2): what a collapsed world does is empty into the ground.**
  Seed 4 under the arm decides at tick 500 and ends 0/2/15 P/C/D with soil at
  2,184,886 mg of a 2,202,302 mg total — **99.2% of the world's matter in the
  ground** — against the same seed's baseline 1158/0/71 and 93,651 mg of soil.
  Seed 9 is the same shape (2,184,330 of 2,209,340). Matter is conserved to the
  milligram in every sample of every run, both arms, so the collapse is a
  redistribution rather than a leak: the grazers strip the stand faster than it
  regrows, die, and nothing is left to lift the matter back out of the soil.

- **2026-08-31 (DC2): the lens flattens the whole widened palette to one
  radius.** `BodyLensProjection::capsule_for` (`lens/src/body.rs:174`) reduces a
  part to an axis, a run, and **one radius — the smaller of the two cross-axis
  half-extents, floored at 1**. Every shape the archetype uses (`[2,1,1]`,
  `[2,2,1]`, `[2,1,0]`, `[3,1,1]`, `[1,1,1]`, `[0,0,0]`) has a cross-section of
  1, so all six project to radius-1 capsules differing only in length, and the
  one-voxel decorative speck draws exactly as large as a working eye. The
  primitive `[2,2,2]` segment projects at radius 2, which is why a
  many-small-parts body reads *thinner* than the blocks it replaced rather than
  more detailed. **DC1's shape vocabulary cannot reach the eye through this
  projection**, and neither can DC4's roster: this belongs beside DC3's render
  work and DC5's colour question, and it is cheaper than either (the radius rule
  is four lines). `lens/src/body.rs`, `dc2_browser.png`.

- **2026-08-30 (founding): part count appears nowhere in the body economy.**
  `mass_ceiling_mg` is total voxel volume × 0.8; the build multiple is a span
  sum over Limb and Sensor parts only; Mass has no span term and Plate has no
  process at all. A body re-carved into more, smaller parts at constant total
  voxel volume is economically identical. §2.2.

- **2026-08-30 (founding): the TD7 and TD11 bounds are properties of the
  palette, written as if they were properties of the game.** "No anatomy can
  price itself past ~7x" is `1 + 100 × 4 / 64`, the primitive limb's build price;
  "no anatomy may see the enclosure, 46 voxels" is `8 × (1 + 100 / 21)`, the
  primitive sensor's. Build price scales as `1 / cross-sectional area`, so a
  palette that thins its limbs from a 3×3 cross-section to 3×1 raises the first
  bound from 7× to 20×, and to 1×1 raises it to 61×, silently. The guard is one
  inequality and it belongs in `PartPalette::validate`. §2.2, §2.3.

- **2026-08-30 (founding): a body cannot contain more than four distinct
  shapes.** `PartPalette` holds exactly one `PartTemplate` per `Role` and
  `develop_body` reads `palette.template(role)` for every part. That, not part
  count, is why bodies read as repeated blocks: Isometry's reference figure uses
  eight distinct box extents and seven colours on one 10×24×8 model, and
  Mesocosm's bodies use one shape and one colour. §4.1.

- **2026-08-30 (founding): past 96 living parts the played body is silently not
  drawn.** `BodyLensProjection::project` returns `TooManyCapsules`
  (`lens/src/body.rs:74`) and the host's `pose_at` maps it to `None`
  (`genet/src/section.rs:475`), documented as tracing the frame without it. The
  first playtest's body reached 61 parts in ten meals. This is a code-verified
  mechanism for the second playtest's "the followed critter disappeared while he
  was not dead", whose cause that plan records as unknown — a candidate to prove
  against the preserved `mark_playtest2` trace, not a confirmation. §3.2.

- **2026-08-30 (founding): a roster member is truncated to its first ten parts
  in document order**, which is the axial chain from the root outward, so a
  truncated body is its head end rather than its silhouette
  (`params.rs:121`). Reordering by descending capsule radius costs nothing and
  is a strict improvement at any part scale. §3.2, DC-R1.

- **2026-08-30 (founding): the downlevel roster budget cannot carry
  many-small-voxel bodies.** `M × (C + 1) ≤ 511` at the WebGL2 uniform limit
  against `40 × 34 = 1,360` needed — short by 2.7×, and no rearrangement of 511
  fixes it. Desktop's `≤ 2,047` carries it at 66%. Meanwhile the *played* pose
  spends only 21% of the downlevel limit at `MAX_CAPSULES = 96`, so that
  particular constant is arbitrary and has room to roughly 500. §3.1, §3.2.

- **2026-08-30 (founding): grazer versus predator is currently "has a limb or
  not".** `Organism::feeding_mode` reads `body.performs(Process::Contract)`
  (`organism.rs:288`). The ruling presumes limbs on every archetype, so
  authoring capable consumers deletes the grazer from the world unless the
  distinction moves first. No default is defensible; it is ruled or the roster
  waits. §4.3.

- **2026-08-30 (founding): symmetry *is* kingdom.** `Kingdom::from_symmetry`
  maps Radial → Producer, Bilateral → Consumer, None → Decomposer, and
  `BodyPlan::mirrors` pairs appendages only for Bilateral. Producer and
  decomposer archetypes therefore get no mirrored growth and no radial
  arrangement mechanism, which authoring will hit on the first frond. §4.3.

- **2026-08-30 (founding): the classifier floors how small a functional part can
  be.** `classify` reads Sensor only when every half-extent is ≤ 1 and Limb only
  when one axis exceeds twice the others, so the smallest legal limb is `[3,1,1]`
  — seven voxels long. Limbs can be made thinner but barely shorter, and thinner
  is the expensive direction. Separately, a `[0,0,0]` sensor is legal, is one
  voxel, and contributes span **zero**, so decorative eye voxels and functional
  sense organs are not the same parts. §2.2.

- **2026-08-30 (founding): a body of very small parts becomes sterile, quietly.**
  A birth realizes the recipe at `parent.biomass_mg() / 4` and `continue`s on
  `InsufficientMass` (`breeding.rs:65`) — sound, but the lineage stops breeding.
  The thresholds are mean part volume **5 voxels** (sterile) and **15.2 voxels**
  (breeds later than TD8's gate). At 33 parts on today's volume the mean is 49
  voxels, so both are far off; at 300 parts they are not. §2.5.

- **2026-08-30 (DC1): the §2.3 guard caps limb *length*, not only thinness.**
  Holding build price at or below the primitive limb's 6.25 refuses `[5,1,1]`
  (6.33) and `[6,1,1]` (6.45) as well as `[3,1,0]` (18.75), because span is
  linear in the long axis while the ceiling is cubic in all three. **At a 3x3
  cross-section the longest admissible Limb is `[4,1,1]`**; a longer limb has
  to be thicker, e.g. `[5,2,2]` at 2.27. This is a constraint on archetype
  authoring that §2.2's table implies but does not state, and DC2's legs
  (`[3,1,1]`, 6.00) sit inside it. `development.rs`, `overpriced`.

- **2026-08-30 (DC1): widening the palette is free at the ecology scale, not
  merely at the body scale.** The ten-seed instrument's per-sample curves —
  alive counts, biomass, births, deaths, soil, total matter — are byte-identical
  to TD11's across all ten baseline seeds and all five control seeds, not just
  equal in verdict. The snapshot format moved (`RoleShapes` and two `Tagma`
  fields), so `state_hash` moved and the demo trace re-recorded; nothing the
  simulation computes moved with it. `dc1_palette.json` against
  `td11_chain.json`.

- **2026-08-31 (DC1.5): the unbinding leaves the verdicts alone and leaves more
  of the world alive.** Ten seeds, same horizon, same constants: **0 breathes /
  10 thins / 0 boil / 0 collapse**, identical to DC1's, control all collapse,
  matter conserved to the milligram in every sample and identical per seed, zero
  escapees. What moved is underneath the verdict. **Decomposers survive to the
  10,000-tick horizon in 9 of 10 seeds against DC1's 5**, and standing biomass
  is up in 8 of 10 (seed 8: 370,074 -> 1,131,018 mg; seed 9: 381,867 ->
  864,135; seed 2 is the one that fell, 691,333 -> 389,601). Consumers still die
  out in 9 of 10, unchanged. The likely mechanism is that a fixing part brings
  its own ceiling, so the stand is made of bigger producers that hold more
  matter in bodies rather than in the ground — the second-order effect the forms
  brief's §A predicted, landing in the direction it hoped for. `dc15_kingdom.json`
  against `dc1_palette.json`.

- **2026-08-31 (DC1.5): the decomposer's anatomy is the *absence* of a feeding
  organ, and that is an honest reading rather than a shrug.** §6.5 named this
  the open design question. There is no positive saprotroph organ to read and
  inventing one would have been a stat flag with a shape on it, but the reading
  was already in the tree: `Process::Intake`'s own doc comment is "a mouth, a
  gut, **an absorbing surface**", and every body's bulk segments perform it. A
  saprotroph digests outside itself and takes the result in across its whole
  surface — no ingesting organ, which is exactly what the real ones do. So
  producer and consumer are positive readings (a part that performs `Fix`; a
  mouth borne under the head) and **decomposer is what a body reads as when it
  carries neither**. It costs no machinery and it is stated as precedence rather
  than discovered. The consequence to watch is that the residual is also what a
  body reads as after it *loses* its mouth: `organism/kingdom.rs`.

- **2026-08-31 (DC1.5): expressing the fixing process by `Role::Plate` makes
  every flat part a leaf.** The role's doc reads "Fins, plates, leaves" and the
  ruling took it as the named geometry, but the three are now one process:
  armour fixes, and a fin fixes. The founding consumer draw therefore had to
  drop its plate arm — a consumer that grew one would be the deferred mixotroph
  — so **founding consumers can no longer be armoured at all**. That is a
  standing constraint on DC4's roster: no archetype can carry a shell without
  reading as a producer until either the reading separates a frond from a plate
  (a second geometry, or a position rule) or the mixotroph question is ruled.
  `process.rs`, `axis.rs::seed`.

- **2026-08-31 (DC1.5): developmental absence could take a mouth, and with it a
  kingdom.** `Soma::develop` drew a 1-in-12 absence per appendage-bearing tagma,
  and the mouth sits on the head tagma, which the founding draw gives one
  segment. Under the new reading that was a 1-in-12 chance for **every consumer
  lineage** of realizing a body with no mouth — a founder born a decomposer into
  a consumer's tier, and the census failing at ~8%. Guarded: absence may not
  take a feeding organ, because an individual missing one limb is variation and
  an individual missing its mouth is a stillbirth. `axis.rs::Soma::develop`.

- **2026-08-31 (DC1.5): a bare root reads Decomposer, so the fixture
  constructor had to grow anatomy.** `Organism::founding` is called from 44
  sites across tests, examples and the lens fixtures, every one of them handing
  it a `Kingdom` and expecting the body to be one. Post-unbinding a single root
  part reads Decomposer whatever it was asked for, so the constructor now
  attaches the smallest organ of the role its kingdom names — a `[3,2,1]` frond
  above, a `[2,1,1]` crop or a `[3,1,1]` jaw below — with the milligram taken
  out of the root, so the body still weighs what it was given. **A consumer gets
  a jaw when its own root is Limb-classified and a crop otherwise**, which
  reproduces every existing fixture's feeding mode exactly, since the old rule
  was "does any part perform `Contract`" and a fixture's only part was its root.
  Two fixtures still moved measurably and are updated in place: a hunter's
  walker height (a jaw hangs below the head) and a metabolize test whose literal
  `[5,0,0]` offset no longer cleared the body the seed draws. `organism.rs`.

- **2026-08-31 (DC1.5): founding *mobile grazers* costs three seeds in ten —
  measured, not argued.** The first cut of the transitional draw took the
  consumer head's geometry from an independent coin flip, which founds something
  the world has never had. Before DC1.5, `Grazer` meant "no living part performs
  `Contract`", which also meant **sessile**: a grazer could not cross the
  enclosure, so its pressure was local by construction. A crop-mouthed line that
  draws limbs can, and a grazer eats *only* producers (`movement.rs:68`), so all
  230 consumers' appetite lands on the stand and nothing eats them back. Ten
  seeds under the coin flip: **1 breathes / 6 thins / 0 boil / 3 collapse**,
  against DC1's 0/10/0/0. The correlation is clean — the three collapses (seeds
  4, 6, 7) are all grazer-tier worlds, and the two predator-tier worlds (1 and
  2) read *breathes* and *thins*, seed 1 being the only breathes in the set.
  Collapsed worlds empty into the ground: seed 4 ends 6/12/0 P/C/D with soil at
  2,079,663 mg of 2,202,302 total, against DC1's 1430/0/0 and 71,814 mg of soil.
  The draw was therefore changed to the null-change rule — **the mouth follows
  the legs** — and founding a mobile grazer is left for DC2 to do deliberately
  with the instrument watching. This is the number a grazer archetype has to
  beat, and it is the first hard evidence that §4.3's first blocker was not only
  a legibility problem.

- **2026-08-31 (DC1.5): incorporation can now mint a mouth, and severing can
  take one.** `growth::resolve` places a Mass-classified part at the plan's Mass
  preference, which defaults to `Below`, nearest the root first — precisely the
  attachment the reading calls a mouth. So a body that eats something bulky
  grows a feeding organ and changes kingdom, and a body whose mouth is severed
  or arrives severed through a `Chronicle` loses one. Both are the point of a
  reading rather than a field, and both are **latent in the shipping loop**: no
  ecology rule severs a part, and only the played critter incorporates. The
  fuller kleptoplasty payoff the forms brief's §A wants is *not* reachable yet —
  `land` grafts the eaten organism as one part shaped like its **root**, and
  every root in a founded world is the Mass template, so nothing can be eaten
  into a fixing part. `growth.rs`, `world/act.rs::land`, `chronicle.rs`.

## Progress

- **2026-08-31 (DC3): the render budget, and the radius rule with it.** Landed
  against §6's DC3 done-conditions. DC-R1 and DC-R2 from §3.3; **DC-R3 is
  untouched and still Mark's**. `mesocosm-core` was not opened.

  **DC-R1, truncation by descending radius.** `RosterPose::from_pose`
  (`lens/src/tracer/params.rs`) picked a member's first ten capsules in
  document order, which is the axial chain from the root outward, so a
  truncated body was its head end. It now keeps the widest capsules — ordered
  by the fatter of the two endpoint radii, ties going to the earlier capsule,
  then re-sorted back into document order so the upload is stable frame to
  frame. Bodies inside the budget take a length comparison and no allocation.
  Capsules the budget could not carry are counted in a new
  `BrickDiagnostics::roster_capsules_dropped` beside the existing
  `roster_dropped`, and the host puts both in its receipt.

  **DC-R2, the numbers.** `MAX_CAPSULES` 96 → **256** and
  `MAX_ROSTER_CAPSULES` 10 → **11**, exactly §3.3. Measured rather than
  recited, and the binding test in `tracer/params.rs` now asserts each one:
  `CritterParams` 3 136 → **8 256 B**, the whole frame uniform **8 464 B =
  51.7%** of the 16 384 B downlevel binding (§3.3 estimated 52%); `RosterPose`
  352 → **384 B**, the roster binding 14 096 → **15 376 B = 93.8%**; and
  `M × (C + 1)` is 480 against the 511 ceiling, where 12 capsules each would be
  520. The heightmap lane's hardcoded `192` in `renderer.rs`, `helpers.rs` and
  `march.wgsl` is gone: the array size is injected from `MAX_CAPSULES` the way
  the tracer already injects the roster's, so the two layouts cannot drift.
  `g2_glprobe` accepts the new binding at `downlevel_webgl2_defaults`, byte for
  byte the same receipt it wrote before.

  **The vanish, retired, and caught in the act.** `BodyLensProjection` gained
  `project_truncated` — additive, `project` refuses past the cap exactly as it
  did, so `paredros-room --features r1-proof` still checks clean. The host's
  `pose_at` (`genet/src/section.rs`) uses it: a body past the budget is drawn
  truncated to its widest parts and the overflow is reported, where the old
  `.ok()` turned the refusal into no body at all. **The receipt is a
  counterfactual on one trace.** A headed `--auto-eat` run grew the played body
  to **349 living parts** (`dc3_vanish.trace.json`, hash `97bf29f3ca1fd587`,
  replays exactly). Replayed at `df2a741`: hash matches, `body_parts: 349`, the
  minimap shows the critter alive — and the section draws **nothing**
  (`dc3_vanish_before.png`, 211 body-tint pixels, all of them the minimap dot).
  Replayed after: same hash, same 349 parts, the body on screen
  (`dc3_vanish_after.png`, 2 092 pixels) and the run says out loud "body: 93 of
  349 parts past the lens capsule budget, drawn truncated to its widest". That
  is §3.2's candidate mechanism confirmed on a live body, not a hypothesis.

  **The preserved `mark_playtest2` trace no longer reaches 304 parts**, and
  this is worth recording rather than working around. Replayed today it
  diverges at once (DC1, DC1.5 and DC2 all moved founding), the played critter
  reaches 58 parts by frame 10 and is dead by frame 30, and the run ends
  `body_parts: 0`. The trace is still a good *world* driver — it is what the
  roster capture pair below uses — but it can no longer grow the body it grew,
  so the vanish had to be demonstrated on a trace recorded today.

  **The capsule radius rule** — DC2's finding, the four lines it called the
  cheapest legibility here. `capsule_for` took the *smaller* of the two
  cross-axis half-extents and floored it at 1; it now takes their **mean**,
  floored at half a voxel, and the run is measured from that. On the
  archetype's six shapes that is three radii where there was one: speck
  `[0,0,0]` and crop `[2,1,0]` at 0.5, eye `[1,1,1]` / slim `[2,1,1]` / leg
  `[3,1,1]` at 1.0, broad `[2,2,1]` at 1.5. The primitive `[2,2,2]` segment is
  unchanged at 2.0, so nothing the palette already drew got fatter — except one
  thing, below.

  **What the two capture pairs actually show, honestly.**

  *The radius rule* (`dc3_radius_before.png` / `dc3_radius_after.png`, the
  `menagerie` example's DC2 arm at the same framing). It is still a segmented
  bead-chain, still one tint, and the legs still read as bumps — DC2's verdict
  stands and DC4's bar is not met. But two things changed and both are the
  right direction: the cropping mouth was a bead the same size as the trunk and
  is now a thin flat blade lying under the head, so it reads as a *different
  organ*; and the broad segments now bulge against the slim ones, so the body
  has a thorax instead of a constant diameter. The decorative specks shrank to
  nubs and no longer draw as working eyes, which was the finding's sharpest
  complaint. So: more legible, not yet a creature.

  *The truncation order* (`dc3_roster_before.png` / `dc3_roster_after.png`,
  `mark_playtest2` replayed to frame 400, DC-R1 alone with nothing else
  changed). The change is unambiguous and **its aesthetic sign is not.** These
  forty roster members are producers: a root mass at ground level with tall
  thin `[4,4,1]` fronds above it. Descending radius keeps the fat root blobs
  and drops the fronds, so the stand goes from about twenty-five visible
  stalks to about thirteen and reads lower and rounder. The plan's argument is
  about silhouette *area* and area is what it delivers; whether a stand of
  bulbs reads better than a stand of blades is a judgement, and on this
  particular frame I would not claim it does. **A candidate refinement for
  Mark: order by capsule *extent* (`radius² × length`) rather than radius**,
  which keeps a long thin frond over a small fat bead and is the same four
  lines. §3.3 says radius, so radius is what landed.

  **One shape got fatter, and it is the same frond.** The primitive `Plate`
  `[4,4,1]` has a 4:1 cross-section, and the mean takes it from radius 1 to
  **2.5**. In the section that turns the standing crop from thin blades into
  thick columns (visible in `dc3_roster_after.png`, which carries both
  changes). A geometric mean would give 2.0 and is no better. A single capsule
  cannot represent a 9×9×3 slab, and giving the plate two capsules is a
  projection redesign this slice was told not to do. **Mark's**, alongside the
  ordering key above.

  **The load-bearing receipt: the demo trace replays to its hash unchanged.**
  `ps1_played.trace.json` replays to `86af868ebb97e90b` and exits 0. DC3 is all
  presentation and reached no trace, exactly as §6 predicted — this is the
  second slice in the series not to break a hash.

  **Files and ceilings.** `lens/src/body.rs` 349 → 495, `tracer/params.rs`
  192 → 291, `genet/src/section.rs` 525 → 569. All under the 600 ceiling;
  `body.rs` is the one to watch next. Receipts in `Code/testing/mesocosm/`:
  `dc3_radius_{before,after}.png`, `dc3_roster_{before,after}.png`,
  `dc3_vanish_{before,after}.{png,json}`, `dc3_vanish.trace.json`. The
  preserved `mark_playtest2.*` files are untouched.

  **Left for Mark.** Three. **DC-R3**, unchanged and still the ruling §3.3
  asks for. **The roster ordering key** — radius, as landed, or extent.
  **The `[4,4,1]` plate**, which one capsule cannot carry at any radius rule.

- **2026-08-31 (DC2): one archetype, measured against the economy.** Landed
  against §6's DC2 done-conditions. **The verdict moved and is reported rather
  than absorbed** — see the third finding above; nothing here is a claim that
  the arm should ship.

  **The body, as authored.** `axis::archetype` is new (286 lines) and is the
  catalogue's sibling and its opposite: the catalogue says "reference points,
  not content", and this module is content. `consumer_browser` is eight tagmata
  and **33 parts**, the same count §2.4's carving B has:

  | tagma | segments | segment shape | bears | shape |
  | --- | ---: | --- | --- | --- |
  | head | 1 | `[2,1,1]` | Mouth x1 | crop `[2,1,0]` |
  | face | 1 | `[2,1,1]` | Feeler x1 | eye `[1,1,1]` (mirrored: 2) |
  | nares | 1 | `[2,1,1]` | Feeler x1 | speck `[0,0,0]` (mirrored: 2) |
  | neck | 5 | `[2,1,1]` | — | — |
  | shoulder | 1 | `[2,2,1]` | Limb x1 | leg `[3,1,1]` (2) |
  | chest | 1 | `[2,2,2]` | — | — |
  | haunches | 2 | `[2,2,1]` | Limb x1 | leg `[3,1,1]` (4) |
  | tail | 10 | `[2,1,1]` | — | — |

  Six distinct shapes, as carving B has. Four of them are new palette entries in
  **spare slots** — `[2,1,1]`, `[2,2,1]` and `[2,1,0]` in `Mass`, `[3,1,1]` in
  `Limb`, `[0,0,0]` in `Sensor` — so `PALETTE_SHAPES` stays four and
  `PartPalette::primitive`'s defaults do not move, which is what makes the arm
  isolable. It reads **Consumer** because the head bears a mouth and nothing on
  it fixes, and **Grazer** because that mouth is a `Mass`-classified crop.
  `variance` is **0**, per §5 ("a hexapod with a varying leg count is a
  different creature"); kin are still not clones because `Soma::develop`'s
  developmental absence survives, so ~30% of founders are born short a leg pair,
  an eye pair or their specks. Absence cannot take the mouth (DC1.5's guard) and
  the body carries no `Plate` at all, so **no individual can develop out of its
  kingdom** — asserted over 256 development seeds.

  **The measurement, against §2.4's table.**

  | reading | §2.4 carving B | DC2 measured | |
  | --- | ---: | ---: | --- |
  | parts | 33 | **33** | ✓ |
  | distinct part shapes | 6 | **6** | ✓ |
  | `mass_ceiling_mg` | 1,284 | **1,284** | ✓ |
  | `actuator_span` | 18 | **18** | ✓ |
  | `sensor_span` | 2 | **2** | ✓ |
  | build multiple | 2.40 | **3,084 / 1,284 = 2.40** | ✓ |
  | rent at adult mass | 9 mg/tick | **9** | ✓ |
  | bite at adult mass | 49 mg | **49** | ✓ |
  | sight horizon | 9 voxels | **9** | ✓ |
  | breeding gate | 423 mg | **423** | ✓ |
  | birth floor | parent >= 132 mg | **132** | ✓ |
  | ticks of reserve at full | 142 | **142** | ✓ |
  | total voxel volume | 1,611 | **1,609** | **-2** |

  **One divergence, and it is arithmetic rather than judgement.** The body is
  two voxels short of carving B's volume. The cause is the first finding above:
  the six dorsal plates cannot be worn by a consumer now that a plate fixes, and
  they were also the only parts absorbing the fractional loss that lets 1,611
  voxels floor to 1,284 mg. Holding the milligram was the choice, because every
  other row in the table reads off the ceiling and the spans and none reads off
  the voxel count. The plates' 294 voxels and 234 mg went back into `Mass`
  bulk — the chest and the broad leg-bearing segments — so the silhouette keeps
  a deep body without claiming armour it cannot have.

  **The instrument, both arms, ten seeds each.**

  | arm | breathes | thins | boil | collapse |
  | --- | ---: | ---: | ---: | ---: |
  | baseline (`Founding::Drawn`, DC1.5's founding) | 0 | 10 | 0 | 0 |
  | archetype (`Founding::BrowsingConsumer`) | **1** | 5 | 0 | **4** |
  | DC1.5's naive mobile grazer, for comparison | 1 | 6 | 0 | 3 |

  The baseline arm is **identical to DC1.5's receipt**, per seed and to the
  milligram (seed 8 ends at 1,131,018 mg, seed 9 at 864,135, seed 2 at 389,601 —
  DC1.5's own numbers), which is the proof that installing the archetype changed
  nothing for the tiers that still draw. Control all collapse, zero escapees,
  matter conserved in every sample of every run.

  The archetype arm is **one seed worse than DC1.5's naive grazer on collapse
  and one better on thins** — the same regime, and the third finding above says
  why it is a regime the body cannot leave: a tier is one interbreeding species,
  so founding the browser founds 230 mobile grazers with a 423 mg breeding gate,
  a working eye, the whole 610-strong stand as food, and nothing eating them
  back, in every seed. The baseline's consumer tier is a *predator* tier in nine
  of ten seeds and consumers die out anyway; the archetype's is the only
  founding in the series that ends with all three kingdoms alive (seed 8,
  453/6/5 — the only `breathes` the baseline instrument has ever read at this
  cohort). **The reading is that DC4 cannot install one consumer archetype: it
  has to install the pursuit form beside the browser, or the tier has to stop
  being one species.** That is §7's first open question and it is Mark's.

  **Receipts.** `cargo test -p mesocosm-core --test matter --release` green (5
  tests, 86 s). Nine new tests: five in `axis::archetype` (the carving-B column
  measured off a developed body, palette admissibility, a speck seeing nothing,
  no individual developing out of Grazer over 256 seeds, and kin resembling
  without cloning) and three in `world::genesis` (the authored tier founds 230
  mobile grazers in seeds 1-10, a founded organism reads the carving-B column,
  and **the arm is isolable** — 687 producer and decomposer bodies per seed
  encode byte-identically under both foundings, with identical positions).
  `cargo test --workspace` green (545 passed, 1 pre-existing ignored;
  mesocosm-lens at `--test-threads=1`), `clippy --workspace --all-targets -D
  warnings` clean,
  `cargo fmt --all --check` clean, `cargo check -p paredros-room --features
  r1-proof` clean. **No `REFERENCE_*` or `*_BASE` constant was touched** — no
  file under `organism/` or `process.rs` is in the diff at all.

  **The fixtures did not move, and that is the receipt.** The demo trace
  re-records to the same 120 intents and the same hash `86af868ebb97e90b` DC1.5
  left it at, because the arm is off by default and no snapshot field changed;
  `--replay` at the default path exits 0 and reports the match, and a trace with
  one bit flipped in its recorded hash exits 1. This is the first slice in the
  series that did not break a hash. Instrument receipt at
  `Code/testing/mesocosm/dc2_browser.json`; DC1's and DC1.5's files are
  untouched. Capture at `Code/testing/mesocosm/dc2_browser.png`, from
  `mesocosm-lens`'s `menagerie` example at the same framing the catalogue's
  reference animals use, so the authored body and the generated ones are
  comparable at a glance.

  **What the capture actually shows, honestly.** Not a browsing hexapod. A
  segmented green bead-chain, closer to a caterpillar: uniform capsules with a
  slightly thicker cluster at the middle and small blobs at the head end. The
  legs are present and read as bumps rather than legs. Two causes, and the
  smaller one is the framing (the camera looks nearly along the lateral axis, so
  the mirrored legs foreshorten into the silhouette). The larger one is the
  fifth finding above: **the lens projects every part to a capsule with one
  radius, the smaller cross-axis half-extent floored at 1**, and all six of the
  archetype's shapes have a cross-section of 1 — so the shape vocabulary DC1
  widened is invisible, a one-voxel decorative speck draws the same size as a
  working eye, and a many-small-parts body reads *thinner* than the `[2,2,2]`
  blocks it replaced. One tint over the whole body (§3.3's open colour question)
  finishes the job. DC4 is the bar and this is not it; the good news is that the
  two things standing between here and there are already scheduled (DC3, DC5)
  and the radius rule is four lines.

  **Split, per the ceiling.** `world/genesis.rs` was 576 and is now 446, with
  its tests in `world/genesis/tests.rs` (308). `axis.rs` is 541 and
  `axis/archetype.rs` 286. `population_instrument.rs` is **598** — it grew a
  third batch and is now the file the next round has to split before adding.

  **Left for Mark.** Four. **Whether the consumer tier founds one archetype or
  three** — DC2's number says one is not viable and §4.2 already proposes three;
  this is §7's question 1 and it now has evidence. **Whether `Founding` stays a
  two-variant enum** or becomes a per-tier set once DC4 has a roster. **The
  capsule radius rule**, which is the cheapest legibility this plan has found
  and belongs somewhere between DC3 and DC5. And **whether the played critter
  should carry the archetype** — DC2's arm gives it to `SpeciesId(1)` along with
  the rest of the consumer tier, which is §5's open question answered
  provisionally rather than ruled.

- **2026-08-31 (DC1.5): kingdom unbinds from symmetry.** Landed against §6's
  DC1.5 done-conditions and both of §6.5's rulings.

  **The reading.** `Kingdom::from_symmetry` is gone. `Kingdom::of_body` and
  `FeedingMode::of_body` (`organism/kingdom.rs`) read four rules off the parts a
  body feeds with:

  | reading | anatomy |
  | --- | --- |
  | `Producer` | a living part performs `Process::Fix` — a plate, a frond, a leaf |
  | `Consumer` | no fixing part, and the head bears a mouth |
  | `Predator` | that mouth is `Limb`-classified: a jaw, which swings |
  | `Grazer` | that mouth is bulk: a crop, which does not |
  | `Decomposer` | neither organ — it absorbs across its own surface |

  **A mouth is a living part attached to the root and hung below it.** Both
  halves carry weight. Without *below*, the next segment along the axis reads as
  a mouth, since a spine is `Mass`-classified parts attached to `Mass`-classified
  parts; without *the root*, any bulk hanging anywhere off a body makes it a
  consumer. The root is the axis' front-most segment by construction of
  `develop_body`, so "at the head" is a fact of the pipeline rather than a
  convention this reading invents. Fixing is checked first, so a body carrying
  both organs reads `Producer` — the deferred mixotroph pinned by stated
  precedence rather than found by accident.

  **A fourth process.** `Process::Fix`, expressed by `Role::Plate`, wired in all
  four places the forms brief's §A listed — the variant, `NATIVE_DEFS`,
  `Role::processes()` and the parity test. `Process::ALL` is new and the parity
  test iterates it, so the trap that brief named (a fifth variant compiles clean
  and panics at runtime inside `Registry::of_native`) cannot recur. **The word
  is provisional**: `Fix` is biology's plain working verb and `CLAUDE.md` forbids
  coining mid-session, so the identifier and `ProcessId`'s `"fix"` are both
  placeholders awaiting a naming round. Renaming moves no rule.

  **`Symmetry` is geometry.** It keeps its three variants and its one job,
  deciding what `BodyPlan::mirrors`; `Kingdom::symmetry()` survives only as the
  silhouette a founding tier opens with, and nothing reads it back.

  **The transitional founding.** The pyramid no longer authors a `Kingdom` onto
  a founder for the body to wear — it picks which body that founder's line
  draws, and the world reads the kingdom back off the body. `axis::seed` takes a
  `Kingdom` instead of a `limbed: bool`: a producer draw gets a guaranteed
  fronded stretch and nothing that contracts, a consumer draw a mouth on its
  head, a decomposer draw neither. Consumers lost their plate arm, because a
  consumer that grew one would be the deferred mixotroph. **The mouth follows
  the legs** — a line that draws pursuit machinery draws the jaw to use it —
  which is both the natural authoring rule and the null change, since the old
  reading was "any part performs `Contract`". Drawing it independently instead
  is a different ecology and it is measured in the findings above.

  **The four bindings, each handled.** `LivingTarget.kingdom` keeps reading
  `o.kingdom()` and the grazer filter is unchanged in code — what a body earns
  by and what it is edible as are now one reading of one anatomy, and the
  register's §13 mixotroph case stays deferred because no body can honestly do
  both. `CohortKey` is unchanged mechanically and documented: it is an anatomy
  class now, so a body can leave it by growing or losing an organ. Hashes moved
  as priced: `FaunaTraits.feeding_mode` reaches `state_hash` through the fauna
  decision trace, and the demo fixture is re-recorded below. `guise` and
  `betrays_itself` keep every line of their logic and get sharper referents —
  the lie is a claim to carry a way of making a living's organs, and the tell is
  "a producer's claim over a body with no part performing `Fix`", which is why
  the tell exists at all rather than a coincidence of two enums lining up.

  **Receipts.** `cargo test -p mesocosm-core --test matter --release` green (5
  tests, 53 s). **The census is a test, not a probe**
  (`world::genesis::tests::every_founding_body_reads_the_kingdom_its_tier_drew`):
  it replays genesis' own seeded tier draws and asserts every one of the 917
  founding bodies in each of seeds 1-10 reads back the kingdom its tier drew —
  9,170 of 9,170 — and a sibling test asserts ten seeds reach both consumer
  readings. The pyramid (610/229/77) and the TD2b kingdom floor still hold, now
  as readings rather than as fields. `population_instrument --release`: **0
  breathes / 10 thins / 0 boil / 0 collapse**, control all collapse, zero
  escapees, matter conserved and per-seed identical to DC1's; per-kingdom end
  states in the findings above. Demo fixture re-recorded — same 120 intents,
  hash `e2aca9bb40e7b0ea` -> `86af868ebb97e90b`; `--replay` at the default path
  exits 0 and reports the match, and a trace with one bit flipped in its
  recorded hash exits 1. `cargo test --workspace` green (537 tests,
  mesocosm-lens at `--test-threads=1`), `clippy --workspace --all-targets -D
  warnings` clean, `cargo fmt --all --check` clean, `cargo check -p
  paredros-room --features r1-proof` clean. Receipt at
  `Code/testing/mesocosm/dc15_kingdom.json`; DC1's file is untouched.

  **Split, per the ceiling.** `organism/kingdom.rs` is new (269 lines) and
  `organism.rs` came down to 556. `process.rs` is 569, `axis.rs` 540,
  `world/genesis.rs` 576 — that last one is close enough that DC2 should expect
  to split it rather than add to it.

  **Left for Mark.** Three things, none of them blocking DC2. **The process
  word** — `Fix` is a placeholder and wants a naming round. **Whether founding
  should reach a mobile grazer at all**, now that the number is measured: the
  draw here declines to, deliberately, and DC2 can take it up with the
  instrument watching. And **the armour collision**: `Role::Plate` is now the
  fixing geometry, so nothing can be armoured without also being a producer, and
  the roster will hit that on its first shelled archetype.

- **2026-08-30 (DC1): the palette learns more than four shapes.** Landed
  against §6's done-conditions.

  **What changed.** `PartPalette`'s four `PartTemplate` fields become four
  `RoleShapes`, each a default template plus `PALETTE_SHAPES - 1` optional
  extras — **four shapes per role**, sized on §2.4's carving B (three Mass
  shapes, one each of Limb, Plate and Sensor) with one spare. The default is a
  plain field rather than slot zero of an array, so "every role always has one
  shape" is a fact of the type instead of a rule `validate` enforces. `Tagma`
  gains `segment_shape` and `appendage_shape`, `u8` selectors where **0 is
  every role's default**; a selector a world does not admit falls back to the
  default rather than failing, which is what keeps the palette's stated
  cross-world promise (`development.rs:40`) true. `develop_body` resolves the
  segment template per tagma (the root from the first tagma's selector) and the
  appendage template from the tagma's own. `PartPalette::validate` now runs the
  classifier over *every* admitted shape, not just the default, and adds the
  §2.3 build-price refusal for Limb and Sensor, priced with the economy's own
  `part_ceiling_mg` rather than a copy of its formula. No archetype: the
  primitive palette is one shape per role, exactly what it was.

  **The null-change receipt.** The new `Tagma` fields and the widened palette
  move the snapshot format, so `state_hash` cannot witness body identity —
  it moved on all ten seeds, as §4.1 predicted. Identity was proven instead by
  digesting, per seed at genesis, every organism's encoded `BodyDocument` and,
  separately, every organism's genesis-drawn scalars (position, stage, age,
  since_offspring, signal, venom, guise, development seed, masses) — the second
  being a positive check that the **draw sequence** did not move, since every
  genesis `rng` draw lands in one of those fields. Both digests and the total
  living-part count are identical across all ten seeds before and after.

  **Receipts.** `cargo test -p mesocosm-core --test matter --release` green (5
  tests, 26 s). Four new refusal tests: a Limb at `[3,1,0]` and a Sensor at
  `[1,1,0]` are refused with `Overpriced` while the primitive palette develops,
  a wrong-role shape in a *later* slot is caught like one in the default, and
  the two bounds are asserted equal to the primitive palette's own span and
  ceiling so guard and palette cannot drift apart. `population_instrument
  --release`: **0 breathes / 10 thins / 0 boil / 0 collapse**, control all
  collapse, matter conserved in every seed, and the curves byte-identical to
  TD11's (see Findings). Demo fixture re-recorded — same 120 intents, hash
  `8f6df49c63923be6` -> `e2aca9bb40e7b0ea`; `--replay` at the default path exits
  0 and reports the match, a falsified trace exits 1. `cargo test --workspace`
  green (mesocosm-lens at `--test-threads=1`, 38 tests), `clippy --workspace
  --all-targets -D warnings` clean, `cargo fmt --all --check` clean, `cargo
  check -p paredros-room --features r1-proof` clean.

  **Two small calls made here, both open to reversal.** The instrument's
  receipt filename moved `td11_chain.json` -> `dc1_palette.json`, per its own
  documented rule that no round overwrites another's; TD11's file is untouched.
  And `development.rs`'s tests moved to `development/tests.rs` under the
  600-LOC ceiling (528 / 310 / axis.rs 452).

  **Left for Mark.** `PALETTE_SHAPES` is four, and the palette is *world*
  state: DC4's eight archetypes share one. If the roster wants more than four
  Mass shapes across all eight, raising the constant is one line, but it is a
  budget question rather than a mechanical one and the number should be ruled
  before the roster is authored, not discovered during it.

- **2026-08-30:** founded. Assessment against the code complete: the body
  machinery (`axis.rs`, `plan.rs`, `development.rs`, `body.rs`, `species.rs`,
  `world/genesis.rs`), the economy coupling (`organism/ecology/rates.rs`,
  `organism.rs`), the render path (`mesocosm-lens/src/{lib,body}.rs`,
  `tracer/params.rs`, `mesocosm-genet/src/section.rs`), and the Isometry
  reference (`isometry/crates/isometry-voxel/src/{demo,recipe,body}.rs`). The
  part-scale arithmetic and the render-budget arithmetic are done and are §2 and
  §3. Nothing implemented; eight questions stand for Mark in §7, of which two
  (§4.3) block DC4.
