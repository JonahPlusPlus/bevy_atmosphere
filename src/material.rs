use bevy::ecs::system::lifetimeless::SRes;
use bevy::ecs::system::SystemParamItem;
use bevy::pbr::{MaterialPipeline, SpecializedMaterial};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_asset::{PrepareAssetError, RenderAsset};
use bevy::render::render_resource::std140::{AsStd140, Std140};
use bevy::render::render_resource::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferInitDescriptor, BufferSize,
    BufferUsages, CompareFunction, RenderPipelineDescriptor, ShaderStages,
};
use bevy::render::renderer::RenderDevice;

pub const SKY_VERTEX_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 17795653402514319180);
pub const SKY_FRAGMENT_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 13775252721647315361);

/// Controls the appearance of the sky
///
#[derive(Debug, TypeUuid, Clone, AsStd140)]
#[uuid = "a57878c4-569e-4511-be7c-b0e5b2c983e2"]
pub struct AtmosphereMat {
    /// Default: (0.0, 6372e3, 0.0)
    pub ray_origin: Vec3,
    /// Default: (0.0, 1.0, 1.0)
    pub sun_position: Vec3,
    /// Default: 22.0
    pub sun_intensity: f32,
    /// Represents Planet radius (Default: 6371e3) and Atmosphere radius (Default: 6471e3)
    pub planet_radius: f32,
    pub atmosphere_radius: f32,
    /// Represents Rayleigh coefficient (Default: (5.5e-6, 13.0e-6, 22.4e-6)) and scale height (Default: 8e3)
    pub rayleigh_coefficient: Vec3,
    pub rayleigh_scale_height: f32,
    /// Represents Mie coefficient (Default: 21e-6), scale height (Default: 1.2e3) and preferred scattering direction (Default: 0.758)
    pub mie_coefficient: f32,
    pub mie_scale_height: f32,
    pub mie_direction: f32,
}

#[allow(dead_code)]
impl AtmosphereMat {
    /// Sets the ray origin
    pub fn set_ray_origin(&mut self, ray_origin: Vec3) {
        self.ray_origin = ray_origin;
    }

    /// Sets the sun's position
    pub fn set_sun_position(&mut self, sun_position: Vec3) {
        self.sun_position = sun_position;
    }

    /// Sets the sun's intensity (brightness)
    pub fn set_sun_intensity(&mut self, sun_intensity: f32) {
        self.sun_intensity = sun_intensity;
    }

    /// Sets the planet's radius (in meters)
    pub fn set_planet_radius(&mut self, planet_radius: f32) {
        self.planet_radius = planet_radius;
    }

    /// Sets the atmosphere's radius (in meters)
    pub fn set_atmosphere_radius(&mut self, atmosphere_radius: f32) {
        self.atmosphere_radius = atmosphere_radius;
    }

    /// Sets the Rayleigh scattering coefficient
    pub fn set_rayleigh_scattering_coefficient(&mut self, coefficient: Vec3) {
        self.rayleigh_coefficient.x = coefficient.x;
        self.rayleigh_coefficient.y = coefficient.y;
        self.rayleigh_coefficient.z = coefficient.z;
    }

    /// Sets the scale height (in meters) for Rayleigh scattering
    pub fn set_rayleigh_scale_height(&mut self, scale: f32) {
        self.rayleigh_scale_height = scale;
    }

    /// Sets the Mie scattering coefficient
    pub fn set_mie_scattering_coefficient(&mut self, coefficient: f32) {
        self.mie_coefficient = coefficient;
    }

    /// Sets the scale height (in meters) for Mie scattering
    pub fn set_mie_scale_height(&mut self, scale: f32) {
        self.mie_scale_height = scale;
    }

    /// Sets the preferred direction for Mie scattering
    pub fn set_mie_scattering_direction(&mut self, direction: f32) {
        self.mie_direction = direction;
    }
}

impl Default for AtmosphereMat {
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

#[derive(Clone)]
pub struct GpuAtmosphereMat {
    _buffer: Buffer,
    bind_group: BindGroup,
}

impl RenderAsset for AtmosphereMat {
    type ExtractedAsset = AtmosphereMat;
    type PreparedAsset = GpuAtmosphereMat;
    type Param = (SRes<RenderDevice>, SRes<MaterialPipeline<Self>>);
    fn extract_asset(&self) -> Self {
        self.clone()
    }

    fn prepare_asset(
        extracted_asset: Self,
        (render_device, material_pipeline): &mut SystemParamItem<Self::Param>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self>> {
        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            contents: extracted_asset.as_std140().as_bytes(),
            label: None,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: None,
            layout: &material_pipeline.material_layout,
        });

        Ok(Self::PreparedAsset {
            _buffer: buffer,
            bind_group,
        })
    }
}

impl SpecializedMaterial for AtmosphereMat {
    type Key = ();

    fn key(_: &<AtmosphereMat as RenderAsset>::PreparedAsset) -> Self::Key {}

    fn specialize(_: Self::Key, descriptor: &mut RenderPipelineDescriptor) {
        descriptor.vertex.entry_point = "main".into();
        descriptor.fragment.as_mut().unwrap().entry_point = "main".into();
        if let Some(depth_stencil_state) = &mut descriptor.depth_stencil {
            depth_stencil_state.depth_compare = CompareFunction::GreaterEqual;
            depth_stencil_state.depth_write_enabled = false;
        }
    }

    fn vertex_shader(_: &AssetServer) -> Option<Handle<Shader>> {
        Some(SKY_VERTEX_SHADER_HANDLE.typed())
    }

    fn fragment_shader(_: &AssetServer) -> Option<Handle<Shader>> {
        Some(SKY_FRAGMENT_SHADER_HANDLE.typed())
    }

    fn bind_group(render_asset: &<Self as RenderAsset>::PreparedAsset) -> &BindGroup {
        &render_asset.bind_group
    }

    fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new(AtmosphereMat::std140_size_static() as u64),
                },
                count: None,
            }],
            label: None,
        })
    }
}
