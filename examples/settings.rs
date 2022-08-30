use bevy::prelude::*;
use bevy_atmosphere::prelude::*;

fn main() {
    App::new()
        .insert_resource(AtmosphereSettings { resolution: 16 })
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

// Change the resolution when the user presses a number key
fn change_resolution(mut commands: Commands, keys: Res<Input<KeyCode>>) {
    if keys.just_pressed(KeyCode::Key1) {
        change(commands, 8); // 8x8
    } else if keys.just_pressed(KeyCode::Key2) {
        change(commands, 16); // 16x16
    } else if keys.just_pressed(KeyCode::Key3) {
        change(commands, 32); // 32x32
    } else if keys.just_pressed(KeyCode::Key4) {
        change(commands, 64); // 64x64
    } else if keys.just_pressed(KeyCode::Key5) {
        change(commands, 128); // 128x128
    } else if keys.just_pressed(KeyCode::Key6) {
        change(commands, 256); // 256x256
    } else if keys.just_pressed(KeyCode::Key7) {
        change(commands, 512); // 512x512
    } else if keys.just_pressed(KeyCode::Key8) {
        change(commands, 1024); // 1024x1024
    } else if keys.just_pressed(KeyCode::Key9) {
        change(commands, 2048); // 2048x2048
    } else if keys.just_pressed(KeyCode::Key0) {
        commands.remove_resource::<AtmosphereSettings>(); // Removes settings, goes back to defaults
        info!("Removed AtmosphereSettings");
    }
}

// A separate `change` fn makes it easier to debug in tracy.
fn change(mut commands: Commands, resolution: u32) {
    #[cfg(feature = "trace")]
    // bevy_atmosphere offers the "trace" feature for when you debug in tracy
    let _change_resolution_executed_span =
        info_span!("executed", name = "settings::change_resolution").entered();
    commands.insert_resource(AtmosphereSettings { resolution });
    info!("Changed resolution to {resolution}");
}
