use bevy::prelude::*;
use bevy_atmosphere::prelude::*;
use bevy_spectator::{Spectator, SpectatorPlugin};

fn main() {
    println!("Demonstrates using the `Nishita` model\n\t- 1-9 number keys: Change preset\n\t- 0 number key: Remove `Nishita` model");
    App::new()
        .add_plugins((DefaultPlugins, AtmospherePlugin, SpectatorPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, change_nishita)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle::default(),
        AtmosphereCamera::default(),
        Spectator,
    ));
}

fn change_nishita(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::Digit1) {
        info!("Changed to Atmosphere Preset 1 (Sunset)");
        commands.insert_resource(AtmosphereModel::new(Nishita {
            sun_position: Vec3::new(0., 0., -1.),
            ..default()
        }));
    } else if keys.just_pressed(KeyCode::Digit2) {
        info!("Changed to Atmosphere Preset 2 (Noir Sunset)");
        commands.insert_resource(AtmosphereModel::new(Nishita {
            sun_position: Vec3::new(0., 0., -1.),
            rayleigh_coefficient: Vec3::new(1e-5, 1e-5, 1e-5),
            ..default()
        }));
    } else if keys.just_pressed(KeyCode::Digit3) {
        info!("Changed to Atmosphere Preset 3 (Magenta)");
        commands.insert_resource(AtmosphereModel::new(Nishita {
            rayleigh_coefficient: Vec3::new(2e-5, 1e-5, 2e-5),
            ..default()
        }));
    } else if keys.just_pressed(KeyCode::Digit4) {
        info!("Changed to Atmosphere Preset 4 (Strong Mie)");
        commands.insert_resource(AtmosphereModel::new(Nishita {
            mie_coefficient: 5e-5,
            ..default()
        }));
    } else if keys.just_pressed(KeyCode::Digit5) {
        info!("Changed to Atmosphere Preset 5 (Larger Scale)");
        commands.insert_resource(AtmosphereModel::new(Nishita {
            rayleigh_scale_height: 16e3,
            mie_scale_height: 2.4e3,
            ..default()
        }));
    } else if keys.just_pressed(KeyCode::Digit6) {
        info!("Changed to Atmosphere Preset 6 (Weak Intensity)");
        commands.insert_resource(AtmosphereModel::new(Nishita {
            sun_intensity: 11.0,
            ..default()
        }));
    } else if keys.just_pressed(KeyCode::Digit7) {
        info!("Changed to Atmosphere Preset 7 (Half Radius)");
        commands.insert_resource(AtmosphereModel::new(Nishita {
            ray_origin: Vec3::new(0., 6372e3 / 2., 0.),
            planet_radius: 6371e3 / 2.,
            atmosphere_radius: 6471e3 / 2.,
            ..default()
        }));
    } else if keys.just_pressed(KeyCode::Digit8) {
        info!("Changed to Atmosphere Preset 8 (Sideways World)");
        commands.insert_resource(AtmosphereModel::new(Nishita {
            ray_origin: Vec3::new(6372e3, 0., 0.),
            ..default()
        }));
    } else if keys.just_pressed(KeyCode::Digit9) {
        info!("Changed to Atmosphere Preset 9 (Inverted Mie Direction)");
        commands.insert_resource(AtmosphereModel::new(Nishita {
            mie_direction: -0.758,
            ..default()
        }));
    } else if keys.just_pressed(KeyCode::Digit0) {
        info!("Reset Atmosphere to Default");
        commands.remove_resource::<AtmosphereModel>();
    }
}
