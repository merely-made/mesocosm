// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What parts *do*, and what a body can do because of it.
//!
//! The first crossing of the bridge between anatomy and action. Before this,
//! a long limb was classified `Role::Limb` and produced nothing: reach was the
//! constant `REACH = 8` whatever a critter was shaped like.
//!
//! # Deliberately one capability
//!
//! The plan's stop rule is *do not add a broad process catalog before one path
//! is played*, and this is that one path. A process vocabulary authored ahead
//! of any consumer becomes a catalog, which is the Spore failure at a smaller
//! scale. Three processes existed here because one capability needed them; the
//! fourth arrived when something asked, and [`Process::Fix`] is that fourth —
//! the default creatures plan's DC1.5 needed a producer to be readable from
//! anatomy rather than from symmetry.
//!
//! # Grown and acquired are different (PD2)
//!
//! The first four are all things a shape simply *does*: grow the shape and the
//! process is there. [`Process::Secrete`] is the first that a body has to be
//! *given*, and that distinction is [`Seeding`]. Geometry seeds four
//! definitions and admits five, so the fifth is only ever where a development
//! put it — which is what makes expressing it a choice rather than a
//! consequence of having grown a plate.
//!
//! # Geometry seeds allocation; nobody edits a number
//!
//! P2's rule was *processes are read, not stored*: a part's processes were
//! derived from its geometry through [`classify`](crate::plan::classify) and
//! kept nowhere. PD1b keeps the principle and moves the boundary. Geometry is
//! now the **seeding rule** — [`Role::processes`] says what a shape expresses
//! when a part is developed — and what it seeds is
//! [allocation](crate::phenotype): tissue on a named part, occupying named
//! cells, citing the exact definition it expresses.
//!
//! The anti-Spore property survives the move intact, in two places. A
//! definition's [`ProcessDef::admits`] still gates every allocation by the
//! part's shape, so a proposal cannot put contraction on a plate; and
//! capability stays a reading over anatomy, allocation and environment rather
//! than a stored verdict. Reshaping a body still reshapes what it can do, and
//! there is still no ability field to raise.

use serde::{Deserialize, Serialize};

use crate::body::{BodyDocument, PartId};
use crate::plan::{Role, classify};

/// What a part contributes.
///
/// Small on purpose. These are *transformations*; what travels between parts
/// is a flow, and flows are a separate and even smaller vocabulary that
/// arrives when a channel graph does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Process {
    /// Turns stored energy into force. An actuator.
    Contract,
    /// Admits material into the body. A mouth, a gut, an absorbing surface.
    Intake,
    /// Receives a signal from outside.
    Sense,
    /// Makes substance out of the world itself rather than out of a meal. A
    /// leaf, a frond, a mat.
    ///
    /// **The word is provisional.** `Fix` is biology's plain working verb for
    /// it (carbon fixation) and this crate coins nothing mid-slice; the
    /// in-product name is a naming round Mark owns. Renaming the variant moves
    /// no rule, only [`ProcessId::name`] and this identifier.
    Fix,
    /// Makes a toxin out of the body's own substance and the ground under it.
    /// A gland, a nettle's hair, a stinging cell. **PD2's played process.**
    ///
    /// The first process no shape grows: a plate *admits* a gland and never
    /// seeds one, so the tissue has to be taken off whatever the plate was
    /// already doing. What it costs is standing rent; what it buys is a bite
    /// that the thing eating this body pays.
    ///
    /// **The word is provisional**, as [`Process::Fix`]'s is. `Secrete` is the
    /// plain working verb; the in-product name is a naming round Mark owns.
    Secrete,
}

impl Process {
    /// Every native binding, so a new variant cannot be added without the
    /// parity receipt below noticing.
    ///
    /// There is no exhaustive match anywhere else on this enum: without this
    /// list a sixth process would compile clean and panic inside
    /// [`Registry::of_native`] at runtime.
    pub const ALL: [Process; 5] = [
        Process::Contract,
        Process::Intake,
        Process::Sense,
        Process::Fix,
        Process::Secrete,
    ];
}

