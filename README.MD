# bevy_atmosphere

### A procedural sky plugin for bevy

### Example
```
use bevy::prelude::*;
use bevy_atmosphere::*;
fn main() {
    App::build()
        .insert_resource(bevy_atmosphere::AtmosphereMat::default()) // Default Earth sky
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_atmosphere::AtmospherePlugin { dynamic: false }) // Set to false since we aren't changing the sky's appearance
        .add_startup_system(setup.system())
        .run();
}
fn setup(mut commands: Commands) {
    commands.spawn_bundle(PerspectiveCameraBundle::default());
}
```

Big thanks to  https://github.com/wwwtyro/glsl-atmosphere and https://github.com/jomala/bevy_skybox, which I essentially tried to merge together

Also thanks to those on the Bevy Discord who took the time to help me 