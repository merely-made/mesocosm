// Fragment-only brick DDA. Pointer zero is air; nonzero values identify one
// dense 8³ material slot in the atlas. It does not write, allocate, or derive
// simulation facts.

struct TraceParams {
    eye: vec3<f32>,
    yaw: f32,
    pitch: f32,
    fov: f32,
    far: f32,
    _pad: f32,
    world_min: vec4<f32>,
    pointer_extent: vec4<u32>,
    atlas_slots: vec4<u32>,
    fog: vec4<f32>,
    look: vec4<f32>,
    critter: CritterParams,
};

// The body remains a presentation-side SDF projection. Brick truth decides
// terrain occlusion; the nearer of the two traces owns a pixel.
struct CritterParams {
    bounds: vec4<f32>,
    tint_count: vec4<f32>,
    eyes: array<vec4<f32>, 2>,
    pairs: array<vec4<f32>, 192>,
};

@group(0) @binding(0) var pointers: texture_3d<u32>;
@group(0) @binding(1) var atlas: texture_3d<u32>;
@group(0) @binding(2) var<uniform> params: TraceParams;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) ndc: vec2<f32>,
};

struct Hit {
    material: u32,
    t: f32,
    normal: vec3<f32>,
    found: bool,
};

@vertex
fn vs(@builtin(vertex_index) index: u32) -> VsOut {
    var corners = array<vec2<f32>, 3>(
        vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0),
    );
    var out: VsOut;
    out.pos = vec4(corners[index], 0.0, 1.0);
    out.ndc = corners[index];
    return out;
}

fn material_at(voxel: vec3<i32>) -> u32 {
    let local = voxel - vec3<i32>(params.world_min.xyz);
    if (any(local < vec3<i32>(0))) {
        return 0u;
    }
    let cell = local / 8;
    let limits = vec3<i32>(params.pointer_extent.xyz);
    if (any(cell < vec3<i32>(0)) || any(cell >= limits)) {
        return 0u;
    }
    let slot = textureLoad(pointers, cell, 0).x;
    if (slot == 0u) {
        return 0u;
    }
    let index = slot - 1u;
    let sx = params.atlas_slots.x;
    let sz = params.atlas_slots.z;
    let slot_x = index % sx;
    let slot_z = (index / sx) % sz;
    let slot_y = index / (sx * sz);
    let within = local - cell * 8;
    let atlas_at = vec3<i32>(
        i32(slot_x * 8u) + within.x,
        i32(slot_y * 8u) + within.y,
        i32(slot_z * 8u) + within.z,
    );
    return textureLoad(atlas, atlas_at, 0).x;
}

fn ray_box(eye: vec3<f32>, direction: vec3<f32>) -> vec2<f32> {
    let low = params.world_min.xyz;
    let high = low + vec3<f32>(params.pointer_extent.xyz) * 8.0;
    var enter = 0.0;
    var exit = params.far;
    for (var axis = 0; axis < 3; axis = axis + 1) {
        let d = direction[axis];
        if (abs(d) < 1e-6) {
            if (eye[axis] < low[axis] || eye[axis] >= high[axis]) {
                return vec2(1.0, -1.0);
            }
            continue;
        }
        let a = (low[axis] - eye[axis]) / d;
        let b = (high[axis] - eye[axis]) / d;
        enter = max(enter, min(a, b));
        exit = min(exit, max(a, b));
    }
    return vec2(enter, exit);
}

fn initial_crossing(position: f32, direction: f32, voxel: i32, start_t: f32) -> f32 {
    if (direction > 1e-6) {
        return start_t + (f32(voxel + 1) - position) / direction;
    }
    if (direction < -1e-6) {
        return start_t + (f32(voxel) - position) / direction;
    }
    return 1e30;
}

