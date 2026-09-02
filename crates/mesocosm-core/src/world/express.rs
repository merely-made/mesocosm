// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The developmental verb: a discovered candidate, expressed and paid for.
//! (PD3)
//!
//! **The door that replaced PD2's editor operation.** `Intent::Rearrange`
//! carried a complete hand-authored allocation and was permitted only until a
//! packed or scripted path could reach the same validator; the processdef
//! plan §9 named its whole deletion checklist, and this file is what walks in
//! after it. What a development *is* now comes from two places a host does not
//! own — the admitted ruleset and the line's own discovery record — so nothing
//! outside the game's rules can decide what tissue moves.
//!
//! # What this file adds that the validator does not
//!
//! The same three things `world/rearrange.rs` added, unchanged, because they
//! were never the temporary part:
//!
//! - **the price.** `Instruction::cost_cells` counts tissue whose expression
//!   changed, times `BodyPhenotype::cell_mg(part)`: the part's own adult-mass
//!   ceiling divided by its mosaic's living cell count.
//! - **the payment.** The milligram leaves the body's reserve and lands in the
//!   ground under it. Rebuilding an organ is work, and work in this world is
//!   matter moving somewhere else, never matter ceasing to exist. (TD6)
//! - **the record.** Expressing a candidate is a biographical event, so it
//!   enters history beside the meals and the burrows.

use crate::discovery::ConditionId;
use crate::flow::{Account, FlowEvent, Subject};
use crate::phenotype::Arrangement;

use super::{Outcome, Rejection, World};

impl World {
    /// Expresses what a condition granted, charging the body for the tissue
    /// that changed hands.
    ///
    /// Ordered so nothing is spent before everything is known: the proposal is
    /// built from the discovery, validated against a clone, the price is read
    /// off the validated instruction, the reserve is checked, and only then
    /// does the real phenotype move. A body that cannot afford the development
    /// keeps both its milligrams and its old arrangement.
    pub(super) fn express(&mut self, condition: ConditionId) -> Outcome {
        if self.controlled().is_none() {
            return Outcome::Rejected(Rejection::Disembodied);
        }
        if !self.discovered(condition) {
            return Outcome::Rejected(Rejection::Undiscovered(condition));
        }
        // The player's hand is on it. `Arrangement` is diagnostic and the
        // validator never reads it, which is what makes the direct and
        // automatic parity receipt a property rather than a coincidence.
        let Some(proposal) = self.candidate_proposal(condition, Arrangement::Direct) else {
            return Outcome::Rejected(Rejection::Nowhere(condition));
        };
        // One part, because the candidates are one organ each and the price is
        // read in that part's own tissue.
        let Some(&part) = proposal.parts.first() else {
            return Outcome::Rejected(Rejection::Nowhere(condition));
        };

        let me = self.controlled().expect("checked above");
        let (id, position, energy_mg) = (me.id, me.position, me.energy_mg);

        // A dry run, so a refusal costs nothing and an unaffordable
        // development does not land half of itself. The clone is one body, at
        // a developmental moment, not per tick.
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
        Outcome::Expressed {
            part,
            cost_mg,
            revision,
        }
    }
}
