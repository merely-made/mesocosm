// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Turning placed quads into vertices.
//!
//! This is where integers become floats, and it is the only direction that
//! conversion runs. Geometry comes from the mesher in exact voxel units;
//! nothing computed here ever goes back to the core.

use bytemuck::{Pod, Zeroable};
use mesocosm_mesh::{BodyMesh, place_point};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: size_of::<[f32; 3]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x3,
            },
        ],
    };
}

/// Maps a material id to a base colour. A placeholder until materials become
/// real content; deliberately deterministic so a body looks the same each run.
pub fn material_colour(material: u8) -> [f32; 3] {
    // Cheap integer hash spread across a pleasant range, so distinct materials
    // are distinguishable without an authored palette.
    let h = material.wrapping_mul(97).wrapping_add(31);
    let r = 0.35 + ((h & 0b111) as f32 / 7.0) * 0.55;
    let g = 0.35 + (((h >> 3) & 0b111) as f32 / 7.0) * 0.55;
    let b = 0.35 + (((h >> 5) & 0b111) as f32 / 7.0) * 0.55;
    [r, g, b]
}

/// How much a face is lit, by which way it points. Top bright, sides mid,
/// bottom dark: the convention that makes untextured voxels read as solid.
pub fn face_shade(axis: u8, positive: bool) -> f32 {
    match (axis, positive) {
        (1, true) => 1.0,
        (1, false) => 0.45,
        (0, true) => 0.78,
        (0, false) => 0.66,
        (_, true) => 0.86,
        (_, false) => 0.58,
    }
}

/// Builds the triangle list for a meshed body.
///
/// Two triangles per quad, wound so the mesher's outward-facing corner order
/// survives. Returns an empty list for a body with no geometry, which the
/// renderer draws as an empty frame rather than treating as an error.
pub fn build_vertices(mesh: &BodyMesh) -> Vec<Vertex> {
    let mut out = Vec::new();

    for placement in &mesh.placements {
        let Some(part_mesh) = mesh.mesh_for(placement.volume) else {
            continue;
        };
        for quad in &part_mesh.quads {
            let shade = face_shade(quad.axis, quad.positive);
            let base = material_colour(quad.material);
            let colour = [base[0] * shade, base[1] * shade, base[2] * shade];

            let corners = quad.corners().map(|corner| {
                let placed = place_point(corner, placement.yaw, placement.offset);
                [placed[0] as f32, placed[1] as f32, placed[2] as f32]
            });

            for index in [0usize, 1, 2, 0, 2, 3] {
                out.push(Vertex { position: corners[index], color: colour });
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use mesocosm_core::{BodyDocument, SpeciesId, VolumeRef};
    use mesocosm_mesh::{Volume, VolumeMap, mesh_body};

    fn source() -> VolumeMap {
        let mut map = VolumeMap::new();
        map.insert(VolumeRef::from_tag(1), Volume::solid([2, 2, 2], 1));
        map
    }

    #[test]
    fn a_cube_becomes_six_quads_of_triangles() {
        let body = BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 100, [1, 1, 1]);
        let mesh = mesh_body(&body, &source()).unwrap();
        let vertices = build_vertices(&mesh);
        assert_eq!(vertices.len(), 6 * 6, "six faces, two triangles each");
    }

    #[test]
    fn an_unresolvable_body_yields_no_vertices() {
        let mesh = BodyMesh::default();
        assert!(build_vertices(&mesh).is_empty());
    }

    #[test]
    fn faces_are_shaded_by_direction() {
        assert!(face_shade(1, true) > face_shade(1, false), "top beats bottom");
        assert!(face_shade(1, true) > face_shade(0, true), "top beats sides");
    }

    #[test]
    fn materials_get_distinct_colours() {
        assert_ne!(material_colour(1), material_colour(2));
        assert_eq!(material_colour(7), material_colour(7));
    }

    #[test]
    fn vertex_is_tightly_packed() {
        assert_eq!(size_of::<Vertex>(), 24);
    }
}
