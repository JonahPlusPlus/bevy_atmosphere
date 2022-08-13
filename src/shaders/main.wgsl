#import bevy_atmosphere::types
#import bevy_atmosphere::math

@group(0) @binding(0)
var<uniform> atmosphere: Atmosphere;

@group(1) @binding(0)
var image: texture_storage_2d<rgba8unorm, read_write>;

let SCALE: f32 = 256f;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let location = vec2<i32>(invocation_id.xy);

    let dither = dither(vec2<f32>(invocation_id.xy));
    
    let dir = vec2<f32>(f32(invocation_id.x)/SCALE, f32(invocation_id.y)/SCALE);

    var ray: vec3<f32>;
    
    switch invocation_id.z {
        case 0u {
            ray = vec3<f32>(1f, 1f - dir.x, 1f - dir.y); // +X
        }
        case 1u {
            ray = vec3<f32>(dir.x - 1f, 1f, 1f - dir.y); // +Y
        }
        case 2u {
            ray = vec3<f32>(dir.x - 1f, 1f - dir.y, 1f); // +Z
        }
        case 3u {
            ray = vec3<f32>(-1f, 1f - dir.y, 1f - dir.x);// -X
        }
        case 4u {
            ray = vec3<f32>(dir.y - 1f, -1f, 1f - dir.x);// -Y
        }
        default {
            ray = vec3<f32>(dir.y - 1f, 1f - dir.x, -1f);// -Z
        }
    }

    let render = render_atmosphere(
        ray, 
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

    textureStore(image, location + vec2<i32>(512i * i32(invocation_id.z), 0), vec4<f32>(render + dither, 1.0));
}
