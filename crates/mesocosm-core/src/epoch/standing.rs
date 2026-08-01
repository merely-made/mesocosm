// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a lineage is actually up against: the world, plus everybody else.
//!
//! This is the small type that makes the whole initiative rule mean something.
//! A world's pressures are fixed, but **the pressure other lineages exert is
//! not** — it is a function of what they currently are. So when the biggest
//! consumer in the roster spends its bank on better jaws and commits, every
//! lineage that has not yet taken its turn is now living somewhere more
//! dangerous, and scores its own candidates accordingly.
//!
//! That is the payoff the ordering was ruled for: descending complexity means
//! the expensive, slow-generating forms commit first, and the simpler ones get
//! to **answer what just happened** instead of guessing. Compressed generation
//! tempo, in one legible round.
//!
//! Without this type the initiative order would be decoration — every lineage
//! would score against the same frozen world and the sequence would not matter.

use super::lineage::{Lineage, Role, Trait};
use super::worlds::{Pressure, WorldProfile};

/// How much one point of a hunter's jaws raises predation on everyone else.
const JAWS_WEIGHT: i32 = 1;

/// How much each living competitor for the same role raises crowding.
const COMPETITOR_WEIGHT: i32 = 2;

/// The world as one lineage experiences it right now.
pub struct Standing<'a> {
    world: &'a WorldProfile,
    roster: &'a [Lineage],
}

impl<'a> Standing<'a> {
    pub fn new(world: &'a WorldProfile, roster: &'a [Lineage]) -> Self {
        Self { world, roster }
    }

    pub fn world(&self) -> &WorldProfile {
        self.world
    }

