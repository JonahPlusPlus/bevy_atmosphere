use bevy::prelude::*;
use bevy_atmosphere::prelude::*;
use bevy_spectator::*;

fn main() {
    println!("Demonstrates using the `AtmosphereSettings` resource\n\t- Spacebar: Toggle dithering\n\t- 1-9 number keys: Change resolution\n\t- 0 number key: Remove `AtmosphereSettings` resource");
    App::new()
        .insert_resource(AtmosphereSettings {
            resolution: 16,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(AtmospherePlugin)
        .add_plugins(SpectatorPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, change_resolution)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle::default(),
        AtmosphereCamera::default(),
        Spectator,
    ));
}

// Change the resolution when the user presses a number key
fn change_resolution(
    mut commands: Commands,
    settings: Option<ResMut<AtmosphereSettings>>,
    keys: Res<Input<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        #[cfg(feature = "bevy/trace")]
        // enabling "bevy/trace" (via "bevy/trace_chrome" or "bevy/trace_tracy") allows you to debug bevy_atmosphere
        let _change_dithering_executed_span =
            info_span!("executed", name = "settings::change_dithering").entered();
        if let Some(mut settings) = settings {
            settings.dithering ^= true;
        } else {
            commands.insert_resource(AtmosphereSettings {
                dithering: false,
                ..default()
            });
        }
        info!("Toggled dithering");
    } else if keys.just_pressed(KeyCode::Key1) {
        change(commands, settings, 8); // 8x8
    } else if keys.just_pressed(KeyCode::Key2) {
        change(commands, settings, 16); // 16x16
    } else if keys.just_pressed(KeyCode::Key3) {
        change(commands, settings, 32); // 32x32
    } else if keys.just_pressed(KeyCode::Key4) {
        change(commands, settings, 64); // 64x64
    } else if keys.just_pressed(KeyCode::Key5) {
        change(commands, settings, 128); // 128x128
    } else if keys.just_pressed(KeyCode::Key6) {
        change(commands, settings, 256); // 256x256
    } else if keys.just_pressed(KeyCode::Key7) {
        change(commands, settings, 512); // 512x512
    } else if keys.just_pressed(KeyCode::Key8) {
        change(commands, settings, 1024); // 1024x1024
    } else if keys.just_pressed(KeyCode::Key9) {
        change(commands, settings, 2048); // 2048x2048
    } else if keys.just_pressed(KeyCode::Key0) {
        commands.remove_resource::<AtmosphereSettings>(); // Removes settings, goes back to defaults
        info!("Removed AtmosphereSettings");
    }
}

// A separate `change` fn makes it easier to debug in tracy.
fn change(mut commands: Commands, settings: Option<ResMut<AtmosphereSettings>>, resolution: u32) {
    #[cfg(feature = "bevy/trace")]
    // enabling "bevy/trace" (via "bevy/trace_chrome" or "bevy/trace_tracy") allows you to debug bevy_atmosphere
    let _change_resolution_executed_span =
        info_span!("executed", name = "settings::change_resolution").entered();
    if let Some(mut settings) = settings {
        settings.resolution = resolution;
    } else {
        commands.insert_resource(AtmosphereSettings {
            resolution,
            ..default()
        });
    }
    info!("Changed resolution to {resolution}");
}