fn dda(eye: vec3<f32>, direction: vec3<f32>) -> Hit {
    let interval = ray_box(eye, direction);
    if (interval.x > interval.y || interval.y < 0.0) {
        return Hit(0u, params.far, vec3(0.0, 1.0, 0.0), false);
    }
    let start_t = max(interval.x, 0.0) + 0.0001;
    let start = eye + direction * start_t;
    var voxel = vec3<i32>(floor(start));
    let step = vec3<i32>(
        select(-1, 1, direction.x >= 0.0),
        select(-1, 1, direction.y >= 0.0),
        select(-1, 1, direction.z >= 0.0),
    );
    var crossing = vec3(
        initial_crossing(start.x, direction.x, voxel.x, start_t),
        initial_crossing(start.y, direction.y, voxel.y, start_t),
        initial_crossing(start.z, direction.z, voxel.z, start_t),
    );
    let delta = vec3(
        select(1e30, 1.0 / abs(direction.x), abs(direction.x) > 1e-6),
        select(1e30, 1.0 / abs(direction.y), abs(direction.y) > 1e-6),
        select(1e30, 1.0 / abs(direction.z), abs(direction.z) > 1e-6),
    );
    var t = start_t;
    var normal = vec3(0.0, 1.0, 0.0);
    for (var count = 0; count < 1024; count = count + 1) {
        let material = material_at(voxel);
        if (material != 0u) {
            return Hit(material, t, normal, true);
        }
        if (crossing.x <= crossing.y && crossing.x <= crossing.z) {
            t = crossing.x;
            crossing.x = crossing.x + delta.x;
            voxel.x = voxel.x + step.x;
            normal = vec3(-f32(step.x), 0.0, 0.0);
        } else if (crossing.y <= crossing.z) {
            t = crossing.y;
            crossing.y = crossing.y + delta.y;
            voxel.y = voxel.y + step.y;
            normal = vec3(0.0, -f32(step.y), 0.0);
        } else {
            t = crossing.z;
            crossing.z = crossing.z + delta.z;
            voxel.z = voxel.z + step.z;
            normal = vec3(0.0, 0.0, -f32(step.z));
        }
        if (t > interval.y || t > params.far) {
            break;
        }
    }
    return Hit(0u, params.far, vec3(0.0, 1.0, 0.0), false);
}

fn material_colour(material: u32) -> vec3<f32> {
    if (material == 3u) {
        return vec3(0.38, 0.24, 0.13); // soil
    }
    if (material == 2u) {
        return vec3(0.32, 0.34, 0.40); // rock
    }
    return vec3(0.66, 0.20, 0.72);
}

fn capsule_distance(p: vec3<f32>, a: vec3<f32>, b: vec3<f32>, r: f32) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / max(dot(ba, ba), 1e-6), 0.0, 1.0);
    return length(pa - ba * h) - r;
}

fn one_capsule(p: vec3<f32>, index: u32) -> f32 {
    let a = params.critter.pairs[index * 2u];
    let b = params.critter.pairs[index * 2u + 1u];
    return capsule_distance(p, a.xyz, b.xyz, (a.w + b.w) * 0.5);
}

fn critter_distance(p: vec3<f32>) -> f32 {
    let count = u32(params.critter.tint_count.w);
    var distance = one_capsule(p, 0u);
    let blend = 0.4;
    for (var index = 1u; index < count; index = index + 1u) {
        let next = one_capsule(p, index);
        let amount = clamp(0.5 + 0.5 * (distance - next) / blend, 0.0, 1.0);
        distance = mix(distance, next, amount) - blend * amount * (1.0 - amount);
    }
    return distance;
}

fn eye_distance(p: vec3<f32>) -> f32 {
    let left = length(p - params.critter.eyes[0].xyz) - params.critter.eyes[0].w;
    let right = length(p - params.critter.eyes[1].xyz) - params.critter.eyes[1].w;
    return min(left, right);
}

fn critter_normal(p: vec3<f32>) -> vec3<f32> {
    let epsilon = 0.08;
    return normalize(vec3(
        critter_distance(p + vec3(epsilon, 0.0, 0.0)) - critter_distance(p - vec3(epsilon, 0.0, 0.0)),
        critter_distance(p + vec3(0.0, epsilon, 0.0)) - critter_distance(p - vec3(0.0, epsilon, 0.0)),
        critter_distance(p + vec3(0.0, 0.0, epsilon)) - critter_distance(p - vec3(0.0, 0.0, epsilon)),
    ));
}

