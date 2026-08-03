// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a host may read off a world.
//!
//! Split out of `world.rs` at the 600-line ceiling. Nothing here mutates
//! anything, which is the whole point of gathering it: a host projects a world
//! and never changes one, so the surface it is allowed to touch is worth being
//! able to see in one place.

use std::collections::{BTreeMap, BTreeSet};

use crate::body::{Aabb, BodyDocument, SpeciesId};
use crate::organism::{Organism, OrganismId};
use crate::places::{PlaceId, Places};
use crate::process::Unmet;
use crate::record::WorldRecord;

use super::{Ineligible, World};

impl World {
    /// Which organism the player is, if any.
    pub fn controlled_id(&self) -> Option<OrganismId> {
        self.controlled
    }

    /// The played organism.
    ///
    /// `None` when nobody is embodied, **and also when the subject is no
    /// longer alive**. A carcass is not a critter you can play: natural death
    /// leaves an organism in the roster as carrion until it is spent, and an
    /// earlier cut checked only that the id was still present, so a dead
    /// critter kept moving and eating while it decomposed.
    pub fn controlled(&self) -> Option<&Organism> {
        let id = self.controlled?;
        self.organisms.iter().find(|o| o.id == id && o.is_alive())
    }

    pub(super) fn controlled_mut(&mut self) -> Option<&mut Organism> {
        let id = self.controlled?;
        self.organisms.iter_mut().find(|o| o.id == id && o.is_alive())
    }

    /// Whether a given organism could be played.
    pub fn is_eligible(&self, organism: OrganismId) -> bool {
        self.eligibility(organism).is_ok()
    }

    /// How elaborate an organism's lineage is, as the frontier reads it.
    ///
    /// The **recipe's** intricacy, not the body's part count: repetition is
    /// cheap and vocabulary is expensive, so reaching a new lineage means
    /// reaching a richer recipe rather than a longer creature. Falls back to
    /// anatomy for an organism whose lineage the registry has lost, which a
    /// consistent world never has.
    pub fn intricacy(&self, organism: &Organism) -> i32 {
        self.lineages
            .get(organism.species)
            .map(|species| species.recipe.complexity() as i32)
            .unwrap_or_else(|| organism.complexity())
    }

    /// Whether an organism could be played, and why not if not.
    ///
    /// **This is where the complexity frontier finally binds.** The rule was
    /// ruled long ago -- an unlocked lineage must be *more* metabolically
    /// complex than the target, so stepping downward into a newly viable niche
    /// is the point and minting an unearned peer at the frontier is not -- and
    /// until now it lived in `epoch::can_switch_to`, which nothing outside its
    /// own tests called. Control could take anything alive, however elaborate.
    ///
    /// It binds here rather than there because that function reasons over
    /// `epoch::Lineage` and its provisional trait array, while control reasons
    /// over organisms. Since P1 every organism has a body, so complexity can
    /// come from anatomy and the two models no longer have to agree first.
    pub fn eligibility(&self, organism: OrganismId) -> Result<(), Ineligible> {
        let Some(target) = self.organisms.iter().find(|o| o.id == organism) else {
            return Err(Ineligible::NoSuchOrganism);
        };
        if !target.is_alive() {
            return Err(Ineligible::NotAlive);
        }
        // A line you have already lived is always yours to return to. The
        // frontier gates reaching *outward*, not going home.
        if self.unlocked.contains(&target.species) {
            return Ok(());
        }

        let frontier = self.frontier();
        let reach = self.intricacy(target);
        if reach < frontier {
            Ok(())
        } else {
            Err(Ineligible::AboveTheFrontier { frontier, target: reach })
        }
    }

    /// The most elaborate thing the player has ever held.
    ///
    /// The ceiling a new lineage must sit below, and it only goes up.
    pub fn frontier(&self) -> i32 {
        self.frontier
    }

    /// Which lineages exist, and how they are related.
    pub fn lineages(&self) -> &crate::species::Lineages {
        &self.lineages
    }

