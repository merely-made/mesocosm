// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What happened, and what it followed from.
//!
//! A [`Codicil`] of [`Event`]s, with causality. Until now `World::apply`
//! returned an outcome and dropped it, so the world had a present and no past,
//! and every proposal that reads history had nothing to read.
//!
//! # It sits beside the world, not inside it
//!
//! History is **derivable**: a seed plus ordered intents reproduces it exactly.
//! So it must not live in the snapshot, or whole-state capture would grow
//! without bound and stop being the cheap memcpy the wing's rollback thinking
//! rests on. It is materialised rather than recomputed only because queries
//! want it now, which makes it a durable cache and exactly what a muniment slot
//! is for.
//!
//! The world therefore buffers at most one tick of events and a caller drains
//! them. That buffer is bounded by definition.
//!
//! # Causality without a bookkeeping burden
//!
//! Every event cites **the last event about each subject it touches**. One
//! `Seq` per organism is the whole apparatus, and it produces a genuine graph
//! rather than a chain: an event involving two creatures cites both of their
//! histories, so eating *joins* two causal lines that were independent until
//! they met.
//!
//! That is the structure significance is made of. `codicil`'s
//! [`effects`](Codicil::effects) then answers what followed from an event,
//! which is the retroactive definition of significance: a thing mattered
//! because of what later depended on it.
//!
//! # What it deliberately does not do
//!
//! It does not decide what was *significant*. Abnormality is a lookup against
//! [`WorldRecord`](crate::record::WorldRecord); this is the traversal half.
//! Two questions, two structures.

use std::collections::BTreeMap;

use codicil::{Codicil, Seq};
use serde::{Deserialize, Serialize};

use crate::body::{PartId, SpeciesId};
use crate::organism::OrganismId;

/// Why a feeding event happened. Keeping this on the event makes predation
/// and scavenging distinguishable without reconstructing world state from
/// later deaths.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MealKind {
    Grazing,
    Predation,
    Scavenging,
}

/// Something that happened to somebody.
///
/// Small on purpose. Each variant names the organisms it touches, because that
/// is what the causal links are built from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    /// A creature entered the world.
    Born { organism: OrganismId, species: SpeciesId, parent: Option<OrganismId> },
    /// A creature came of age.
    Matured { organism: OrganismId },
    /// A creature took substance from another. The join that makes this a
    /// graph rather than a set of chains.
    Fed { eater: OrganismId, from: OrganismId, mass_mg: u64, kind: MealKind },
    /// A creature changed position under its own drive.
    Moved { organism: OrganismId, from: [i32; 3], to: [i32; 3] },
    /// A creature took a meal and kept it as body.
    Grew { organism: OrganismId, part: PartId },
    /// A creature took a meal and spent it.
    Burned { organism: OrganismId, energy_mg: u64 },
    /// A creature lost a part, and everything below it.
    Severed { organism: OrganismId, part: PartId },
    /// A creature died.
    Died { organism: OrganismId, species: SpeciesId },
    /// A carcass returned to the world.
    Returned { organism: OrganismId },
    /// The player took a body.
    Inhabited { organism: OrganismId },
    /// A line split off another, and was named.
    Speciated { species: SpeciesId, from: SpeciesId, founder: OrganismId },
    /// A creature removed ground: a burrow, a bore, a den. World-shaping
    /// enters the same history as eating, because a carved refuge is as
    /// biographical as a meal.
    Carved { organism: OrganismId, at: [i32; 3], removed: u32 },
    /// A line learned to grow something, by eating something that had it.
    ///
    /// The discovery half of kleptoplasty: a meal that teaches is a different
    /// kind of event from a meal that feeds, and only the first is recorded.
    Learned { organism: OrganismId, species: SpeciesId, appendage: crate::axis::Appendage },
}

