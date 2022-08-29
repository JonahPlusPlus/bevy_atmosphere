![bevy_atmosphere logo](/assets/logo.svg)
# bevy_atmosphere
[![Crates.io](https://img.shields.io/crates/d/bevy_atmosphere)](https://crates.io/crates/bevy_atmosphere) [![docs.rs](https://img.shields.io/docsrs/bevy_atmosphere)](https://docs.rs/bevy_atmosphere/)

## A procedural sky plugin for the [Bevy game engine](https://bevyengine.org/).

Uses Rayleigh and Mie scattering to simulate a realistic sky.

## ["basic" Example](/examples/basic.rs)

```rust
use bevy::prelude::*;
use bevy_atmosphere::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AtmospherePlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands
        .spawn_bundle(Camera3dBundle::default())
        .insert(AtmosphereCamera(None));
}
```

## 0.4 Change Log

* To change the sky simulation parameters, you would add/update an `Atmosphere` resource with custom values.
* The plugin doesn't just pick the first camera, but can be used on select cameras using the `AtmosphereCamera` component, which holds an optional render layer for the spawned skybox to be on.
* The plugin will automatically create skyboxes for atmosphere cameras during the `ATMOSPHERE_INIT` startup stage, which can be disabled by turning off the "automatic" feature.
* Created skyboxes now have the `AtmosphereSkyBox` component. Only skyboxes with the component and that have a parent with `AtmosphereCamera` will have their rotation corrected.
* To change the resolution, you can add an `AtmosphereSettings` resource and set the `resolution` field (which should be a multiple of 8). This could be used as part of quality settings in games.