# Phenotype: what a body is for

**Status: decisions plan, 2026-07-31. Nothing here is built.** This is a
decision document, not an implementation spec. Each entry states a question,
the options, a recommendation with its cost, and who rules it. Implementation
plans follow the rulings, not the other way round.

Ordering stays with the [execution waves plan](2026-07-31_execution_waves_plan.md);
this owns *what a body means* and hands the sequencing back.

---

## 1. Why this exists now

Two things landed on 2026-07-31 that together made a planning pass necessary
rather than optional.

**The anatomy ruling.** Bodies are part trees and loss cascades, wing-wide
(games wing record §2). `anatomy.rs` implements descent, depth, and severing.
That ruling presumes capability comes from anatomy, because a cascade is
pointless if losing a limb costs nothing.

**A design review** (relayed by Mark) that named the gap precisely: a part
should contribute one or two biological processes, geometry modifies their
effectiveness, attachment decides whether they form a working path, and the
world decides whether the result is useful.

Both point at the same hole, and it is now the largest one in the project:

> **Mesocosm has three competent models of a creature that do not explain one
> another.** A voxel body graph, a live organism ecology, and an adaptation
> trait sheet. A long limb is classified `Role::Limb` and produces no
> locomotion. `Trait::Jaws` exists in the lineage model and is not a mouth
> assembled from voxels. A consumer can graze and its body neither grants nor
> constrains the act.

Verified, not asserted:

- `Role` is derived by `classify(half_extent)` and never consulted outside
  placement. Its only non-`plan.rs` references are two lines of test code.
- `Intent::Move` costs `distance * MOVE_COST_MG` with `MOVE_COST_MG = 1`. The
  body is not consulted. A twelve-limbed creature and a single cube move
  identically.
- `GRAZE_RANGE` and `GRAZES_MG` are flat constants; `Kingdom` is the only gate.
- Before today, `total_mass_mg()` was read by no rule at all, only by tests and
  a `println!`. The body was a pure accumulator: growing cost nothing and
  granted nothing.

---

## 2. What is already ruled

Recorded so this plan does not re-litigate settled things.

| Ruling | Where |
| ------ | ----- |
| Bodies are part trees; loss cascades to dependents | wing record §2, 2026-07-31 |
| Capability is a fold over surviving parts, never a stored number | wing record §2 |
| The tree is shared identity; the fold is per-vessel rules | wing record §2 |
| The mind is not in the tree (Kenshi rule) | Paredros founding plan |
| Parts come only from incorporation, and carry provenance | Mesocosm founding plan |
| Placement is automatic and symmetric by body plan; full manual placement is an editor mode, never the resting state | Mark, 2026-07-31 |
| Initiative is descending metabolic complexity | Mesocosm founding plan |
| Three authored worlds, not procedural generation | waves plan §2.2 |
| Games couple by data, not by types | waves plan §1.4 |

---

## 3. The decisions

In dependency order. **D1 blocks D2 and D3. D4 blocks nothing**, which is the
single most useful fact in this document.

### D1. What produces a capability?

**Question.** How does a body become an ability?

**Options.**

1. *Nothing* (status quo). Capabilities are constants; anatomy is decoration.
2. *Fold.* Sum a contribution over parts. Reach is the longest limb, armour is
   total plate. My earlier proposal.
3. *Processes and paths.* Each part contributes one or two biological
   processes; geometry scales them; **the attachment graph decides whether they
   compose into a working path**; the world decides whether the result is worth
   having. The review's proposal.

**Recommendation: option 3.** A fold cannot express the thing that makes this
interesting. A jaw alone is not a bite: it needs an actuator behind it and an
intake surface in front. Under a fold that is inexpressible; under paths it is
the default case. Three consequences follow that option 2 cannot produce:

- **Severing breaks a path**, so losing one part can disable a capability that
  physically lives elsewhere. That is what makes the cascade ruling pay.
