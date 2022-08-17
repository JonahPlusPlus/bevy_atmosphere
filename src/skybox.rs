use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{Indices, Mesh, MeshVertexBufferLayout, PrimitiveTopology},
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef,
        },
    },
};

/// The material used by all skyboxes.
pub struct AtmosphereSkyBoxMaterial(pub Handle<SkyBoxMaterial>);

pub const ATMOSPHERE_SKYBOX_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 04511926918914205353);

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "b460ff90-0ee4-42df-875f-0a62ecd1301c"]
pub struct SkyBoxMaterial {
    #[texture(0, dimension = "cube")]
    #[sampler(1)]
    pub sky_texture: Handle<Image>,
}

impl Material for SkyBoxMaterial {
    fn fragment_shader() -> ShaderRef {
        ATMOSPHERE_SKYBOX_SHADER_HANDLE.typed().into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        #[cfg(feature = "dither")]
        if let Some(fragment) = &mut descriptor.fragment {
            fragment.shader_defs.push(String::from("DITHER"));
        }
        Ok(())
    }
}

/// Generates an inverted box mesh with face UVs that fit inside a `pipeline::SIZE` square with a 1 pixel border.
pub fn mesh(far: f32) -> Mesh {
    let size = (far * f32::sqrt(0.5)) - 1.0; // sqrt(0.5) is the ratio between squares separated by a circle
                                             // where one lies on the outside of the circle (edges) and the other lies on the inside of the circle (corners)
                                             // this is necessary since while the faces of the skybox may be seen, the corners and edges probably won't, since they don't lie on the radius of the far plane
    let norm = f32::sqrt(1. / 3.); // component of normalized (1, 1, 1)
    let (vertices, indices) = (
        &[
            ([size, size, size], [norm, norm, norm]),       // 0(+, +, +)
            ([-size, size, size], [-norm, norm, norm]),     // 1(-, +, +)
            ([size, -size, size], [norm, -norm, norm]),     // 2(+, -, +)
            ([size, size, -size], [norm, norm, -norm]),     // 3(+, +, -)
            ([-size, -size, size], [-norm, -norm, norm]),   // 4(-, -, +)
            ([size, -size, -size], [norm, -norm, -norm]),   // 5(+, -, -)
            ([-size, size, -size], [-norm, norm, -norm]),   // 6(-, +, -)
            ([-size, -size, -size], [-norm, -norm, -norm]), // 7(-, -, -)
        ],
        &[
            0, 5, 2, 5, 0, 3, // +X
            6, 4, 7, 4, 6, 1, // -X
            0, 6, 3, 6, 0, 1, // +Y
            2, 7, 4, 7, 2, 5, // -Y
            1, 2, 4, 2, 1, 0, // +Z
            3, 7, 5, 7, 3, 6, // -Z
        ],
    );

    let positions: Vec<_> = vertices.iter().map(|(p, _)| *p).collect();
    let normals: Vec<_> = vertices.iter().map(|(_, n)| *n).collect();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_indices(Some(Indices::U16(indices.to_vec())));
    mesh
}
