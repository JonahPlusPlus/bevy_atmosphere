//! A procedural sky plugin for bevy
//!
//! ## Example
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_atmosphere::*;
//!
//! fn main() {
//!     App::new()
//!         .insert_resource(bevy_atmosphere::AtmosphereMat::default()) // Default Earth sky
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(bevy_atmosphere::AtmospherePlugin {
//!             dynamic: false,  // Set to false since we aren't changing the sky's appearance
//!             sky_radius: 100.0
//!         })
//!         .add_startup_system(setup)
//!         .run();
//! }
//!
//! fn setup(mut commands: Commands) {
//!     commands.spawn_bundle(PerspectiveCameraBundle::default());
//! }
//! ```

pub mod skybox;
pub mod pipeline;
pub mod plugin;
pub mod resource;

pub mod prelude {
    pub use crate::plugin::{AtmosphereCamera, AtmospherePlugin};
    pub use crate::resource::Atmosphere;
}
