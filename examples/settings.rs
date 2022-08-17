use bevy::prelude::*;
use bevy_atmosphere::prelude::*;

fn main() {
    App::new()
        .insert_resource(AtmosphereSettings { resolution: 8 })
        // .insert_resource(Atmosphere {
        //     sun_position: Vec3::new(0.0, 0.0, 1.0),
        //     ..default()
        // })
        .add_plugins(DefaultPlugins)
        .add_plugin(AtmospherePlugin)
        .add_plugin(bevy_flycam::NoCameraPlayerPlugin)
        .add_startup_system(setup)
        .add_system(change_resolution)
        .run();
}

fn setup(mut commands: Commands) {
    commands
        .spawn_bundle(Camera3dBundle::default())
        .insert(AtmosphereCamera(None))
        .insert(bevy_flycam::FlyCam);
}

fn change_resolution(mut settings: ResMut<AtmosphereSettings>, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::Key1) {
        settings.resolution = 8;
        info!("Changed resolution to 8");
    } else if keys.just_pressed(KeyCode::Key2) {
        settings.resolution = 16;
        info!("Changed resolution to 16");
    } else if keys.just_pressed(KeyCode::Key3) {
        settings.resolution = 32;
        info!("Changed resolution to 32");
    } else if keys.just_pressed(KeyCode::Key4) {
        settings.resolution = 64;
        info!("Changed resolution to 64");
    } else if keys.just_pressed(KeyCode::Key5) {
        settings.resolution = 128;
        info!("Changed resolution to 128");
    } else if keys.just_pressed(KeyCode::Key6) {
        settings.resolution = 256;
        info!("Changed resolution to 256");
    } else if keys.just_pressed(KeyCode::Key7) {
        settings.resolution = 512;
        info!("Changed resolution to 512");
    } else if keys.just_pressed(KeyCode::Key8) {
        settings.resolution = 1024;
        info!("Changed resolution to 1024");
    }
}
