//! a procedural sky plugin for bevy
//! 
//!
//! ## "basic" Example
//! ```no_run
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

pub mod pipeline;
pub mod plugin;
pub mod resource;
pub mod skybox;

pub mod prelude {
    pub use crate::plugin::{AtmosphereCamera, AtmospherePlugin};
    pub use crate::resource::Atmosphere;
}
