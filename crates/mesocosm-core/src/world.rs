// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The world: an enclosure holding one critter and the loose matter it can
//! metabolize.
//!
//! A world is a pure function of its seed and the ordered intents applied to
//! it. There are no clock reads, no unordered iteration that reaches the
//! simulation, and no randomness outside the seeded stream, so replaying the
//! same trace against the same seed reproduces the same world exactly.

use serde::{Deserialize, Serialize};

use crate::body::{
    Aabb, Attachment, BodyDocument, Origin, PartId, Provenance, SpeciesId, VolumeRef, Yaw,
};
use crate::organism::{Kingdom, Organism, OrganismId, Signal, Stage};
use crate::process::Unmet;
use crate::rng::Rng;

/// How an incorporated part finds its site.
///
/// A **policy**, not a destination. Placing a part explicitly is a different
/// way of growing, not a different thing to do with a meal, and folding it in
/// beside `Burn` conflated the two questions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Placement {
    /// The body plan decides. **The default**: growth is automatic and
    /// symmetric, and the player shapes the plan rather than the placement.
    Planned,
    /// An explicit site. The editor path: total control is possible, but it is
    /// never the resting state.
    Explicit { parent: PartId, offset: [i32; 3], yaw: Yaw },
}

/// Where a meal goes.
///
/// **This is the game's central question, and before it existed there was no
/// question.** Eating used to grant a part *and* half the mass as energy, so
/// the most important verb asked the player nothing. Splitting the destination
/// is what makes every meal ask: live now, or grow later?
///
/// Later destinations arrive when the systems that receive them do:
/// provisioning reproduction, depositing or building a niche, and cultivating
/// something outside the skin. Each is a different answer to *where does this
/// capability live: in me, in a relationship, or in the world?*
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Route {
    /// Burn it now. Full mass becomes usable energy and no part is kept.
    Burn,
    /// Commit it to growth. Yields **no** immediate energy, which is the whole
    /// tradeoff.
    Incorporate { placement: Placement },
}

/// What a host may ask the world to do. Hosts send intents; they never mutate
/// world state directly.
///
/// `Clone` rather than `Copy`, because naming a lineage carries a name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Intent {
    /// Move the critter by a voxel delta.
    Move { delta: [i32; 3] },
    /// Eat something, and decide what it becomes. **The one verb.**
    ///
    /// Metabolize means routing matter through the organism rather than
    /// swallowing it: the meal is the same, the destination is the choice.
    Metabolize { organism: OrganismId, route: Route },
    /// Return mass to the enclosure as carrion.
    Deposit { mass_mg: u64 },
    /// Split the line you are in, and name it.
    ///
    /// **The act that makes a species.** Splitting is not a threshold anybody
    /// crosses; it is something a player does, and the name is the doing.
    /// Takes the creature you are holding and nothing else, so a new line
    /// begins with one founder and its former kin keep the old one.
    Speciate { name: String },
    /// Inhabit another critter.
    ///
    /// **A recorded intent, not a side door.** Lineage switching is gameplay,
    /// and ordered intents are the only way world state changes. A control
    /// change made outside this path would replay every fact about a run
    /// except who was living it.
    TakeControl { organism: OrganismId },
    /// Advance one tick without acting.
    Idle,
}

/// Why an intent could not be applied. Rejections are part of the recorded
/// outcome, so a replay that rejects the same intents is still identical.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rejection {
    /// Nobody may inhabit that, and this says why.
    Ineligible(Ineligible),
    /// A critter cannot eat itself.
    ///
    /// Only expressible since the played critter joined the organism
    /// vector: before that it was not a thing anyone could target.
    Itself,
    /// Nobody is being played, so there is nothing to act with. A world
    /// running with no one in it is a legitimate state.
    Disembodied,
    NoSuchOrganism(OrganismId),
    NoSuchParent(PartId),
    /// The played critter could not touch it, and this says why: no actuator
    /// at all, or one that does not extend far enough.
    OutOfReach(Unmet),
    InsufficientMass,
    /// The body plan found nowhere for a part of this shape to go. Refusing is
    /// correct: forcing it would overlap existing parts, and a plan that
    /// cannot place something is telling you to change the plan.
    NoRoom,
}

