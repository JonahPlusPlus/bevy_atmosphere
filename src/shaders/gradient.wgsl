
struct Gradient {
    sky: vec4<f32>,
    horizon: vec4<f32>,
    ground: vec4<f32>,
}

fn render_gradient(r: vec3<f32>, g: Gradient) -> vec3<f32> {
    let r = normalize(r);
    let y = r.y;

    let p_sky = max(y, 0f);
    let p_horizon = 1f-abs(y);
    let p_ground = max(-y, 0f);

    let color = (g.sky * p_sky) + (g.horizon * p_horizon) + (g.ground * p_ground);

    return color.xyz;
}

@group(0) @binding(0)
var<uniform> gradient: Gradient;

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

    let render = render_gradient(
        ray,
        gradient
    );

    textureStore(
        image,
        vec2<i32>(invocation_id.xy),
        i32(invocation_id.z),
        vec4<f32>(render, 1.0)
    );
}
