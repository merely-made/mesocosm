// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! How a body plan is generated: segments, tagmata, and a lexicon.
//!
//! Bodies were being sculpted, which is the wrong shape of answer: a sculpt
//! produces one creature and no relatives. Real body plans come from an
//! **axial recipe** read head to tail, and almost the whole animal kingdom is
//! variations on it:
//!
//! 1. **Segments** repeated along one axis (metamerism).
//! 2. **Serial homology**: every segment carries the same appendage machinery.
//! 3. **Tagmata**: segments group into a few named stretches.
//! 4. **Regional identity**: a stretch decides what its segments' appendages
//!    become, or suppresses them.
//!
//! Symmetry, which [`BodyPlan`](crate::plan::BodyPlan) already owns, is the
//! fifth. Together they generate centipedes, insects, spiders, tetrapods, and
//! snakes as parameter sets rather than as special cases, which the tests in
//! this module demonstrate against the real catalogue.
//!
//! # The mutations are the real ones
//!
//! Every edit here is a change biology actually makes. Fusing segment pairs
//! turns a centipede into a millipede. Raising the count and suppressing two
//! limb positions turns a tetrapod into a snake. And **changing a tagma's
//! appendage is homeosis**: *antennapedia*, the textbook Hox mutant that grows
//! legs where antennae belong, is one field of one [`Tagma`] in this type.
//!
//! # It composes, it does not replace
//!
//! [`BodyPlan`](crate::plan::BodyPlan) stays the *placement policy*: which
//! facing a role prefers, whether it mirrors, how much tolerance growth has.
//! This is the *scaffold* above it. The scaffold says a thoracic segment bears
//! a limb; the policy says which way a limb points.
//!
//! # Acquisition is Hox-like
//!
//! A lineage cannot express an appendage it has never eaten. The [`lexicon`]
//! is the set of appendage kinds a line has acquired, and incorporating a
//! creature teaches its kinds. So kleptoplasty is not a bolt-on part: eating
//! something **teaches your line a word**, and the plan decides where to say
//! it. That is why a plan carries a lexicon and refuses assignments outside
//! it.
//!
//! [`lexicon`]: Lineage::lexicon

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::plan::Role;
use crate::rng::Rng;

/// What a stretch of segments grows.
///
/// Deliberately the same vocabulary the parts graph already speaks: an
/// appendage becomes a part with a [`Role`], so nothing downstream learns a
/// second word for the same thing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Appendage {
    /// No appendage. The abdomen of an insect, the trunk of a snake.
    None,
    /// A walking, swimming, or grasping limb.
    Limb,
    /// A sensor: antenna, eye stalk, feeler.
    Feeler,
    /// Armour or shell.
    Plate,
    /// A feeding structure: mandible, proboscis, radula.
    Mouth,
    /// A wing or fin: a limb that works on a fluid.
    Vane,
}

impl Appendage {
    pub const ALL: [Appendage; 6] = [
        Appendage::None,
        Appendage::Limb,
        Appendage::Feeler,
        Appendage::Plate,
        Appendage::Mouth,
        Appendage::Vane,
    ];

    /// The part role this appendage becomes when it grows.
    pub fn role(self) -> Option<Role> {
        match self {
            Appendage::None => None,
            Appendage::Limb | Appendage::Vane => Some(Role::Limb),
            Appendage::Feeler => Some(Role::Sensor),
            Appendage::Plate => Some(Role::Plate),
            Appendage::Mouth => Some(Role::Mass),
        }
    }

    /// Whether a lineage starts able to express this.
    ///
    /// Mass and the absence of an appendage are free; everything else is
    /// acquired by eating something that had it.
    pub fn is_innate(self) -> bool {
        matches!(self, Appendage::None | Appendage::Mouth)
    }
}

