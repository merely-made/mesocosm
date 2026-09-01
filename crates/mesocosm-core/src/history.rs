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
//!
//! It also does not carry resource movement. Every milligram of upkeep and soil
//! draw in a permanent causal log would be the wrong record at the wrong
//! frequency; [`flow`](crate::flow) is the other half, and the two share one
//! commit point and one [`Envelope`](crate::flow::Envelope).

use std::collections::BTreeMap;

use codicil::{Codicil, Seq};
use serde::{Deserialize, Serialize};

use crate::body::{PartId, SpeciesId};
use crate::flow::RecordedEvent;
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
    Born {
        organism: OrganismId,
        species: SpeciesId,
        parent: Option<OrganismId>,
    },
    /// A creature came of age.
    Matured { organism: OrganismId },
    /// A creature took substance from another. The join that makes this a
    /// graph rather than a set of chains.
    Fed {
        eater: OrganismId,
        from: OrganismId,
        mass_mg: u64,
        kind: MealKind,
    },
    /// A creature changed position under its own drive.
    Moved {
        organism: OrganismId,
        from: [i32; 3],
        to: [i32; 3],
    },
    /// A creature took a meal and kept it as body.
    Grew { organism: OrganismId, part: PartId },
    /// A creature took a meal and spent it.
    Burned {
        organism: OrganismId,
        energy_mg: u64,
    },
    /// A creature lost a part, and everything below it.
    Severed { organism: OrganismId, part: PartId },
    /// A creature died.
    Died {
        organism: OrganismId,
        species: SpeciesId,
    },
    /// A carcass returned to the world.
    Returned { organism: OrganismId },
    /// The player took a body.
    Inhabited { organism: OrganismId },
    /// A line split off another, and was named.
    Speciated {
        species: SpeciesId,
        from: SpeciesId,
        founder: OrganismId,
    },
    /// A creature removed ground: a burrow, a bore, a den. World-shaping
    /// enters the same history as eating, because a carved refuge is as
    /// biographical as a meal.
    Carved {
        organism: OrganismId,
        at: [i32; 3],
        removed: u32,
    },
    /// A line came to a new developmental option, and this names the condition
    /// it came through. (PE2)
    ///
    /// **Replaces `Learned`.** That variant recorded one appendage word per
    /// meal, because `World::learn_from` taught every non-innate appendage in
    /// the donor's whole recipe; the plan named it a migration input. What is
    /// recorded now is the *condition*, whose digest resolves to the matched
    /// evidence, the route, the realized candidate and its parameters through
    /// [`World::discoveries`](crate::World::discoveries). A meal that teaches
    /// is still a different kind of event from a meal that feeds — it is just
    /// no longer only a meal that can teach.
    Discovered {
        organism: OrganismId,
        species: SpeciesId,
        condition: crate::discovery::ConditionId,
    },
    /// A creature rebuilt one of its organs, and paid for it. (PD2)
    ///
    /// The first event about a body changing what it *does* rather than what
    /// it is made of. Growing and severing are already here; this is the third
    /// way a body becomes different, and PD1a ruled it must never happen
    /// except through a discrete event with a cost and a cause — so it is as
    /// biographical as a meal, and recorded like one.
    Rearranged {
        organism: OrganismId,
        part: PartId,
        cost_mg: u64,
    },
}

impl Event {
    /// The organisms this event is about, in a fixed order.
    ///
    /// What the causal links are built from, so it must be exhaustive: an
    /// organism left out here has its history quietly forked.
    pub fn subjects(&self) -> Vec<OrganismId> {
        match *self {
            Event::Born {
                organism, parent, ..
            } => parent
                .map(|p| vec![p, organism])
                .unwrap_or_else(|| vec![organism]),
            Event::Fed { eater, from, .. } => vec![eater, from],
            Event::Matured { organism }
            | Event::Grew { organism, .. }
            | Event::Burned { organism, .. }
            | Event::Severed { organism, .. }
            | Event::Moved { organism, .. }
            | Event::Died { organism, .. }
            | Event::Returned { organism }
            | Event::Inhabited { organism }
            | Event::Rearranged { organism, .. }
            | Event::Carved { organism, .. } => vec![organism],
            // The founder's line continues through the split, which is what
            // makes a speciation visible in its own history rather than only
            // in the world's.
            Event::Speciated { founder, .. } => vec![founder],
            Event::Discovered { organism, .. } => vec![organism],
        }
    }
}

