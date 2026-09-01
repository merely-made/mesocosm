// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Anatomy and allocation, held together.
//!
//! # Why a wrapper
//!
//! PD1a compared three owners for process allocation and ruled this one. The
//! two it rejected are worth keeping visible, because both are the shape this
//! file exists to prevent:
//!
//! - **Allocation fields inside [`Part`](crate::body::Part)** would make the
//!   structural anatomy document depend on Mesocosm's process semantics, and
//!   change the body-only bytes mesh, Lens and other vessels consume.
//! - **A freely mutable allocation state beside a freely mutable
//!   [`BodyDocument`]** permits exactly the split account PD0 spent a
//!   migration removing: a part attached, severed or restored without its
//!   allocation following.
//!
//! So both representations stay independently *readable* while every mutation
//! that affects both is one operation. The fields here are private and no
//! caller can obtain `&mut BodyDocument` from a live organism; body-only
//! authoring still uses `BodyDocument` directly, which keeps primitive
//! topology usable without loading the biology system.
//!
//! # The invariants
//!
//! - Every living part has exactly one mosaic, and every living allocation
//!   names a living part. The mosaics are index-aligned with
//!   `BodyDocument::parts`, so an anatomy tombstone **is** an allocation
//!   tombstone; they cannot disagree because there is nothing to synchronize.
//! - Every part's mosaic conserves capacity: occupied plus free equals the
//!   count of its living cells.
//! - Rearrangement happens only through [`BodyPhenotype::develop`], which
//!   validates a complete proposal, bumps the revision, and returns the record
//!   of what it did. There is no other way in.
//!
//! # Severing keeps history
//!
//! A severed part's mosaic stays addressable through [`BodyPhenotype::mosaic`]
//! so an injury is still explainable, and is excluded from
//! [`BodyPhenotype::allocations`] so it cannot contribute. Historical cells and
//! sites remain readable; they are not capacity and they express nothing.

use serde::{Deserialize, Serialize};

use crate::body::{
    AttachError, Attachment, BodyDocument, PartId, Provenance, SpeciesId, VolumeRef,
};
use crate::process::{ProcessRef, Registry};

pub mod develop;
pub mod mosaic;

pub use develop::{
    Aim, AllocationProposal, Arrangement, Development, Instruction, ProposedSite, Refusal, arrange,
};
pub use mosaic::{CellId, Expressed, MAX_CELLS, MAX_SITES, Mosaic, Site, SiteId};

/// One critter's anatomy and its process allocation, as one transactional
/// value.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BodyPhenotype {
    body: BodyDocument,
    /// Index-aligned with `body.parts`. That alignment is the invariant: a
    /// part and its mosaic are addressed by the same number.
    mosaics: Vec<Mosaic>,
    /// Bumped by every committed development, so a proposal authored against
    /// an earlier arrangement is refusable and the order of rearrangements is
    /// on the record.
    revision: u32,
}

impl BodyPhenotype {
    /// Wraps a finished anatomy, seeding one mosaic per part from its shape.
    ///
    /// The only constructor. A `BodyDocument` becomes an organism's body by
    /// passing through here, so there is no path that produces a live body
    /// with no allocation.
    pub fn seed(body: BodyDocument) -> Self {
        let mosaics = body.parts.iter().map(Mosaic::seed).collect();
        let phenotype = Self {
            body,
            mosaics,
            revision: 0,
        };
        debug_assert!(phenotype.conserves(), "a seeded mosaic must conserve");
        phenotype
    }

    /// The anatomy, for everything that draws, weighs, places or projects it.
    pub fn body(&self) -> &BodyDocument {
        &self.body
    }

    /// The anatomy alone — the **carry this body** projection's structural
    /// half, and what a body-only consumer receives.
    pub fn into_body(self) -> BodyDocument {
        self.body
    }

    /// Every living part's allocation, in part order.
    pub fn allocations(&self) -> impl Iterator<Item = (PartId, &Mosaic)> {
        self.body
            .living()
            .map(|part| (part.id, &self.mosaics[part.id.0 as usize]))
    }

    /// One part's mosaic, severed parts included, because an injury still has
    /// to be explainable.
    pub fn mosaic(&self, part: PartId) -> Option<&Mosaic> {
        self.mosaics.get(part.0 as usize)
    }

    /// How many developments this phenotype has committed.
    pub fn revision(&self) -> u32 {
        self.revision
    }

    /// What a proposal must expect, and what a commit reports.
    ///
    /// Over anatomy, allocation and revision together, so a body that grew, an
    /// allocation that moved, and a rearrangement that happened are all
    /// staleness.
    pub fn digest(&self) -> u64 {
        let bytes = crate::snapshot::encode(self).expect("a phenotype is always encodable");
        crate::snapshot::hash_bytes(&bytes)
    }

    /// Whether any living allocation expresses this definition.
    ///
    /// The allocation-side reading of what a body does. It agrees with
    /// [`BodyDocument::performs`] by construction while geometry is the whole
    /// seeding rule, and the receipts assert that it does.
    pub fn expresses(&self, process: ProcessRef) -> bool {
        self.allocations()
            .any(|(_, mosaic)| mosaic.sites().iter().any(|site| site.process == process))
    }