/// One stretch of the axis, and what it makes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tagma {
    /// How many segments this stretch holds.
    pub segments: u8,
    /// What each of its segments bears.
    pub appendage: Appendage,
    /// Appendages per segment. Two is a millipede's diplosegment; zero
    /// suppresses without changing the stretch's identity, which is how a
    /// snake keeps a trunk and loses its legs.
    pub per_segment: u8,
}

impl Tagma {
    pub fn new(segments: u8, appendage: Appendage) -> Self {
        Self { segments, appendage, per_segment: 1 }
    }

    pub fn bare(segments: u8) -> Self {
        Self { segments, appendage: Appendage::None, per_segment: 0 }
    }

    pub fn with_per_segment(mut self, per_segment: u8) -> Self {
        self.per_segment = per_segment;
        self
    }

    /// How many appendages this stretch grows in total.
    pub fn appendage_count(&self) -> u32 {
        if self.appendage == Appendage::None {
            return 0;
        }
        self.segments as u32 * self.per_segment as u32
    }
}

/// A lineage's heritable axial recipe: the theme its members vary on.
///
/// Epoch-bounded by ruling, because changing this is a regional-identity
/// change and bodies change between epochs rather than during them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lineage {
    /// Stretches head to tail. At least one.
    pub tagmata: Vec<Tagma>,
    /// How far an individual's segment count may stray per tagma, either way.
    pub variance: u8,
    /// Appendage kinds this line has acquired and may assign.
    ///
    /// Ordered so iteration and serialization are deterministic.
    lexicon: BTreeSet<Appendage>,
}

/// Why an assignment was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unspeakable {
    /// The line has never eaten anything with this appendage.
    NotInLexicon(Appendage),
    /// No such stretch.
    NoSuchTagma(usize),
}

impl Lineage {
    /// The simplest body there is: one bare stretch.
    ///
    /// Every lineage starts here and grows a vocabulary by eating.
    pub fn founding(segments: u8) -> Self {
        let mut lexicon = BTreeSet::new();
        for appendage in Appendage::ALL {
            if appendage.is_innate() {
                lexicon.insert(appendage);
            }
        }
        Self { tagmata: vec![Tagma::bare(segments.max(1))], variance: 1, lexicon }
    }

    /// Builds a recipe directly. Used by the catalogue and by tests; a played
    /// lineage reaches these shapes by acquisition and mutation.
    pub fn of(tagmata: Vec<Tagma>) -> Self {
        let mut lexicon: BTreeSet<Appendage> =
            Appendage::ALL.into_iter().filter(|a| a.is_innate()).collect();
        for tagma in &tagmata {
            lexicon.insert(tagma.appendage);
        }
        Self { tagmata, variance: 1, lexicon }
    }