impl Role {
    /// What a part of this shape contributes.
    ///
    /// A long thin part is an actuator, a bulky one admits material, a small
    /// one senses, and a flat one spreads itself against the world and fixes.
    /// That mapping is the whole vocabulary today, and since PD1b it is the
    /// **seeding rule**: it decides what a newly developed part expresses,
    /// not what a part is permanently obliged to be.
    ///
    /// **What a shape grows, not what it will tolerate** (PD2). A plate admits
    /// [`Process::Secrete`] and does not seed it; the two questions are
    /// [`Registry::seeds`] and [`ProcessDef::admits`], and only the first one
    /// is this.
    pub fn processes(self) -> &'static [Process] {
        // The registry is the definition of record (PD1b); this remains the
        // fast native view of it, and the parity receipt in `process/tests.rs`
        // keeps the two from drifting. Seeding itself reads the registry.
        match self {
            Role::Limb => &[Process::Contract],
            Role::Mass => &[Process::Intake],
            Role::Sensor => &[Process::Sense],
            // "Fins, plates, leaves": two long axes and one short is the shape
            // that presents area to the world, which is what fixing needs. A
            // plate also resists things, and resisting is still not a process
            // because nothing reads it; it becomes one when damage does.
            Role::Plate => &[Process::Fix],
        }
    }
}

/// What a body can currently do.
///
/// One variant, and it stays one until a second capability is actually played.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Capability {
    /// Touching something at a distance, which is what eating requires.
    Reach,
}

/// What a living body does with available matter. This is derived from the
/// body's trophic signature and expressed processes, so the ecology does not
/// need a parallel predator flag.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum FeedingMode {
    Producer,
    Grazer,
    Predator,
    Scavenger,
}

/// Why a body cannot do something.
///
/// Carried into rejections so a receipt says *which embodied requirement
/// failed* rather than only that something failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Unmet {
    /// No living part performs the process this capability needs.
    NoProcess {
        capability: Capability,
        needs: Process,
    },
    /// The body can do it, but not that far.
    TooFar { reach: i32, distance: i32 },
}

/// Reach a body has without any actuator: its own bulk.
///
/// A creature can always touch what is against it. This is not a floor added
/// to make the game work, it is what having a body means.
pub const BULK_REACH: i32 = 1;

/// A namespaced process identity (PD1b slice 1).
///
/// The registry is keyed by these rather than by enum variants, so a pack
/// can one day mint `("reef", "filter")` without colliding with a native.
/// Static strs because every definition today is native; admission of owned
/// strings arrives with packs (PD3), not before.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProcessId {
    pub namespace: &'static str,
    pub name: &'static str,
}

/// A definition's content address: a hash over its rule-bearing bytes.
///
/// This is what stops a friendly id silently changing meaning. Two worlds
/// that both know `mesocosm:fix` but disagree about which roles express it
/// hold different digests, and an allocation citing one cannot be resolved
/// against the other.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct DefinitionDigest(pub u64);

/// What an expressed process names: the exact admitted definition.
///
/// **This is the identity a phenotype stores**, and it is deliberately not a
/// [`Process`] variant. The enum is a native binding for engine fast paths;
/// allocation cites a definition, resolves it through a [`Registry`], and
/// refuses when the registry does not hold it. That is the PD1b migration:
/// the closed enum stopped being identity authority.
///
/// Only the digest travels, because [`ProcessId`]'s static strs cannot be
/// deserialized into an owned world. PD3 admits owned ids and this record
/// widens to carry both; the qualified id is recovered through
/// [`Registry::resolve`] until then.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProcessRef {
    pub definition: DefinitionDigest,
}

/// Whether a shape that admits a definition also **grows** it.
///
/// PD1b had only one answer, because all four natives were things a shape
/// simply does. PD2 needs the other: a definition a part will carry but never
/// develops on its own, so the only way to express it is a development that
/// takes the tissue off something else. That is the difference between a
/// consequence and a choice, and it is rule-bearing — a world whose plates
/// grew glands is a different world — so it is in the digest.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Seeding {
    /// Growing an admitting shape expresses it. The four originals.
    Geometry,
    /// Only a validated development places it. Nothing grows one.
    Acquired,
}