- **Incorporation can disappoint.** You can eat an excellent part and gain
  nothing, because you have no route to it. A fold always rewards eating.
- **The bijection dies.** The ecology lab converged and produced zero
  extinctions across 600 lineage-rounds because every pressure had exactly one
  trait that answered it. Paths cannot be a lookup table: a plate answers
  corrosion *and* adds mass *and* occludes what is behind it.

**Cost.** The largest single change in the project so far. It replaces the
trait vocabulary, changes how the adaptation phase scores, and needs a process
vocabulary authored carefully enough to avoid becoming a catalog (the Spore
failure) while staying small enough to reason about.

**Rules: Mark.**

### D2. Where is capability computed?

**Question.** Which crate owns the derivation?

**The constraint, verified.** `VolumeRef` is documented: *the core never reads
volume contents; it only carries the reference*. A `Part` carries a box
(`half_extent`), a pivot, an attachment, and a mass. So **volume, leverage, and
orientation are available in core; true surface area and occlusion are not.**
The review's list of what voxels provide overshoots by two entries.

**Options.**

1. *Box proxies in core.* Surface is computed from `half_extent` as a box;
   occlusion is approximated from the attachment graph and neighbouring boxes.
2. *Move derivation to `mesocosm-mesh`*, which resolves volumes and can measure
   real exposed surface.
3. *Cache a surface figure per part in core*, written when a part is attached.

**Recommendation: option 1, and revisit at option 3 rather than option 2.**
Boxes give exact face areas for free, and every part in the game today *is* a
box. Option 2 puts a core game rule in a projection crate and makes the
simulation depend on voxel content, which would end the clean split that makes
the body document portable. If box proxies prove too coarse, caching a
resolved figure at attachment time keeps the rule in core and pays the cost
once instead of every tick.

**Cost of the recommendation.** Occlusion is approximate, so "exposed
photosynthetic tissue" means "a face not covered by a child part's box" rather
than a true visibility test. Good enough to be a real constraint, not good
enough to be a lighting model.

**Rules: me, unless Mark disagrees.** This is an implementation boundary rather
than a design question, and it is reversible.

### D3. Do traits survive?

**Question.** What happens to `epoch::Trait` and `Lineage::traits: [i32; 7]`?

**The finding that decides it.** `BodyPlan` already holds heritable growth
parameters: `symmetry`, `preferences: [Facing; 4]` per role, and `tolerance`.
The review's "traits become heritable growth parameters" is therefore not a new
system. It is the one that already exists.

**Recommendation: delete the trait array.** Adaptation spends its bank on
`BodyPlan` changes, the body grows under the plan, and capability is read off
the grown body. This is a fix by subtraction, and it serves the anti-Spore rule
directly: the adaptation phase currently has its own private creature model,
which is a stage growing its own engine.

**Cost, which the review does not name.** Evaluating a candidate plan requires
**growing a body from it**. Thrive's N=5 hill climb becomes five body growths
rather than five array sums. That is Karl Sims, which is presumably why Sims is
cited, and it is still cheap at these body sizes. But it changes the adaptation
phase's cost model and should be chosen rather than discovered.

**Rules: Mark**, because it deletes a shipped subsystem.

### D4. Does a meal have a destination?

**Question.** When you metabolize something, what does it become?

**The defect, verified.** `world.rs:332` is
`self.energy_mg += eaten.mass_mg / 2;` inside `incorporate`. Eating grants a
part *and* half the mass as energy, automatically. The single most important
verb in the game asks the player nothing.

**Recommendation: split it, and do this first.** `Metabolize` stays the one
verb and gains a destination: burn it now, incorporate it, give it to
offspring, or build with it. Every meal then asks *live now, grow later, or
change the world*.

**Why first.** It is small, it touches one function and one intent, and
**it does not depend on D1, D2, or D3 at all.** It answers whether the central
tradeoff is interesting to make before anything expensive is built on the
assumption that it is. If routing turns out to be tedious rather than tense,
that is far cheaper to learn now.

