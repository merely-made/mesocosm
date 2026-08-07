// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Develops an axial recipe into the authoritative parts graph.
//!
//! [`Recipe`](crate::axis::Recipe) and [`Soma`](crate::axis::Soma) used to stop
//! at a renderer-only capsule body. That left the heritable program and the
//! organism's actual anatomy unrelated. This module is the join: segments and
//! appendages become ordinary [`Part`](crate::body::Part)s, so every later
//! projection reads the same body document.

use crate::axis::{Appendage, Recipe, Soma};
use crate::body::{AttachError, Attachment, BodyDocument, Provenance, SpeciesId, VolumeRef, Yaw};
use crate::plan::{Role, classify};
use serde::{Deserialize, Serialize};

/// The shape and content address used when development expresses one role.
///
/// The core owns the role and exact integer shape, while an asset-bearing host
/// supplies the content address. This prevents development from inventing
/// renderer assets or assuming that one volume format is universal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartTemplate {
    pub volume: VolumeRef,
    pub half_extent: [i32; 3],
}

/// Templates available to one developmental realization.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartPalette {
    pub mass: PartTemplate,
    pub limb: PartTemplate,
    pub plate: PartTemplate,
    pub sensor: PartTemplate,
}

impl PartPalette {
    /// The current world's baseline admitted part vocabulary.
    ///
    /// The references are fixture content addresses while the project has no
    /// pack loader. Keeping the palette as world state means another world can
    /// admit different materials without changing a lineage's recipe.
    pub fn primitive() -> Self {
        Self {
            mass: PartTemplate {
                volume: VolumeRef::from_tag(1),
                half_extent: [2, 2, 2],
            },
            limb: PartTemplate {
                volume: VolumeRef::from_tag(2),
                half_extent: [4, 1, 1],
            },
            plate: PartTemplate {
                volume: VolumeRef::from_tag(3),
                half_extent: [4, 4, 1],
            },
            sensor: PartTemplate {
                volume: VolumeRef::from_tag(4),
                half_extent: [1, 1, 1],
            },
        }
    }

    pub fn template(self, role: Role) -> PartTemplate {
        match role {
            Role::Mass => self.mass,
            Role::Limb => self.limb,
            Role::Plate => self.plate,
            Role::Sensor => self.sensor,
        }
    }

    fn validate(self) -> Result<(), DevelopmentError> {
        for role in Role::ALL {
            let actual = classify(self.template(role).half_extent);
            if actual != role {
                return Err(DevelopmentError::WrongRole {
                    expected: role,
                    actual,
                });
            }
        }
        Ok(())
    }
}

