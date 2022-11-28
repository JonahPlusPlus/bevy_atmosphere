use std::any::Any;

use bevy::prelude::*;
use bevy::reflect::GetTypeRegistration;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{BindGroup, BindGroupLayout, CachedComputePipelineId};
use bevy::render::renderer::RenderDevice;
use bevy::render::texture::FallbackImage;

#[cfg(feature = "nishita")]
pub mod nishita;

pub trait AtmosphereModel: Send + Sync + std::fmt::Debug + Reflect + Any + 'static {
    fn as_bind_group(
        &self,
        layout: &BindGroupLayout,
        render_device: &RenderDevice,
        images: &RenderAssets<Image>,
        fallback_image: &FallbackImage,
    ) -> BindGroup;

    fn clone_dynamic(&self) -> Box<dyn AtmosphereModel>;

    fn as_reflect(&self) -> &dyn Reflect;

    fn as_reflect_mut(&mut self) -> &mut dyn Reflect;
}

impl Clone for Box<dyn AtmosphereModel> {
    fn clone(&self) -> Self {
        self.clone_dynamic()
    }
}

#[derive(Clone)]
pub struct AtmosphereModelMetadata {
    pub bind_group_layout: BindGroupLayout,
    pub pipeline: CachedComputePipelineId,
}

pub trait RegisterAtmosphereModel: GetTypeRegistration {
    fn register(app: &mut App);

    fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout;
}

pub trait AddAtmosphereModel {
    fn add_atmosphere_model<T: RegisterAtmosphereModel>(&mut self) -> &mut App;
}

impl AddAtmosphereModel for App {
    fn add_atmosphere_model<T: RegisterAtmosphereModel>(&mut self) -> &mut App {
        T::register(self);

        self
    }
}