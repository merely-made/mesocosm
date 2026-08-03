// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

// The march: per-pixel raymarch over a heightfield, the Voxel Space lineage
// generalised from per-column spans to free pitch. Inputs are two images and
// a camera; outputs are colour (rgb) and normalised hit distance (a), which
// the grade pass reads as its fog and depth term.

struct MarchParams {
    // Camera position in map units (x, height, z).
    eye: vec3<f32>,
    yaw: f32,
    pitch: f32,
    // Horizontal field of view in radians.
    fov: f32,
    // Furthest march distance, map units.
    far: f32,
    // Map side length in texels (square maps).
    map_side: f32,
};

@group(0) @binding(0) var height_map: texture_2d<f32>;
@group(0) @binding(1) var color_map: texture_2d<f32>;
@group(0) @binding(2) var map_sampler: sampler;
@group(0) @binding(3) var<uniform> params: MarchParams;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) ndc: vec2<f32>,
};

@vertex
fn vs(@builtin(vertex_index) index: u32) -> VsOut {
    // One fullscreen triangle.
    var corners = array<vec2<f32>, 3>(
        vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0),
    );
    var out: VsOut;
    out.pos = vec4(corners[index], 0.0, 1.0);
    out.ndc = corners[index];
    return out;
}

fn terrain_height(at: vec2<f32>) -> f32 {
    let uv = at / params.map_side;
    return textureSampleLevel(height_map, map_sampler, uv, 0.0).r * 255.0;
}

fn terrain_color(at: vec2<f32>) -> vec3<f32> {
    let uv = at / params.map_side;
    return textureSampleLevel(color_map, map_sampler, uv, 0.0).rgb;
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
    // Ray direction from yaw, pitch, and the pixel's ndc offset.
    let half_fov = params.fov * 0.5;
    let ray_yaw = params.yaw + in.ndc.x * half_fov;
    // Vertical fov scales by the frame's implied aspect through ndc.y.
    let ray_pitch = params.pitch + in.ndc.y * half_fov * 0.5625;
    let dir = vec3(
        sin(ray_yaw) * cos(ray_pitch),
        sin(ray_pitch),
        cos(ray_yaw) * cos(ray_pitch),
    );

    // March with a growing step: fine nearby where detail reads, coarse far
    // away where fog eats it anyway. On a hit, one binary refinement pass
    // tightens the surface so nearby terrain does not stairstep.
    // Budget check: 0.35-unit steps to ~12 units, then 3% geometric growth
    // reaches past 2000 units well inside 300 iterations, so params.far is
    // the binding limit rather than the loop. An earlier cut grew at 1.2%
    // and silently topped out near 155 units, which painted every distant
    // vista as sky.
    var t = 0.5;
    var hit = false;
    var at = params.eye;
    for (var i = 0; i < 300; i = i + 1) {
        at = params.eye + dir * t;
        if (terrain_height(at.xz) > at.y) {
            hit = true;
            break;
        }
        if (t > params.far) {
            break;
        }
        t = t + max(0.35, t * 0.03);
    }

    if (!hit) {
        // Sky: a vertical gradient the grade pass tints. Distance 1.0 marks
        // "no terrain" for the fog term.
        let sky = mix(vec3(0.65, 0.72, 0.80), vec3(0.35, 0.45, 0.62), clamp(ray_pitch * 3.0 + 0.3, 0.0, 1.0));
        return vec4(sky, 1.0);
    }

    var lo = max(t - max(0.35, t * 0.03), 0.0);
    var hi = t;
    for (var i = 0; i < 6; i = i + 1) {
        let mid = (lo + hi) * 0.5;
        let p = params.eye + dir * mid;
        if (terrain_height(p.xz) > p.y) {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    at = params.eye + dir * hi;

    // Slope shading from the heightfield gradient: cheap, and the grade
    // pass's ramp decides how harsh it reads.
    let e = 1.5;
    let dx = terrain_height(at.xz + vec2(e, 0.0)) - terrain_height(at.xz - vec2(e, 0.0));
    let dz = terrain_height(at.xz + vec2(0.0, e)) - terrain_height(at.xz - vec2(0.0, e));
    let normal = normalize(vec3(-dx, 2.0 * e, -dz));
    let sun = normalize(vec3(0.4, 0.8, 0.3));
    let light = clamp(dot(normal, sun), 0.0, 1.0);

    let base = terrain_color(at.xz);
    let lit = base * (0.45 + 0.55 * light);
    return vec4(lit, clamp(hi / params.far, 0.0, 0.999));
}
