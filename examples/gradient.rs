use bevy::prelude::*;
use bevy_atmosphere::prelude::*;
use bevy_spectator::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Atmosphere::new(Gradient {
            ground: Color::rgb(0.5, 0.2, 0.1),
            horizon: Color::rgb(0.5, 0.1, 0.6),
            sky: Color::rgb(0.1, 0.5, 0.9),
        }))
        .add_plugin(AtmospherePlugin)
        .add_plugin(SpectatorPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera3dBundle::default(), AtmosphereCamera(None), Spectator));
}