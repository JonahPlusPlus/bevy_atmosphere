use crate::model::Atmospheric;
use bevy::{prelude::*, render::render_resource::*};

// #ifdef LUMEN_ALPHA
// let weighted = dot(render, vec3<f32>(0.2126, 0.7152, 0.0722));
// let luminosity = 1.0 - exp(-(PI * (render.x + render.y + render.z)));
// let pixel = vec4<f32>(render, luminosity);
// #else
// let pixel = vec4<f32>(render, 1.0);
// #endif

/// The Nishita sky model, with Applesky20 augmentation for ozone.
///
/// An atmospheric model that uses Ozone absorption and Rayleigh/Mie scattering to simulate a realistic sky.
#[derive(AsBindGroup, Atmospheric, ShaderType, Reflect, Debug, Clone)]
#[uniform(0, NishitaPrecompute)]
#[precompute()]
#[internal("shaders/nishita_precompute.wgsl")]
pub struct NishitaPrecompute {
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
    /// Ozone Absorbtion Coefficient (Default: `(7.0e-7, 15.0e-7, 3.2e-7)`).
    ///
    /// Strongly influences the color of the sky during twilight.
    pub ozone_coefficient: Vec3,
    /// Rayleigh Scattering Coefficient (Default: `(5.5e-6, 13.0e-6, 22.4e-6)`).
    ///
    /// Strongly influences the color of the sky.
    pub rayleigh_coefficient: Vec3,
    /// Rayleigh Scattering Scale Height (Default: `8e3`).
    ///
    /// Controls the amount of Rayleigh scattering.
    /// Reference altitude for the type of particle (dry air) in meters that rayleigh scattering can occur
    pub rayleigh_scale_height: f32,
    /// Mie Scattering Coefficient (Default: `21e-6`).
    ///
    /// Strongly influences the color of the horizon.
    pub mie_coefficient: f32,
    /// Mie Scattering Scale Height (Default: `1.2e3`).
    ///
    /// Controls the amount of Mie scattering.
    /// Reference altitude for the type of particle (dust) in meters that mie scattering can occur
    pub mie_scale_height: f32,
    /// Mie Scattering Preferred Direction (Default: `0.758`).
    ///
    /// Controls the general direction of Mie scattering.
    pub mie_direction: f32,
}

impl From<&NishitaPrecompute> for NishitaPrecompute {
    fn from(other: &NishitaPrecompute) -> Self {
        other.clone()
    }
}