    /// Every pressure bearing on `who`, world and neighbours together.
    ///
    /// Yields all pressures, including ones at zero, so a caller does not have
    /// to know which axes a world touches to score against it.
    pub fn pressures_on(&self, who: &Lineage) -> impl Iterator<Item = (Pressure, i32)> + '_ {
        let id = who.id;
        let role = who.role;
        Pressure::ALL
            .into_iter()
            .map(move |pressure| (pressure, self.strength_on(id, role, pressure)))
    }

    /// How hard one pressure bears on a lineage.
    pub fn on(&self, who: &Lineage, pressure: Pressure) -> i32 {
        self.strength_on(who.id, who.role, pressure)
    }

    fn strength_on(&self, id: u32, role: Role, pressure: Pressure) -> i32 {
        let world = self.world.strength(pressure);
        world + self.ecological(id, role, pressure)
    }

    /// The part of a pressure that other lineages are responsible for.
    ///
    /// The whole reason a round has an order.
    fn ecological(&self, id: u32, role: Role, pressure: Pressure) -> i32 {
        let living = || self.roster.iter().filter(|other| !other.extinct && other.id != id);

        match pressure {
            // Somebody has to be doing the eating. Producers are not a threat
            // however well armed, which keeps the trophic roles meaningful
            // rather than decorative.
            Pressure::Predation => living()
                .filter(|other| other.role != Role::Producer)
                .map(|other| other.level(Trait::Jaws) * JAWS_WEIGHT)
                .sum(),
            // You are crowded by the things that want what you want.
            Pressure::Crowding => {
                living().filter(|other| other.role == role).count() as i32 * COMPETITOR_WEIGHT
            }
            _ => 0,
        }
    }

    /// The most complex lineage still alive. The world's **complexity
    /// frontier**: the ceiling a player may not mint a new peer at.
    pub fn frontier(&self) -> i32 {
        self.roster
            .iter()
            .filter(|lineage| !lineage.extinct)
            .map(Lineage::complexity)
            .max()
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::epoch::worlds::TIDAL_SHELF;

    fn hunter(id: u32, jaws: i32) -> Lineage {
        let mut it = Lineage::new(id, "hunter", Role::Consumer, [0; 7]);
        it.set_level(Trait::Jaws, jaws);
        it
    }

    fn grazer(id: u32) -> Lineage {
        Lineage::new(id, "grazer", Role::Consumer, [1, 0, 0, 0, 0, 1, 0])
    }

    #[test]
    fn a_world_alone_is_the_floor() {
        let roster = [];
        let standing = Standing::new(&TIDAL_SHELF, &roster);
        let lone = grazer(0);
        assert_eq!(standing.on(&lone, Pressure::Cold), TIDAL_SHELF.strength(Pressure::Cold));
        assert_eq!(standing.on(&lone, Pressure::Predation), TIDAL_SHELF.strength(Pressure::Predation));
    }

    #[test]
    fn a_hunter_raises_predation_on_everyone_else() {
        // The mechanic the round order exists to expose.
        let roster = [grazer(0), hunter(1, 5)];
        let standing = Standing::new(&TIDAL_SHELF, &roster);

        let alone = Standing::new(&TIDAL_SHELF, &roster[..1]);
        assert!(
            standing.on(&roster[0], Pressure::Predation)
                > alone.on(&roster[0], Pressure::Predation),
            "the grazer's world got more dangerous because of its neighbour"
        );
    }

    #[test]
    fn nothing_preys_on_itself() {
        let roster = [hunter(0, 9)];
        let standing = Standing::new(&TIDAL_SHELF, &roster);
        assert_eq!(
            standing.on(&roster[0], Pressure::Predation),
            TIDAL_SHELF.strength(Pressure::Predation),
            "its own jaws are not a threat to it"
        );
    }

    #[test]
    fn producers_are_not_predators_however_armed() {
        // Trophic role has to mean something. A well-defended plant is not a
        // reason for anybody else to grow a shell.
        let mut plant = Lineage::new(1, "plant", Role::Producer, [0; 7]);
        plant.set_level(Trait::Jaws, 9);
        let roster = [grazer(0), plant];
        let standing = Standing::new(&TIDAL_SHELF, &roster);

        assert_eq!(
            standing.on(&roster[0], Pressure::Predation),
            TIDAL_SHELF.strength(Pressure::Predation)
        );
    }

    #[test]
    fn competitors_for_the_same_role_crowd_each_other() {
        let same = [grazer(0), grazer(1), grazer(2)];
        let mixed = [
            grazer(0),
            Lineage::new(1, "moss", Role::Producer, [0; 7]),
            Lineage::new(2, "rot", Role::Decomposer, [0; 7]),
        ];

        let crowded = Standing::new(&TIDAL_SHELF, &same).on(&same[0], Pressure::Crowding);
        let roomy = Standing::new(&TIDAL_SHELF, &mixed).on(&mixed[0], Pressure::Crowding);
        assert!(crowded > roomy, "three grazers crowd; one of each does not");
    }

    #[test]
    fn the_extinct_stop_pressing() {
        let mut dead = hunter(1, 9);
        dead.extinct = true;
        let roster = [grazer(0), dead];
        let standing = Standing::new(&TIDAL_SHELF, &roster);

        assert_eq!(
            standing.on(&roster[0], Pressure::Predation),
            TIDAL_SHELF.strength(Pressure::Predation),
            "a dead hunter hunts nobody"
        );
    }

    #[test]
    fn the_frontier_is_the_most_complex_living_lineage() {
        let mut fancy = Lineage::new(2, "fancy", Role::Consumer, [4, 4, 4, 4, 4, 4, 4]);
        let roster = [grazer(0), hunter(1, 3), fancy.clone()];
        assert_eq!(Standing::new(&TIDAL_SHELF, &roster).frontier(), fancy.complexity());

        fancy.extinct = true;
        let after = [grazer(0), hunter(1, 3), fancy];
        assert_eq!(
            Standing::new(&TIDAL_SHELF, &after).frontier(),
            hunter(1, 3).complexity().max(grazer(0).complexity()),
            "losing the frontier lineage lowers the ceiling"
        );
    }
}
