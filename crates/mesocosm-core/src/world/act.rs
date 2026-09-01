// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What an intent does.
//!
//! Split out of `world.rs` at the 600-line ceiling. Everything here mutates the
//! world, and nothing else in the crate does: `World::apply` is the only door,
//! and this is the room behind it.

use crate::body::{Attachment, BodyDocument, Origin, PartId, Provenance};
use crate::flow::{Account, FlowEvent, Subject};
use crate::organism::{Organism, OrganismId};
use crate::places::step_for;

use super::records::Landed;
use super::{Intent, Outcome, Placement, Rejection, Route, World};

/// Energy spent per unit of movement, in milligrams.
const MOVE_COST_MG: u64 = 1;

impl World {
    pub(super) fn resolve(&mut self, intent: Intent) -> Outcome {
        // Every acting intent needs somebody to act. Nobody home is a
        // refusal, not a panic: a world can outlive whoever was in it.
        if !matches!(
            intent,
            Intent::Idle | Intent::Resume | Intent::TakeControl { .. }
        ) && !self.is_embodied()
        {
            return Outcome::Rejected(Rejection::Disembodied);
        }

        match intent {
            Intent::Idle => Outcome::Idled,

            // Answering costs nothing and moves nothing. What it does is reset
            // the idle run — a hand that says "carry on" is a hand on the
            // critter, and the ecology must not take the body back for having
            // been asked a question. (PE1.)
            Intent::Resume => Outcome::Resumed,

            Intent::Carve { at, radius } => {
                // Reach is anatomy's, same as eating: a stubby body digs at
                // its feet, a limbed one reaches further.
                if !self.in_reach(at) {
                    let Some(me) = self.controlled() else {
                        return Outcome::Rejected(Rejection::Disembodied);
                    };
                    let distance = (0..3)
                        .map(|a| (at[a] - me.position[a]).abs())
                        .max()
                        .unwrap_or(0);
                    return Outcome::Rejected(Rejection::OutOfReach(
                        crate::process::Unmet::TooFar {
                            reach: self.reach(),
                            distance,
                        },
                    ));
                }
                if !(1..=2).contains(&radius) {
                    return Outcome::Rejected(Rejection::OutOfReach(
                        crate::process::Unmet::TooFar {
                            reach: 2,
                            distance: radius,
                        },
                    ));
                }
                let removed = self.ground.carve(at, radius);
                Outcome::Carved { at, removed }
            }

            Intent::Move { delta } => {
                let Some((from, energy_mg, shape)) = self.controlled().map(|organism| {
                    (
                        organism.position,
                        organism.energy_mg,
                        organism.walker_shape(),
                    )
                }) else {
                    return Outcome::Rejected(Rejection::Disembodied);
                };
                let toward = [
                    from[0].saturating_add(delta[0]),
                    from[1].saturating_add(delta[1]),
                    from[2].saturating_add(delta[2]),
                ];
                let next = step_for(&self.ground, shape, from, toward);
                // Gravity is not an exertion. Spend only for horizontal ground
                // actually covered, so a wall does not consume an arbitrary
                // requested displacement and a long input cannot buy a
                // teleport.
                let distance = u64::from((next[0] - from[0]).unsigned_abs())
                    + u64::from((next[2] - from[2]).unsigned_abs());
                let cost = distance * MOVE_COST_MG;
                if cost > energy_mg {
                    return Outcome::Rejected(Rejection::InsufficientMass);
                }
                let Some(me) = self.controlled_mut() else {
                    return Outcome::Rejected(Rejection::Disembodied);
                };
                me.energy_mg -= cost;
                me.position = next;
                let traveller = Subject::of(me);
                // Travel is paid in substance, and it lands in the ground it
                // was covered over rather than vanishing. (TD6)
                let column = self.soil.column_at(from);
                self.soil.deposit(column, cost);
                self.flow(
                    from,
                    FlowEvent::returned(
                        crate::flow::Process::Travel,
                        traveller,
                        Account::Reserve,
                        cost,
                    ),
                );
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
                Outcome::Speciated {
                    species,
                    from,
                    founder: id,
                }
            }

            Intent::TakeControl { organism } => {
                if let Err(why) = self.eligibility(organism) {
                    return Outcome::Rejected(Rejection::Ineligible(why));
                }
                if let Some(species) = self
                    .organisms
                    .iter()
                    .find(|o| o.id == organism)
                    .map(|o| o.species)
                {
                    self.unlocked.insert(species);
                }
                if let Some(taken) = self.organisms.iter().find(|o| o.id == organism) {
                    self.frontier = self.frontier.max(self.intricacy(taken));
                }
                self.controlled = Some(organism);
                Outcome::Inhabited { organism }
            }

            Intent::Metabolize {
                organism,
                placement,
            } => self.metabolize(organism, placement),

            // **Enriching the ground**, since TD6. It used to spawn a carcass
            // — a scrap of loose matter waiting for a decomposer — which was
            // the only detritus the enclosure had. Now the enclosure has a
            // soil store, so a deposit is what it always sounded like: the
            // player putting matter back into the column they are standing on,
            // where a producer can draw it up again. The `organism` named by
            // the outcome is the depositor, not a corpse that no longer
            // exists.
            Intent::Deposit { mass_mg } => {
                let Some((id, position, depositor)) = self
                    .controlled()
                    .map(|me| (me.id, me.position, Subject::of(me)))
                else {
                    return Outcome::Rejected(Rejection::Disembodied);
                };
                {
                    let me = self.controlled_mut().expect("checked above");
                    if mass_mg == 0 || mass_mg > me.energy_mg {
                        return Outcome::Rejected(Rejection::InsufficientMass);
                    }
                    me.energy_mg -= mass_mg;
                }
                let column = self.soil.column_at(position);
                self.soil.deposit(column, mass_mg);
                self.flow(
                    position,
                    FlowEvent::returned(
                        crate::flow::Process::Deposit,
                        depositor,
                        Account::Reserve,
                        mass_mg,
                    ),
                );
                Outcome::Deposited { organism: id }
            }
        }
    }