/// One process as a record: identity, the roles whose geometry expresses
/// it, and a digest over its rule-bearing bytes.
///
/// This is the PD1b migration's first half. The enum below remains the
/// *native binding* (fast, exhaustive matching for engine code), but the
/// definition of record is this struct: what expresses a process is data,
/// and changing one rule-bearing byte changes the digest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessDef {
    pub id: ProcessId,
    pub native: Process,
    /// The roles whose shape may express this process: the **site
    /// requirement**, and what [`ProcessDef::admits`] answers.
    pub expressed_by: &'static [Role],
    /// Whether growing one of those shapes also grows this process.
    pub seeding: Seeding,
}

impl ProcessDef {
    /// Digest over the rule-bearing bytes: identity, site requirement, and
    /// whether geometry grows it.
    pub fn digest(&self) -> DefinitionDigest {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.id.namespace.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(self.id.name.as_bytes());
        bytes.push(0);
        for role in self.expressed_by {
            bytes.push(*role as u8);
        }
        bytes.push(0);
        bytes.push(self.seeding as u8);
        DefinitionDigest(crate::snapshot::hash_bytes(&bytes))
    }

    /// Whether growing an admitting shape expresses this definition.
    pub fn seeded(&self) -> bool {
        self.seeding == Seeding::Geometry
    }

    /// What a phenotype stores when it expresses this definition.
    pub fn reference(&self) -> ProcessRef {
        ProcessRef {
            definition: self.digest(),
        }
    }

    /// Whether a part of this shape may express this process.
    ///
    /// The site requirement, and the reason a part cannot acquire a
    /// capability by editing a number: allocation can only put a process
    /// where the geometry already expresses it, so changing what a part does
    /// still means changing what a part *is*.
    pub fn admits(&self, role: Role) -> bool {
        self.expressed_by.contains(&role)
    }
}

/// The registry: every admitted process definition, ordered.
///
/// Deterministic by construction (a fixed native table today; PD3 admits
/// packs through validation, never around it). The ruleset digest is what a
/// snapshot will cite when admission becomes dynamic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Registry {
    defs: &'static [ProcessDef],
}

const NATIVE_DEFS: &[ProcessDef] = &[
    ProcessDef {
        id: ProcessId {
            namespace: "mesocosm",
            name: "contract",
        },
        native: Process::Contract,
        expressed_by: &[Role::Limb],
        seeding: Seeding::Geometry,
    },
    ProcessDef {
        id: ProcessId {
            namespace: "mesocosm",
            name: "intake",
        },
        native: Process::Intake,
        expressed_by: &[Role::Mass],
        seeding: Seeding::Geometry,
    },
    ProcessDef {
        id: ProcessId {
            namespace: "mesocosm",
            name: "sense",
        },
        native: Process::Sense,
        expressed_by: &[Role::Sensor],
        seeding: Seeding::Geometry,
    },
    ProcessDef {
        id: ProcessId {
            namespace: "mesocosm",
            // Provisional, with the variant: the naming round is Mark's.
            name: "fix",
        },
        native: Process::Fix,
        expressed_by: &[Role::Plate],
        seeding: Seeding::Geometry,
    },
    ProcessDef {
        id: ProcessId {
            namespace: "mesocosm",
            // Provisional, with the variant: the naming round is Mark's.
            name: "secrete",
        },
        native: Process::Secrete,
        // **The same shape that fixes.** Area against the world is what a
        // toxin surface needs too, which is why a nettle's sting is on its
        // leaf; and it puts the tradeoff where PD1a wanted it, inside one
        // organ. A plate on a consumer is armour rather than a frond, so the
        // same rule lets an animal arm its shell without becoming a plant.
        expressed_by: &[Role::Plate],
        // **Nothing grows a gland.** This is the whole of PD2's first
        // done-condition: expressing it is an act with a record, so a body
        // that has one was given one.
        seeding: Seeding::Acquired,
    },
];

impl Registry {
    pub fn native() -> Self {
        Self { defs: NATIVE_DEFS }
    }

    pub fn all(&self) -> impl Iterator<Item = &ProcessDef> {
        self.defs.iter()
    }

    pub fn get(&self, id: ProcessId) -> Option<&ProcessDef> {
        self.defs.iter().find(|def| def.id == id)
    }