    /// The lineage registry, mutably. The adaptation phase edits recipes
    /// through here; ordinary play does not.
    pub fn lineages_mut(&mut self) -> &mut crate::species::Lineages {
        &mut self.lineages
    }

    /// How far apart two creatures' ancestries are, in forks.
    ///
    /// One of the axes graft compatibility was ruled to scale with. It was
    /// uncomputable until lineages could split: every pair was identical.
    pub fn kinship(&self, a: OrganismId, b: OrganismId) -> Option<u32> {
        let species = |id: OrganismId| self.organisms.iter().find(|o| o.id == id).map(|o| o.species);
        self.lineages.distance(species(a)?, species(b)?)
    }

    /// Lineages the player has inhabited.
    pub fn unlocked(&self) -> impl Iterator<Item = SpeciesId> + '_ {
        self.unlocked.iter().copied()
    }

    /// The enclosure, divided into regions.
    pub fn places(&self) -> &Places {
        &self.places
    }

    /// Which region the played critter is in.
    pub fn place(&self) -> Option<PlaceId> {
        self.places.at(self.position()?)
    }

    /// Everywhere each lineage has been.
    pub fn ranges(&self) -> &BTreeMap<SpeciesId, BTreeSet<PlaceId>> {
        &self.ranges
    }

    /// Everywhere one lineage has been.
    pub fn range(&self, species: SpeciesId) -> BTreeSet<PlaceId> {
        self.ranges.get(&species).cloned().unwrap_or_default()
    }

    /// What this world has seen anyone do.
    pub fn record(&self) -> &WorldRecord {
        &self.record
    }

    /// The played critter's anatomy.
    pub fn body(&self) -> Option<&BodyDocument> {
        self.controlled().map(|o| &o.body)
    }

    /// Where the played critter is.
    pub fn position(&self) -> Option<[i32; 3]> {
        self.controlled().map(|o| o.position)
    }

    /// The played critter's budget.
    pub fn energy_mg(&self) -> Option<u64> {
        self.controlled().map(|o| o.energy_mg)
    }

    /// Whether anyone is being played.
    pub fn is_embodied(&self) -> bool {
        self.controlled().is_some()
    }

    /// Who the world stopped being able to play on the most recent tick.
    ///
    /// The seam the design calls for: losing a body is where witnessing, world
    /// examination, adaptation, and choosing another eligible critter happen.
    pub fn control_lost(&self) -> Option<OrganismId> {
        self.control_lost
    }

    /// Whether the played critter could touch `target`, and why not if not.
    ///
    /// **Anatomy answers this now.** It used to be `REACH = 8` regardless of
    /// what a critter was shaped like, so a twelve-limbed creature and a cube
    /// reached exactly as far.
    pub(super) fn reach_to(&self, target: [i32; 3]) -> Result<(), Unmet> {
        let Some(me) = self.controlled() else {
            return Err(Unmet::TooFar { reach: 0, distance: i32::MAX });
        };
        let distance = (0..3)
            .map(|axis| (target[axis] - me.position[axis]).abs())
            .max()
            .unwrap_or(0);
        me.body.can_reach(distance)
    }

    /// Whether the played critter can touch a point. Anatomy decides.
    pub fn in_reach(&self, target: [i32; 3]) -> bool {
        self.within_reach(target)
    }

    /// How far the played critter can touch, from its anatomy.
    pub fn reach(&self) -> i32 {
        self.body().map(|b| b.reach()).unwrap_or(0)
    }

    fn within_reach(&self, target: [i32; 3]) -> bool {
        self.reach_to(target).is_ok()
    }

    /// The played body's collision extent in body space, when there is one.
    pub fn collision(&self) -> Option<Aabb> {
        self.body().map(|b| b.aabb())
    }

    pub fn total_mass_mg(&self) -> u64 {
        self.body().map(|b| b.total_mass_mg()).unwrap_or(0)
    }
}
