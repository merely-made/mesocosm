// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Snapshots and state hashing.
//!
//! A snapshot is the whole world, captured at once, by one call. That is the
//! property the wing's determinism constraint is for: hand-written per-field
//! capture has a failure mode, a field added later and not added to the
//! snapshot, that whole-state capture cannot have.
//!
//! The state hash is over the snapshot bytes, so two worlds hash equal exactly
//! when they would serialize equal.

use serde::{Serialize, de::DeserializeOwned};

use crate::world::World;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapshotError {
    Encode,
    Decode,
}

/// Captures the whole world as bytes.
pub fn snapshot(world: &World) -> Result<Vec<u8>, SnapshotError> {
    encode(world)
}

/// Restores a world captured by [`snapshot`].
pub fn restore(bytes: &[u8]) -> Result<World, SnapshotError> {
    decode(bytes)
}

/// Round-trips any core value. Used by the body-document tests and by hosts
/// that carry parts of the world without carrying all of it.
pub fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, SnapshotError> {
    postcard::to_allocvec(value).map_err(|_| SnapshotError::Encode)
}

pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, SnapshotError> {
    postcard::from_bytes(bytes).map_err(|_| SnapshotError::Decode)
}

/// FNV-1a over the snapshot bytes. Chosen for being a few integer operations
/// with no platform-dependent behaviour; this is an equality witness for
/// replay, not a cryptographic digest.
pub fn state_hash(world: &World) -> u64 {
    let bytes = snapshot(world).expect("a world is always encodable");
    hash_bytes(&bytes)
}

pub fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::Yaw;
    use crate::world::{Intent, Placement, Route, World};
    use crate::organism::OrganismId;

    #[test]
    fn snapshot_round_trips() {
        let world = World::new(31, 10);
        let bytes = snapshot(&world).unwrap();
        let restored = restore(&bytes).unwrap();
        assert_eq!(world, restored);
    }

    #[test]
    fn identical_worlds_hash_equal() {
        assert_eq!(state_hash(&World::new(5, 8)), state_hash(&World::new(5, 8)));
    }

    #[test]
    fn divergent_worlds_hash_differently() {
        let mut a = World::new(5, 8);
        let b = World::new(5, 8);
        a.apply(Intent::Move { delta: [1, 0, 0] });
        assert_ne!(state_hash(&a), state_hash(&b));
    }

    #[test]
    fn restoring_mid_run_continues_identically() {
        let trace = [
            Intent::Move { delta: [1, 0, 1] },
            Intent::Idle,
            Intent::Deposit { mass_mg: 50 },
            Intent::Move { delta: [-2, 0, 0] },
        ];

        let mut straight = World::new(77, 12);
        straight.apply_all(&trace);

        let mut forked = World::new(77, 12);
        forked.apply_all(&trace[..2]);
        let bytes = snapshot(&forked).unwrap();
        let mut resumed = restore(&bytes).unwrap();
        resumed.apply_all(&trace[2..]);

        assert_eq!(state_hash(&straight), state_hash(&resumed));
    }

    #[test]
    fn rejected_intents_are_part_of_the_recorded_state() {
        let mut a = World::new(13, 6);
        let mut b = World::new(13, 6);
        a.apply(Intent::Metabolize { organism: OrganismId(9999), route: Route::Incorporate { placement: Placement::Explicit { parent: a.body.root, offset: [0, 0, 0], yaw: Yaw::Zero } } });
        b.apply(Intent::Idle);
        // Both were rejected-or-idle and advanced one tick, but the worlds are
        // still equal because neither changed anything else.
        assert_eq!(state_hash(&a), state_hash(&b));
    }
}
