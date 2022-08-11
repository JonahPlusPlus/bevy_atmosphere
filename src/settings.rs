use std::ops::RangeInclusive;

use bevy::render::extract_resource::ExtractResource;

#[derive(ExtractResource, Clone)]
pub struct AtmosphereSettings {
    /// Priority to render atmosphere (Default: -1000)
    pub priority: isize,
    /// Component of the resolution of a skybox face (8 * size = resolution) (Default: 64, or a resolution of 512)
    pub size: u32,
    /// Range of `RenderLayer`s in which cameras can render skyboxes (Default: (2..=2))
    pub skybox_layers: RangeInclusive<u8>,
}

impl Default for AtmosphereSettings {
    fn default() -> Self {
        Self {
            priority: -1000,
            size: 64,
            skybox_layers: (2..=2),
        }
    }
}
