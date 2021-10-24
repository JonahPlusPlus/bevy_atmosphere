use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::pipeline::{PipelineDescriptor, RenderPipeline};
use bevy::render::render_graph::{base, RenderGraph, AssetRenderResourcesNode};
use bevy::render::shader::{ShaderStage, ShaderStages};
use bevy::render::renderer::RenderResources;

const SKY_VERTEX_SHADER: &str = include_str!("shaders/sky.vert");
const SKY_FRAGMENT_SHADER: &str = include_str!("shaders/sky.frag");

/// Controls the appearance of the sky
///
/// Due to constraints on the shader, namely the number of uniforms in a set being capped off at 8, some fields were combined, therefore, functions are provided to set individual fields
#[derive(Debug, RenderResources, TypeUuid, Clone)]
#[uuid = "a57878c4-569e-4511-be7c-b0e5b2c983e2"]
pub struct AtmosphereMat {
    /// Default: (0.0, 6372e3, 0.0)
    pub ray_origin: Vec3,
    /// Default: (0.0, 0.0, 1.0)
    pub sun_position: Vec3,
    /// Default: 22.0
    pub sun_intensity: f32,
    /// Represents Planet radius (Default: 6371e3) and Atmosphere radius (Default: 6471e3)
    pub radius: Vec2,
    /// Represents Rayleigh coefficient (Default: (5.5e-6, 13.0e-6, 22.4e-6)) and scale height (Default: 8e3)
    pub rayleigh: Vec4,
    /// Represents Mie coefficient (Default: 21e-6), scale height (Default: 1.2e3) and preferred scattering direction (Default: 0.758)
    pub mie: Vec3
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
        self.radius.x = planet_radius;
    }

    /// Sets the atmosphere's radius (in meters)
    pub fn set_atmosphere_radius(&mut self, atmosphere_radius: f32) {
        self.radius.y = atmosphere_radius;
    }

    /// Sets the Rayleigh scattering coefficient
    pub fn set_rayleigh_scattering_coefficient(&mut self, coefficient: Vec3) {
        self.rayleigh.x = coefficient.x.clone();
        self.rayleigh.y = coefficient.y.clone();
        self.rayleigh.z = coefficient.z.clone();
    }

    /// Sets the scale height (in meters) for Rayleigh scattering
    pub fn set_rayleigh_scale_height(&mut self, scale: f32) {
        self.rayleigh.w = scale;
    }

    /// Sets the Mie scattering coefficient
    pub fn set_mie_scattering_coefficient(&mut self, coefficient: f32) {
        self.mie.x = coefficient;
    }

    /// Sets the scale height (in meters) for Mie scattering
    pub fn set_mie_scale_height(&mut self, scale: f32) {
        self.mie.y = scale;
    }

    /// Sets the preferred direction for Mie scattering
    pub fn set_mie_scattering_direction(&mut self, direction: f32) {
        self.mie.z = direction;
    }
}

impl Default for AtmosphereMat {
    fn default() -> Self {
        Self {
            ray_origin: Vec3::new(0.0, 6372e3, 0.0),
            sun_position: Vec3::new(0.0, 0.0, 1.0),
            sun_intensity: 22.0,
            radius: Vec2::new(6371e3, 6471e3),
            rayleigh: Vec4::new(5.5e-6, 13.0e-6, 22.4e-6, 8e3),
            mie: Vec3::new(21e-6, 1.2e3, 0.758),
        }
    }
}

impl AtmosphereMat {
    pub fn pipeline(
        mut pipelines: ResMut<Assets<PipelineDescriptor>>,
        mut shaders: ResMut<Assets<Shader>>,
        mut render_graph: ResMut<RenderGraph>
    ) -> RenderPipelines {
        let mut descriptor = PipelineDescriptor::default_config(ShaderStages {
            vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, SKY_VERTEX_SHADER)),
            fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, SKY_FRAGMENT_SHADER)))
        });
        descriptor.depth_stencil = descriptor.depth_stencil.map(|mut depth_stencil_state| {
            depth_stencil_state.depth_compare = bevy::render::pipeline::CompareFunction::LessEqual;
            depth_stencil_state.depth_write_enabled = false;
            depth_stencil_state
        });

        let sky_pipeline_handle = pipelines.add(descriptor);
        render_graph.add_system_node(
            "AtmosphereMat",
            AssetRenderResourcesNode::<AtmosphereMat>::new(true),
        );
        render_graph.add_node_edge("AtmosphereMat", base::node::MAIN_PASS).unwrap();

        let render_pipelines = RenderPipelines::from_pipelines(vec![RenderPipeline::new(sky_pipeline_handle)]);
        render_pipelines
    }
}