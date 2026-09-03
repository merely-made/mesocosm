// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Branch transfer, as one world transaction. (P3)
//!
//! [`crate::phenotype::graft`] owns what a branch *is* and how one is set into
//! a body. This owns the four things that are the world's rather than a
//! phenotype's, and they are the same four every landing verb here owns:
//!
//! - **who may donate.** A carcass, and not its root. PE2 settled why a corpse
//!   and not a severed part: a severed part's milligrams have already left the
//!   conservation account, so taking one would create matter. Live
//!   dismemberment is a further gate and is not this proof's.
//! - **whether it fits.** The branch root takes an ordinary plan-resolved site,
//!   and the whole arriving branch has to clear the parts already there and
//!   still leave a body that stands on this ground.
//! - **the terms.** This world's affinity table returns a verdict for the
//!   donor's tissue domain into the recipient's, and that verdict decides
//!   whether a *carry* is feasible at all and what a feasible one lands with.
//! - **the ledger.** The branch's exact milligrams leave one body's substance
//!   and arrive in the other's, as one recorded transfer naming both subjects;
//!   the development's price leaves the reserve and lands in the ground.
//!
//! # One transaction
//!
//! Everything that can refuse is checked against a **candidate** phenotype
//! before anything is published, and the donor loses nothing until the
//! recipient has landed the branch. A refusal leaves both bodies and both
//! ledgers byte-identical.

use serde::{Deserialize, Serialize};

use crate::body::{PartId, SpeciesId};
use crate::flow::{Account, FlowEvent, Subject};
use crate::graft::{Crossing, Verdict};
use crate::organism::OrganismId;
use crate::phenotype::Lowering;

use super::{Outcome, Rejection, World};

/// A branch transfer, as the world recorded it.
///
/// World state, so it survives a snapshot and comes back the same on a replay.
/// What each arriving part *is* lives on that part's own provenance; this is
/// the transaction the parts arrived through.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Graft {
    pub tick: u64,
    /// The body that took it.
    ///
    /// **Scoped, because a world outlives a body.** The record is the world's
    /// and survives succession; the branch does not, because it was on the
    /// creature that died. A panel that read this against whoever is being
    /// played now would name parts of somebody else's anatomy.
    pub recipient: OrganismId,
    /// The carcass the branch came off.
    pub donor: OrganismId,
    pub donor_line: SpeciesId,
    /// The donor-local id of the branch root: its source address.
    pub donor_part: PartId,
    /// The branch root's new id here.
    pub root: PartId,
    /// Every arriving part's new id, in branch order. Parents before children.
    pub parts: Vec<PartId>,
    pub mass_mg: u64,
    pub crossing: Crossing,
    pub verdict: Verdict,
    /// What the transfer's development cost, out of the reserve.
    pub cost_mg: u64,
    /// The phenotype revision the transfer created.
    pub revision: u32,
}

impl World {
    /// What this world says about carrying `from`'s tissue into `into`'s.
    ///
    /// The destination declaring its compatibility, which is the wing
    /// contract's own requirement. A host shows this before a player commits to
    /// a crossing rather than after.
    pub fn verdict_between(&self, from: SpeciesId, into: SpeciesId) -> Verdict {
        self.affinity
            .verdict(self.lineages.domain(from), self.lineages.domain(into))
    }

    /// What this world says about carrying that organism's tissue into the
    /// played body. `None` while nobody is embodied.
    pub fn verdict_for(&self, donor: OrganismId) -> Option<Verdict> {
        let me = self.controlled()?;
        let line = self.organisms.iter().find(|o| o.id == donor)?.species;
        Some(self.verdict_between(line, me.species))
    }

    /// This world's graft affinity table.
    pub fn affinity(&self) -> &crate::graft::Affinity {
        &self.affinity
    }

    /// The most recent branch transfer this world recorded, whoever took it.
    ///
    /// Raw state: it is what a snapshot carries and what a replay has to agree
    /// about. A reading that speaks *about the body in front of the player*
    /// wants [`Self::carried_branch`] instead.
    pub fn last_graft(&self) -> Option<&Graft> {
        self.last_graft.as_ref()
    }

    /// The branch the **played** body is carrying, if it took one.
    ///
    /// `None` once control has moved on, because the branch went with the body
    /// that held it. This is the reading a panel wants.
    pub fn carried_branch(&self) -> Option<&Graft> {
        self.last_graft
            .as_ref()
            .filter(|graft| Some(graft.recipient) == self.controlled)
    }

