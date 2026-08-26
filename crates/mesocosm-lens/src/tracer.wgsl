// Mesocosm presentation over conatus-brick's product-neutral DDA.

struct TraceCamera {
    origin: vec3<f32>,
    projection: u32,
    forward: vec3<f32>,
    far: f32,
    right: vec3<f32>,
    _right_pad: f32,
    up: vec3<f32>,
    _up_pad: f32,
};

struct TraceParams {
    camera: TraceCamera,
    space: BrickTraceSpace,
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

@group(0) @binding(2) var<uniform> params: TraceParams;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) ndc: vec2<f32>,
};

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
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
    let exit = min(-projection + sqrt(discriminant), params.camera.far);
    if (travel > exit) {
        return -1.0;
    }
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
    var fog = clamp((t / params.camera.far - params.fog.w) / max(1.0 - params.fog.w, 0.001), 0.0, 1.0);
    if (params.look.y > 0.5) {
        fog = floor(fog * params.look.y) / params.look.y;
    }
    var out = mix(colour, params.fog.xyz, fog);
    if (params.look.z > 0.5) {
        out = floor(clamp(out + vec3(bayer(pixel) * params.look.x), vec3(0.0), vec3(1.0)) * 5.0) / 4.0;
    }
    return out;
}

fn camera_ray(ndc: vec2<f32>) -> Ray {
    if (params.camera.projection == 1u) {
        return Ray(
            params.camera.origin + params.camera.right * ndc.x + params.camera.up * ndc.y,
            params.camera.forward,
        );
    }
    return Ray(
        params.camera.origin,
        normalize(
            params.camera.forward + params.camera.right * ndc.x + params.camera.up * ndc.y,
        ),
    );
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
    let ray = camera_ray(in.ndc);
    let hit = brick_dda(params.space, params.camera.far, ray.origin, ray.direction);
    let pixel = vec2<u32>(u32(in.pos.x), u32(in.pos.y));
    let body_t = trace_critter(ray.origin, ray.direction);
    if (body_t > 0.0 && (!hit.found || body_t < hit.t)) {
        let point = ray.origin + ray.direction * body_t;
        let normal = critter_normal(point);
        let sun = normalize(vec3(0.4, 0.8, 0.3));
        let light = clamp(dot(normal, sun), 0.0, 1.0);
        let rim = pow(1.0 - clamp(dot(normal, -ray.direction), 0.0, 1.0), 2.0) * 0.35;
        var base = params.critter.tint_count.xyz;
        if (eye_distance(point) < 0.05) {
            base = base * 0.12;
        }
        let lit = base * (0.4 + 0.6 * light) + vec3(rim * 0.6);
        return vec4(grade(lit, body_t, pixel), 1.0);
    }
    if (!hit.found) {
        let sky = mix(vec3(0.65, 0.72, 0.80), vec3(0.35, 0.45, 0.62), clamp(ray.direction.y * 3.0 + 0.3, 0.0, 1.0));
        return vec4(grade(sky, params.camera.far, pixel), 1.0);
    }
    let sun = normalize(vec3(0.4, 0.8, 0.3));
    let light = 0.38 + 0.62 * max(0.0, dot(hit.normal, sun));
    return vec4(grade(material_colour(hit.material) * light, hit.t, pixel), 1.0);
}
