//! Provides [`AtmosphereSettings`] resource, a type that controls how the sky is rendered.

use bevy::render::extract_resource::ExtractResource;

/// Provides settings for how the sky is rendered.
#[derive(ExtractResource, Debug, Clone, Copy)]
pub struct AtmosphereSettings {
    /// Resolution of a face of a skybox (Default: 512).
    ///
    /// It should be a multiple of 8, any different and there may be issues.
    pub resolution: u32,
}

impl Default for AtmosphereSettings {
    fn default() -> Self {
        Self { resolution: 512 }
    }
}
