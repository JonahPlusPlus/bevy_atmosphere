# [![bevy_atmosphere logo](/assets/logo.svg)](https://github.com/JonahPlusPlus/bevy_atmosphere)
[![Crates.io](https://img.shields.io/crates/v/bevy_atmosphere)](https://crates.io/crates/bevy_atmosphere)
[![Crates.io](https://img.shields.io/crates/d/bevy_atmosphere)](https://crates.io/crates/bevy_atmosphere)
[![docs.rs](https://img.shields.io/docsrs/bevy_atmosphere)](https://docs.rs/bevy_atmosphere/)
[![Discord](https://img.shields.io/discord/691052431525675048.svg?label=&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.com/channels/691052431525675048/1035260359952576603)

A procedural sky plugin for the [Bevy game engine](https://bevyengine.org).

## ["basic" Example](/examples/basic.rs)

![basic example image](examples/images/basic-example.png)

```rust
use bevy::prelude::*;
use bevy_atmosphere::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, AtmospherePlugin))
        .add_system(Startup, setup)
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

## Bevy compatibility

| bevy | bevy_atmosphere |
|------|-----------------|
| 0.11 | 0.7             |
| 0.10 | 0.6             |
| 0.9  | 0.5             |
| 0.8  | 0.4             |
| 0.7  | 0.3             |
| 0.6  | 0.1             |

### 🚧 Warning: Incompatible with WebGL 🚧

Versions 0.4 and higher break compatibility with WebGL by using a compute shader for efficiency.
