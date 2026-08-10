// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What an intent does.
//!
//! Split out of `world.rs` at the 600-line ceiling. Everything here mutates the
//! world, and nothing else in the crate does: `World::apply` is the only door,
//! and this is the room behind it.

use crate::body::{Attachment, BodyDocument, Origin, PartId, Provenance, VolumeRef};
use crate::organism::{Kingdom, Organism, OrganismId, Stage};

use super::{Intent, Outcome, Placement, Rejection, Route, World};

/// Energy spent per unit of movement, in milligrams.
const MOVE_COST_MG: u64 = 1;

impl World {
    pub(super) fn resolve(&mut self, intent: Intent) -> Outcome {
        // Every acting intent needs somebody to act. Nobody home is a
        // refusal, not a panic: a world can outlive whoever was in it.
        if !matches!(intent, Intent::Idle | Intent::TakeControl { .. }) && !self.is_embodied() {
            return Outcome::Rejected(Rejection::Disembodied);
        }

        match intent {
            Intent::Idle => Outcome::Idled,

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
        for appendage in learned {
            self.pending.push(crate::history::Event::Learned {
                organism: eater.0,
                species: eater.1,
                appendage,
            });
        }
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
                Outcome::Burned {
                    organism: eaten.id,
                    energy_mg: eaten.biomass_mg(),
                },
                eaten.biomass_mg(),
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
                    Ok(part) => (Outcome::Incorporated { part }, 0),
                    Err(_) => (Outcome::Rejected(Rejection::NoSuchParent(parent)), 0),
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
            origin: Origin::Incorporated {
                from_species: eaten.species,
                from_part: PartId(0),
            },
            epoch: self.epoch,
        }
    }
}