impl Event {
    /// The organisms this event is about, in a fixed order.
    ///
    /// What the causal links are built from, so it must be exhaustive: an
    /// organism left out here has its history quietly forked.
    pub fn subjects(&self) -> Vec<OrganismId> {
        match *self {
            Event::Born { organism, parent, .. } => {
                parent.map(|p| vec![p, organism]).unwrap_or_else(|| vec![organism])
            }
            Event::Fed { eater, from, .. } => vec![eater, from],
            Event::Matured { organism }
            | Event::Grew { organism, .. }
            | Event::Burned { organism, .. }
            | Event::Severed { organism, .. }
            | Event::Moved { organism, .. }
            | Event::Died { organism, .. }
            | Event::Returned { organism }
            | Event::Inhabited { organism }
            | Event::Carved { organism, .. } => vec![organism],
            // The founder's line continues through the split, which is what
            // makes a speciation visible in its own history rather than only
            // in the world's.
            Event::Speciated { founder, .. } => vec![founder],
            Event::Learned { organism, .. } => vec![organism],
        }
    }
}

/// The world's past.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct History {
    log: Codicil<Event>,
    /// The most recent event about each organism, which is how a new one finds
    /// its causes. Ordered, so recording is deterministic.
    latest: BTreeMap<OrganismId, Seq>,
}

impl History {
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends an event, citing the last event about each subject it touches.
    ///
    /// A creature's first event has no cause, which is honest: nothing in this
    /// log led to it. Every later one continues that creature's line, and an
    /// event touching two creatures joins theirs.
    pub fn record(&mut self, event: Event) -> Seq {
        let subjects = event.subjects();
        let causes: Vec<Seq> =
            subjects.iter().filter_map(|who| self.latest.get(who).copied()).collect();

        let seq = self
            .log
            .append_caused_by(causes, event)
            .expect("causes are drawn from this log, so they always exist");

        for who in subjects {
            self.latest.insert(who, seq);
        }
        seq
    }

    /// Records a tick's worth of events, in order.
    pub fn record_all(&mut self, events: impl IntoIterator<Item = Event>) {
        for event in events {
            self.record(event);
        }
    }

    /// The log itself, for causal queries.
    pub fn log(&self) -> &Codicil<Event> {
        &self.log
    }

    pub fn len(&self) -> usize {
        self.log.len()
    }

    pub fn is_empty(&self) -> bool {
        self.log.is_empty()
    }

    /// The event, if any.
    pub fn get(&self, seq: Seq) -> Option<&Event> {
        self.log.get(seq)
    }

    /// The most recent event about an organism.
    pub fn latest(&self, organism: OrganismId) -> Option<Seq> {
        self.latest.get(&organism).copied()
    }

    /// Everything that followed from an event.
    ///
    /// **The significance query.** An event mattered because of what later
    /// depended on it, and a flat log cannot answer that at all.
    pub fn consequences(&self, seq: Seq) -> Vec<Seq> {
        self.log.effects(seq)
    }

    /// Everything that led to an event, nearest first.
    pub fn antecedents(&self, seq: Seq) -> Vec<Seq> {
        self.log.causes(seq)
    }

    /// Whether neither event led to the other.
    pub fn concurrent(&self, a: Seq, b: Seq) -> bool {
        self.log.concurrent(a, b)
    }

    /// An organism's own line, oldest first.
    pub fn line_of(&self, organism: OrganismId) -> Vec<Seq> {
        let Some(latest) = self.latest(organism) else {
            return Vec::new();
        };
        let mut line: Vec<Seq> = self
            .antecedents(latest)
            .into_iter()
            .filter(|seq| self.touches(*seq, organism))
            .collect();
        line.push(latest);
        line.sort();
        line
    }

