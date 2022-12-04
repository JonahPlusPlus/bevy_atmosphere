# Technical Docs

There have been questions on how this plugin actually work, so this document goes over that!

## At a glance

It's easier to understand how bevy_atmosphere got here if you understand what it started as.

Before, this plugin was just a fragment shader on an inverted cube. It's still that, but with a few other things to make it optimized.

A realistic atmosphere shader is actually pretty slow to run, and the parameters don't get changed that often in most games.
So, rather than render it every frame, it's better to render it once and cache the result as a texture, then render that texture instead.
This also allows us to use the same texture for multiple local players, making it very scalable.

## Rendering the Texture

First, we need some way to render the shader to a texture.
A naive approach would be to render a plane to a texture target (admittingly one of my first ideas).
Instead, the correct approach is to employ the use of a compute shader, which can write directly to the texture.

Of course, there are some caveats with using compute shaders.
First, WebGL goes out the window. But I think it's better to focus on optimizing for native targets, than to let some browser standard that will eventually be obsoleted get in the way (the upcoming WebGPU standard will support compute shaders).

Second, you can't write to cube textures (you can't do this with fragment shaders either, but you don't have to if you use them; you just render the model directly).
Cube textures are neat because they allow for proper interpolation of a six-sided texture (the texture would usually bleed through the wrong sides, since it is really just a 2D texture underneath).
The solution here is to create two texture "views": one as a cube texture and another as a 2D array texture.
The skybox material can then use the cube texture view and the compute shader can use the 2D array texture view.

Now, our compute shader can write to a texture, but how does it know which side is which?

There are two things to know:

First, a compute shader has access to some built-in information.
There is a particular piece of information called the `global_invocation_id` (in WGSL).
When you call a compute shader, you also pass the number of "workgroups" to it.
There is a hierarchy of work; from finest to coarsest, this is: `invocation -> subgroup -> local workgroup -> global workgroup`.
The workgroup size and number determine how many "invocations" there are; both are 3D vectors.

For instance, if we were rendering a 512 by 512 texture, we could use a workgroup size of (8, 8, 1) and run (64, 64, 1) workgroups.
Then we would use the `global_invocation_id.xy` to save a pixel to the texture.

Second, the cube map texture is laid out as a 2D texture, with each side ordered on the y-axis.
[The OpenGL wiki actually has a useful page on this.](https://www.khronos.org/opengl/wiki/Cubemap_Texture)

So, the solution is to set the workgroup size to (8, 8, 1) and call (resolution/8, resolution/8, 6) workgroups.
Inside the shader, we can then switch on `global_invocation_id.z` to determine the face and use it to calculate a `ray` value that represents the direction of the pixel.

For reference, here is that code:
```wgsl
let size = textureDimensions(image).x;
let scale = f32(size)/2f;

let dir = vec2<f32>((f32(global_invocation_id.x)/scale) - 1f, (f32(global_invocation_id.y)/scale) - 1f);

var ray: vec3<f32>;

switch global_invocation_id.z {
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
```

We can then use that `ray` value to calculate the color in some direction, then store it to the texture like so:

```wgsl
textureStore(
    image,
    vec2<i32>(global_invocation_id.xy),
    i32(global_invocation_id.z),
    vec4<f32>(render, 1.0)
);
```

(The parameters of `textureStore` are `(texture, location, array_index, color)`)

## The Compute Pipeline

So, we have a shader for rendering this texture, but to actually use it we need a compute pipeline.

In Bevy, this means creating a render graph node and inserting it into the render graph inside the render app.
We also have to create the necessary systems for preparing data for the node to use.

However, creating this pipeline also gives us the opportunity to make some optimizations.
For instance, we don't need to dispatch this shader every frame, so we can track information about user-accessable parameters, like the model parameters and pipeline settings and only dispatch the shader when these change.
This requires extracting and processing resources from the main world into the render world.

## The Skybox

The final step is to display this texture.

The skybox is simply an inverted cube, generated to fit into the camera's far plane.
The skybox's material uses a shader that renders the cube texture and dithers the result to get rid of color banding (Bevy's screen dithering has a limited effect in helping), though that can be disabled through the settings or through a feature gate.

If you use the `AtmosphereCamera` component, the skybox is made an child of the entity and is locally rotated by a system to get rid of global rotation caused by the parent.

## Conclusion

And that's basically how the plugin works, at the core of it. If you have more questions, just make an issue. I'll probably create an FAQ if I get enough.
