//! Provides the [`Atmosphere`] resource, a type that controls the appearance of the sky.

use bevy::{
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        render_asset::RenderAssets,
        render_resource::{AsBindGroupError, BindGroup, BindGroupLayout, PreparedBindGroup},
        renderer::RenderDevice,
        texture::FallbackImage,
    },
};

use crate::model::AtmosphereModel;

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

    pub fn to<T: AtmosphereModel>(&self) -> Option<&T> {
        AtmosphereModel::as_reflect(&*self.0).downcast_ref()
    }

    pub fn to_mut<T: AtmosphereModel>(&mut self) -> Option<&mut T> {
        AtmosphereModel::as_reflect_mut(&mut *self.0).downcast_mut()
    }
}

impl std::ops::Deref for Atmosphere {
    type Target = Box<dyn AtmosphereModel>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Atmosphere {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "nishita")] {
        impl Default for Atmosphere {
            fn default() -> Self {
                use crate::model::nishita::Nishita;
                Self(Box::new(Nishita::default()))
            }
        }
    } else {
        impl Default for Atmosphere {
            fn default() -> Self {
                panic!("Enable at least one atmospheric model!");
            }
        }
    }
}
