# [![bevy_atmosphere logo](/assets/logo.svg)](https://github.com/JonahPlusPlus/bevy_atmosphere)
[![bevy](https://img.shields.io/badge/Bevy-0.8-blue)](https://crates.io/crates/bevy/0.8.0)
[![Crates.io](https://img.shields.io/crates/v/bevy_atmosphere)](https://crates.io/crates/bevy_atmosphere)
[![Crates.io](https://img.shields.io/crates/d/bevy_atmosphere)](https://crates.io/crates/bevy_atmosphere)
[![docs.rs](https://img.shields.io/docsrs/bevy_atmosphere)](https://docs.rs/bevy_atmosphere/)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/JonahPlusPlus/bevy_atmosphere#license)

A procedural sky plugin for the [Bevy game engine](https://bevyengine.org/).

### 🚧 Warning: Under Development 🚧

v0.4 breaks compatibility with WebGL by using a compute shader.
WebGPU should resolve this when shipped.

If you need to test a web build, you can try enabling your browser's respective experiment flag for WebGPU.

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

## License

bevy_atmosphere is dual-licensed under MIT and Apache-2.0! That means you can choose to use `bevy_atmosphere` under either for your project.

## 0.4 Change Log

* To change the sky simulation parameters, you would add/update an `Atmosphere` resource with custom values.
* The plugin doesn't just pick the first camera, but can be used on select cameras using the `AtmosphereCamera` component, which holds an optional render layer for the spawned skybox to be on.
* The plugin will automatically create skyboxes for atmosphere cameras during the `ATMOSPHERE_INIT` startup stage, which can be disabled by turning off the "automatic" feature.
* Created skyboxes now have the `AtmosphereSkyBox` component. Only skyboxes with the component and that have a parent with `AtmosphereCamera` will have their rotation corrected.
* To change the resolution, you can add an `AtmosphereSettings` resource and set the `resolution` field (which should be a multiple of 8). This could be used as part of quality settings in games.

### 0.4.1 Patch
* Removed `ATMOSPHERE_INIT` stage and "init" feature.
* Added new "detection" feature that checks for new `AtmosphereCamera` components each frame, instead of just at startup. (Removal detection will be added in a future release)
* Removed unnecessary "radsort" dependency.
* Made removing `Atmosphere` and `AtmosphereSettings` resources set back to default.
* `settings` example now shows removing `AtmosphereSettings`.
* Added files to `package.exclude` of `Cargo.toml`, in order to reduce package size.