    /// The definition a stored reference names, or `None` when this world's
    /// ruleset does not hold it.
    ///
    /// `None` is a real answer and must never be substituted for a similar
    /// local definition (plan §6, missing packs). Allocation refuses rather
    /// than guessing.
    pub fn resolve(&self, reference: ProcessRef) -> Option<&ProcessDef> {
        self.defs
            .iter()
            .find(|def| def.digest() == reference.definition)
    }

    /// The definition a native binding resolves to. Total for the natives
    /// by construction; the bijection is receipted below.
    pub fn of_native(&self, process: Process) -> &ProcessDef {
        self.defs
            .iter()
            .find(|def| def.native == process)
            .expect("every native process is registered")
    }

    /// **The seeding rule**: the definitions growing this shape expresses.
    ///
    /// Not the same question as [`ProcessDef::admits`], which is the site
    /// requirement a proposal must satisfy. A plate is admitted for two
    /// definitions and grows one.
    pub fn seeds(&self, role: Role) -> impl Iterator<Item = &ProcessDef> {
        self.defs
            .iter()
            .filter(move |def| def.seeded() && def.expressed_by.contains(&role))
    }

    /// Digest over the whole admitted ruleset, order-sensitive.
    pub fn digest(&self) -> u64 {
        let mut bytes = Vec::new();
        for def in self.defs {
            bytes.extend_from_slice(&def.digest().0.to_le_bytes());
        }
        crate::snapshot::hash_bytes(&bytes)
    }
}

impl Process {
    /// This native binding's qualified identity.
    pub fn id(self) -> ProcessId {
        Registry::native().of_native(self).id
    }
}

impl BodyDocument {
    /// The processes a part contributes, from its shape.
    pub fn processes(&self, id: PartId) -> &'static [Process] {
        match self.part(id) {
            Some(part) if !part.severed => classify(part.half_extent).processes(),
            _ => &[],
        }
    }

    /// Whether any living part performs `process`.
    pub fn performs(&self, process: Process) -> bool {
        self.living()
            .any(|part| classify(part.half_extent).processes().contains(&process))
    }

    /// How far this body can touch.
    ///
    /// **A satisfied path, not a measurement.** Reaching needs an actuator, so
    /// the answer is the distance to the furthest living part that
    /// [`Process::Contract`]s, plus that part's own extent. A body with no
    /// actuator reaches only as far as its own bulk.
    ///
    /// Nothing stores this. Grow a limb and it grows; sever the limb and it
    /// shrinks; and neither required editing a number.
    pub fn reach(&self) -> i32 {
        let Some(origin) = self.world_pivot(self.root) else {
            return 0;
        };

        let actuated = self
            .living()
            .filter(|part| {
                classify(part.half_extent)
                    .processes()
                    .contains(&Process::Contract)
            })
            .filter_map(|part| {
                let at = self.world_pivot(part.id)?;
                let span = (0..3)
                    .map(|axis| (at[axis] - origin[axis]).abs() + part.half_extent[axis].abs())
                    .max()
                    .unwrap_or(0);
                Some(span)
            })
            .max();

        // Bulk is what you can touch without reaching for it, and it is the
        // extent of the root rather than a constant.
        let bulk = self
            .part(self.root)
            .map(|root| BULK_REACH + root.half_extent.iter().map(|d| d.abs()).max().unwrap_or(0))
            .unwrap_or(0);

        actuated.map(|span| span.max(bulk)).unwrap_or(bulk)
    }

    /// Whether this body could reach `distance`, and why not when it could not.
    pub fn can_reach(&self, distance: i32) -> Result<(), Unmet> {
        let reach = self.reach();
        if distance <= reach {
            return Ok(());
        }
        // Distinguish "no arm" from "arm too short". A body that has never
        // grown an actuator has a different problem from one whose actuator is
        // simply not long enough, and a player deserves to be told which.
        if !self.performs(Process::Contract) {
            return Err(Unmet::NoProcess {
                capability: Capability::Reach,
                needs: Process::Contract,
            });
        }
        Err(Unmet::TooFar { reach, distance })
    }
}

#[cfg(test)]
mod tests;