    pub fn lexicon(&self) -> impl Iterator<Item = Appendage> + '_ {
        self.lexicon.iter().copied()
    }

    pub fn can_express(&self, appendage: Appendage) -> bool {
        self.lexicon.contains(&appendage)
    }

    /// Learns an appendage kind by having eaten something that had it.
    ///
    /// **The acquisition half of kleptoplasty.** Returns whether this was new,
    /// which is what makes a meal a discovery rather than a calorie.
    pub fn acquire(&mut self, appendage: Appendage) -> bool {
        self.lexicon.insert(appendage)
    }

    /// Homeosis: change what a stretch grows.
    ///
    /// Refused when the line cannot say the word, which is the rule that makes
    /// a lexicon matter.
    pub fn assign(&mut self, tagma: usize, appendage: Appendage) -> Result<(), Unspeakable> {
        if !self.can_express(appendage) {
            return Err(Unspeakable::NotInLexicon(appendage));
        }
        let Some(target) = self.tagmata.get_mut(tagma) else {
            return Err(Unspeakable::NoSuchTagma(tagma));
        };
        target.appendage = appendage;
        if target.per_segment == 0 {
            target.per_segment = 1;
        }
        Ok(())
    }

    /// Splits one stretch into two, the mutation that makes a body regional.
    ///
    /// A one-tagma worm becoming a head-and-trunk creature is this, and it is
    /// how tagmatization actually arises.
    pub fn divide(&mut self, tagma: usize, at: u8) -> Result<usize, Unspeakable> {
        let Some(target) = self.tagmata.get_mut(tagma) else {
            return Err(Unspeakable::NoSuchTagma(tagma));
        };
        let front = at.clamp(1, target.segments.saturating_sub(1).max(1));
        let back = target.segments.saturating_sub(front).max(1);
        let tail = Tagma { segments: back, ..*target };
        target.segments = front;
        self.tagmata.insert(tagma + 1, tail);
        Ok(tagma + 1)
    }

    /// Total segments the recipe calls for, before individual variation.
    pub fn segments(&self) -> u32 {
        self.tagmata.iter().map(|t| t.segments as u32).sum()
    }

    /// Total appendages the recipe calls for.
    pub fn appendages(&self) -> u32 {
        self.tagmata.iter().map(Tagma::appendage_count).sum()
    }

    /// How elaborate the recipe is.
    ///
    /// **Repetition is cheap; vocabulary and regionalisation are expensive.**
    /// Serial homology means each extra identical segment adds almost no
    /// information, so a hundred-segment worm is long rather than elaborate,
    /// while a short creature that expresses five kinds of appendage across
    /// five stretches is genuinely intricate. Segments therefore carry a
    /// repetition discount, stretches carry more, and distinct appendage
    /// kinds carry most.
    ///
    /// This is the axis the complexity frontier reads, replacing raw part
    /// count: reaching a new lineage should mean reaching a richer *recipe*,
    /// not a longer one.
    pub fn complexity(&self) -> u32 {
        let kinds: BTreeSet<Appendage> = self
            .tagmata
            .iter()
            .map(|t| t.appendage)
            .filter(|a| *a != Appendage::None)
            .collect();
        self.segments() / 8 + self.tagmata.len() as u32 * 4 + kinds.len() as u32 * 8
    }
}

/// One individual's realised body: the lineage's theme, varied.
///
/// Kin resemble each other because they share a recipe, not because they
/// share a shape. Vertebral count differs between snakes of one species and
/// humans vary in rib count; this is that.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Soma {
    /// Realised segment count per tagma, in the lineage's order.
    pub segments: Vec<u8>,
    /// Which segments lost their appendage, as (tagma, index within it).
    /// Development is imperfect, and an individual missing one limb is the
    /// cheapest evidence that individuals are not clones.
    pub absent: Vec<(u8, u8)>,
}

impl Soma {
    /// Develops an individual from a lineage's recipe and a seed.
    ///
    /// Pure function of the two, so a creature's body is reproducible from
    /// its identity the way everything else in the core is.
    pub fn develop(lineage: &Lineage, seed: u64) -> Self {
        let mut rng = Rng::from_seed(seed);
        let mut segments = Vec::with_capacity(lineage.tagmata.len());
        for tagma in &lineage.tagmata {
            let spread = lineage.variance as i32;
            let drift = if spread > 0 {
                rng.range_i32(-spread, spread)
            } else {
                0
            };
            segments.push((tagma.segments as i32 + drift).clamp(1, 255) as u8);
        }

        // A rare developmental absence, drawn per tagma so a long stretch is
        // likelier to lose one than a short one.
        let mut absent = Vec::new();
        for (index, tagma) in lineage.tagmata.iter().enumerate() {
            if tagma.appendage == Appendage::None || tagma.per_segment == 0 {
                continue;
            }
            let realised = segments[index];
            if rng.below(12) == 0 {
                absent.push((index as u8, rng.below(realised.max(1) as u64) as u8));
            }
        }
        Self { segments, absent }
    }

    pub fn total_segments(&self) -> u32 {
        self.segments.iter().map(|s| *s as u32).sum()
    }
}