fn trace_critter(eye: vec3<f32>, direction: vec3<f32>) -> f32 {
    if (params.critter.tint_count.w < 0.5) {
        return -1.0;
    }
    let offset = eye - params.critter.bounds.xyz;
    let projection = dot(offset, direction);
    let radius = dot(offset, offset) - params.critter.bounds.w * params.critter.bounds.w;
    if (projection > 0.0 && radius > 0.0) {
        return -1.0;
    }
    let discriminant = projection * projection - radius;
    if (discriminant < 0.0) {
        return -1.0;
    }
    var travel = max(-projection - sqrt(discriminant), 0.05);
    let exit = -projection + sqrt(discriminant);
    for (var step = 0; step < 48; step = step + 1) {
        let distance = critter_distance(eye + direction * travel);
        if (distance < 0.02) {
            return travel;
        }
        travel = travel + max(distance, 0.015);
        if (travel > exit) {
            break;
        }
    }
    return -1.0;
}

fn bayer(pixel: vec2<u32>) -> f32 {
    var values = array<f32, 16>(
        0.0, 8.0, 2.0, 10.0,
        12.0, 4.0, 14.0, 6.0,
        3.0, 11.0, 1.0, 9.0,
        15.0, 7.0, 13.0, 5.0,
    );
    return values[(pixel.y % 4u) * 4u + (pixel.x % 4u)] / 16.0 - 0.5;
}

fn grade(colour: vec3<f32>, t: f32, pixel: vec2<u32>) -> vec3<f32> {
    var fog = clamp((t / params.far - params.fog.w) / max(1.0 - params.fog.w, 0.001), 0.0, 1.0);
    if (params.look.y > 0.5) {
        fog = floor(fog * params.look.y) / params.look.y;
    }
    var out = mix(colour, params.fog.xyz, fog);
    if (params.look.z > 0.5) {
        out = floor(clamp(out + vec3(bayer(pixel) * params.look.x), vec3(0.0), vec3(1.0)) * 5.0) / 4.0;
    }
    return out;
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
    let half_fov = params.fov * 0.5;
    let ray_yaw = params.yaw + in.ndc.x * half_fov;
    let ray_pitch = params.pitch + in.ndc.y * half_fov * 0.5625;
    let direction = vec3(
        sin(ray_yaw) * cos(ray_pitch),
        sin(ray_pitch),
        cos(ray_yaw) * cos(ray_pitch),
    );
    let hit = dda(params.eye, direction);
    let pixel = vec2<u32>(u32(in.pos.x), u32(in.pos.y));
    let body_t = trace_critter(params.eye, direction);
    if (body_t > 0.0 && (!hit.found || body_t < hit.t)) {
        let point = params.eye + direction * body_t;
        let normal = critter_normal(point);
        let sun = normalize(vec3(0.4, 0.8, 0.3));
        let light = clamp(dot(normal, sun), 0.0, 1.0);
        let rim = pow(1.0 - clamp(dot(normal, -direction), 0.0, 1.0), 2.0) * 0.35;
        var base = params.critter.tint_count.xyz;
        if (eye_distance(point) < 0.05) {
            base = base * 0.12;
        }
        let lit = base * (0.4 + 0.6 * light) + vec3(rim * 0.6);
        return vec4(grade(lit, body_t, pixel), 1.0);
    }
    if (!hit.found) {
        let sky = mix(vec3(0.65, 0.72, 0.80), vec3(0.35, 0.45, 0.62), clamp(ray_pitch * 3.0 + 0.3, 0.0, 1.0));
        return vec4(grade(sky, params.far, pixel), 1.0);
    }
    let sun = normalize(vec3(0.4, 0.8, 0.3));
    let light = 0.38 + 0.62 * max(0.0, dot(hit.normal, sun));
    return vec4(grade(material_colour(hit.material) * light, hit.t, pixel), 1.0);
}
