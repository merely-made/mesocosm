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
//! scale. Three processes exist here because one capability needs them; the
//! fourth arrives when something asks.
//!
//! # Processes are read, not stored
//!
//! A part does not carry a list of what it does. Its processes are derived
//! from its geometry through [`classify`](crate::plan::classify), which is the
//! same rule that already decides where growth puts things. So a part cannot
//! be given an ability it has no shape for, and reshaping a body reshapes what
//! it can do without anybody editing a number.

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
}

impl Role {
    /// What a part of this shape contributes.
    ///
    /// A long thin part is an actuator, a bulky one admits material, a small
    /// one senses, and a flat one does neither while still being armour. That
    /// mapping is the whole vocabulary today.
    pub fn processes(self) -> &'static [Process] {
        // The registry is the definition of record (PD1b); this remains the
        // fast native view of it, and the parity receipt below keeps the two
        // from drifting.
        match self {
            Role::Limb => &[Process::Contract],
            Role::Mass => &[Process::Intake],
            Role::Sensor => &[Process::Sense],
            // A plate resists things. Resisting is not yet a process because
            // nothing reads it; it becomes one when damage does.
            Role::Plate => &[],
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
    NoProcess { capability: Capability, needs: Process },
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
    /// The roles whose shape expresses this process. Geometry seeding:
    /// today's whole allocation rule, carried as data.
    pub expressed_by: &'static [Role],
}

impl ProcessDef {
    /// Digest over the rule-bearing bytes: identity plus expression rule.
    pub fn digest(&self) -> u64 {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.id.namespace.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(self.id.name.as_bytes());
        bytes.push(0);
        for role in self.expressed_by {
            bytes.push(*role as u8);
        }
        crate::snapshot::hash_bytes(&bytes)
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
        id: ProcessId { namespace: "mesocosm", name: "contract" },
        native: Process::Contract,
        expressed_by: &[Role::Limb],
    },
    ProcessDef {
        id: ProcessId { namespace: "mesocosm", name: "intake" },
        native: Process::Intake,
        expressed_by: &[Role::Mass],
    },
    ProcessDef {
        id: ProcessId { namespace: "mesocosm", name: "sense" },
        native: Process::Sense,
        expressed_by: &[Role::Sensor],
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

    /// The definition a native binding resolves to. Total for the natives
    /// by construction; the bijection is receipted below.
    pub fn of_native(&self, process: Process) -> &ProcessDef {
        self.defs
            .iter()
            .find(|def| def.native == process)
            .expect("every native process is registered")
    }

    /// The processes a role's geometry expresses, per the registry.
    pub fn expressed_by(&self, role: Role) -> impl Iterator<Item = &ProcessDef> {
        self.defs.iter().filter(move |def| def.expressed_by.contains(&role))
    }

    /// Digest over the whole admitted ruleset, order-sensitive.
    pub fn digest(&self) -> u64 {
        let mut bytes = Vec::new();
        for def in self.defs {
            bytes.extend_from_slice(&def.digest().to_le_bytes());
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
        self.living().any(|part| classify(part.half_extent).processes().contains(&process))
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
            .filter(|part| classify(part.half_extent).processes().contains(&Process::Contract))
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
            .map(|root| {
                BULK_REACH + root.half_extent.iter().map(|d| d.abs()).max().unwrap_or(0)
            })
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
mod tests {
    use super::*;
    use crate::body::{Attachment, Provenance, SpeciesId, VolumeRef, Yaw};

    /// A bulk root, with an optional long limb reaching out along +x.
    fn critter(limb: bool) -> (BodyDocument, Option<PartId>) {
        let mut body =
            BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 1_000, [2, 2, 2]);
        let root = body.root;
        if !limb {
            return (body, None);
        }
        let arm = body
            .attach(
                VolumeRef::from_tag(2),
                200,
                // Long in one axis only, so `classify` reads it as a limb.
                [6, 1, 1],
                Attachment { parent: root, offset: [8, 0, 0], yaw: Yaw::Zero },
                Provenance::founding(),
            )
            .expect("attaches");
        (body, Some(arm))
    }

    #[test]
    fn a_parts_processes_come_from_its_shape() {
        let (body, arm) = critter(true);
        assert_eq!(body.processes(body.root), &[Process::Intake], "a bulk root admits");
        assert_eq!(body.processes(arm.unwrap()), &[Process::Contract], "a long part acts");
    }

    #[test]
    fn a_body_without_an_actuator_reaches_only_its_own_bulk() {
        let (body, _) = critter(false);
        assert!(!body.performs(Process::Contract));
        assert_eq!(body.reach(), BULK_REACH + 2, "its own half-extent, and no further");
    }

    #[test]
    fn growing_a_limb_extends_reach() {
        // The first embodied consequence. Two bodies, different reach, and no
        // capability number was written anywhere.
        let (bare, _) = critter(false);
        let (limbed, _) = critter(true);

        assert!(limbed.reach() > bare.reach(), "{} vs {}", limbed.reach(), bare.reach());
    }

    #[test]
    fn severing_the_limb_takes_the_reach_with_it() {
        // The other half: a capability that came from anatomy leaves with it.
        let (mut body, arm) = critter(true);
        let reached = body.reach();

        body.sever(arm.unwrap());

        assert!(body.reach() < reached, "the reach went with the arm");
        assert_eq!(body.reach(), BULK_REACH + 2, "back to bulk");
        assert!(!body.performs(Process::Contract), "and nothing acts any more");
    }

    #[test]
    fn a_longer_limb_reaches_further_than_a_short_one() {
        let mut short = BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 100, [1, 1, 1]);
        let root = short.root;
        let mut long = short.clone();

        short
            .attach(
                VolumeRef::from_tag(2),
                50,
                [3, 1, 1],
                Attachment { parent: root, offset: [4, 0, 0], yaw: Yaw::Zero },
                Provenance::founding(),
            )
            .unwrap();
        long.attach(
            VolumeRef::from_tag(2),
            50,
            [9, 1, 1],
            Attachment { parent: root, offset: [10, 0, 0], yaw: Yaw::Zero },
            Provenance::founding(),
        )
        .unwrap();

        assert!(long.reach() > short.reach(), "length is the reach");
    }

    #[test]
    fn a_failure_says_which_requirement_was_unmet() {
        // "No arm" and "arm too short" are different problems and a receipt
        // has to be able to say which.
        let (bare, _) = critter(false);
        assert_eq!(
            bare.can_reach(50),
            Err(Unmet::NoProcess { capability: Capability::Reach, needs: Process::Contract })
        );

        let (limbed, _) = critter(true);
        let reach = limbed.reach();
        assert_eq!(limbed.can_reach(50), Err(Unmet::TooFar { reach, distance: 50 }));
        assert_eq!(limbed.can_reach(reach), Ok(()), "and what it can do, it can do");
    }

    #[test]
    fn the_registry_and_the_native_view_agree() {
        // PD1b slice 1's load-bearing receipt: expression is defined by
        // registry data, and the enum fast-path may never drift from it.
        let registry = Registry::native();
        for role in [Role::Limb, Role::Mass, Role::Sensor, Role::Plate] {
            let via_registry: Vec<Process> =
                registry.expressed_by(role).map(|def| def.native).collect();
            assert_eq!(
                via_registry,
                role.processes().to_vec(),
                "{role:?} expresses differently in data and in code"
            );
        }
        // The binding is a bijection: every native resolves, and ids are
        // qualified and distinct.
        let mut ids = std::collections::BTreeSet::new();
        for process in [Process::Contract, Process::Intake, Process::Sense] {
            let def = registry.of_native(process);
            assert_eq!(def.native, process);
            assert!(ids.insert(def.id), "duplicate id {:?}", def.id);
            assert_eq!(def.id.namespace, "mesocosm");
        }
    }

    #[test]
    fn a_rule_bearing_byte_changes_the_digest() {
        let registry = Registry::native();
        let contract = registry.of_native(Process::Contract);
        // Same identity, different expression rule: a different definition.
        let tampered = ProcessDef {
            id: contract.id,
            native: contract.native,
            expressed_by: &[Role::Limb, Role::Plate],
        };
        assert_ne!(contract.digest(), tampered.digest());
        // And the ruleset digest is stable across constructions.
        assert_eq!(Registry::native().digest(), Registry::native().digest());
    }

    #[test]
    fn a_plate_is_not_an_actuator() {
        // Armour resists; it does not reach. Without this a body could grow
        // reach by growing anything at all, and shape would stop mattering.
        let mut body = BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 100, [1, 1, 1]);
        let root = body.root;
        body.attach(
            VolumeRef::from_tag(2),
            50,
            // Wide and flat: two long axes, one short.
            [4, 4, 1],
            Attachment { parent: root, offset: [6, 0, 0], yaw: Yaw::Zero },
            Provenance::founding(),
        )
        .unwrap();

        assert_eq!(classify([4, 4, 1]), Role::Plate);
        assert!(!body.performs(Process::Contract));
        assert_eq!(body.reach(), BULK_REACH + 1, "a plate bought no reach");
    }
}
