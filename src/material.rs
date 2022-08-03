use bevy::{prelude::*, reflect::TypeUuid, render::render_resource::{AsBindGroup, ShaderRef, CompareFunction, ShaderType}};

pub const ATMOSPHERE_MAIN_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 05132991701789555342);
pub const ATMOSPHERE_MATH_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 07843425155352921761);
pub const ATMOSPHERE_TYPES_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 09615256157423613453);

/// Controls the appearance of the sky
#[derive(ShaderType, AsBindGroup, Debug, TypeUuid, Clone, Copy)]
#[uuid = "a57878c4-569e-4511-be7c-b0e5b2c983e2"]
#[uniform(0, Atmosphere)]
pub struct Atmosphere {
    /// Ray Origin (Default: (0.0, 6372e3, 0.0))
    pub ray_origin: Vec3,
    /// Sun Position (Default: (0.0, 1.0, 1.0))
    pub sun_position: Vec3,
    /// Sun Intensity (Default: 22.0)
    pub sun_intensity: f32,
    /// Planet Radius (Default: 6371e3)
    pub planet_radius: f32,
    /// Atmosphere Radius (Default: 6471e3)
    pub atmosphere_radius: f32,
    /// Rayleigh Scattering Coefficient (Default: (5.5e-6, 13.0e-6, 22.4e-6))
    pub rayleigh_coefficient: Vec3,
    /// Rayleigh Scattering Scale Height (Default: 8e3)
    pub rayleigh_scale_height: f32,
    /// Mie Scattering Coefficient (Default: 21e-6)
    pub mie_coefficient: f32,
    /// Mie Scattering Scale Height (Default: 1.2e3)
    pub mie_scale_height: f32,
    /// Mie Scattering Preferred Direction (Default: 0.758)
    pub mie_direction: f32,
}

impl From<&Atmosphere> for Atmosphere {
    fn from(atmosphere: &Atmosphere) -> Self {
        *atmosphere
    }
}

impl Default for Atmosphere {
    fn default() -> Self {
        Self {
            ray_origin: Vec3::new(0.0, 6372e3, 0.0),
            sun_position: Vec3::new(0.0, 1.0, 1.0),
            sun_intensity: 22.0,
            planet_radius: 6371e3,
            atmosphere_radius: 6471e3,
            rayleigh_coefficient: Vec3::new(5.5e-6, 13.0e-6, 22.4e-6),
            rayleigh_scale_height: 8e3,
            mie_coefficient: 21e-6,
            mie_scale_height: 1.2e3,
            mie_direction: 0.758,
        }
    }
}

impl Material for Atmosphere {
    fn fragment_shader() -> ShaderRef {
        ATMOSPHERE_MAIN_SHADER_HANDLE.typed().into()
    }

    fn vertex_shader() -> ShaderRef {
        ATMOSPHERE_MAIN_SHADER_HANDLE.typed().into()
    }

    fn specialize(
            _pipeline: &bevy::pbr::MaterialPipeline<Self>,
            descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
            layout: &bevy::render::mesh::MeshVertexBufferLayout,
            _key: bevy::pbr::MaterialPipelineKey<Self>,
        ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {

            let vertex_layout = layout.get_layout(&[
                Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
                Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
                Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            ])?;

            descriptor.vertex.buffers = vec![vertex_layout];

            if let Some (depth_stencil_state) = &mut descriptor.depth_stencil {
                depth_stencil_state.depth_compare = CompareFunction::GreaterEqual;
                depth_stencil_state.depth_write_enabled = false;
            }
            Ok(())
    }
}
