use bevy::render::extract_resource::ExtractResource;

#[derive(ExtractResource, Debug, Clone, Copy)]
pub struct AtmosphereSettings {
    pub resolution: u32,
}

impl Default for AtmosphereSettings {
    fn default() -> Self {
        Self { resolution: 1024 }
    }
}
