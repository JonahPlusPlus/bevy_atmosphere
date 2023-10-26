use crate::model::Atmospheric;
use crate::pipeline::AtmospherePrecomputeImage;
use bevy::prelude::*;
use bevy::render::render_resource::{self, *};

use super::nishita_precompute::NishitaPrecompute;

/// The Nishita sky model, with Applesky and Blender ("Cyclesky") augmentation for ozone.
///
/// An atmospheric model that uses Ozone absorption and Rayleigh/Mie scattering to simulate a realistic sky.
/// Creates a more accurate twilight color than default Nishita model.
#[derive(Atmospheric, Reflect, Debug, Clone)]
#[internal("shaders/applesky.wgsl")]
#[after(NishitaPrecompute)]
pub struct Applesky {
    /// Ray Origin (Default: `(0.0, 6372e3, 0.0)`).
    ///
    /// Controls orientation of the sky and height of the sun.
    /// It can be thought of as the up-axis and values should be somewhere between planet radius and atmosphere radius (with a bias towards lower values).
    /// When used with `planet_radius` and `atmosphere_radius`, it can be used to change sky brightness and falloff
    pub ray_origin: Vec3,
    /// Sun Direction (Default: `(1.0, 1.0, 1.0)`).
    ///
    /// Controls direction of the sun in the sky.
    /// Scale doesn't matter, as it will be normalized.
    pub sun_direction: Vec3,
    /// Sun Intensity (Default: `(1.466, 1.756, 1.715)`).
    ///
    /// The sun's intensity at the top of the atmosphere.
    pub sun_intensity: Vec3,
    /// Sun Angular Radius (Default: 4.675e-3)
    ///
    /// Controls the size of the sun.
    pub sun_angular_radius: f32,
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
    /// Air Density (Default: `1.0`).
    ///
    /// Controls the density of the air.
    pub air_density: f32,
    /// Dust Density (Default: `1.0`).
    ///
    /// Controls the density of dust.
    pub dust_density: f32,
    /// Ozone Density (Default: `1.0`).
    ///
    /// Controls the density of the ozone.
    pub ozone_density: f32,
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
    /// The precomputed data.
    pub precomputed: Handle<Image>,
}

#[derive(ShaderType, Reflect, Debug, Clone)]
pub struct GpuApplesky {
    pub ray_origin: Vec3,
    pub sun_direction: Vec3,
    pub sun_intensity: Vec3,
    pub sun_angular_radius: f32,
    pub planet_radius: f32,
    pub atmosphere_radius: f32,
    pub air_density: f32,
    pub dust_density: f32,
    pub ozone_density: f32,
    pub ozone_coefficient: Vec3,
    pub rayleigh_coefficient: Vec3,
    pub rayleigh_scale_height: f32,
    pub mie_coefficient: f32,
    pub mie_scale_height: f32,
    pub mie_direction: f32,
}

impl AsBindGroup for Applesky {
    type Data = ();

    fn as_bind_group(
        &self,
        layout: &BindGroupLayout,
        render_device: &bevy::render::renderer::RenderDevice,
        images: &bevy::render::render_asset::RenderAssets<Image>,
        _fallback_image: &bevy::render::texture::FallbackImage,
    ) -> Result<PreparedBindGroup<Self::Data>, AsBindGroupError> {
        let bindings = Vec::from([
            OwnedBindingResource::Buffer({
                let mut buffer = render_resource::encase::UniformBuffer::new(Vec::new());
                buffer.write(&GpuApplesky::from(self)).unwrap();
                render_device.create_buffer_with_data(&BufferInitDescriptor {
                    label: Some("applesky_shader_type"),
                    contents: buffer.as_ref(),
                    usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                })
            }),
            OwnedBindingResource::TextureView(
                images
                    .get(&self.precomputed)
                    .ok_or(AsBindGroupError::RetryNextUpdate)?
                    .texture_view
                    .clone(),
            ),
        ]);
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("applesky_bind_group"),
            layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: bindings[0].get_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: bindings[1].get_binding(),
                },
            ],
        });
        Ok(PreparedBindGroup {
            bindings,
            bind_group,
            data: (),
        })
    }

    fn bind_group_layout(render_device: &bevy::render::renderer::RenderDevice) -> BindGroupLayout
    where
        Self: Sized,
    {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("applesky_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(GpuApplesky::min_size()),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadOnly,
                        format: TextureFormat::Rgba32Float,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        })
    }
}

impl FromWorld for Applesky {
    fn from_world(world: &mut World) -> Self {
        Self {
            ray_origin: Vec3::new(0.0, 6372e3, 0.0),
            sun_direction: Vec3::new(1.0, 1.0, 1.0),
            sun_intensity: Vec3::new(1.466, 1.756, 1.715),
            sun_angular_radius: 2.675e-2, // should be smaller, e.g. 4.675e-3, but small sun looks terrible with 512x512 skybox
            planet_radius: 6371e3,
            atmosphere_radius: 6471e3,
            air_density: 1.0,
            dust_density: 1.0,
            ozone_density: 1.0,
            ozone_coefficient: Vec3::new(7.0e-7, 15.0e-7, 3.2e-7),
            rayleigh_coefficient: Vec3::new(5.5e-6, 13.0e-6, 22.4e-6),
            rayleigh_scale_height: 7994.0,
            mie_coefficient: 21e-6,
            mie_scale_height: 1.2e3,
            mie_direction: 0.758,
            precomputed: world
                .get_resource::<AtmospherePrecomputeImage>()
                .expect("Applesky atmosphere model requires AtmospherePrecomputeImage to be added first")
                .handle
                .clone_weak(),
        }
    }
}

impl From<&Applesky> for GpuApplesky {
    fn from(applesky: &Applesky) -> Self {
        Self {
            ray_origin: applesky.ray_origin,
            sun_direction: applesky.sun_direction,
            sun_intensity: applesky.sun_intensity,
            sun_angular_radius: applesky.sun_angular_radius,
            planet_radius: applesky.planet_radius,
            atmosphere_radius: applesky.atmosphere_radius,
            air_density: applesky.air_density,
            dust_density: applesky.dust_density,
            ozone_density: applesky.ozone_density,
            ozone_coefficient: applesky.ozone_coefficient,
            rayleigh_coefficient: applesky.rayleigh_coefficient,
            rayleigh_scale_height: applesky.rayleigh_scale_height,
            mie_coefficient: applesky.mie_coefficient,
            mie_scale_height: applesky.mie_scale_height,
            mie_direction: applesky.mie_direction,
        }
    }
}

impl From<&Applesky> for NishitaPrecompute {
    fn from(applesky: &Applesky) -> Self {
        Self {
            planet_radius: applesky.planet_radius,
            atmosphere_radius: applesky.atmosphere_radius,
            ozone_coefficient: applesky.ozone_coefficient,
            rayleigh_coefficient: applesky.rayleigh_coefficient,
            rayleigh_scale_height: applesky.rayleigh_scale_height,
            mie_coefficient: applesky.mie_coefficient,
            mie_scale_height: applesky.mie_scale_height,
            mie_direction: applesky.mie_direction,
        }
    }
}
