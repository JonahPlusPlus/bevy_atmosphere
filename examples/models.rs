use bevy::prelude::*;
use bevy_atmosphere::prelude::*;
use bevy_spectator::{Spectator, SpectatorPlugin};

fn main() {
    println!("Demonstrates changing the atmosphere model\n\t- G: Gradient\n\t- N: Nishita");

    App::new()
        .add_plugins((DefaultPlugins, AtmospherePlugin, SpectatorPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, change_model)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        AtmosphereCamera::default(),
        Spectator,
    ));
}

fn change_model(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::KeyG) {
        info!("Changed to Gradient atmosphere model");
        commands.insert_resource(AtmosphereModel::new(Gradient::default()));
    } else if keys.just_pressed(KeyCode::KeyN) {
        info!("Changed to Nishita atmosphere model");
        commands.insert_resource(AtmosphereModel::new(Nishita::default()));
    } else if keys.just_pressed(KeyCode::Digit0) {
        info!("Reset atmosphere model to default");
        commands.remove_resource::<AtmosphereModel>();
    }
}
