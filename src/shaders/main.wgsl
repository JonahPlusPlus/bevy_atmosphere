#import bevy_atmosphere::types
#import bevy_atmosphere::math

@group(0) @binding(0)
var<uniform> atmosphere: Atmosphere;

@group(1) @binding(0)
var image: texture_storage_2d<rgba8unorm, read_write>;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let location = vec2<i32>(i32(invocation_id.x), i32(invocation_id.y));

    let size = i32(num_workgroups.x) * 8;
    let scale = f32(size)/2f;

    //let dither = dither(vec2<f32>(location));

    // X
    let render = render_atmosphere(
        vec3<f32>(1f, 1f - f32(location.x)/scale, 1f - f32(location.y)/scale), 
        atmosphere.ray_origin,
        atmosphere.sun_position,
        atmosphere.sun_intensity,
        atmosphere.planet_radius,
        atmosphere.atmosphere_radius,
        atmosphere.rayleigh_coefficient,
        atmosphere.mie_coefficient,
        atmosphere.rayleigh_scale_height,
        atmosphere.mie_scale_height,
        atmosphere.mie_direction,
    );
    
    textureStore(image, location + vec2<i32>(size * 0 + 1, 1), vec4<f32>(render , 1.0));

    // Y
    let render = render_atmosphere(
        vec3<f32>(f32(location.x)/scale - 1f, 1f, 1f - f32(location.y)/scale), 
        atmosphere.ray_origin,
        atmosphere.sun_position,
        atmosphere.sun_intensity,
        atmosphere.planet_radius,
        atmosphere.atmosphere_radius,
        atmosphere.rayleigh_coefficient,
        atmosphere.mie_coefficient,
        atmosphere.rayleigh_scale_height,
        atmosphere.mie_scale_height,
        atmosphere.mie_direction,
    );

    textureStore(image, location + vec2<i32>(size * 1 + 3, 1), vec4<f32>(render , 1.0));

    // Z
    let render = render_atmosphere(
        vec3<f32>(f32(location.x)/scale - 1f, 1f - f32(location.y)/scale, 1f), 
        atmosphere.ray_origin,
        atmosphere.sun_position,
        atmosphere.sun_intensity,
        atmosphere.planet_radius,
        atmosphere.atmosphere_radius,
        atmosphere.rayleigh_coefficient,
        atmosphere.mie_coefficient,
        atmosphere.rayleigh_scale_height,
        atmosphere.mie_scale_height,
        atmosphere.mie_direction,
    );

    textureStore(image, location + vec2<i32>(size * 2 + 5, 1), vec4<f32>(render , 1.0));

    // -X
    let render = render_atmosphere(
        vec3<f32>(-1f, 1f - f32(location.y)/scale, 1f - f32(location.x)/scale), 
        atmosphere.ray_origin,
        atmosphere.sun_position,
        atmosphere.sun_intensity,
        atmosphere.planet_radius,
        atmosphere.atmosphere_radius,
        atmosphere.rayleigh_coefficient,
        atmosphere.mie_coefficient,
        atmosphere.rayleigh_scale_height,
        atmosphere.mie_scale_height,
        atmosphere.mie_direction,
    );

    textureStore(image, location + vec2<i32>(size * 3 + 7, 1), vec4<f32>(render , 1.0));

    // -Y
    let render = render_atmosphere(
        vec3<f32>(f32(location.y)/scale - 1f, -1f, 1f - f32(location.x)/scale), 
        atmosphere.ray_origin,
        atmosphere.sun_position,
        atmosphere.sun_intensity,
        atmosphere.planet_radius,
        atmosphere.atmosphere_radius,
        atmosphere.rayleigh_coefficient,
        atmosphere.mie_coefficient,
        atmosphere.rayleigh_scale_height,
        atmosphere.mie_scale_height,
        atmosphere.mie_direction,
    );

    textureStore(image, location + vec2<i32>(size * 4 + 9, 1), vec4<f32>(render , 1.0));

    // -Z
    let render = render_atmosphere(
        vec3<f32>(f32(location.y)/scale - 1f, 1f - f32(location.x)/scale, -1f), 
        atmosphere.ray_origin,
        atmosphere.sun_position,
        atmosphere.sun_intensity,
        atmosphere.planet_radius,
        atmosphere.atmosphere_radius,
        atmosphere.rayleigh_coefficient,
        atmosphere.mie_coefficient,
        atmosphere.rayleigh_scale_height,
        atmosphere.mie_scale_height,
        atmosphere.mie_direction,
    );

    textureStore(image, location + vec2<i32>(size * 5 + 11, 1), vec4<f32>(render , 1.0)); // -Z
}
