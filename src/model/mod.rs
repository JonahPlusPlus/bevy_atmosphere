use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{BindGroupLayout, BindGroup};
use bevy::render::renderer::RenderDevice;
use bevy::render::texture::FallbackImage;

pub mod nishita;

pub trait AtmosphereModel: Send + Sync + std::fmt::Debug {
    fn as_bind_group(
        &self,
        layout: &BindGroupLayout,
        render_device: &RenderDevice,
        images: &RenderAssets<Image>,
        fallback_image: &FallbackImage,
    ) -> BindGroup;

    fn bind_group_layout(&self, render_device: &RenderDevice) -> BindGroupLayout;

    fn clone_dynamic(&self) -> Box<dyn AtmosphereModel>;
}

impl Clone for Box<dyn AtmosphereModel> {
    fn clone(&self) -> Self {
        self.clone_dynamic()
    }
}
