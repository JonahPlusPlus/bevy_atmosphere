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
//!     commands
//!         .spawn_bundle(Camera3dBundle::default())
//!         .insert(AtmosphereCamera(None));
//! }
//! ```
//!
//! To change the sky's appearance, use the [`Atmosphere`](crate::system_param::Atmosphere) and [`AtmosphereMut`](crate::system_param::AtmosphereMut) system params.
//! ```no_run
//! # use bevy::utils::default;
//! # use bevy::math::Vec3;
//! # use bevy_atmosphere::prelude::*;
//! fn read_nishita(atmosphere: Atmosphere<Nishita>) {
//!     let sun_position = atmosphere.sun_position;
//!     println!("Sun is at {sun_position}");
//! }
//! 
//! fn write_gradient(mut atmosphere: AtmosphereMut<Gradient>) {
//!     atmosphere.horizon = Color::RED;
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
//!     resolution: 1024
//! }
//! # ;
//! ```
//!
//! To see more examples, view the ["examples"](https://github.com/JonahPlusPlus/bevy_atmosphere/tree/master/examples) directory.

pub mod model;
pub mod models;
pub mod pipeline;
pub mod plugin;
pub mod resource;
pub mod settings;
pub mod skybox;
pub mod system_param;

pub mod prelude {
    //! `use bevy_atmosphere::prelude::*;` to import the most commonly used items.
    pub use crate::plugin::{AtmosphereCamera, AtmospherePlugin};
    pub use crate::resource::AtmosphereModel;
    pub use crate::settings::AtmosphereSettings;
    pub use crate::system_param::{Atmosphere, AtmosphereMut};

    #[cfg(any(doc, feature = "nishita"))]
    pub use crate::models::nishita::Nishita;

    #[cfg(any(doc, feature = "gradient"))]
    pub use crate::models::gradient::Gradient;
}
