//! Provides the [`Atmosphere`] resource, a type that controls the appearance of the sky.

use bevy::{
    prelude::*,
    render::{
        extract_resource::ExtractResource, render_resource::{BindGroup, BindGroupLayout, AsBindGroupError}, renderer::RenderDevice, render_asset::RenderAssets, texture::FallbackImage,
    },
};

use crate::model::{AtmosphereModel, nishita::Nishita};

/// Controls the appearance of the atmosphere.
///
/// How the atmosphere is simulated is based off of Rayleigh and Mie scattering.
///
/// Rayleigh scattering is caused by light passing through particles smaller than the wavelength.
/// It is the cause for the color of the sky and sunset.
///
/// Mie scattering is caused by light passing through particles of similar size to the wavelength.
/// It is the cause for the sky getting lighter toward the horizon.
#[derive(Resource, ExtractResource, Debug, Clone)]
pub struct Atmosphere(Box<dyn AtmosphereModel>);

impl From<&Atmosphere> for Atmosphere {
    fn from(atmosphere: &Atmosphere) -> Self {
        atmosphere.clone()
    }
}

impl Atmosphere {
    pub fn new(model: impl AtmosphereModel + 'static) -> Self {
        Self(Box::new(model))
    }

    pub fn as_bind_group(
        &self,
        layout: &BindGroupLayout,
        render_device: &RenderDevice,
        images: &RenderAssets<Image>,
        fallback_image: &FallbackImage,
    ) -> Result<BindGroup, AsBindGroupError> {
        Ok(self.0.as_bind_group(layout, render_device, images, fallback_image))
    }

    pub fn bind_group_layout(&self, render_device: &RenderDevice) -> BindGroupLayout {
        self.0.bind_group_layout(render_device)
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "nishita")] {
        impl Default for Atmosphere {
            fn default() -> Self {
                Self(Box::new(Nishita::default()))
            }
        }
    } else {
        compile_error!("Enable at least one atmospheric model!")
    }
}
