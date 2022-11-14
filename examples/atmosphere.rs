use bevy::prelude::*;
use bevy_atmosphere::prelude::*;
use bevy_spectator::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AtmospherePlugin)
        .add_plugin(SpectatorPlugin)
        .add_startup_system(setup)
        .add_system(change_atmosphere)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera3dBundle::default(), AtmosphereCamera(None), Spectator));
}

fn change_atmosphere(mut commands: Commands, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::Key1) {
        info!("Changed to Atmosphere Preset 1 (Sunset)");
        commands.insert_resource(Atmosphere {
            sun_position: Vec3::new(0., 0., -1.),
            ..default()
        });
    } else if keys.just_pressed(KeyCode::Key2) {
        info!("Changed to Atmosphere Preset 2 (Noir Sunset)");
        commands.insert_resource(Atmosphere {
            sun_position: Vec3::new(0., 0., -1.),
            rayleigh_coefficient: Vec3::new(1e-5, 1e-5, 1e-5),
            ..default()
        });
    } else if keys.just_pressed(KeyCode::Key3) {
        info!("Changed to Atmosphere Preset 3 (Magenta)");
        commands.insert_resource(Atmosphere {
            rayleigh_coefficient: Vec3::new(2e-5, 1e-5, 2e-5),
            ..default()
        });
    } else if keys.just_pressed(KeyCode::Key4) {
        info!("Changed to Atmosphere Preset 4 (Strong Mie)");
        commands.insert_resource(Atmosphere {
            mie_coefficient: 5e-5,
            ..default()
        });
    } else if keys.just_pressed(KeyCode::Key5) {
        info!("Changed to Atmosphere Preset 5 (Larger Scale)");
        commands.insert_resource(Atmosphere {
            rayleigh_scale_height: 16e3,
            mie_scale_height: 2.4e3,
            ..default()
        });
    } else if keys.just_pressed(KeyCode::Key6) {
        info!("Changed to Atmosphere Preset 6 (Weak Intensity)");
        commands.insert_resource(Atmosphere {
            sun_intensity: 11.0,
            ..default()
        });
    } else if keys.just_pressed(KeyCode::Key7) {
        info!("Changed to Atmosphere Preset 7 (Half Radius)");
        commands.insert_resource(Atmosphere {
            ray_origin: Vec3::new(0., 6372e3 / 2., 0.),
            planet_radius: 6371e3 / 2.,
            atmosphere_radius: 6471e3 / 2.,
            ..default()
        });
    } else if keys.just_pressed(KeyCode::Key8) {
        info!("Changed to Atmosphere Preset 8 (Sideways World)");
        commands.insert_resource(Atmosphere {
            ray_origin: Vec3::new(6372e3, 0., 0.),
            ..default()
        });
    } else if keys.just_pressed(KeyCode::Key9) {
        info!("Changed to Atmosphere Preset 9 (Inverted Mie Direction)");
        commands.insert_resource(Atmosphere {
            mie_direction: -0.758,
            ..default()
        });
    } else if keys.just_pressed(KeyCode::Key0) {
        info!("Reset Atmosphere to Default");
        commands.remove_resource::<Atmosphere>();
    }
}
