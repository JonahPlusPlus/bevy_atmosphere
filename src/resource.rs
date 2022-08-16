use bevy::{
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        render_resource::{AsBindGroup, ShaderType},
    },
};

/// Controls the appearance of the sky
#[derive(AsBindGroup, ShaderType, ExtractResource, Debug, Clone, Copy)]
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

impl From<&Atmosphere> for Atmosphere {
    fn from(atmosphere: &Atmosphere) -> Self {
        *atmosphere
    }
}
