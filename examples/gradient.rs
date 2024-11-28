use bevy::color::palettes;
use bevy::prelude::*;
use bevy_atmosphere::prelude::*;
use bevy_spectator::{Spectator, SpectatorPlugin};

fn main() {
    println!("Demonstrates using the `Gradient` model\n\t- 1-9 number keys: Change preset\n\t- 0 number key: Remove `Gradient` model");
    App::new()
        .insert_resource(AtmosphereModel::new(Gradient::default()))
        .add_plugins((DefaultPlugins, AtmospherePlugin, SpectatorPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, change_gradient)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        AtmosphereCamera::default(),
        Spectator,
    ));
}

fn change_gradient(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::Digit1) {
        info!("Changed to Atmosphere Preset 1 (Default Gradient)");
        commands.insert_resource(AtmosphereModel::new(Gradient::default()));
    } else if keys.just_pressed(KeyCode::Digit2) {
        info!("Changed to Atmosphere Preset 2 (Cotton Candy)");
        commands.insert_resource(AtmosphereModel::new(Gradient {
            ground: Color::srgb(1.0, 0.5, 0.75).into(),
            horizon: LinearRgba::WHITE,
            sky: Color::srgb(0.5, 0.75, 1.0).into(),
        }));
    } else if keys.just_pressed(KeyCode::Digit3) {
        info!("Changed to Atmosphere Preset 3 (80's Sunset)");
        commands.insert_resource(AtmosphereModel::new(Gradient {
            sky: palettes::css::PURPLE.into(),
            horizon: palettes::css::PINK.into(),
            ground: palettes::css::ORANGE.into(),
        }));
    } else if keys.just_pressed(KeyCode::Digit4) {
        info!("Changed to Atmosphere Preset 4 (Winter)");
        commands.insert_resource(AtmosphereModel::new(Gradient {
            ground: Color::srgb(0.0, 0.1, 0.2).into(),
            horizon: Color::srgb(0.3, 0.4, 0.5).into(),
            sky: Color::srgb(0.7, 0.8, 0.9).into(),
        }));
    } else if keys.just_pressed(KeyCode::Digit5) {
        info!("Changed to Atmosphere Preset 5 (Nether)");
        commands.insert_resource(AtmosphereModel::new(Gradient {
            ground: LinearRgba::BLACK,
            horizon: Color::srgb(0.2, 0.0, 0.0).into(),
            sky: Color::srgb(0.5, 0.1, 0.0).into(),
        }));
    } else if keys.just_pressed(KeyCode::Digit6) {
        info!("Changed to Atmosphere Preset 6 (Golden)");
        commands.insert_resource(AtmosphereModel::new(Gradient {
            ground: palettes::css::ORANGE_RED.into(),
            horizon: palettes::css::ORANGE.into(),
            sky: palettes::css::GOLD.into(),
        }));
    } else if keys.just_pressed(KeyCode::Digit7) {
        info!("Changed to Atmosphere Preset 7 (Noir)");
        commands.insert_resource(AtmosphereModel::new(Gradient {
            ground: LinearRgba::BLACK,
            horizon: LinearRgba::BLACK,
            sky: LinearRgba::WHITE,
        }));
    } else if keys.just_pressed(KeyCode::Digit8) {
        info!("Changed to Atmosphere Preset 8 (Midnight)");
        commands.insert_resource(AtmosphereModel::new(Gradient {
            ground: LinearRgba::BLACK,
            horizon: LinearRgba::BLACK,
            sky: palettes::css::MIDNIGHT_BLUE.into(),
        }));
    } else if keys.just_pressed(KeyCode::Digit9) {
        info!("Changed to Atmosphere Preset 9 (Greenery)");
        commands.insert_resource(AtmosphereModel::new(Gradient {
            ground: Color::srgb(0.1, 0.2, 0.0).into(),
            horizon: Color::srgb(0.3, 0.4, 0.1).into(),
            sky: Color::srgb(0.6, 0.8, 0.2).into(),
        }));
    } else if keys.just_pressed(KeyCode::Digit0) {
        info!("Reset Atmosphere to Default");
        commands.remove_resource::<AtmosphereModel>();
    }
}