    /// Every living part that expresses a definition, in part order.
    pub fn expressing(&self, process: ProcessRef) -> impl Iterator<Item = PartId> + '_ {
        self.allocations().filter_map(move |(part, mosaic)| {
            mosaic
                .sites()
                .iter()
                .any(|site| site.process == process)
                .then_some(part)
        })
    }

    /// Every expressed process and how it came to be there, sorted.
    ///
    /// What survives a re-realization: the same program under different
    /// declared conditions may grow a different body, and this is the identity
    /// and provenance that must not have moved with it.
    pub fn expressed(&self) -> Vec<(ProcessRef, Expressed)> {
        let mut found: Vec<(ProcessRef, Expressed)> = self
            .allocations()
            .flat_map(|(_, mosaic)| mosaic.sites())
            .map(|site| (site.process, site.cause))
            .collect();
        found.sort_unstable();
        found.dedup();
        found
    }

    /// A capability trace for one part: what it expresses, on how much tissue,
    /// how much is free, and where the expression came from.
    ///
    /// `None` for a part this body does not have. A severed part answers with
    /// its history, because "this used to fix, and the branch is gone" is the
    /// receipt a player is owed.
    pub fn explain(&self, part: PartId) -> Option<Explanation> {
        let mosaic = self.mosaic(part)?;
        let registry = Registry::native();
        Some(Explanation {
            part,
            living: self.body.is_living(part),
            capacity: mosaic.capacity(),
            free: mosaic.free(),
            sites: mosaic
                .sites()
                .iter()
                .map(|site| SiteReading {
                    id: site.id,
                    process: site.process,
                    // `None` is the missing-ruleset diagnostic, not a licence
                    // to name a similar local process instead.
                    named: registry.resolve(site.process).map(|def| def.id),
                    cells: site.cells.len() as u32,
                    cause: site.cause,
                })
                .collect(),
        })
    }

    /// Whether every invariant this wrapper exists to hold, holds.
    pub fn conserves(&self) -> bool {
        if self.body.parts.len() != self.mosaics.len() {
            return false;
        }
        self.allocations()
            .all(|(_, mosaic)| mosaic.conserves() && mosaic.capacity() <= MAX_CELLS)
    }

    /// Attaches a part **and** seeds its mosaic, in one operation.
    ///
    /// Either the body gained a part with an allocation or it gained nothing.
    /// This is the whole reason `Organism` does not expose `&mut BodyDocument`.
    pub fn attach(
        &mut self,
        volume: VolumeRef,
        mass_mg: u64,
        half_extent: [i32; 3],
        attachment: Attachment,
        provenance: Provenance,
    ) -> Result<PartId, AttachError> {
        let id = self
            .body
            .attach(volume, mass_mg, half_extent, attachment, provenance)?;
        let part = self.body.part(id).expect("just attached");
        self.mosaics.push(Mosaic::seed(part));
        debug_assert!(self.conserves(), "an attach must leave the mosaics whole");
        Ok(id)
    }

    /// Severs a part and everything under it, taking their allocations out of
    /// activity in the same commit.
    ///
    /// Returns what was lost. The mosaics stay addressable as history.
    pub fn sever(&mut self, part: PartId) -> Vec<PartId> {
        self.body.sever(part)
    }

    /// Which lineage this body belongs to. Forking is the only caller.
    pub fn set_species(&mut self, species: SpeciesId) {
        self.body.species = species;
    }

    /// Adds substance to the root part.
    ///
    /// **Mass is not allocation.** Growing or starving changes what a body
    /// weighs and never where its organs are: reallocating tissue requires a
    /// discrete developmental event with a cost and a causal record, which is
    /// what keeps a whole roster tractable.
    pub fn gain_root_mass(&mut self, mg: u64) -> bool {
        let root = self.body.root;
        match self.body.parts.get_mut(root.0 as usize) {
            Some(part) => {
                part.mass_mg = part.mass_mg.saturating_add(mg);
                true
            }
            None => false,
        }
    }

    /// Removes substance across the living body in stable part order,
    /// returning what could not be paid.
    pub fn spend_mass(&mut self, mg: u64) -> u64 {
        let mut unpaid = mg;
        for part in self.body.parts.iter_mut().filter(|part| !part.severed) {
            let paid = part.mass_mg.min(unpaid);
            part.mass_mg -= paid;
            unpaid -= paid;
            if unpaid == 0 {
                break;
            }
        }
        unpaid
    }

    /// **The only way allocation moves.**
    ///
    /// Validates a complete proposal against this phenotype and either commits
    /// all of it or none of it. A refusal leaves body, allocation and revision
    /// byte-identical; an acceptance bumps the revision and returns the record
    /// of what changed hands.
    pub fn develop(&mut self, proposal: &AllocationProposal) -> Result<Development, Refusal> {
        let validated = develop::validate(self, proposal)?;
        let revision = self.revision + 1;
        let mut sites = Vec::new();
        for (part, desired) in validated.rewrites {
            let mosaic = &mut self.mosaics[part.0 as usize];
            mosaic.rewrite(desired, revision);
            for site in mosaic.sites() {
                sites.push((part, site.id, site.process));
            }
        }
        self.revision = revision;
        debug_assert!(
            self.conserves(),
            "a committed development must leave the mosaics whole"
        );
        Ok(Development {
            instruction: Instruction {
                revision,
                parts: proposal.parts.clone(),
                sites,
                cost_cells: validated.cost_cells,
                digest: self.digest(),
            },
            source: proposal.source,
        })
    }
}

/// One part's allocation, in terms a journal or a failure receipt can print.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Explanation {
    pub part: PartId,
    /// `false` for a severed part, whose sites are history.
    pub living: bool,
    pub capacity: u32,
    pub free: u32,
    pub sites: Vec<SiteReading>,
}

/// One expressed process, explained.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SiteReading {
    pub id: SiteId,
    pub process: ProcessRef,
    /// The qualified id, when this world's ruleset holds the definition.
    /// `None` is the missing-ruleset diagnostic.
    pub named: Option<crate::process::ProcessId>,
    pub cells: u32,
    pub cause: Expressed,
}

#[cfg(test)]
mod tests;