    fn touches(&self, seq: Seq, organism: OrganismId) -> bool {
        self.get(seq).is_some_and(|event| event.subjects().contains(&organism))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: OrganismId = OrganismId(1);
    const B: OrganismId = OrganismId(2);
    const SPECIES: SpeciesId = SpeciesId(7);

    #[test]
    fn a_first_event_has_no_cause() {
        let mut history = History::new();
        let born = history.record(Event::Born { organism: A, species: SPECIES, parent: None });

        assert!(history.antecedents(born).is_empty(), "nothing in this log led to it");
        assert_eq!(history.latest(A), Some(born));
    }

    #[test]
    fn a_creatures_events_form_its_line() {
        let mut history = History::new();
        let born = history.record(Event::Born { organism: A, species: SPECIES, parent: None });
        let matured = history.record(Event::Matured { organism: A });
        let died = history.record(Event::Died { organism: A, species: SPECIES });

        assert_eq!(history.antecedents(matured), vec![born]);
        assert_eq!(history.antecedents(died), vec![matured, born], "nearest first");
        assert_eq!(history.line_of(A), vec![born, matured, died]);
    }

    #[test]
    fn independent_creatures_have_independent_lines() {
        // The property a flat log destroys: two creatures that never met are
        // concurrent, not ordered.
        let mut history = History::new();
        let a = history.record(Event::Born { organism: A, species: SPECIES, parent: None });
        let b = history.record(Event::Born { organism: B, species: SPECIES, parent: None });

        assert!(history.concurrent(a, b), "neither led to the other");
        assert!(history.consequences(a).is_empty());
    }

    #[test]
    fn eating_joins_two_lines() {
        // The join that makes this a graph. Until they met, these creatures
        // had nothing to do with each other; afterwards, one's past is part of
        // the other's.
        let mut history = History::new();
        let a_born = history.record(Event::Born { organism: A, species: SPECIES, parent: None });
        let b_born = history.record(Event::Born { organism: B, species: SPECIES, parent: None });
        assert!(history.concurrent(a_born, b_born));

        let meal = history.record(Event::Fed { eater: A, from: B, mass_mg: 40, kind: MealKind::Predation });

        assert_eq!(history.antecedents(meal), vec![a_born, b_born], "both lines are cited");
        assert_eq!(history.consequences(a_born), vec![meal]);
        assert_eq!(history.consequences(b_born), vec![meal], "the eaten one led here too");
    }

    #[test]
    fn a_birth_descends_from_its_parent() {
        let mut history = History::new();
        let parent = history.record(Event::Born { organism: A, species: SPECIES, parent: None });
        let child =
            history.record(Event::Born { organism: B, species: SPECIES, parent: Some(A) });

        assert_eq!(history.antecedents(child), vec![parent]);
        assert_eq!(history.consequences(parent), vec![child]);
        assert_eq!(history.latest(A), Some(child), "the parent's line continues through it");
    }

    #[test]
    fn consequences_reach_through_a_chain() {
        // The retroactive definition of significance: what a thing led to,
        // however far downstream.
        let mut history = History::new();
        history.record(Event::Born { organism: A, species: SPECIES, parent: None });
        let b_born = history.record(Event::Born { organism: B, species: SPECIES, parent: None });
        let meal = history.record(Event::Fed { eater: A, from: B, mass_mg: 40, kind: MealKind::Predation });
        let grew = history.record(Event::Grew { organism: A, part: PartId(1) });

        assert_eq!(
            history.consequences(b_born),
            vec![meal, grew],
            "being eaten led, eventually, to somebody else's new limb"
        );
    }

    #[test]
    fn severing_continues_the_line_it_happened_to() {
        let mut history = History::new();
        history.record(Event::Born { organism: A, species: SPECIES, parent: None });
        let grew = history.record(Event::Grew { organism: A, part: PartId(1) });
        let lost = history.record(Event::Severed { organism: A, part: PartId(1) });

        assert!(history.antecedents(lost).contains(&grew), "you can only lose what you grew");
    }

    #[test]
    fn a_history_round_trips() {
        let mut history = History::new();
        history.record(Event::Born { organism: A, species: SPECIES, parent: None });
        history.record(Event::Fed { eater: A, from: B, mass_mg: 5, kind: MealKind::Predation });

        let bytes = crate::snapshot::encode(&history).unwrap();
        assert_eq!(crate::snapshot::decode::<History>(&bytes).unwrap(), history);
    }

    #[test]
    fn every_variant_names_its_subjects() {
        // The links are built from `subjects`, so a variant that forgets one
        // silently forks that creature's history. Cheap to assert, expensive
        // to discover later.
        let all = [
            Event::Born { organism: A, species: SPECIES, parent: Some(B) },
            Event::Matured { organism: A },
            Event::Fed { eater: A, from: B, mass_mg: 1, kind: MealKind::Predation },
            Event::Grew { organism: A, part: PartId(0) },
            Event::Burned { organism: A, energy_mg: 1 },
            Event::Severed { organism: A, part: PartId(0) },
            Event::Died { organism: A, species: SPECIES },
            Event::Returned { organism: A },
            Event::Inhabited { organism: A },
        ];
        for event in all {
            assert!(!event.subjects().is_empty(), "{event:?} names nobody");
        }
    }
}
