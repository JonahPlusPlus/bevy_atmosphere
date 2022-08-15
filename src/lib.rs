//! A procedural sky plugin for bevy.
//! 
//!
//! ## "basic" Example
//! ```norun
//! use bevy::prelude::*;
//! use bevy_atmosphere::prelude::*;
//! 
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(AtmospherePlugin::default())
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
//! To change the sky's appearance, use the [Atmosphere](crate::resource::Atmosphere) resource
//! ```no_run
//! Atmosphere {
//!     // changes the sky color from Rayleigh scattering
//!     rayleigh_coefficient: Vec3::new(22.4e-6, 5.5e-6, 13.0e-6),
//!     ..default()
//! }
//! ```
//! 
//! To see more examples, view the ["examples"](https://www.github.com/JonahPlusPlus/examples) directory

pub mod pipeline;
pub mod plugin;
pub mod resource;
pub mod skybox;

pub mod prelude {
    //! `use bevy_atmosphere::prelude::*;` to import the most commonly used items.
    pub use crate::plugin::{AtmosphereCamera, AtmospherePlugin};
    pub use crate::resource::Atmosphere;
}
