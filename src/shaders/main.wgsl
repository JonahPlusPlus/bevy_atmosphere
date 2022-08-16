#import bevy_atmosphere::types
#import bevy_atmosphere::math

@group(0) @binding(0)
var<uniform> atmosphere: Atmosphere;

@group(1) @binding(0)
var image: texture_storage_2d_array<rgba16float, write>;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) invocation_id: vec3<u32>, @builtin(num_workgroups) num_workgroups: vec3<u32>) {
    let size = textureDimensions(image).x;
    let scale = f32(size)/2f;
    
    let dir = vec2<f32>((f32(invocation_id.x)/scale) - 1f, (f32(invocation_id.y)/scale) - 1f);

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
        default {
            ray = vec3<f32>(-dir.x, -dir.y, -1f);// -Z
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

    textureStore(
        image,
        vec2<i32>(invocation_id.xy),
        i32(invocation_id.z),
        vec4<f32>(render, 1.0)
    );
}