    /// Eat something. The one verb, in one place, so every meal pays the same
    /// costs whatever it becomes.
    ///
    /// **The body routes it.** Burn-or-build is not carried by the intent and
    /// not asked of the player: a critter whose budget is inside
    /// [`STARVED_UPKEEP_TICKS`] of empty burns the meal, and one with room to
    /// spare builds with it at `placement`. The state that decides is the one
    /// the vitals panel is already showing, which is what makes the choice
    /// diegetic rather than a second hotkey (ruled 2026-08-29, TD4). Recorded
    /// traces are unaffected: the budget is world state, so a replay reaches
    /// the same decision on the same tick.
    ///
    /// **One transaction.** Everything that can refuse is checked before the
    /// organism leaves the roster, a failed attachment puts it back, and the
    /// energy ledger moves only once the meal has actually landed. An earlier
    /// cut consumed the meal and charged its venom before the attachment was
    /// known to succeed.
    ///
    /// [`STARVED_UPKEEP_TICKS`]: super::STARVED_UPKEEP_TICKS
    fn metabolize(&mut self, organism: OrganismId, placement: Placement) -> Outcome {
        let route = if self.is_starved() {
            Route::Burn
        } else {
            Route::Incorporate { placement }
        };
        if Some(organism) == self.controlled {
            return Outcome::Rejected(Rejection::Itself);
        }
        let Some(index) = self.organisms.iter().position(|m| m.id == organism) else {
            return Outcome::Rejected(Rejection::NoSuchOrganism(organism));
        };
        if let Route::Incorporate {
            placement: Placement::Explicit { parent, .. },
        } = route
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
            Route::Incorporate {
                placement: Placement::Planned,
            } => {
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
        let body_before = matches!(route, Route::Incorporate { .. })
            .then(|| self.body().cloned())
            .flatten();

        // Read before the meal lands, because landing it changes the anatomy a
        // subject is read off. A meal is credited to the body that took it.
        let Some((eater, eater_at)) = self.controlled().map(|me| (Subject::of(me), me.position))
        else {
            return Outcome::Rejected(Rejection::Disembodied);
        };
        let eaten = self.organisms.remove(index);
        let meal = Subject::of(&eaten);
        let (outcome, landed) = self.land(&eaten, route, growth);

        if matches!(outcome, Outcome::Rejected(_)) {
            // Nothing landed, so nothing was eaten and nothing is owed.
            self.organisms.insert(index, eaten);
            return outcome;
        }

        // A body cannot grow through Ground. Land against a cloned rollback
        // point because planned incorporation may add a mirrored pair: either
        // the complete live anatomy fits this stance, or the meal and body are
        // both restored.
        if let Some(before) = body_before
            && self.controlled().is_some_and(|organism| {
                !organism
                    .walker_shape()
                    .stands(&self.ground, organism.position)
            })
        {
            *self.controlled_body_mut() = before;
            self.organisms.insert(index, eaten);
            return Outcome::Rejected(Rejection::NoRoom);
        }

        // **Gains before costs.** Subtracting venom first let a nearly starved
        // critter lose part of a toxin to the zero floor and then collect the
        // full meal, so being close to death made a venomous thing *safer*.
        //
        // The floor itself remains: energy is unsigned, so venom beyond what a
        // critter has is forgiven rather than owed. A debt or damage model is a
        // later decision, recorded in the phenotype plan.
        let mut spilled = 0;
        if let Some(me) = self.controlled_mut() {
            let before = me.energy_mg + landed.budget_mg;
            me.energy_mg = before.saturating_sub(eaten.venom_mg);
            spilled = before - me.energy_mg;
        }
        // Everything the meal was that the eater did not keep goes into the
        // ground under it: the reserve it was carrying, the half of a mirrored
        // pair that would not attach, an odd milligram a split could not
        // halve, and what a bite of venom cost to bring up. (TD6)
        let column = self.soil.column_at(eaten.position);
        let unkept = eaten.biomass_mg() - landed.budget_mg - landed.body_mg;
        self.soil
            .deposit(column, unkept + eaten.energy_mg + spilled);
        self.record_meal(&eaten, meal, eater, eater_at, &landed, unkept, spilled);
        self.learn_from(&eaten);
        outcome
    }

    /// Teaches the eater's line whatever the meal knew how to grow.
    ///
    /// **The acquisition half of kleptoplasty**, ruled 2026-08-03: a lineage
    /// cannot express an appendage it has never eaten, so incorporation is
    /// developmental rather than decorative. Eating teaches your line a word;
    /// the recipe decides where to say it. A word already known is just food,
    /// which is what makes the first one a discovery.
    fn learn_from(&mut self, eaten: &Organism) {
        let Some(eater) = self.controlled().map(|o| (o.id, o.species)) else {
            return;
        };
        let Some(taught) = self.lineages.get(eaten.species).map(|s| {
            s.recipe
                .tagmata
                .iter()
                .map(|t| t.appendage)
                .filter(|a| !a.is_innate())
                .collect::<Vec<_>>()
        }) else {
            return;
        };

        let mut learned = Vec::new();
        if let Some(species) = self.lineages.get_mut(eater.1) {
            for appendage in taught {
                if species.recipe.acquire(appendage) {
                    learned.push(appendage);
                }
            }
        }
        let place = self.acted_at(Some(eater.0));
        let tick = self.tick;
        for appendage in learned {
            self.pending.push(crate::flow::Envelope::new(
                tick,
                place,
                crate::history::Event::Learned {
                    organism: eater.0,
                    species: eater.1,
                    appendage,
                },
            ));
        }
    }

    /// Attempts the routed outcome, returning it and where the meal's mass
    /// went. Mutates the body but never the roster or the ledger.
    fn land(
        &mut self,
        eaten: &Organism,
        route: Route,
        growth: Option<crate::growth::Growth>,
    ) -> (Outcome, Landed) {
        match route {
            Route::Burn => (
                Outcome::Burned {
                    organism: eaten.id,
                    energy_mg: eaten.biomass_mg(),
                },
                Landed {
                    budget_mg: eaten.biomass_mg(),
                    body_mg: 0,
                },
            ),
            Route::Incorporate {
                placement:
                    Placement::Explicit {
                        parent,
                        offset,
                        yaw,
                    },
            } => {
                let provenance = self.taken_from(eaten);
                let attached = self.controlled_body_mut().attach(
                    eaten.volume(),
                    eaten.biomass_mg(),
                    eaten.half_extent(),
                    Attachment {
                        parent,
                        offset,
                        yaw,
                    },
                    provenance,
                );
                match attached {
                    Ok(part) => (
                        Outcome::Incorporated { part },
                        Landed {
                            budget_mg: 0,
                            body_mg: eaten.biomass_mg(),
                        },
                    ),
                    Err(_) => (
                        Outcome::Rejected(Rejection::NoSuchParent(parent)),
                        Landed::default(),
                    ),
                }
            }
            Route::Incorporate {
                placement: Placement::Planned,
            } => {
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
                    return (Outcome::Rejected(Rejection::NoRoom), Landed::default());
                };

                // No energy. Growing is the slow answer, and a meal cannot be
                // both meals.
                let (outcome, body_mg) = match crate::growth::mirror_attachment(&growth) {
                    Some(mirrored) => match self.controlled_body_mut().attach(
                        eaten.volume(),
                        each,
                        eaten.half_extent(),
                        mirrored,
                        provenance,
                    ) {
                        Ok(mirror) => (Outcome::IncorporatedPair { part, mirror }, each * 2),
                        Err(_) => (Outcome::Incorporated { part }, each),
                    },
                    None => (Outcome::Incorporated { part }, each),
                };
                (
                    outcome,
                    Landed {
                        budget_mg: 0,
                        body_mg,
                    },
                )
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
            origin: Origin::Incorporated {
                from_species: eaten.species,
                from_part: PartId(0),
            },
            epoch: self.epoch,
        }
    }
}