    /// Takes a branch off a carcass and sets it on the played body.
    pub(super) fn graft(
        &mut self,
        organism: OrganismId,
        part: PartId,
        crossing: Crossing,
    ) -> Outcome {
        if Some(organism) == self.controlled {
            return Outcome::Rejected(Rejection::Itself);
        }
        let Some(index) = self.organisms.iter().position(|o| o.id == organism) else {
            return Outcome::Rejected(Rejection::NoSuchOrganism(organism));
        };
        let donor = &self.organisms[index];
        // Live dismemberment is a further gate (phenotype D3a). This proof's
        // donor has stopped.
        if donor.is_alive() {
            return Outcome::Rejected(Rejection::StillLiving(organism));
        }
        let body = donor.body();
        let Some(found) = body.part(part) else {
            return Outcome::Rejected(Rejection::NoSuchPart(part));
        };
        if part == body.root {
            return Outcome::Rejected(Rejection::WholeBody(part));
        }
        if found.severed {
            return Outcome::Rejected(Rejection::NothingLeft(part));
        }
        let (donor_at, line) = (donor.position, donor.species);
        let carrion = Subject::of(donor);
        let Some(branch) = donor.phenotype.harvest(part) else {
            return Outcome::Rejected(Rejection::NothingLeft(part));
        };
        let mass_mg = branch.mass_mg();
        if let Err(unmet) = self.reach_to(donor_at) {
            return Outcome::Rejected(Rejection::OutOfReach(unmet));
        }

        let Some(me) = self.controlled() else {
            return Outcome::Rejected(Rejection::Disembodied);
        };
        // **The terms, declared before anything moves.** A carry the table
        // refuses is refused here and not silently rewritten into a regrowth;
        // the player is told which boundary failed and regrowth is the route
        // that remains.
        let (from, into) = (self.lineages.domain(line), self.lineages.domain(me.species));
        let verdict = self.affinity.verdict(from, into);
        let lowering = match (crossing, verdict) {
            (Crossing::Carry, Verdict::Native) => Lowering::Carried,
            (Crossing::Carry, Verdict::Adapter) => Lowering::Adapted,
            (Crossing::Carry, Verdict::Refused) => {
                return Outcome::Rejected(Rejection::Incompatible { from, into });
            }
            (Crossing::Regrow, _) => Lowering::Regrown,
        };

        // Where the branch goes, by the ordinary body plan. The plan is asked
        // about the **branch**, not about its root: a branch keeps its own
        // joints, so room for the part at the top of it is not room for the
        // thing. Resolved before the corpse gives anything up.
        let (centre, half_extent) = branch.bounds();
        let Some(growth) = crate::growth::resolve(me.body(), half_extent) else {
            return Outcome::Rejected(Rejection::NoRoom);
        };
        // The plan sited the branch's box; the root is offset from its centre
        // by however far off-centre the root sits inside its own branch.
        let mut site = crate::growth::attachment(&growth);
        site.offset = [0, 1, 2].map(|axis| site.offset[axis] - centre[axis]);
        let (id, position, energy_mg, eater) = (me.id, me.position, me.energy_mg, Subject::of(me));

        // A candidate, so a refusal costs nothing. One clone of one body at a
        // deliberate moment, the same discipline the developmental verb uses.
        let mut candidate = me.phenotype.clone();
        let graftage = match candidate.receive(self.ruleset(), &branch, site, self.epoch, lowering)
        {
            Ok(graftage) => graftage,
            Err(refusal) => return Outcome::Rejected(Rejection::Refused(refusal)),
        };
        // A branch is placed by its root and shaped by its own joints, so
        // clearing the root's site is not enough: every arriving part has to
        // clear what was already there.
        if !clears(candidate.body(), &graftage.parts) {
            return Outcome::Rejected(Rejection::NoRoom);
        }
        // A body cannot grow through Ground. Asked of the candidate rather than
        // of a published body, so this refusal has nothing to undo: the whole
        // transaction is still a value nobody else can see.
        if !crate::places::WalkerShape::from_aabb(candidate.body().aabb())
            .stands(&self.ground, position)
        {
            return Outcome::Rejected(Rejection::NoRoom);
        }
        if graftage.cost_mg > energy_mg {
            return Outcome::Rejected(Rejection::InsufficientMass);
        }

        let revision = graftage.development.instruction.revision;
        let cost_mg = graftage.cost_mg;
        {
            let organism = self
                .organisms
                .iter_mut()
                .find(|o| o.id == id)
                .expect("the controlled organism was just read");
            organism.phenotype = candidate;
            organism.energy_mg -= cost_mg;
        }

        // **The source loses the branch**, and only now. Severing cascades, so
        // this takes exactly what was harvested.
        let lost = self.organisms[index].phenotype.sever(part);
        debug_assert_eq!(lost.len(), branch.len(), "the branch left whole");

        self.flow(
            position,
            FlowEvent::between(
                crate::flow::Process::Graft,
                carrion,
                Account::Substance,
                eater,
                Account::Substance,
                mass_mg,
            ),
        );
        if cost_mg > 0 {
            let column = self.soil.column_at(position);
            self.soil.deposit(column, cost_mg);
            self.flow(
                position,
                FlowEvent::returned(
                    crate::flow::Process::Develop,
                    eater,
                    Account::Reserve,
                    cost_mg,
                ),
            );
        }
        self.last_graft = Some(Graft {
            tick: self.tick,
            recipient: id,
            donor: organism,
            donor_line: line,
            donor_part: part,
            root: graftage.root,
            parts: graftage.parts.clone(),
            mass_mg,
            crossing,
            verdict,
            cost_mg,
            revision,
        });
        Outcome::Grafted {
            root: graftage.root,
            parts: graftage.parts.len() as u32,
            from: organism,
            from_part: part,
            mass_mg,
            crossing,
            verdict,
        }
    }
}

/// Whether every newly arrived part clears the parts that were already there.
///
/// Flush placement makes touching parts exactly adjacent rather than
/// overlapping, so this is the same strict comparison `growth` uses, asked of
/// the whole branch instead of one site.
///
/// **Pairs inside the branch are skipped on purpose.** The branch was a
/// disjoint piece of a real body a moment ago and it arrives rigid, joint for
/// joint, so its parts cannot have started overlapping each other on the way
/// across. What is new is the company it is keeping.
fn clears(body: &crate::body::BodyDocument, arrived: &[PartId]) -> bool {
    for part in arrived {
        let (Some(at), Some(found)) = (body.world_pivot(*part), body.part(*part)) else {
            return false;
        };
        for other in body.living() {
            if arrived.contains(&other.id) {
                continue;
            }
            let Some(centre) = body.world_pivot(other.id) else {
                continue;
            };
            let overlaps = (0..3).all(|axis| {
                (at[axis] - centre[axis]).abs()
                    < found.half_extent[axis].abs() + other.half_extent[axis].abs()
            });
            if overlaps {
                return false;
            }
        }
    }
    true
}
