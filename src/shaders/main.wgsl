#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings

// Derived from bevy_pbr::mesh_functions to avoid naga logs about skipped functions
fn mesh_position_local_to_clip(model: mat4x4<f32>, vertex_position: vec4<f32>) -> vec4<f32> {
    let world_position = model * vertex_position;
    return view.view_proj * world_position;
}

#import bevy_atmosphere::types
#import bevy_atmosphere::math

struct Vertex {
    @location(0)
        position: vec3<f32>,
    @location(1)
        normal: vec3<f32>,
    @location(2)
        uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position)
        clip_position: vec4<f32>,
    @location(0)
        ray: vec3<f32>,
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let position = vertex.position;
    var out: VertexOutput;
    out.clip_position = mesh_position_local_to_clip(mesh.model, vec4<f32>(position, 1.0));
    out.ray = position;
    return out;
}

@group(1) @binding(0)
var<uniform> atmosphere: Atmosphere;

struct FragmentOutput {
    @location(0)
        color: vec4<f32>,
}

@fragment
fn fragment(in: VertexOutput) -> FragmentOutput {
    var out: FragmentOutput;
    
    let render = render_atmosphere(
        in.ray, 
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
    
    out.color = vec4<f32>(1f - exp(-1f * render), 1f);

    return out;
}
