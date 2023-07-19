
#ifdef DITHER
// From https://alex.vlachos.com/graphics/Alex_Vlachos_Advanced_VR_Rendering_GDC2015.pdf
// and https://www.shadertoy.com/view/MslGR8 (5th one starting from the bottom)
fn dither(frag_coord: vec2<f32>) -> vec3<f32> {
	// Iestyn's RGB dither (7 asm instructions) from Portal 2 X360, slightly modified for VR.
    var dither = vec3<f32>(dot(vec2<f32>(171.0, 231.0), frag_coord));
    dither = fract(dither.rgb / vec3<f32>(103.0, 71.0, 97.0));
	// Subtract 0.5 to avoid slightly brightening the whole viewport.
    return (dither.rgb - 0.5) / 255.0;
}
#endif

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
#ifdef DITHER
    return vec4<f32>(color + dither(position.xy), 1f);
#else
    return vec4<f32>(color, 1f);
#endif
}