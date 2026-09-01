// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The developmental verb: one part's tissue, moved and paid for. (PD2)
//!
//! **A temporary authoring path.** The processdef plan permits PD2 exactly one
//! of these — "a native developmental fixture or an explicit editor operation"
//! — so that a process can be played before packs and the Piccolo bridge
//! exist, and requires it to be deleted when they arrive. PD3 encodes the
//! definition as pack data; PD4 removes this door. What is *not* temporary is
//! everything under it: [`BodyPhenotype::develop`] is the one validator, and
//! automatic arrangement already comes through it.
//!
//! # What this file adds that the validator does not
//!
//! Three things, and they are all the world's rather than the phenotype's:
//!
//! - **the price.** `Instruction::cost_cells` counts tissue whose expression
//!   changed; PD1b deliberately left it a count, because inventing a milligram
//!   with no consumer would have been a number picked to look reasonable. It
//!   has a consumer now, so it is priced here in the part's own cells.
//! - **the payment.** The milligram leaves the body's reserve and lands in the
//!   ground under it. Rebuilding an organ is work, and work in this world is
//!   matter moving somewhere else, never matter ceasing to exist. (TD6)
//! - **the record.** A rearrangement is a biographical event, so it enters
//!   history beside the meals and the burrows.

use crate::body::PartId;
use crate::flow::{Account, FlowEvent, Subject};
use crate::phenotype::{AllocationProposal, Arrangement, ProposedSite};

use super::intent::Allocate;
use super::{Outcome, Rejection, World};

impl World {
    /// Moves one part's allocation, charging the body for the tissue that
    /// changed hands.
    ///
    /// Ordered so nothing is spent before everything is known: the proposal is
    /// validated against a clone, the price is read off the validated
    /// instruction, the reserve is checked, and only then does the real
    /// phenotype move. A body that cannot afford the development keeps both
    /// its milligrams and its old arrangement.
    pub(super) fn rearrange(&mut self, part: PartId, sites: &[Allocate]) -> Outcome {
        let Some(me) = self.controlled() else {
            return Outcome::Rejected(Rejection::Disembodied);
        };
        let (id, position, energy_mg) = (me.id, me.position, me.energy_mg);
        let proposal = AllocationProposal {
            expect: me.phenotype.digest(),
            // The player's hand is on it. `Arrangement` is diagnostic and the
            // validator never reads it, which is what makes the direct and
            // automatic parity receipt a property rather than a coincidence.
            source: Arrangement::Direct,
            parts: vec![part],
            sites: sites
                .iter()
                .map(|site| ProposedSite {
                    part,
                    process: site.process,
                    cells: site.cells.clone(),
                })
                .collect(),
        };

        // A dry run, so a refusal costs nothing and an unaffordable
        // development does not land half of itself. The clone is one body, at
        // an editor moment, not per tick.
        let mut candidate = me.phenotype.clone();
        let development = match candidate.develop(&proposal) {
            Ok(development) => development,
            Err(refusal) => return Outcome::Rejected(Rejection::Refused(refusal)),
        };
        let cost_mg = u64::from(development.instruction.cost_cells) * me.phenotype.cell_mg(part);
        if cost_mg > energy_mg {
            return Outcome::Rejected(Rejection::InsufficientMass);
        }

        let revision = development.instruction.revision;
        let subject = Subject::of(me);
        let organism = self
            .organisms
            .iter_mut()
            .find(|organism| organism.id == id)
            .expect("the controlled organism was just read");
        organism.phenotype = candidate;
        organism.energy_mg -= cost_mg;

        let column = self.soil.column_at(position);
        self.soil.deposit(column, cost_mg);
        self.flow(
            position,
            FlowEvent::returned(
                crate::flow::Process::Develop,
                subject,
                Account::Reserve,
                cost_mg,
            ),
        );
        // The history event is `records::event_for`'s, like every other
        // outcome's: one commit point, so a refusal cannot leave a record of a
        // thing that did not happen.
        Outcome::Rearranged {
            part,
            cost_mg,
            revision,
        }
    }
}