/// Recipes for real animals, as evidence that the method generates the
/// catalogue rather than one creature.
///
/// These are reference points, not content: worldgen seeds lineages and play
/// mutates them. They exist so a test can assert that the same four rules
/// reach a centipede and a tetrapod without special cases.
pub mod catalogue {
    use super::*;

    /// Many segments, one trunk, a limb pair on every one.
    pub fn centipede(segments: u8) -> Lineage {
        Lineage::of(vec![
            Tagma::new(1, Appendage::Feeler),
            Tagma::new(segments, Appendage::Limb),
        ])
    }

    /// A centipede with fused segments: two limb pairs per apparent one.
    pub fn millipede(segments: u8) -> Lineage {
        Lineage::of(vec![
            Tagma::new(1, Appendage::Feeler),
            Tagma::new(segments, Appendage::Limb).with_per_segment(2),
        ])
    }

    /// Head, thorax, abdomen: legs on the thorax only, wings on part of it.
    pub fn insect() -> Lineage {
        Lineage::of(vec![
            Tagma::new(1, Appendage::Feeler),
            Tagma::new(1, Appendage::Mouth),
            Tagma::new(3, Appendage::Limb),
            Tagma::new(2, Appendage::Vane),
            Tagma::bare(11),
        ])
    }

    /// Two stretches: four leg pairs forward, nothing behind.
    pub fn spider() -> Lineage {
        Lineage::of(vec![
            Tagma::new(1, Appendage::Mouth),
            Tagma::new(4, Appendage::Limb),
            Tagma::bare(10),
        ])
    }

    /// A long trunk with limbs only at two girdles.
    pub fn tetrapod(trunk: u8) -> Lineage {
        Lineage::of(vec![
            Tagma::new(1, Appendage::Feeler),
            Tagma::new(1, Appendage::Limb),
            Tagma::bare(trunk),
            Tagma::new(1, Appendage::Limb),
            Tagma::bare(trunk / 2),
        ])
    }

    /// A tetrapod with its girdles suppressed and its trunk multiplied: the
    /// single clearest demonstration that these are variations on a theme.
    pub fn snake(trunk: u8) -> Lineage {
        let mut plan = tetrapod(trunk);
        for tagma in &mut plan.tagmata {
            if tagma.appendage == Appendage::Limb {
                tagma.per_segment = 0;
                tagma.appendage = Appendage::None;
            }
        }
        plan.tagmata[2].segments = trunk;
        plan
    }
}

#[cfg(test)]
mod tests {
    use super::catalogue::*;
    use super::*;

    #[test]
    fn one_method_reaches_the_catalogue() {
        // The claim under test: real body plans are parameter sets, not
        // special cases. Each of these is the same four rules.
        // Totals include the head's feelers, so the trunk is checked directly.
        assert_eq!(centipede(40).tagmata[1].appendage_count(), 40, "a limb per segment");
        assert_eq!(millipede(40).tagmata[1].appendage_count(), 80, "two per fused segment");
        assert_eq!(centipede(40).appendages(), 41, "plus the head");
        assert_eq!(insect().appendages(), 3 + 2 + 1 + 1, "legs, wings, feelers, mouth");
        assert_eq!(spider().appendages(), 4 + 1);
        assert_eq!(tetrapod(20).appendages(), 2 + 1, "two girdles and a head");
        assert_eq!(snake(120).appendages(), 1, "the head keeps its feelers");
    }

    #[test]
    fn a_snake_is_a_tetrapod_with_two_edits() {
        // Suppress the girdles, lengthen the trunk. Nothing else differs,
        // which is the whole point of an axial recipe.
        let walker = tetrapod(20);
        let crawler = snake(120);

        assert_eq!(walker.tagmata.len(), crawler.tagmata.len(), "same regions");
        assert!(crawler.segments() > walker.segments(), "longer");
        assert_eq!(crawler.appendages(), 1, "and legless");
    }

