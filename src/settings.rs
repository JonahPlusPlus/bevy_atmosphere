//! Provides [`AtmosphereSettings`] resource, a type that controls how the sky is rendered.

use bevy::{prelude::Resource, render::extract_resource::ExtractResource};

/// Available methods for determining the size of the auto created skybox
#[cfg(feature = "detection")]
#[derive(Debug, Clone, Copy)]
pub enum SkyboxCreationMode {
    /// Uses the camera projection `far` value or the specified fallback if `far` is `None`.
    FromProjectionFarWithFallback(f32),
    /// Ignores any camera projection `far` value and always uses the specified value.
    FromSpecifiedFar(f32),
}

#[cfg(feature = "detection")]
impl Default for SkyboxCreationMode {
    fn default() -> Self {
        Self::FromProjectionFarWithFallback(1000.0)
    }
}

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
    /// Method used to determine the size of the auto created skybox.
    ///
    /// See [`SkyboxCreationMode`]
    ///
    /// Default: `FromProjectionFarWithFallback(1000.0)`.
    #[cfg(feature = "detection")]
    pub skybox_creation_mode: SkyboxCreationMode,
}

impl Default for AtmosphereSettings {
    fn default() -> Self {
        Self {
            resolution: 512,
            #[cfg(feature = "dithering")]
            dithering: true,
            #[cfg(feature = "detection")]
            skybox_creation_mode: SkyboxCreationMode::default(),
        }
    }
}
