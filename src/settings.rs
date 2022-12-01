//! Provides [`AtmosphereSettings`] resource, a type that controls how the sky is rendered.

use bevy::{prelude::Resource, render::extract_resource::ExtractResource};

/// Provides settings for how the sky is rendered.
#[derive(Resource, ExtractResource, Debug, Clone, Copy)]
pub struct AtmosphereSettings {
    /// Resolution of a face of a skybox (Default: `512`).
    ///
    /// It should be a multiple of 8, any different and there may be issues.
    pub resolution: u32,
    /// Controls whether or not dithering is applied (Default: `true`).
    ///
    /// Dithering will prevent noticeable color banding in some models.
    ///
    /// This option can be removed by disabling the "dithering" feature
    #[cfg(feature = "dithering")]
    pub dithering: bool,
}

impl Default for AtmosphereSettings {
    fn default() -> Self {
        Self {
            resolution: 512,
            #[cfg(feature = "dithering")]
            dithering: true,
        }
    }
}