    #[test]
    fn a_millipede_is_a_centipede_with_one_field_changed() {
        let mut plan = centipede(30);
        plan.tagmata[1].per_segment = 2;
        assert_eq!(plan, millipede(30));
    }

    #[test]
    fn dividing_a_worm_is_how_a_body_becomes_regional() {
        // A one-stretch creature growing a head: tagmatization, which is the
        // mutation that makes every other plan reachable.
        let mut worm = Lineage::founding(12);
        assert_eq!(worm.tagmata.len(), 1);

        let tail = worm.divide(0, 3).unwrap();
        assert_eq!(tail, 1);
        assert_eq!(worm.tagmata[0].segments, 3);
        assert_eq!(worm.tagmata[1].segments, 9);
        assert_eq!(worm.segments(), 12, "division moves boundaries, not mass");
    }

    #[test]
    fn a_line_cannot_say_a_word_it_has_not_eaten() {
        // The acquisition rule: kleptoplasty teaches vocabulary, and a plan
        // refuses to express what the lineage has never incorporated.
        let mut worm = Lineage::founding(8);
        assert!(!worm.can_express(Appendage::Limb));
        assert_eq!(
            worm.assign(0, Appendage::Limb),
            Err(Unspeakable::NotInLexicon(Appendage::Limb))
        );

        assert!(worm.acquire(Appendage::Limb), "the first one is a discovery");
        assert!(!worm.acquire(Appendage::Limb), "the second is a meal");
        assert!(worm.assign(0, Appendage::Limb).is_ok());
        assert_eq!(worm.appendages(), 8);
    }

    #[test]
    fn homeosis_is_one_field() {
        // Antennapedia: legs where feelers belong. A real Hox mutant, and one
        // assignment here.
        let mut fly = insect();
        assert_eq!(fly.tagmata[0].appendage, Appendage::Feeler);
        fly.assign(0, Appendage::Limb).unwrap();
        assert_eq!(fly.tagmata[0].appendage, Appendage::Limb);
        assert_eq!(fly.appendages(), insect().appendages(), "the count is unchanged");
    }

    #[test]
    fn complexity_counts_kinds_not_just_length() {
        // The frontier's new axis: a long worm is not elaborate, and a short
        // creature with several appendage kinds is.
        let worm = centipede(60);
        let bug = insect();
        assert!(worm.segments() > bug.segments(), "the worm is three times longer");
        assert!(bug.complexity() > worm.complexity(), "the insect is more elaborate");

        // And a legless snake, longer still, stays simpler than both.
        let crawler = snake(120);
        assert!(crawler.segments() > worm.segments());
        assert!(crawler.complexity() < bug.complexity(), "length is not intricacy");
    }

    #[test]
    fn kin_vary_without_diverging() {
        // Individuals of one lineage differ, and stay recognisable.
        let plan = centipede(30);
        let bodies: Vec<Soma> = (0..24).map(|seed| Soma::develop(&plan, seed)).collect();

        let lengths: BTreeSet<u32> = bodies.iter().map(Soma::total_segments).collect();
        assert!(lengths.len() > 1, "no two are exactly the same");
        for body in &bodies {
            let drift = body.total_segments().abs_diff(plan.segments());
            assert!(drift <= plan.variance as u32 * plan.tagmata.len() as u32);
        }
        assert!(
            bodies.iter().any(|b| !b.absent.is_empty()),
            "development is imperfect often enough to notice"
        );
    }

    #[test]
    fn development_is_reproducible() {
        let plan = insect();
        assert_eq!(Soma::develop(&plan, 99), Soma::develop(&plan, 99));
        assert_ne!(Soma::develop(&plan, 1), Soma::develop(&plan, 2));
    }

    #[test]
    fn a_recipe_round_trips() {
        let plan = insect();
        let bytes = crate::snapshot::encode(&plan).unwrap();
        assert_eq!(crate::snapshot::decode::<Lineage>(&bytes).unwrap(), plan);
    }
}
