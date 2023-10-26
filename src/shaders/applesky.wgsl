// #import bevy_atmosphere::utils // for wgsl-analyzer
#import bevy_atmosphere::utils PI,M_PI_2F,D_PI_2F,D_2_PI,D_1_PI,D_1_6F,quadrature_nodes,quadrature_weights,density_rayleigh,density_mie,density_ozone,rsi,ray_optical_depth,shell_index_to_height,height_to_shell_index

// notes:
//
// I tried keeping it as close to "nishita.wgsl" as possible
//
// for mie scattering based on atmospheric condition
// g can be g = (5/9)u − ((4/3) − (25/81)uu)x^(−1/3) + x^(1/3)
// x can be x = (5/9)u + (125/729)uuu + ((64/27) - (325/243)uu + (1250/2187)uuuu)^(1/2)
// where u controls the atmospheric condition and can vary from 0.7 to 0.85 (dust), and as a result x can vary from 1.81 to 1.88, g from 0.64 to 0.81,
//
// We could pre-compute the second ("J") optical depths outside of the main render pipeline, since "JSTEP" isn't user controlled and is const.
// I don't know how to set up a pre-compute stage with the current state of this crate, so that's an exercise for another day.

// credit to Blender Authors for referenced source code
//
// SPDX-FileCopyrightText: 2011-2020 Blender Authors
//
// SPDX-License-Identifier: Apache-2.0

struct Applesky {
    ray_origin: vec3<f32>,
    sun_direction: vec3<f32>,
    sun_intensity: vec3<f32>,
    sun_angular_radius: f32,
    planet_radius: f32,
    atmosphere_radius: f32,
    air_density: f32,
    dust_density: f32,
    ozone_density: f32,
    ozone_coefficient: vec3<f32>,
    rayleigh_coefficient: vec3<f32>,
    rayleigh_scale_height: f32,
    mie_coefficient: f32,
    mie_scale_height: f32,
    mie_direction: f32,
}

const ISTEPS: u32 = 16u;
const JSTEPS: u32 = 8u;

@group(0) @binding(0)
var<uniform> applesky: Applesky;

@group(0) @binding(1)
var optical_depths: texture_storage_2d<rgba32float, read>;

fn light_optical_depth(shell_index: f32, cylinder_index: f32) -> vec3<f32> {
    // could just use sampling here, but I heard there's problems with 32Float and wasm/metal
    let index = vec2<f32>(shell_index, cylinder_index);
    let index_floor = floor(index);
    let index_ceil = ceil(index);
    let l = index - index_floor;

    return mix(
        mix(
            textureLoad(optical_depths, vec2<i32>(index_floor)).xyz,
            textureLoad(optical_depths, vec2<i32>(i32(index_floor.x), i32(index_ceil.y))).xyz,
            l.y
        ),
        mix(
            textureLoad(optical_depths, vec2<i32>(i32(index_ceil.x), i32(index_floor.y))).xyz,
            textureLoad(optical_depths, vec2<i32>(index_ceil)).xyz,
            l.y
        ),
        l.x
    );
}

@group(1) @binding(0)
var image: texture_storage_2d_array<rgba16float, write>;

