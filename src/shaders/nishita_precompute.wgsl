// #import bevy_atmosphere::utils // for wgsl-analyzer
#import bevy_atmosphere::utils PI,M_PI_2F,D_PI_2F,D_2_PI,D_1_PI,D_1_6F,quadrature_nodes,quadrature_weights,density_rayleigh,density_mie,density_ozone,rsi,ray_optical_depth,shell_index_to_height,height_to_shell_index

// credit to Blender Authors for referenced source code
//
// SPDX-FileCopyrightText: 2011-2020 Blender Authors
//
// SPDX-License-Identifier: Apache-2.0

struct Precompute {
    planet_radius: f32,
    atmosphere_radius: f32,
    ozone_coefficient: vec3<f32>,
    rayleigh_coefficient: vec3<f32>,
    rayleigh_scale_height: f32,
    mie_coefficient: f32,
    mie_scale_height: f32,
    mie_direction: f32,
}
@group(0) @binding(0)
var<uniform> precompute: Precompute;

// a texture is used to store the precomputed optical depths
// store the light optical depths for each of the shell radii and angle from sun
@group(1) @binding(0)
var optical_depths: texture_storage_2d<rgba32float, write>;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let dims = vec2<f32>(textureDimensions(optical_depths)) - 1.0;

    let height = (precompute.atmosphere_radius - precompute.planet_radius) * shell_index_to_height(f32(invocation_id.x), dims.x);

    let shell_radius = precompute.planet_radius + height;
    let cylinder_angle = (PI * f32(invocation_id.y) / dims.y);

    // we are assuming uniform dome, so the sun direction doesn't matter for the precomputed optical depths
    let sun_direction = vec3<f32>(0.0, 1.0, 0.0);
    let ray_origin = shell_radius * vec3<f32>(0., cos(cylinder_angle), sin(cylinder_angle));

    textureStore(
        optical_depths,
        vec2<i32>(invocation_id.xy),
        vec4<f32>(ray_optical_depth(ray_origin, sun_direction, precompute.planet_radius, precompute.atmosphere_radius, precompute.rayleigh_scale_height, precompute.mie_scale_height), 1.0)
    );
}