**Rules: Mark**, but this is the one I would start today.

### D5. Where does the bank come from?

**Question.** The ecology lab produced zero extinctions in 600 lineage-rounds
and converged by round six. Income is flat and uncontested, so every lineage
eventually solves its world and nothing can lose.

**Options.**

1. *Contested income.* Bank is a share of a finite pool weighted by standing,
   so falling behind compounds and extinction is a consequence of competition.
2. *Evented disturbance.* Worlds throw glaciations and plagues, so a settled
   roster is periodically made wrong.

**Recommendation: 1 is the fix, 2 is the seasoning**, and 1 belongs to the
epoch half, since how bank is *earned* is what the played phase is for. Note
that D4 and D5 are the same question at two scales: what a meal becomes, and
what a population's meals become.

**Rules: Mark.** Deferred until D4 is played, because D4 may answer it.

### D6. Does the tree travel?

**Question.** `mesocosm.chronicle/v0` carries `parts` as a flat list. Bodies
are now trees. Does the record carry parent links?

**Recommendation: yes, at v1**, but not yet. If the tree is shared identity,
dropping parent links loses exactly what the wing says is not a vessel's to
lose, and re-entry cannot rebuild what was never carried. But there is no
consumer yet: `Chronicle::found` currently rebuilds descendants as a star, and
until capability depends on depth, a star costs nothing. Once D1 lands, a star
descendant is systematically weaker than its ancestor, and that is the moment
v1 is needed.

The refusal path built on 2026-07-31 is what makes the change cheap: magic and
version sit ahead of the payload and both sides refuse an unknown version
before decoding.

**Rules: me, on D1's completion.** Mark rules only if the answer should be no.

### D7. What makes witnessing rewarding?

**Question.** The weakest of the three verbs. The adaptation phase produces a
transcript, which is a changelog rather than a story.

**The review's answer**, which I agree with: a field journal of observed
behaviours, uncertain hypotheses, lineage diagrams, remembered tells, and
before/after epoch summaries. Knowledge persists across descendants without
becoming permanent stat power.

**Cost the review does not name.** This is a UI-heavy feature and Mesocosm has
a winit window and no UI framework decision. Its one paragraph is much cheaper
than its implementation.

**Recommendation: defer, but stop losing the material for it.** The data a
journal would show should be recorded as it happens, so the feature is later
assembly rather than later archaeology.

**Rules: Mark, later.** Not a wave 2 item.

---

## 4. What this plan does not decide

Named so they are visibly parked rather than forgotten:

- **Situations generated from ecological dependencies** rather than authored
  sidequests. Strong idea, downstream of D1 and D5.
- **The three-clock resolution ladder** (moment, epoch, world). Partly implied
  by existing rulings; wants its own pass once D1 fixes what a moment contains.
- **Co-op ordering**, with lineage grafting first. Still deferred-to-last per
  the wing record; the proposed order is good and changes nothing today.
- **Parasitism and distribution**, the two unbuilt strategy-table entries.
  Both become expressible under D1: a parasite is a part with its own agenda,
  and distribution is growing a part *to be eaten*.
- **Whether "borg" survives word clearance** (wing record open question 3).

---

## 5. Sequencing

The recommendation, in one line: **D4 first, alone, and played.** Then D1 with
D2 and D3 behind it. D5 and D6 follow from what those teach. D7 last.

**Done when**, for the first step: a player choosing where a meal goes finds
the choice tense rather than clerical, on a body that is otherwise unchanged.
That is a judgment and it is Mark's, in the same way wave 2.1 is.

**Done when**, for D1: a critter's abilities can be read off its anatomy with
no capability number stored anywhere; severing a limb removes an ability that
depended on it; and the ecology lab, rerun unchanged, stops converging by round
six.

That last condition is the honest test of whether this whole plan was right.