fn render_applesky(ray_dir: vec3<f32>, ray_origin: vec3<f32>, sun_dir: vec3<f32>, sun_intensity: vec3<f32>, sun_angular_radius: f32, planet_radius: f32, atmosphere_radius: f32, air_density: f32, dust_density: f32, ozone_density: f32, ozone_coefficients: vec3<f32>, rayleigh_coefficients: vec3<f32>, mie_coefficient: f32, rayleigh_scale_height: f32, mie_scale_height: f32, g: f32) -> vec3<f32> {
    let scale_density = vec3<f32>(air_density, dust_density, ozone_density);

    // Normalize the ray direction and sun position.
    let ray_dir = normalize(ray_dir);
    let sun_dir = normalize(sun_dir);

    // Calculate the step size of the primary ray.
    let p_atmos = rsi(ray_dir, ray_origin, atmosphere_radius);
    var p = p_atmos;
    if p.x > p.y { return vec3<f32>(0f); }
    let p_planet = rsi(ray_dir, ray_origin, planet_radius);
    p.y = min(p.y, p_planet.x);
    let i_step_size = (p.y - p.x) / f32(ISTEPS);
    let segment = i_step_size * ray_dir;

    // Initialize the primary ray depth.
    var pos = ray_origin + (0.5 * segment);

    // Initialize accumulators for Rayleigh and Mie scattering.
    var total_rayleigh = vec3<f32>(0f);
    var total_mie = vec3<f32>(0f);

    // Initialize optical depth accumulators for the primary ray.
    var i_optical_depth = vec3<f32>(0f);

    // Calculate the Rayleigh and Mie phases.
    let mu = dot(ray_dir, sun_dir);
    let mumu = mu * mu;
    let gg = g * g;
    let rayleigh_phase = (3.0 / (16.0 * PI)) * (1.0 + mumu);
    let mie_phase = (3.0 * (1.0 - gg) * (1.0 + mumu)) / (8.0 * PI * (2.0 + gg) * pow((1.0 + gg - (2.0 * g * mu)), 1.5));

    let dims = vec2<f32>(textureDimensions(optical_depths)) - 1.0;
    let ray_sun_theta = acos(mu);
    let shell_index_coef = 1.0 / (atmosphere_radius - planet_radius);
    let cylinder_index_coef = D_1_PI * dims.y;

    // Sample the primary ray.
    for (var i = 0u; i < ISTEPS; i++) {
        // Calculate the height of the sample.
        let len = length(pos);
        let height = len - planet_radius;

        // Calculate the optical depth of the Rayleigh and Mie scattering for this step.
        let density = scale_density * vec3<f32>(
            density_rayleigh(height, rayleigh_scale_height),
            density_mie(height, mie_scale_height),
            density_ozone(height)
        );

        // Accumulate optical depth.
        i_optical_depth += i_step_size * density;

        // Calculate light scattering point optical depth.
        let j_optical_depth = scale_density * light_optical_depth(
            height_to_shell_index(height * shell_index_coef, dims.x),
            clamp(acos(dot(pos, sun_dir) / len) * cylinder_index_coef, 0f, dims.y)
        );

        // Calculate attenuation.
        let total_optical_depth = i_optical_depth + j_optical_depth;
        let attenuation = exp(-((rayleigh_coefficients * total_optical_depth.x) + (1.11 * mie_coefficient * total_optical_depth.y) + (ozone_coefficients * total_optical_depth.z)));

        // Accumulate scattering.
        total_rayleigh += density.x * attenuation;
        total_mie += density.y * attenuation;

        // Increment the primary ray depth.
        pos += segment;
    }

    // Calculate and return the final color.
    let atmos = i_step_size * ((rayleigh_phase * rayleigh_coefficients * total_rayleigh) + (mie_phase * mie_coefficient * total_mie));

    // If we are outside the sun disk, or we intersect planet, we just return the atmosphere color
    if ray_sun_theta > sun_angular_radius || p_planet.y > 0f {
        return sun_intensity * atmos;
    } else {
        // Else we add transmittance to render a simple sun disk
        let optical_depth = scale_density * ray_optical_depth(ray_origin, ray_dir, planet_radius, atmosphere_radius, rayleigh_scale_height, mie_scale_height);
        let transmittance = exp(-((rayleigh_coefficients * optical_depth.x) + (1.11 * mie_coefficient * optical_depth.y))) / (M_PI_2F * (1f - cos(sun_angular_radius)));
        return sun_intensity * (atmos + transmittance);
    }
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let placeholder = textureDimensions(optical_depths);

    let size = textureDimensions(image).x;
    let scale = (f32(size) - 1f) / 2f;

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

    let render = render_applesky(
        ray,
        applesky.ray_origin,
        applesky.sun_direction,
        applesky.sun_intensity,
        applesky.sun_angular_radius,
        applesky.planet_radius,
        applesky.atmosphere_radius,
        applesky.air_density,
        applesky.dust_density,
        applesky.ozone_density,
        applesky.ozone_coefficient,
        applesky.rayleigh_coefficient,
        applesky.mie_coefficient,
        applesky.rayleigh_scale_height,
        applesky.mie_scale_height,
        applesky.mie_direction,
    );

    textureStore(
        image,
        vec2<i32>(invocation_id.xy),
        i32(invocation_id.z),
        vec4<f32>(render, 1.0)
    );
}
