# bevy_atmosphere

### A procedural sky plugin for bevy

[![Crates.io](https://img.shields.io/crates/d/bevy_atmosphere)](https://crates.io/crates/bevy_atmosphere) [![docs.rs](https://img.shields.io/docsrs/bevy_atmosphere)](https://docs.rs/bevy_atmosphere/) 

### Example
```rust
use bevy::prelude::*;
use bevy_atmosphere::*;
fn main() {
    App::new()
        .insert_resource(bevy_atmosphere::AtmosphereMat::default()) // Default Earth sky
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_atmosphere::AtmospherePlugin {
            dynamic: false,  // Set to false since we aren't changing the sky's appearance
            sky_radius: 10.0,
        })
        .add_startup_system(setup)
        .run();
}
fn setup(mut commands: Commands) {
    commands.spawn_bundle(PerspectiveCameraBundle::default());
}
```

## Change Log

* v0.3.1: Changed default radius from 10 to 100 (larger values can reduce visual artifacts)
* v0.3.0: Updated to bevy v0.7
