//! A procedural sky plugin for the [Bevy game engine](https://bevyengine.org/).
//!
//! Provides a framework for creating and using atmospheric models.
//!
//! ## "basic" Example
//! ```no_run
//! # use bevy::utils::default;
//! use bevy::prelude::*;
//! use bevy_atmosphere::prelude::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(AtmospherePlugin)
//!         .add_startup_system(setup)
//!         .run();
//! }
//!
//! fn setup(mut commands: Commands) {
//!     commands.spawn((Camera3dBundle::default(), AtmosphereCamera::default()));
//! }
//! ```
//!
//! How the sky is rendered is described by an [`Atmospheric`](crate::model::Atmospheric) model.
//! bevy_atmosphere provides a [collection of models to use](crate::collection), but you can [create your own as well](crate::model).
//!
//! To read and modify the atmospheric model, use the [`Atmosphere<T>`](crate::system_param::Atmosphere) and
//! [`AtmosphereMut<T>`](crate::system_param::AtmosphereMut) system params or
//! the [`AtmosphereModel`](struct@crate::model::AtmosphereModel) resource.
//! ```no_run
//! # use bevy::utils::default;
//! # use bevy::math::Vec3;
//! # use bevy::prelude::*;
//! # use bevy_atmosphere::prelude::*;
//! fn read_nishita(atmosphere: Atmosphere<Nishita>) {
//!     let sun_position = atmosphere.sun_position;
//!     println!("Sun is at {sun_position}");
//! }
//!
//! fn write_gradient(mut atmosphere: AtmosphereMut<Gradient>) {
//!     atmosphere.horizon = Color::RED;
//! }
//!
//! fn check_model(atmosphere: Res<AtmosphereModel>) {
//!     if let Some(nishita) = atmosphere.to_ref::<Nishita>() {
//!         println!("Sun is at {}", nishita.sun_position);
//!     } else {
//!         println!("Model isn't Nishita");
//!     }
//! }
//! # ;
//! ```
//!
//! Use the [`AtmosphereSettings`](crate::settings::AtmosphereSettings) resource to change how the sky is rendered.
//! ```no_run
//! # use bevy_atmosphere::settings::AtmosphereSettings;
//! # let _ =
//! AtmosphereSettings {
//!     // changes the resolution (should be a multiple of 8)
//!     resolution: 1024,
//!     // turns off dithering
//!     dithering: false,
//! }
//! # ;
//! ```
//!
//! To see more examples, view the ["examples"](https://github.com/JonahPlusPlus/bevy_atmosphere/tree/master/examples) directory.

pub mod collection;
pub mod model;
pub mod pipeline;
pub mod plugin;
pub mod settings;
pub mod skybox;
pub mod system_param;

pub mod prelude {

    //! `use bevy_atmosphere::prelude::*;` to import the most commonly used items.
    pub use crate::model::{AddAtmosphereModel, AtmosphereModel, Atmospheric};
    pub use crate::plugin::{AtmosphereCamera, AtmospherePlugin};
    pub use crate::settings::AtmosphereSettings;
    pub use crate::system_param::{Atmosphere, AtmosphereMut};

    #[cfg(any(doc, feature = "nishita"))]
    pub use crate::collection::nishita::Nishita;

    #[cfg(any(doc, feature = "gradient"))]
    pub use crate::collection::gradient::Gradient;
}
