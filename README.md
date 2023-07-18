# [![bevy_atmosphere logo](/assets/logo.svg)](https://github.com/JonahPlusPlus/bevy_atmosphere)
[![bevy](https://img.shields.io/badge/Bevy-0.10-blue)](https://crates.io/crates/bevy/0.10.0)
[![Crates.io](https://img.shields.io/crates/v/bevy_atmosphere)](https://crates.io/crates/bevy_atmosphere)
[![Crates.io](https://img.shields.io/crates/d/bevy_atmosphere)](https://crates.io/crates/bevy_atmosphere)
[![docs.rs](https://img.shields.io/docsrs/bevy_atmosphere)](https://docs.rs/bevy_atmosphere/)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/JonahPlusPlus/bevy_atmosphere#license)
[![Discord](https://img.shields.io/discord/691052431525675048.svg?label=&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.com/channels/691052431525675048/1035260359952576603)

A procedural sky plugin for the [Bevy game engine](https://bevyengine.org/).

## ["basic" Example](/examples/basic.rs)

![basic example image](examples/images/basic-example.png)

```rust
use bevy::prelude::*;
use bevy_atmosphere::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AtmospherePlugin)
        .add_systems(Update, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera3dBundle::default(), AtmosphereCamera::default()));
}
```

## ["cycle" Example](/examples/cycle.rs)

![cycle example image](examples/images/cycle-example.png)

## Getting Started

To learn more, read the [docs](https://docs.rs/bevy_atmosphere/) or check out the [examples](/examples/).

For more information on the technicalities, you can check out the [technical docs](/docs/) or check out [my blog](https://jonahplusplus.dev/).

### 🚧 Warning: Incompatible with WebGL 🚧

Versions 0.4 and higher break compatibility with WebGL by using a compute shader for efficiency.
WebGPU should resolve this when shipped.

As of writing, Bevy uses WebGL internally. A custom fork can be used to enable WebGPU in Bevy and feature flags can be used to enable WebGPU in most browsers.

## License

bevy_atmosphere is dual-licensed under MIT and Apache-2.0! That means you can choose to use `bevy_atmosphere` under either for your project.

## 0.6 Change Log

- Updated bevy to 0.10
- Updated bevy_spectator to 0.2 (for examples)
