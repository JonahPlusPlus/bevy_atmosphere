//! Provides the [`AtmosphereModel`] resource, a type stores an [`Atmospheric`](crate::model::Atmospheric) model.

use bevy::{prelude::*, render::extract_resource::ExtractResource};

use crate::model;

/// A [`Resource`] that stores an [`Atmospheric`](crate::model::Atmospheric) model.
/// 
/// Acts as a wrapper for accessing an [`Atmospheric`](crate::model::Atmospheric) model as a resource.
#[derive(Resource, ExtractResource, Debug, Clone)]
pub struct AtmosphereModel {
    model: Box<dyn model::Atmospheric>
}

impl From<&AtmosphereModel> for AtmosphereModel {
    fn from(atmosphere: &AtmosphereModel) -> Self {
        atmosphere.clone()
    }
}

impl AtmosphereModel {
    /// Creates a new `AtmospherModel` from a [`Atmospheric`](crate::model::Atmospheric) model.
    pub fn new(model: impl model::Atmospheric + 'static) -> Self {
        Self {
            model: Box::new(model)
        }
    }

    /// Get a reference of the [`Atmospheric`](crate::model::Atmospheric) trait object.
    #[inline]
    pub fn model(&self) -> &dyn model::Atmospheric {
        &*self.model
    }

    /// Get a mutable reference of the [`Atmospheric`](crate::model::Atmospheric) trait object.
    #[inline]
    pub fn model_mut(&mut self) -> &mut dyn model::Atmospheric {
        &mut *self.model
    }

    /// Cast to a reference of the specified [`Atmospheric`](crate::model::Atmospheric) model.
    pub fn to_ref<T: model::Atmospheric>(&self) -> Option<&T> {
        model::Atmospheric::as_reflect(&*self.model).downcast_ref()
    }

    /// Cast to a mutable reference of the specified [`Atmospheric`](crate::model::Atmospheric) model.
    pub fn to_mut<T: model::Atmospheric>(&mut self) -> Option<&mut T> {
        model::Atmospheric::as_reflect_mut(&mut *self.model).downcast_mut()
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "nishita")] {
        impl Default for AtmosphereModel {
            fn default() -> Self {
                use crate::models::nishita::Nishita;
                Self::new(Nishita::default())
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
