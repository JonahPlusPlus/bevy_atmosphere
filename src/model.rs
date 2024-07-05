//! Provides the [`trait@Atmospheric`] trait and [`AtmosphereModel`] resource.
//!
//! # Examples
//! Most models will look something like the following:
//! ```ignore
//! # use bevy::prelude::*;
//! # use bevy::render::render_resource::ShaderType;
//! # use bevy_atmosphere::prelude::*;
//! #[derive(Atmospheric, ShaderType, Reflect, Debug, Clone)]
//! // This is shorthand for when all fields are uniform.
//! // Use other `AsBindGroup` attributes if this doesn't apply.
//! #[uniform(0, MyModel)]
//! // The `external` attribute loads external assets.
//! // The `internal` attribute is for loading internal assets.
//! #[external("shader.wgsl")]
//! struct MyModel {
//!     color: Color,
//! }
//!
//! // Required `#[uniform(0, MyModel)]`
//! // (if we were using a different GPU representation, this is where we would convert to it)
//! impl From<&MyModel> for MyModel {
//!     fn from(model: &MyModel) -> Self {
//!         model.clone()
//!     }
//! }
//! ```
//!
//! It can then be registered by calling [`AddAtmosphereModel::add_atmosphere_model`].

use std::any::{Any, TypeId};

use bevy::{
    prelude::*,
    reflect::GetTypeRegistration,
    render::{
        extract_resource::ExtractResource,
        render_asset::RenderAssets,
        render_resource::{BindGroup, BindGroupLayout, CachedComputePipelineId},
        renderer::RenderDevice,
        texture::{FallbackImage, GpuImage},
    },
};

/// A derive macro for implementing [`Atmospheric`].
pub use bevy_atmosphere_macros::Atmospheric;

/// A trait for defining atmosphere models.
///
/// Since `AsBindGroup` is not object-safe, this trait and [`RegisterAtmosphereModel`] split it into two, for dynamic and static contexts.
///
/// The recommended way to use `Atmospheric` is to derive it with the [`Atmospheric`](derive@Atmospheric) macro.
pub trait Atmospheric: Send + Sync + Reflect + Any + 'static {
    fn as_bind_group(
        &self,
        layout: &BindGroupLayout,
        render_device: &RenderDevice,
        images: &RenderAssets<GpuImage>,
        fallback_image: &FallbackImage,
    ) -> BindGroup;

    fn clone_dynamic(&self) -> Box<dyn Atmospheric>;

    fn as_reflect(&self) -> &dyn Reflect;

    fn as_reflect_mut(&mut self) -> &mut dyn Reflect;
}

impl Clone for Box<dyn Atmospheric> {
    fn clone(&self) -> Self {
        self.clone_dynamic()
    }
}

/// The `TypeData` for [`Atmospheric`] models.
#[derive(Clone)]
pub struct AtmosphereModelMetadata {
    /// Used to test if the model has changed.
    pub id: TypeId,
    /// Used to create the `BindGroup`.
    pub bind_group_layout: BindGroupLayout,
    /// Used to get the shader's pipeline.
    pub pipeline: CachedComputePipelineId,
}

/// A trait for registering [`AtmosphereModelMetadata`].
pub trait RegisterAtmosphereModel: GetTypeRegistration {
    fn register(app: &mut App);

    fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout;
}

/// A trait for using [`RegisterAtmosphereModel`] from `App`.
///
/// # Examples
/// ```ignore
/// # use bevy::prelude::*;
/// # use bevy_atmosphere::prelude::*;
/// fn main() {
///     App::new()
///         .add_plugins((DefaultPlugins, AtmospherePlugin))
///         .add_atmosphere_model::<MyModel>()
///         .insert_resource(AtmosphereModel::new(MyModel::default()))
///         .run();
/// }
/// ```
pub trait AddAtmosphereModel {
    fn add_atmosphere_model<T: RegisterAtmosphereModel>(&mut self) -> &mut App;
}

impl AddAtmosphereModel for App {
    fn add_atmosphere_model<T: RegisterAtmosphereModel>(&mut self) -> &mut App {
        T::register(self);

        self
    }
}

/// A `Resource` that stores an [`Atmospheric`] model.
///
/// Acts as a wrapper for accessing an [`Atmospheric`] model as a resource.
#[derive(Resource, ExtractResource, Clone)]
pub struct AtmosphereModel {
    model: Box<dyn Atmospheric>,
}

impl From<&AtmosphereModel> for AtmosphereModel {
    fn from(atmosphere: &AtmosphereModel) -> Self {
        atmosphere.clone()
    }
}

impl AtmosphereModel {
    /// Creates a new `AtmosphereModel` from a [`Atmospheric`] model.
    pub fn new(model: impl Atmospheric + 'static) -> Self {
        Self {
            model: Box::new(model),
        }
    }

    /// Get a reference of the underlying [`Atmospheric`] trait object.
    #[inline]
    pub fn model(&self) -> &dyn Atmospheric {
        &*self.model
    }

    /// Get a mutable reference of the underlying [`Atmospheric`] trait object.
    #[inline]
    pub fn model_mut(&mut self) -> &mut dyn Atmospheric {
        &mut *self.model
    }

    /// Convert the underlying model to a reference of the specified [`Atmospheric`] model.
    pub fn to_ref<T: Atmospheric>(&self) -> Option<&T> {
        Atmospheric::as_reflect(&*self.model).downcast_ref()
    }

    /// Convert the underlying model to a mutable reference of the specified [`Atmospheric`] model.
    pub fn to_mut<T: Atmospheric>(&mut self) -> Option<&mut T> {
        Atmospheric::as_reflect_mut(&mut *self.model).downcast_mut()
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "nishita")] {
        impl Default for AtmosphereModel {
            fn default() -> Self {
                use crate::collection::nishita::Nishita;
                Self::new(Nishita::default())
            }
        }
    } else if #[cfg(feature = "gradient")] {
        impl Default for AtmosphereModel {
            fn default() -> Self {
                use crate::collection::gradient::Gradient;
                Self::new(Gradient::default())
            }
        }
    } else {
        impl Default for AtmosphereModel {
            fn default() -> Self {
                panic!("Enable at least one atmospheric model!");
            }
        }
    }
}
