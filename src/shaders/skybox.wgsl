@group(1) @binding(0)
var sky_texture: texture_cube<f32>;
@group(1) @binding(1)
var sky_sampler: sampler;

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
    @builtin(position) position: vec4<f32>
) -> @location(0) vec4<f32> {
    let color = textureSample(sky_texture, sky_sampler, world_normal).xyz;
    return vec4<f32>(color, 1f);
}