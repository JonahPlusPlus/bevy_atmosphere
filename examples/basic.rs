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