/// The world's past.
///
/// Entries are [`RecordedEvent`]s: the event plus when and where it happened.
/// The envelope arrived with PE0's flow record and is shared with it, because a
/// past with no tick on it cannot answer *how many died in the last two hundred
/// ticks*, which is what the first ecology reading is made of.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct History {
    log: Codicil<RecordedEvent>,
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
    pub fn record(&mut self, event: RecordedEvent) -> Seq {
        let subjects = event.record.subjects();
        let causes: Vec<Seq> = subjects
            .iter()
            .filter_map(|who| self.latest.get(who).copied())
            .collect();

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
    pub fn record_all(&mut self, events: impl IntoIterator<Item = RecordedEvent>) {
        for event in events {
            self.record(event);
        }
    }

    /// The log itself, for causal queries.
    pub fn log(&self) -> &Codicil<RecordedEvent> {
        &self.log
    }

    pub fn len(&self) -> usize {
        self.log.len()
    }

    pub fn is_empty(&self) -> bool {
        self.log.is_empty()
    }

    /// The record, if any: the event with its tick and place.
    pub fn get(&self, seq: Seq) -> Option<&RecordedEvent> {
        self.log.get(seq)
    }

    /// Just the event, for a caller that only wants what happened.
    pub fn event(&self, seq: Seq) -> Option<&Event> {
        self.get(seq).map(|recorded| &recorded.record)
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

    /// Everyone descended from an organism, nearest generation first.
    ///
    /// **Descent is already written down.** `Event::Born` has carried its
    /// parent since the breeding transaction existed, so PE1 needs no second
    /// link on the body and no lineal table beside the world: the question is a
    /// walk over the record that answers it. Transitive, because a line
    /// continues through a grandchild when the child is gone, and the visited
    /// set is what keeps a walk over a log finite.
    ///
    /// Ids within a generation come out ascending, so "the eldest" is a
    /// deterministic thing to ask for rather than whatever iteration found
    /// first.
    pub fn descendants(&self, of: OrganismId) -> Vec<OrganismId> {
        let mut children: BTreeMap<OrganismId, Vec<OrganismId>> = BTreeMap::new();
        for recorded in self.log.entries() {
            if let Event::Born {
                organism,
                parent: Some(parent),
                ..
            } = recorded.record
            {
                children.entry(parent).or_default().push(organism);
            }
        }

        let mut found = Vec::new();
        let mut seen = std::collections::BTreeSet::from([of]);
        let mut generation = children.get(&of).cloned().unwrap_or_default();
        while !generation.is_empty() {
            generation.sort_unstable();
            let mut next = Vec::new();
            for child in generation {
                if !seen.insert(child) {
                    continue;
                }
                found.push(child);
                next.extend(children.get(&child).into_iter().flatten().copied());
            }
            generation = next;
        }
        found
    }

    fn touches(&self, seq: Seq, organism: OrganismId) -> bool {
        self.event(seq)
            .is_some_and(|event| event.subjects().contains(&organism))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: OrganismId = OrganismId(1);
    const B: OrganismId = OrganismId(2);
    const SPECIES: SpeciesId = SpeciesId(7);

    /// An event with its envelope. The causal claims below are about what is
    /// inside it; the tick is what a windowed reading later counts by.
    fn at(tick: u64, event: Event) -> RecordedEvent {
        RecordedEvent::new(tick, None, event)
    }

    fn born(organism: OrganismId, parent: Option<OrganismId>) -> Event {
        Event::Born {
            organism,
            species: SPECIES,
            parent,
        }
    }

    fn meal() -> Event {
        Event::Fed {
            eater: A,
            from: B,
            mass_mg: 40,
            kind: MealKind::Predation,
        }
    }

    #[test]
    fn a_first_event_has_no_cause() {
        let mut history = History::new();
        let seq = history.record(at(1, born(A, None)));

        assert!(
            history.antecedents(seq).is_empty(),
            "nothing in this log led to it"
        );
        assert_eq!(history.latest(A), Some(seq));
    }

    #[test]
    fn a_record_keeps_the_tick_it_happened_on() {
        // The envelope's whole job: a past with no tick cannot answer how many
        // died in the last two hundred ticks.
        let mut history = History::new();
        let seq = history.record(at(931, born(A, None)));
        assert_eq!(history.get(seq).map(|record| record.tick), Some(931));
        assert_eq!(history.event(seq), Some(&born(A, None)));
    }

    #[test]
    fn a_creatures_events_form_its_line() {
        let mut history = History::new();
        let opening = history.record(at(1, born(A, None)));
        let matured = history.record(at(2, Event::Matured { organism: A }));
        let died = history.record(at(
            3,
            Event::Died {
                organism: A,
                species: SPECIES,
            },
        ));

        assert_eq!(history.antecedents(matured), vec![opening]);
        assert_eq!(
            history.antecedents(died),
            vec![matured, opening],
            "nearest first"
        );
        assert_eq!(history.line_of(A), vec![opening, matured, died]);
    }

    #[test]
    fn independent_creatures_have_independent_lines() {
        // The property a flat log destroys: two creatures that never met are
        // concurrent, not ordered.
        let mut history = History::new();
        let a = history.record(at(1, born(A, None)));
        let b = history.record(at(1, born(B, None)));

        assert!(history.concurrent(a, b), "neither led to the other");
        assert!(history.consequences(a).is_empty());
    }

    #[test]
    fn eating_joins_two_lines() {
        // The join that makes this a graph. Until they met, these creatures
        // had nothing to do with each other; afterwards, one's past is part of
        // the other's.
        let mut history = History::new();
        let a_born = history.record(at(1, born(A, None)));
        let b_born = history.record(at(1, born(B, None)));
        assert!(history.concurrent(a_born, b_born));

        let fed = history.record(at(9, meal()));

        assert_eq!(
            history.antecedents(fed),
            vec![a_born, b_born],
            "both lines are cited"
        );
        assert_eq!(history.consequences(a_born), vec![fed]);
        assert_eq!(
            history.consequences(b_born),
            vec![fed],
            "the eaten one led here too"
        );
    }

    #[test]
    fn a_birth_descends_from_its_parent() {
        let mut history = History::new();
        let parent = history.record(at(1, born(A, None)));
        let child = history.record(at(40, born(B, Some(A))));

        assert_eq!(history.antecedents(child), vec![parent]);
        assert_eq!(history.consequences(parent), vec![child]);
        assert_eq!(
            history.latest(A),
            Some(child),
            "the parent's line continues through it"
        );
    }

    #[test]
    fn consequences_reach_through_a_chain() {
        // The retroactive definition of significance: what a thing led to,
        // however far downstream.
        let mut history = History::new();
        history.record(at(1, born(A, None)));
        let b_born = history.record(at(1, born(B, None)));
        let fed = history.record(at(9, meal()));
        let grew = history.record(at(
            10,
            Event::Grew {
                organism: A,
                part: PartId(1),
            },
        ));

        assert_eq!(
            history.consequences(b_born),
            vec![fed, grew],
            "being eaten led, eventually, to somebody else's new limb"
        );
    }

    #[test]
    fn severing_continues_the_line_it_happened_to() {
        let mut history = History::new();
        history.record(at(1, born(A, None)));
        let grew = history.record(at(
            2,
            Event::Grew {
                organism: A,
                part: PartId(1),
            },
        ));
        let lost = history.record(at(
            3,
            Event::Severed {
                organism: A,
                part: PartId(1),
            },
        ));

        assert!(
            history.antecedents(lost).contains(&grew),
            "you can only lose what you grew"
        );
    }

    #[test]
    fn a_history_round_trips() {
        let mut history = History::new();
        history.record(at(1, born(A, None)));
        history.record(at(2, meal()));

        let bytes = crate::snapshot::encode(&history).unwrap();
        assert_eq!(crate::snapshot::decode::<History>(&bytes).unwrap(), history);
    }

    #[test]
    fn every_variant_names_its_subjects() {
        // The links are built from `subjects`, so a variant that forgets one
        // silently forks that creature's history. Cheap to assert, expensive
        // to discover later.
        let all = [
            born(A, Some(B)),
            Event::Matured { organism: A },
            meal(),
            Event::Grew {
                organism: A,
                part: PartId(0),
            },
            Event::Burned {
                organism: A,
                energy_mg: 1,
            },
            Event::Severed {
                organism: A,
                part: PartId(0),
            },
            Event::Died {
                organism: A,
                species: SPECIES,
            },
            Event::Returned { organism: A },
            Event::Inhabited { organism: A },
        ];
        for event in all {
            assert!(!event.subjects().is_empty(), "{event:?} names nobody");
        }
    }
}