/// Why an organism cannot be inhabited.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ineligible {
    NoSuchOrganism,
    /// A carcass is not a critter you can play.
    NotAlive,
    /// More elaborate than anything you have earned.
    ///
    /// Stepping *down* into a newly viable niche is the point of switching;
    /// stepping across into an unearned peer is what this refuses.
    AboveTheFrontier { frontier: i32, target: i32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    Moved,
    /// A meal became energy and nothing else.
    Burned { organism: OrganismId, energy_mg: u64 },
    Incorporated { part: PartId },
    /// A bilateral plan grew a mirrored pair from one meal, splitting its mass.
    IncorporatedPair { part: PartId, mirror: PartId },
    Deposited { organism: OrganismId },
    /// Control moved to another critter.
    Inhabited { organism: OrganismId },
    /// A line split, and was named.
    Speciated { species: SpeciesId, from: SpeciesId, founder: OrganismId },
    Idled,
    Rejected(Rejection),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct World {
    pub tick: u64,
    pub epoch: u64,
    rng: Rng,
    /// Which organism the player is, if any.
    ///
    /// **Control is a pointer, not a shape.** Switching lineage moves this and
    /// reconstructs nothing; a played critter and an unplayed one serialize
    /// identically; and no rule can branch on who is being played, because
    /// there is no flag to branch on.
    ///
    /// `None` is a first-class state, not a stale pointer. A world does not
    /// require anyone to be in it, and the difference between *nobody is
    /// embodied* and *this id no longer names anything* is one a caller has to
    /// be able to see.
    controlled: Option<OrganismId>,
    /// Who the world stopped being able to play on the most recent tick.
    control_lost: Option<OrganismId>,
    /// Lineages the player has inhabited.
    ///
    /// What "unlocked" means for the complexity frontier. Ordered, so the
    /// frontier is deterministic.
    #[serde(default)]
    unlocked: std::collections::BTreeSet<SpeciesId>,
    /// The most elaborate thing the player has ever held.
    ///
    /// **A high-water mark, not a current reading.** An earlier cut computed
    /// this from living organisms of unlocked species, which meant a lineage
    /// dying out collapsed the frontier to zero and left the world permanently
    /// uninhabitable. That contradicts the ruling that disembodiment is a seam
    /// rather than a dead end: losing a body must not unearn what reaching it
    /// cost.
    #[serde(default)]
    frontier: i32,
    /// Which lineages exist, what they are called, and what they came from.
    #[serde(default)]
    lineages: crate::species::Lineages,
    /// Ordered by id, so iteration never depends on hashing.
    pub organisms: Vec<Organism>,
    next_organism: u32,
    last_tally: crate::organism::Tally,
    /// What happened on the most recent tick, waiting to be drained.
    ///
    /// **One tick, never more.** History is derivable from a seed and ordered
    /// intents, so keeping it in the world would grow every snapshot without
    /// bound and cost the cheap whole-state capture the wing's rollback
    /// thinking rests on. A caller drains this into a [`History`], which lives
    /// beside the world and persists in its own slot.
    /// Not skipped when empty: postcard is positional, so a field written
    /// conditionally cannot be read back. That trap already cost one decode
    /// failure here.
    #[serde(default)]
    pending: Vec<crate::history::Event>,
}

/// The event an outcome amounts to, if any.
///
/// Refusals produce nothing: a history records what happened, and a rejected
/// intent is a thing that did not.
fn event_for(outcome: &Outcome, actor: Option<OrganismId>) -> Option<crate::history::Event> {
    use crate::history::Event;
    match *outcome {
        // Burning names the meal, growing names the grower. Both need the
        // actor, because a history is keyed by who a thing happened to and an
        // event citing nobody would fork that creature's line.
        Outcome::Burned { energy_mg, .. } => {
            Some(Event::Burned { organism: actor?, energy_mg })
        }
        Outcome::Incorporated { part } | Outcome::IncorporatedPair { part, .. } => {
            Some(Event::Grew { organism: actor?, part })
        }
        Outcome::Inhabited { organism } => Some(Event::Inhabited { organism }),
        Outcome::Speciated { species, from, founder } => {
            Some(Event::Speciated { species, from, founder })
        }
        Outcome::Moved | Outcome::Deposited { .. } | Outcome::Idled | Outcome::Rejected(_) => None,
    }
}

/// The half-extent of an organism, by its volume tag.
///
/// Shapes vary so that [`crate::plan::classify`] finds real roles: a world of
/// identical cubes can only ever grow mass. A host's volumes must match, since
/// this is what placement and physics read.
pub fn organism_extent(tag: u8) -> [i32; 3] {
    match tag % 4 {
        0 => [3, 1, 1],
        1 => [1, 3, 1],
        2 => [3, 1, 3],
        _ => [2, 2, 2],
    }
}

/// Energy spent per unit of movement, in milligrams.
const MOVE_COST_MG: u64 = 1;

impl World {
    /// Builds the standard fixture: one critter and a deterministic scatter of
    /// organisms drawn from the seeded stream.
    pub fn new(seed: u64, organism_count: u32) -> Self {
        let mut rng = Rng::from_seed(seed);

        // The played critter is organism zero, built by the same constructor
        // as everything else. It is distinguished only by being pointed at.
        let mut organisms = Vec::with_capacity(organism_count as usize + 1);
        organisms.push(Organism::founding(
            OrganismId(0),
            SpeciesId(1),
            Kingdom::Consumer,
            VolumeRef::from_tag(1),
            [2, 2, 2],
            [0, 0, 0],
            1_000,
        ));

        for index in 1..=organism_count {
            // Draws happen in a fixed order, so the scatter is reproducible.
            let x = rng.range_i32(-16, 16);
            let y = rng.range_i32(-2, 2);
            let z = rng.range_i32(-16, 16);
            let mass = 100 + rng.below(400);
            let species = SpeciesId(2 + (rng.below(3) as u32));
            // A mix that can actually sustain itself: mostly producers,
            // because a world without income runs down.
            let kingdom = match rng.below(6) {
                0 => Kingdom::Consumer,
                1 => Kingdom::Decomposer,
                _ => Kingdom::Producer,
            };
            // Staggered ages, so the enclosure is mid-life rather than all
            // hatching on the same tick.
            let age = rng.below(200) as u32;

            // Most things are honest. A minority lie, in both directions: a
            // harmless thing wearing a warning, and a dangerous thing wearing
            // none. Both are rare, because a world of liars teaches nothing.
            let (signal, venom_mg, guise) = match rng.below(10) {
                0 => (Signal::Warning, 0, kingdom),
                1 => (Signal::Plain, 90 + rng.below(60), Kingdom::Producer),
                2..=3 => (Signal::Warning, 60 + rng.below(60), kingdom),
                _ => (Signal::Plain, 0, kingdom),
            };
            let volume = VolumeRef::from_tag(16 + (index % 8) as u8);
            let half_extent = organism_extent(16 + (index % 8) as u8);
            organisms.push(Organism {
                body: BodyDocument::new(species, volume, mass, half_extent),
                energy_mg: mass,
                position: [x, y, z],
                stage: Stage::Mature,
                age,
                since_offspring: 0,
                signal,
                venom_mg,
                guise,
                ..Organism::founding(
                    OrganismId(index),
                    species,
                    kingdom,
                    volume,
                    half_extent,
                    [x, y, z],
                    mass,
                )
            });
        }

        // The founding population enters the record. Without this a seeded
        // creature's first event is whatever happened *to* it, so its origin
        // is invisible and its causal line begins in the middle.
        let pending = organisms
            .iter()
            .map(|o| crate::history::Event::Born {
                organism: o.id,
                species: o.species,
                parent: None,
            })
            .collect();

        Self {
            tick: 0,
            epoch: 0,
            rng,
            controlled: Some(OrganismId(0)),
            control_lost: None,
            unlocked: std::collections::BTreeSet::from([SpeciesId(1)]),
            // The starting body already counts: the player is holding it, so
            // the frontier begins where they begin rather than at nothing.
            frontier: organisms.first().map(Organism::complexity).unwrap_or(0),
            lineages: {
                // Everything the world began with is a founding lineage: no
                // parent, no name, because nobody was there to give it one.
                let mut lineages = crate::species::Lineages::new();
                for organism in &organisms {
                    lineages.found(organism.species);
                }
                lineages
            },
            organisms,
            next_organism: organism_count + 1,
            last_tally: crate::organism::Tally::default(),
            pending,
        }
    }

    /// Which organism the player is, if any.
    pub fn controlled_id(&self) -> Option<OrganismId> {
        self.controlled
    }

    /// The played organism.
    ///
    /// `None` when nobody is embodied, **and also when the subject is no
    /// longer alive**. A carcass is not a critter you can play: natural death
    /// leaves an organism in the roster as carrion until it is spent, and an
    /// earlier cut checked only that the id was still present, so a dead
    /// critter kept moving and eating while it decomposed.
    pub fn controlled(&self) -> Option<&Organism> {
        let id = self.controlled?;
        self.organisms.iter().find(|o| o.id == id && o.is_alive())
    }

    fn controlled_mut(&mut self) -> Option<&mut Organism> {
        let id = self.controlled?;
        self.organisms.iter_mut().find(|o| o.id == id && o.is_alive())
    }

    /// Whether a given organism could be played.
    pub fn is_eligible(&self, organism: OrganismId) -> bool {
        self.eligibility(organism).is_ok()
    }

    /// Whether an organism could be played, and why not if not.
    ///
    /// **This is where the complexity frontier finally binds.** The rule was
    /// ruled long ago -- an unlocked lineage must be *more* metabolically
    /// complex than the target, so stepping downward into a newly viable niche
    /// is the point and minting an unearned peer at the frontier is not -- and
    /// until now it lived in `epoch::can_switch_to`, which nothing outside its
    /// own tests called. Control could take anything alive, however elaborate.
    ///
    /// It binds here rather than there because that function reasons over
    /// `epoch::Lineage` and its provisional trait array, while control reasons
    /// over organisms. Since P1 every organism has a body, so complexity can
    /// come from anatomy and the two models no longer have to agree first.
    pub fn eligibility(&self, organism: OrganismId) -> Result<(), Ineligible> {
        let Some(target) = self.organisms.iter().find(|o| o.id == organism) else {
            return Err(Ineligible::NoSuchOrganism);
        };
        if !target.is_alive() {
            return Err(Ineligible::NotAlive);
        }
        // A line you have already lived is always yours to return to. The
        // frontier gates reaching *outward*, not going home.
        if self.unlocked.contains(&target.species) {
            return Ok(());
        }

        let frontier = self.frontier();
        if target.complexity() < frontier {
            Ok(())
        } else {
            Err(Ineligible::AboveTheFrontier { frontier, target: target.complexity() })
        }
    }

    /// The most elaborate thing the player has ever held.
    ///
    /// The ceiling a new lineage must sit below, and it only goes up.
    pub fn frontier(&self) -> i32 {
        self.frontier
    }

    /// Which lineages exist, and how they are related.
    pub fn lineages(&self) -> &crate::species::Lineages {
        &self.lineages
    }

    /// How far apart two creatures' ancestries are, in forks.
    ///
    /// One of the axes graft compatibility was ruled to scale with. It was
    /// uncomputable until lineages could split: every pair was identical.
    pub fn kinship(&self, a: OrganismId, b: OrganismId) -> Option<u32> {
        let species = |id: OrganismId| self.organisms.iter().find(|o| o.id == id).map(|o| o.species);
        self.lineages.distance(species(a)?, species(b)?)
    }

    /// Lineages the player has inhabited.
    pub fn unlocked(&self) -> impl Iterator<Item = SpeciesId> + '_ {
        self.unlocked.iter().copied()
    }

    /// The played critter's anatomy.
    pub fn body(&self) -> Option<&BodyDocument> {
        self.controlled().map(|o| &o.body)
    }

    /// Where the played critter is.
    pub fn position(&self) -> Option<[i32; 3]> {
        self.controlled().map(|o| o.position)
    }

    /// The played critter's budget.
    pub fn energy_mg(&self) -> Option<u64> {
        self.controlled().map(|o| o.energy_mg)
    }

    /// Whether anyone is being played.
    pub fn is_embodied(&self) -> bool {
        self.controlled().is_some()
    }

    /// Who the world stopped being able to play on the most recent tick.
    ///
    /// The seam the design calls for: losing a body is where witnessing, world
    /// examination, adaptation, and choosing another eligible critter happen.
    pub fn control_lost(&self) -> Option<OrganismId> {
        self.control_lost
    }

    /// Applies one intent and advances the tick. This is the only way world
    /// state changes.
    pub fn apply(&mut self, intent: Intent) -> Outcome {
        let actor = self.controlled;
        let outcome = self.resolve(intent);

        // Recorded before the ecology steps, because that is the order these
        // happened in and a history that reversed them would let a creature's
        // death precede its last act. It also means the actor is still whoever
        // acted, even if this tick kills them.
        if let Some(event) = event_for(&outcome, actor) {
            self.pending.push(event);
        }

        // The enclosure lives whether or not the player acted. This is what
        // separates an ecology from a field of pickups: things grow, breed,
        // starve, and rot on their own schedule.
        self.last_tally = crate::organism::step(
            &mut self.organisms,
            &mut self.next_organism,
            &mut self.rng,
            &mut self.pending,
        );

        // What you reach, you keep. The frontier rises with the body you are
        // holding and never falls, so a lineage dying out costs you that body
        // rather than everything it earned.
        if let Some(held) = self.controlled().map(Organism::complexity) {
            self.frontier = self.frontier.max(held);
        }

        // Control ends with the life it was attached to. Nothing exempts the
        // played critter from the ecology, so this is where a run can lose its
        // body without the world ending.
        self.control_lost = match self.controlled {
            Some(id) if !self.is_eligible(id) => {
                self.controlled = None;
                Some(id)
            }
            _ => None,
        };

        self.tick += 1;
        outcome
    }

    /// Takes the events of the most recent tick, leaving the world empty of
    /// them. Recording them is a caller's business; the world only reports.
    pub fn drain_events(&mut self) -> Vec<crate::history::Event> {
        std::mem::take(&mut self.pending)
    }

    /// The events of the most recent tick, without taking them.
    pub fn events(&self) -> &[crate::history::Event] {
        &self.pending
    }

    /// What the most recent tick did to the enclosure.
    pub fn last_tally(&self) -> crate::organism::Tally {
        self.last_tally
    }

    /// Living organisms, in id order.
    pub fn living(&self) -> impl Iterator<Item = &Organism> {
        self.organisms.iter().filter(|o| o.is_alive())
    }

    /// Applies an ordered trace, returning every outcome in order.
    pub fn apply_all(&mut self, trace: &[Intent]) -> Vec<Outcome> {
        trace.iter().map(|i| self.apply(i.clone())).collect()
    }

    fn resolve(&mut self, intent: Intent) -> Outcome {
        // Every acting intent needs somebody to act. Nobody home is a
        // refusal, not a panic: a world can outlive whoever was in it.
        if !matches!(intent, Intent::Idle | Intent::TakeControl { .. }) && !self.is_embodied() {
            return Outcome::Rejected(Rejection::Disembodied);
        }

        match intent {
            Intent::Idle => Outcome::Idled,

            Intent::Move { delta } => {
                let distance = delta.iter().map(|d| d.unsigned_abs() as u64).sum::<u64>();
                let cost = distance * MOVE_COST_MG;
                let Some(me) = self.controlled_mut() else {
                    return Outcome::Rejected(Rejection::Disembodied);
                };
                if cost > me.energy_mg {
                    return Outcome::Rejected(Rejection::InsufficientMass);
                }
                me.energy_mg -= cost;
                for (axis, step) in me.position.iter_mut().zip(delta) {
                    *axis += step;
                }
                Outcome::Moved
            }

            Intent::Speciate { ref name } => {
                let Some(me) = self.controlled().map(|o| (o.id, o.species)) else {
                    return Outcome::Rejected(Rejection::Disembodied);
                };
                let (id, from) = me;
                let Some(species) = self.lineages.fork(from, name.clone(), self.tick) else {
                    return Outcome::Rejected(Rejection::NoSuchOrganism(id));
                };

                // Only the founder crosses. Its offspring inherit the new line;
                // its former kin keep the old one, which is what makes forking
                // a commitment rather than a rename.
                if let Some(organism) = self.organisms.iter_mut().find(|o| o.id == id) {
                    organism.species = species;
                    organism.body.species = species;
                }
                self.unlocked.insert(species);
                Outcome::Speciated { species, from, founder: id }
            }

            Intent::TakeControl { organism } => {
                if let Err(why) = self.eligibility(organism) {
                    return Outcome::Rejected(Rejection::Ineligible(why));
                }
                if let Some(species) = self.organisms.iter().find(|o| o.id == organism).map(|o| o.species) {
                    self.unlocked.insert(species);
                }
                if let Some(taken) = self.organisms.iter().find(|o| o.id == organism) {
                    self.frontier = self.frontier.max(taken.complexity());
                }
                self.controlled = Some(organism);
                Outcome::Inhabited { organism }
            }

            Intent::Metabolize { organism, route } => self.metabolize(organism, route),

            Intent::Deposit { mass_mg } => {
                let (species, position) = {
                    let Some(me) = self.controlled() else {
                        return Outcome::Rejected(Rejection::Disembodied);
                    };
                    (me.species, me.position)
                };
                {
                    let me = self.controlled_mut().expect("checked above");
                    if mass_mg == 0 || mass_mg > me.energy_mg {
                        return Outcome::Rejected(Rejection::InsufficientMass);
                    }
                    me.energy_mg -= mass_mg;
                }

                let id = OrganismId(self.next_organism);
                self.next_organism += 1;
                self.organisms.push(Organism {
                    // Deposited matter is dead matter: it feeds decomposers
                    // and returns to the world rather than growing.
                    stage: Stage::Carrion,
                    ..Organism::founding(
                        id,
                        species,
                        Kingdom::Decomposer,
                        VolumeRef::from_tag(64),
                        [1, 1, 1],
                        position,
                        mass_mg,
                    )
                });
                Outcome::Deposited { organism: id }
            }
        }
    }

    /// Eats a organism and grows it where the plan says.
    /// Eat something and route it. The one verb, in one place, so every meal
    /// pays the same costs whatever it becomes.
    ///
    /// **One transaction.** Everything that can refuse is checked before the
    /// organism leaves the roster, a failed attachment puts it back, and the
    /// energy ledger moves only once the meal has actually landed. An earlier
    /// cut consumed the meal and charged its venom before the attachment was
    /// known to succeed.
    fn metabolize(&mut self, organism: OrganismId, route: Route) -> Outcome {
        if Some(organism) == self.controlled {
            return Outcome::Rejected(Rejection::Itself);
        }
        let Some(index) = self.organisms.iter().position(|m| m.id == organism) else {
            return Outcome::Rejected(Rejection::NoSuchOrganism(organism));
        };
        if let Route::Incorporate { placement: Placement::Explicit { parent, .. } } = route
            && self.body().is_some_and(|b| b.part(parent).is_none())
        {
            return Outcome::Rejected(Rejection::NoSuchParent(parent));
        }
        if let Err(unmet) = self.reach_to(self.organisms[index].position) {
            return Outcome::Rejected(Rejection::OutOfReach(unmet));
        }

        // Resolve planned placement before the meal is consumed, so a body
        // with nowhere to put a part refuses without eating anything.
        let growth = match route {
            Route::Incorporate { placement: Placement::Planned } => {
                let extent = self.organisms[index].half_extent();
                let Some(body) = self.body() else {
                    return Outcome::Rejected(Rejection::Disembodied);
                };
                match crate::growth::resolve(body, extent) {
                    Some(growth) => Some(growth),
                    None => return Outcome::Rejected(Rejection::NoRoom),
                }
            }
            _ => None,
        };

        let eaten = self.organisms.remove(index);
        let (outcome, gain_mg) = self.land(&eaten, route, growth);

        if matches!(outcome, Outcome::Rejected(_)) {
            // Nothing landed, so nothing was eaten and nothing is owed.
            self.organisms.insert(index, eaten);
            return outcome;
        }

        // **Gains before costs.** Subtracting venom first let a nearly starved
        // critter lose part of a toxin to the zero floor and then collect the
        // full meal, so being close to death made a venomous thing *safer*.
        //
        // The floor itself remains: energy is unsigned, so venom beyond what a
        // critter has is forgiven rather than owed. A debt or damage model is a
        // later decision, recorded in the phenotype plan.
        if let Some(me) = self.controlled_mut() {
            me.energy_mg = (me.energy_mg + gain_mg).saturating_sub(eaten.venom_mg);
        }
        outcome
    }

    /// Attempts the routed outcome, returning it and the energy it yields.
    /// Mutates the body but never the roster or the ledger.
    fn land(
        &mut self,
        eaten: &Organism,
        route: Route,
        growth: Option<crate::growth::Growth>,
    ) -> (Outcome, u64) {
        match route {
            Route::Burn => (
                Outcome::Burned { organism: eaten.id, energy_mg: eaten.biomass_mg() },
                eaten.biomass_mg(),
            ),
            Route::Incorporate { placement: Placement::Explicit { parent, offset, yaw } } => {
                let provenance = self.taken_from(eaten);
                let attached = self.controlled_body_mut().attach(
                    eaten.volume(),
                    eaten.biomass_mg(),
                    eaten.half_extent(),
                    Attachment { parent, offset, yaw },
                    provenance,
                );
                match attached {
                    Ok(part) => (Outcome::Incorporated { part }, 0),
                    Err(_) => (Outcome::Rejected(Rejection::NoSuchParent(parent)), 0),
                }
            }
            Route::Incorporate { placement: Placement::Planned } => {
                let growth = growth.expect("resolved above for this route");
                let provenance = self.taken_from(eaten);

                // A mirrored pair splits the mass it came from, so the budget
                // stays honest however symmetric the body becomes.
                let parts = if growth.mirror.is_some() { 2 } else { 1 };
                let each = eaten.biomass_mg() / parts;

                let Ok(part) = self.controlled_body_mut().attach(
                    eaten.volume(),
                    each,
                    eaten.half_extent(),
                    crate::growth::attachment(&growth),
                    provenance.clone(),
                ) else {
                    return (Outcome::Rejected(Rejection::NoRoom), 0);
                };

                // No energy. Growing is the slow answer, and a meal cannot be
                // both meals.
                let outcome = match crate::growth::mirror_attachment(&growth) {
                    Some(mirrored) => match self.controlled_body_mut().attach(
                        eaten.volume(),
                        each,
                        eaten.half_extent(),
                        mirrored,
                        provenance,
                    ) {
                        Ok(mirror) => Outcome::IncorporatedPair { part, mirror },
                        Err(_) => Outcome::Incorporated { part },
                    },
                    None => Outcome::Incorporated { part },
                };
                (outcome, 0)
            }
        }
    }

    /// The played critter's anatomy, mutably. Only reached after control
    /// has been confirmed, so the fallback is unreachable in practice.
    fn controlled_body_mut(&mut self) -> &mut BodyDocument {
        let id = self.controlled.expect("metabolize checks embodiment first");
        &mut self
            .organisms
            .iter_mut()
            .find(|o| o.id == id)
            .expect("metabolize checks embodiment before landing a meal")
            .body
    }

    /// Provenance for a part taken off `eaten`.
    fn taken_from(&self, eaten: &Organism) -> Provenance {
        Provenance {
            origin: Origin::Incorporated { from_species: eaten.species, from_part: PartId(0) },
            epoch: self.epoch,
        }
    }

    /// Whether the played critter could touch `target`, and why not if not.
    ///
    /// **Anatomy answers this now.** It used to be `REACH = 8` regardless of
    /// what a critter was shaped like, so a twelve-limbed creature and a cube
    /// reached exactly as far.
    fn reach_to(&self, target: [i32; 3]) -> Result<(), Unmet> {
        let Some(me) = self.controlled() else {
            return Err(Unmet::TooFar { reach: 0, distance: i32::MAX });
        };
        let distance = (0..3)
            .map(|axis| (target[axis] - me.position[axis]).abs())
            .max()
            .unwrap_or(0);
        me.body.can_reach(distance)
    }

    /// Whether the played critter can touch a point. Anatomy decides.
    pub fn in_reach(&self, target: [i32; 3]) -> bool {
        self.within_reach(target)
    }

    /// How far the played critter can touch, from its anatomy.
    pub fn reach(&self) -> i32 {
        self.body().map(|b| b.reach()).unwrap_or(0)
    }

    fn within_reach(&self, target: [i32; 3]) -> bool {
        self.reach_to(target).is_ok()
    }

    /// The played body's collision extent in body space, when there is one.
    pub fn collision(&self) -> Option<Aabb> {
        self.body().map(|b| b.aabb())
    }

    pub fn total_mass_mg(&self) -> u64 {
        self.body().map(|b| b.total_mass_mg()).unwrap_or(0)
    }

    /// Ends the current epoch. Bodies change between epochs, not during them.
    pub fn end_epoch(&mut self) {
        self.epoch += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Walks the critter to its nearest neighbour and returns it.
    ///
    /// Since reach became anatomy rather than a constant, a starting critter
    /// touches about three voxels, so a fixture has to travel like a player
    /// does instead of assuming a meal is adjacent.
    fn near_organism(world: &mut World) -> OrganismId {
        for _ in 0..400 {
            let here = world.position().expect("embodied");
            let Some((id, at)) = world
                .organisms
                .iter()
                .filter(|m| Some(m.id) != world.controlled_id() && m.is_alive())
                .map(|m| (m.id, m.position))
                .min_by_key(|(_, at): &(_, [i32; 3])| {
                    (0..3).map(|a| (at[a] - here[a]).abs()).max().unwrap_or(0)
                })
            else {
                break;
            };
            if world.in_reach(at) {
                return id;
            }
            let step = [0, 1, 2].map(|a| (at[a] - here[a]).signum());
            world.apply(Intent::Move { delta: step });
        }
        panic!("nothing came within reach")
    }

    #[test]
    fn same_seed_builds_the_same_world() {
        let a = World::new(1234, 12);
        let b = World::new(1234, 12);
        assert_eq!(a, b);
    }

    #[test]
    fn different_seeds_build_different_worlds() {
        let a = World::new(1, 12);
        let b = World::new(2, 12);
        assert_ne!(a.organisms, b.organisms);
    }

    #[test]
    fn metabolize_grows_mass_and_collision() {
        let mut world = World::new(99, 24);
        let target = near_organism(&mut world);
        let mass_before = world.total_mass_mg();
        let box_before = world.collision().unwrap();

        let outcome = world.apply(Intent::Metabolize { organism: target, route: Route::Incorporate { placement: Placement::Explicit { parent: world.body().unwrap().root, offset: [5, 0, 0], yaw: Yaw::Zero } } });

        assert!(matches!(outcome, Outcome::Incorporated { .. }));
        assert!(world.total_mass_mg() > mass_before);
        assert!(world.collision().unwrap().extent()[0] > box_before.extent()[0]);
    }

    #[test]
    fn metabolize_records_where_the_part_came_from() {
        let mut world = World::new(7, 24);
        let target = near_organism(&mut world);
        let eaten_species = world
            .organisms
            .iter()
            .find(|m| m.id == target)
            .map(|m| m.species)
            .unwrap();

        let Outcome::Incorporated { part } = world.apply(Intent::Metabolize { organism: target, route: Route::Incorporate { placement: Placement::Explicit { parent: world.body().unwrap().root, offset: [4, 0, 0], yaw: Yaw::Zero } } }) else {
            panic!("expected incorporation");
        };

        let provenance = &world.body().unwrap().part(part).unwrap().provenance;
        assert_eq!(
            provenance.origin,
            Origin::Incorporated { from_species: eaten_species, from_part: PartId(0) }
        );
    }

    #[test]
    fn out_of_reach_organisms_are_refused() {
        let mut world = World::new(5, 4);
        world.organisms.push(Organism {
            stage: Stage::Mature,
            ..Organism::founding(
                OrganismId(900),
                SpeciesId(3),
                Kingdom::Producer,
                VolumeRef::from_tag(2),
                [1, 1, 1],
                [500, 0, 0],
                100,
            )
        });
        let outcome = world.apply(Intent::Metabolize { organism: OrganismId(900), route: Route::Incorporate { placement: Placement::Explicit { parent: world.body().unwrap().root, offset: [1, 0, 0], yaw: Yaw::Zero } } });
        // Five hundred voxels away, and the body says why rather than only
        // that it failed.
        assert!(matches!(
            outcome,
            Outcome::Rejected(Rejection::OutOfReach(crate::process::Unmet::NoProcess { .. }))
        ), "got {outcome:?}");
    }

    #[test]
    fn rejected_intents_still_advance_the_tick() {
        let mut world = World::new(11, 2);
        let before = world.tick;
        let outcome = world.apply(Intent::Metabolize { organism: OrganismId(4242), route: Route::Incorporate { placement: Placement::Explicit { parent: world.body().unwrap().root, offset: [0, 0, 0], yaw: Yaw::Zero } } });
        assert_eq!(outcome, Outcome::Rejected(Rejection::NoSuchOrganism(OrganismId(4242))));
        assert_eq!(world.tick, before + 1);
    }

    #[test]
    fn movement_spends_the_budget() {
        // Movement is five, and upkeep takes its share of the same budget in
        // the same tick. Before the ledgers were reconciled, upkeep came out
        // of a scalar the player's budget could not see.
        let mut world = World::new(3, 2);
        let before = world.energy_mg().unwrap();
        let upkeep = world.controlled().unwrap().upkeep_mg();

        world.apply(Intent::Move { delta: [3, 0, -2] });

        assert_eq!(world.energy_mg().unwrap(), before - 5 - upkeep);
    }

    #[test]
    fn a_bigger_body_costs_more_to_carry() {
        // The reconciliation, in one assertion: upkeep is a function of what
        // a critter is made of. Flat upkeep is why growing used to be free.
        let world = World::new(3, 2);
        let small = world.controlled().unwrap().upkeep_mg();

        let mut grown = world.clone();
        let me = grown.controlled_id().unwrap();
        {
            let organism = grown.organisms.iter_mut().find(|o| o.id == me).unwrap();
            organism.gain_mass(5_000);
        }

        assert!(
            grown.controlled().unwrap().upkeep_mg() > small,
            "a heavier body pays more rent: {} vs {}",
            grown.controlled().unwrap().upkeep_mg(),
            small
        );
    }

    #[test]
    fn deposit_returns_matter_to_the_enclosure() {
        let mut world = World::new(3, 2);
        let count = world.organisms.len();
        let outcome = world.apply(Intent::Deposit { mass_mg: 200 });
        assert!(matches!(outcome, Outcome::Deposited { .. }));
        assert_eq!(world.organisms.len(), count + 1);
    }
}
