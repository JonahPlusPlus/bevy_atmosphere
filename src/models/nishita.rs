use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;
use bevy_atmosphere_macros::AtmosphereModel;

/// Nishita sky model.
#[derive(AtmosphereModel, ShaderType, Reflect, Debug, Clone)]
#[uniform(0, Nishita)]
#[internal("shaders/nishita.wgsl")]
pub struct Nishita {
    /// Ray Origin (Default: `(0.0, 6372e3, 0.0)`).
    ///
    /// Controls orientation of the sky and height of the sun.
    /// It can be thought of as the up-axis and values should be somewhere between planet radius and atmosphere radius (with a bias towards lower values).
    /// When used with `planet_radius` and `atmosphere_radius`, it can be used to change sky brightness and falloff
    pub ray_origin: Vec3,
    /// Sun Position (Default: `(1.0, 1.0, 1.0)`).
    ///
    /// Controls position of the sun in the sky.
    /// Scale doesn't matter, as it will be normalized.
    pub sun_position: Vec3,
    /// Sun Intensity (Default: `22.0`).
    ///
    /// Controls how intense the sun's brightness is.
    pub sun_intensity: f32,
    /// Planet Radius (Default: `6371e3`).
    ///
    /// Controls the radius of the planet.
    /// Heavily interdependent with `atmosphere_radius`
    pub planet_radius: f32,
    /// Atmosphere Radius (Default: `6471e3`).
    ///
    /// Controls the radius of the atmosphere.
    /// Heavily interdependent with `planet_radius`.
    pub atmosphere_radius: f32,
    /// Rayleigh Scattering Coefficient (Default: `(5.5e-6, 13.0e-6, 22.4e-6)`).
    ///
    /// Strongly influences the color of the sky.
    pub rayleigh_coefficient: Vec3,
    /// Rayleigh Scattering Scale Height (Default: `8e3`).
    ///
    /// Controls the amount of Rayleigh scattering.
    pub rayleigh_scale_height: f32,
    /// Mie Scattering Coefficient (Default: `21e-6`).
    ///
    /// Strongly influences the color of the horizon.
    pub mie_coefficient: f32,
    /// Mie Scattering Scale Height (Default: `1.2e3`).
    ///
    /// Controls the amount of Mie scattering.
    pub mie_scale_height: f32,
    /// Mie Scattering Preferred Direction (Default: `0.758`).
    ///
    /// Controls the general direction of Mie scattering.
    pub mie_direction: f32,
}

impl Default for Nishita {
    fn default() -> Self {
        Self {
            ray_origin: Vec3::new(0.0, 6372e3, 0.0),
            sun_position: Vec3::new(1.0, 1.0, 1.0),
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

impl From<&Nishita> for Nishita {
    fn from(nishita: &Nishita) -> Self {
        nishita.clone()
    }
}
