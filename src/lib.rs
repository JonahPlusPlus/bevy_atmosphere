//! A procedural sky plugin for the [Bevy game engine](https://bevyengine.org/).
//!
//! Uses Rayleigh and Mie scattering to simulate a realistic sky.
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
//! To change the sky's appearance, use the [`Atmosphere`](crate::resource::Atmosphere) resource.
//! ```no_run
//! # use bevy::utils::default;
//! # use bevy::math::Vec3;
//! # use bevy_atmosphere::resource::Atmosphere;
//! # let _ =
//! Atmosphere {
//!     // changes the sky color using Rayleigh scattering
//!     rayleigh_coefficient: Vec3::new(22.4e-6, 5.5e-6, 13.0e-6),
//!     ..default()
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

pub mod pipeline;
pub mod plugin;
pub mod resource;
pub mod settings;
pub mod skybox;

pub mod prelude {
    //! `use bevy_atmosphere::prelude::*;` to import the most commonly used items.
    pub use crate::plugin::{AtmosphereCamera, AtmospherePlugin};
    pub use crate::resource::Atmosphere;
    pub use crate::settings::AtmosphereSettings;
}
