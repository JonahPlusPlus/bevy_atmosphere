// #import bevy_atmosphere::utils // for wgsl-analyzer
#import bevy_atmosphere::utils PI,M_PI_2F,D_PI_2F,D_2_PI,D_1_PI,D_1_6F,rsi

struct Nishita {
    ray_origin: vec3<f32>,
    sun_position: vec3<f32>,
    sun_intensity: f32,
    planet_radius: f32,
    atmosphere_radius: f32,
    rayleigh_coefficient: vec3<f32>,
    rayleigh_scale_height: f32,
    mie_coefficient: f32,
    mie_scale_height: f32,
    mie_direction: f32,
}

const ISTEPS: u32 = 16u;
const JSTEPS: u32 = 8u;

fn render_nishita(r: vec3<f32>, r0: vec3<f32>, p_sun: vec3<f32>, i_sun: f32, r_planet: f32, r_atmos: f32, k_rlh: vec3<f32>, k_mie: f32, sh_rlh: f32, sh_mie: f32, g: f32) -> vec3<f32> {
    // Normalize the ray direction and sun position.
    let r = normalize(r);
    let p_sun = normalize(p_sun);

    // Calculate the step size of the primary ray.
    var p = rsi(r, r0, r_atmos);
    if p.x > p.y { return vec3<f32>(0f); }
    p.y = min(p.y, rsi(r, r0, r_planet).x);
    let i_step_size = (p.y - p.x) / f32(ISTEPS);

    // Initialize the primary ray depth.
    var i_depth = 0.0;

    // Initialize accumulators for Rayleigh and Mie scattering.
    var total_rlh = vec3<f32>(0f);
    var total_mie = vec3<f32>(0f);

    // Initialize optical depth accumulators for the primary ray.
    var i_od_rlh = 0f;
    var i_od_mie = 0f;

    // Calculate the Rayleigh and Mie phases.
    let mu = dot(r, p_sun);
    let mumu = mu * mu;
    let gg = g * g;
    let p_rlh = 3.0 / (16.0 * PI) * (1.0 + mumu);
    let p_mie = 3.0 / (8.0 * PI) * ((1.0 - gg) * (mumu + 1.0)) / (pow(1.0 + gg - 2.0 * mu * g, 1.5) * (2.0 + gg));

    // Sample the primary ray.
    for (var i = 0u; i < ISTEPS; i++) {
        // Calculate the primary ray sample position.
        let i_pos = r0 + r * (i_depth + i_step_size * 0.5);

        // Calculate the height of the sample.
        let i_height = length(i_pos) - r_planet;

        // Calculate the optical depth of the Rayleigh and Mie scattering for this step.
        let od_step_rlh = exp(-i_height / sh_rlh) * i_step_size;
        let od_step_mie = exp(-i_height / sh_mie) * i_step_size;

        // Accumulate optical depth.
        i_od_rlh += od_step_rlh;
        i_od_mie += od_step_mie;

        // Calculate the step size of the secondary ray.
        let j_step_size = rsi(p_sun, i_pos, r_atmos).y / f32(JSTEPS);

        // Initialize the secondary ray depth.
        var j_depth = 0f;

        // Initialize optical depth accumulators for the secondary ray.
        var j_od_rlh = 0f;
        var j_od_mie = 0f;

        // Sample the secondary ray.
        for (var j = 0u; j < JSTEPS; j++) {

            // Calculate the secondary ray sample position.
            let j_pos = i_pos + p_sun * (j_depth + j_step_size * 0.5);

            // Calculate the height of the sample.
            let j_height = length(j_pos) - r_planet;

            // Accumulate the optical depth.
            j_od_rlh += exp(-j_height / sh_rlh) * j_step_size;
            j_od_mie += exp(-j_height / sh_mie) * j_step_size;

            // Increment the secondary ray depth.
            j_depth += j_step_size;
        }

        // Calculate attenuation.
        let attn = exp(-(k_mie * (i_od_mie + j_od_mie) + k_rlh * (i_od_rlh + j_od_rlh)));

        // Accumulate scattering.
        total_rlh += od_step_rlh * attn;
        total_mie += od_step_mie * attn;

        // Increment the primary ray depth.
        i_depth += i_step_size;
    }

    // Calculate and return the final color.
    return i_sun * (p_rlh * k_rlh * total_rlh + p_mie * k_mie * total_mie);
}

@group(0) @binding(0)
var<uniform> nishita: Nishita;

@group(1) @binding(0)
var image: texture_storage_2d_array<rgba16float, write>;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let size = textureDimensions(image).x;
    let scale = f32(size) / 2f;

    let dir = vec2<f32>((f32(invocation_id.x) / scale) - 1f, (f32(invocation_id.y) / scale) - 1f);

    var ray: vec3<f32>;

    switch invocation_id.z {
        case 0u {
            ray = vec3<f32>(1f, -dir.y, -dir.x); // +X
        }
        case 1u {
            ray = vec3<f32>(-1f, -dir.y, dir.x);// -X
        }
        case 2u {
            ray = vec3<f32>(dir.x, 1f, dir.y); // +Y
        }
        case 3u {
            ray = vec3<f32>(dir.x, -1f, -dir.y);// -Y
        }
        case 4u {
            ray = vec3<f32>(dir.x, -dir.y, 1f); // +Z
        }
        default: {
            ray = vec3<f32>(-dir.x, -dir.y, -1f);// -Z
        }
    }

    let render = render_nishita(
        ray,
        nishita.ray_origin,
        nishita.sun_position,
        nishita.sun_intensity,
        nishita.planet_radius,
        nishita.atmosphere_radius,
        nishita.rayleigh_coefficient,
        nishita.mie_coefficient,
        nishita.rayleigh_scale_height,
        nishita.mie_scale_height,
        nishita.mie_direction,
    );

    textureStore(
        image,
        vec2<i32>(invocation_id.xy),
        i32(invocation_id.z),
        vec4<f32>(render, 1.0)
    );
}