impl Default for PartPalette {
    fn default() -> Self {
        Self::primitive()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DevelopmentError {
    SomaLength { tagmata: usize, realised: usize },
    EmptyAxis,
    InvalidAbsence { tagma: u8, segment: u8 },
    WrongRole { expected: Role, actual: Role },
    TooManyParts,
    InsufficientMass { mass_mg: u64, parts: u32 },
    Attach(AttachError),
}

impl From<AttachError> for DevelopmentError {
    fn from(value: AttachError) -> Self {
        Self::Attach(value)
    }
}

/// Realizes one individual's anatomy from its lineage recipe.
///
/// Axial segments form the dependency spine. Appendages attach to the segment
/// that expressed them, so severing a segment cascades through everything it
/// bore. Every part has positive mass and the result conserves `mass_mg`
/// exactly. The initial proof uses founding provenance; learned-source
/// addresses remain the later filial-provenance gate.
pub fn develop_body(
    species: SpeciesId,
    recipe: &Recipe,
    soma: &Soma,
    mass_mg: u64,
    palette: PartPalette,
) -> Result<BodyDocument, DevelopmentError> {
    palette.validate()?;
    let count = minimum_body_mass_mg(recipe, soma)?;
    if mass_mg < u64::from(count) {
        return Err(DevelopmentError::InsufficientMass {
            mass_mg,
            parts: count,
        });
    }
    let each = mass_mg / u64::from(count);
    let root_mass = each + mass_mg % u64::from(count);
    let segment_template = palette.template(Role::Mass);
    let mut body = BodyDocument::new(
        species,
        segment_template.volume,
        root_mass,
        segment_template.half_extent,
    );

    let mut previous_segment = body.root;
    let mut first = true;
    for (tagma_index, tagma) in recipe.tagmata.iter().enumerate() {
        let realised = soma.segments[tagma_index];
        for segment_index in 0..realised {
            let segment = if first {
                first = false;
                body.root
            } else {
                let previous_half = body
                    .part(previous_segment)
                    .expect("development only records attached segments")
                    .half_extent;
                let offset = [
                    0,
                    0,
                    previous_half[2].abs() + segment_template.half_extent[2].abs(),
                ];
                body.attach(
                    segment_template.volume,
                    each,
                    segment_template.half_extent,
                    Attachment {
                        parent: previous_segment,
                        offset,
                        yaw: Yaw::Zero,
                    },
                    Provenance::founding(),
                )?
            };
            previous_segment = segment;

            if is_absent(soma, tagma_index, segment_index) {
                continue;
            }
            attach_appendages(
                &mut body,
                segment,
                tagma.appendage,
                tagma.per_segment,
                each,
                palette,
            )?;
        }
    }

    debug_assert_eq!(body.len(), count as usize);
    debug_assert_eq!(body.total_mass_mg(), mass_mg);
    Ok(body)
}

/// The least biomass that can express this realization while keeping every
/// structural part positive-mass.
///
/// A birth below this floor waits for provisioning. It never clones anatomy
/// the parent did not pay for and never silently drops part of the recipe.
pub fn minimum_body_mass_mg(recipe: &Recipe, soma: &Soma) -> Result<u32, DevelopmentError> {
    validate_soma(recipe, soma)?;
    part_count(recipe, soma)
}

fn validate_soma(recipe: &Recipe, soma: &Soma) -> Result<(), DevelopmentError> {
    if recipe.tagmata.len() != soma.segments.len() {
        return Err(DevelopmentError::SomaLength {
            tagmata: recipe.tagmata.len(),
            realised: soma.segments.len(),
        });
    }
    if soma.segments.iter().all(|segments| *segments == 0) {
        return Err(DevelopmentError::EmptyAxis);
    }
    for &(tagma, segment) in &soma.absent {
        let Some(realised) = soma.segments.get(tagma as usize) else {
            return Err(DevelopmentError::InvalidAbsence { tagma, segment });
        };
        if segment >= *realised {
            return Err(DevelopmentError::InvalidAbsence { tagma, segment });
        }
    }
    Ok(())
}

fn part_count(recipe: &Recipe, soma: &Soma) -> Result<u32, DevelopmentError> {
    let mut count = 0u32;
    for (tagma_index, tagma) in recipe.tagmata.iter().enumerate() {
        let realised = soma.segments[tagma_index];
        count = count
            .checked_add(u32::from(realised))
            .ok_or(DevelopmentError::TooManyParts)?;
        for segment in 0..realised {
            if is_absent(soma, tagma_index, segment) {
                continue;
            }
            let pieces = appendage_pieces(tagma.appendage, tagma.per_segment);
            count = count
                .checked_add(pieces)
                .ok_or(DevelopmentError::TooManyParts)?;
        }
    }
    if count == 0 {
        return Err(DevelopmentError::EmptyAxis);
    }
    Ok(count)
}

fn appendage_pieces(appendage: Appendage, per_segment: u8) -> u32 {
    let count = u32::from(per_segment);
    match appendage {
        Appendage::None => 0,
        Appendage::Limb | Appendage::Feeler | Appendage::Vane => count * 2,
        Appendage::Plate | Appendage::Mouth => count,
    }
}

fn is_absent(soma: &Soma, tagma: usize, segment: u8) -> bool {
    u8::try_from(tagma)
        .ok()
        .is_some_and(|tagma| soma.absent.contains(&(tagma, segment)))
}

fn attach_appendages(
    body: &mut BodyDocument,
    segment: crate::body::PartId,
    appendage: Appendage,
    per_segment: u8,
    mass_mg: u64,
    palette: PartPalette,
) -> Result<(), DevelopmentError> {
    let Some(role) = appendage.role() else {
        return Ok(());
    };
    let template = palette.template(role);
    let segment_half = body
        .part(segment)
        .expect("development only addresses attached segments")
        .half_extent;

    for ordinal in 0..per_segment {
        let along = slot_offset(ordinal, per_segment, template.half_extent[2].abs());
        match appendage {
            Appendage::Limb | Appendage::Feeler | Appendage::Vane => {
                for side in [-1, 1] {
                    let offset = [
                        side * (segment_half[0].abs() + template.half_extent[0].abs()),
                        0,
                        along,
                    ];
                    body.attach(
                        template.volume,
                        mass_mg,
                        template.half_extent,
                        Attachment {
                            parent: segment,
                            offset,
                            yaw: Yaw::Zero,
                        },
                        Provenance::founding(),
                    )?;
                }
            }
            Appendage::Plate => {
                let offset = [
                    0,
                    segment_half[1].abs() + template.half_extent[1].abs(),
                    along,
                ];
                body.attach(
                    template.volume,
                    mass_mg,
                    template.half_extent,
                    Attachment {
                        parent: segment,
                        offset,
                        yaw: Yaw::Zero,
                    },
                    Provenance::founding(),
                )?;
            }
            Appendage::Mouth => {
                let offset = [
                    0,
                    -(segment_half[1].abs() + template.half_extent[1].abs()),
                    along,
                ];
                body.attach(
                    template.volume,
                    mass_mg,
                    template.half_extent,
                    Attachment {
                        parent: segment,
                        offset,
                        yaw: Yaw::Zero,
                    },
                    Provenance::founding(),
                )?;
            }
            Appendage::None => {}
        }
    }
    Ok(())
}

fn slot_offset(ordinal: u8, count: u8, half: i32) -> i32 {
    (i32::from(ordinal) * 2 + 1 - i32::from(count)) * half.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::axis::{Tagma, catalogue};

    fn palette() -> PartPalette {
        PartPalette {
            mass: PartTemplate {
                volume: VolumeRef::from_tag(1),
                half_extent: [2, 2, 2],
            },
            limb: PartTemplate {
                volume: VolumeRef::from_tag(2),
                half_extent: [4, 1, 1],
            },
            plate: PartTemplate {
                volume: VolumeRef::from_tag(3),
                half_extent: [4, 4, 1],
            },
            sensor: PartTemplate {
                volume: VolumeRef::from_tag(4),
                half_extent: [1, 1, 1],
            },
        }
    }

    fn exact_soma(recipe: &Recipe) -> Soma {
        Soma {
            segments: recipe.tagmata.iter().map(|tagma| tagma.segments).collect(),
            absent: Vec::new(),
        }
    }

    #[test]
    fn appendage_roles_become_actual_parts() {
        let recipe = Recipe::of(vec![
            Tagma::new(1, Appendage::Feeler),
            Tagma::new(1, Appendage::Mouth),
            Tagma::new(1, Appendage::Limb),
            Tagma::new(1, Appendage::Plate),
        ]);
        let body = develop_body(
            SpeciesId(7),
            &recipe,
            &exact_soma(&recipe),
            1_000,
            palette(),
        )
        .unwrap();
        let roles: Vec<_> = body
            .living()
            .map(|part| classify(part.half_extent))
            .collect();

        assert_eq!(
            roles.iter().filter(|role| **role == Role::Sensor).count(),
            2
        );
        assert_eq!(roles.iter().filter(|role| **role == Role::Limb).count(), 2);
        assert_eq!(roles.iter().filter(|role| **role == Role::Plate).count(), 1);
        assert_eq!(roles.iter().filter(|role| **role == Role::Mass).count(), 5);
    }

    #[test]
    fn a_snake_and_tetrapod_now_have_different_part_graphs() {
        let tetrapod = catalogue::tetrapod(6);
        let snake = catalogue::snake(16);
        let legs = develop_body(
            SpeciesId(1),
            &tetrapod,
            &exact_soma(&tetrapod),
            4_000,
            palette(),
        )
        .unwrap();
        let legless =
            develop_body(SpeciesId(2), &snake, &exact_soma(&snake), 4_000, palette()).unwrap();

        assert!(
            legs.living()
                .any(|part| classify(part.half_extent) == Role::Limb)
        );
        assert!(
            !legless
                .living()
                .any(|part| classify(part.half_extent) == Role::Limb)
        );
        assert!(
            legless.living().count() > legs.living().count(),
            "length remains anatomy"
        );
    }

    #[test]
    fn developmental_absence_removes_the_appendages_not_the_segment() {
        let recipe = catalogue::centipede(4);
        let complete = exact_soma(&recipe);
        let mut absent = complete.clone();
        absent.absent.push((1, 2));
        let whole = develop_body(SpeciesId(1), &recipe, &complete, 1_000, palette()).unwrap();
        let varied = develop_body(SpeciesId(1), &recipe, &absent, 1_000, palette()).unwrap();

        assert_eq!(
            whole.len() - varied.len(),
            2,
            "one bilateral appendage pair is absent"
        );
        let whole_mass = whole
            .living()
            .filter(|p| classify(p.half_extent) == Role::Mass)
            .count();
        let varied_mass = varied
            .living()
            .filter(|p| classify(p.half_extent) == Role::Mass)
            .count();
        assert_eq!(whole_mass, varied_mass, "the axial segment still developed");
    }

    #[test]
    fn development_conserves_mass_and_is_deterministic() {
        let recipe = catalogue::insect();
        let soma = Soma::develop(&recipe, 19);
        let a = develop_body(SpeciesId(3), &recipe, &soma, 4_003, palette()).unwrap();
        let b = develop_body(SpeciesId(3), &recipe, &soma, 4_003, palette()).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.total_mass_mg(), 4_003);
        assert!(a.living().all(|part| part.mass_mg > 0));
    }

    #[test]
    fn a_palette_cannot_lie_about_a_role() {
        let recipe = catalogue::insect();
        let soma = exact_soma(&recipe);
        let mut wrong = palette();
        wrong.limb.half_extent = [2, 2, 2];
        assert_eq!(
            develop_body(SpeciesId(1), &recipe, &soma, 2_000, wrong),
            Err(DevelopmentError::WrongRole {
                expected: Role::Limb,
                actual: Role::Mass
            })
        );
    }

    #[test]
    fn the_expression_floor_is_the_number_of_parts() {
        let recipe = catalogue::centipede(4);
        let soma = exact_soma(&recipe);
        let minimum = minimum_body_mass_mg(&recipe, &soma).unwrap();
        let body =
            develop_body(SpeciesId(1), &recipe, &soma, u64::from(minimum), palette()).unwrap();

        assert_eq!(minimum as usize, body.len());
        assert!(body.living().all(|part| part.mass_mg == 1));
    }
}
