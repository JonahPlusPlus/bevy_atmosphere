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

Thank you [kbieganski](https://github.com/kbieganski) for doing the 0.2.0 update!

Big thanks to [glsl-atmosphere](https://github.com/wwwtyro/glsl-atmosphere) and [bevy_skybox](https://github.com/jomala/bevy_skybox), which I essentially tried to merge together

Also thanks to those on the Bevy Discord who took the time to help me 