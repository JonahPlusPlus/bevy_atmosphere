use bevy::prelude::*;
use bevy_atmosphere::prelude::*;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(AtmosphereSettings {
            resolution: 16
        })
        .insert_resource(Atmosphere::default()) // Default Atmosphere material, we can edit it to simulate another planet
        .insert_resource(CycleTimer(Timer::new(
            bevy::utils::Duration::from_millis(100), // Update our atmosphere every 100ms (in a real game, this would be much slower, but for the sake of an example we use a faster update)
            true,
        )))
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_flycam::NoCameraPlayerPlugin) // Simple movement for this example
        .add_plugin(AtmospherePlugin) // Default AtmospherePlugin
        .add_startup_system(setup_environment)
        .add_system(daylight_cycle)
        .add_system(change_resolution)
        .run();
}

// Marker for updating the position of the light, not needed unless we have multiple lights
#[derive(Component)]
struct Sun;

// Timer for updating the daylight cycle (updating the atmosphere every frame is slow, so it's better to do incremental changes)
struct CycleTimer(Timer);

// We can edit the Atmosphere resource and it will be updated automatically
fn daylight_cycle(
    mut atmosphere: ResMut<Atmosphere>,
    mut query: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    mut timer: ResMut<CycleTimer>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());

    let mut pos = atmosphere.sun_position;
    let t = time.time_since_startup().as_millis() as f32 / 2000.0;
    pos.y = t.sin();
    pos.z = t.cos();
    atmosphere.sun_position = pos;

    if let Some((mut light_trans, mut directional)) = query.single_mut().into() {
        light_trans.rotation = Quat::from_rotation_x(-pos.y.atan2(pos.z));
        directional.illuminance = t.sin().max(0.0).powf(2.0) * 100000.0;
    }

    if timer.0.finished() {}
}

// Simple environment
fn setup_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Our Sun
    commands
        .spawn_bundle(DirectionalLightBundle {
            ..Default::default()
        })
        .insert(Sun); // Marks the light as Sun

    // Simple transform shape just for reference
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(StandardMaterial::from(Color::rgb(0.8, 0.8, 0.8))),
        ..Default::default()
    });

    // X axis
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.5 })),
        material: materials.add(StandardMaterial::from(Color::rgb(0.8, 0.0, 0.0))),
        transform: Transform::from_xyz(1., 0., 0.),
        ..Default::default()
    });

    // Y axis
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.5 })),
        material: materials.add(StandardMaterial::from(Color::rgb(0.0, 0.8, 0.0))),
        transform: Transform::from_xyz(0., 1., 0.),
        ..Default::default()
    });

    // Z axis
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.5 })),
        material: materials.add(StandardMaterial::from(Color::rgb(0.0, 0.0, 0.8))),
        transform: Transform::from_xyz(0., 0., 1.),
        ..Default::default()
    });

    // Spawn our camera
    commands
        .spawn_bundle(Camera3dBundle {
            transform: Transform::from_xyz(5., 0., 5.),
            ..default()
        })
        .insert(AtmosphereCamera(None)) // Marks camera as having an atmosphere that isn't on a specific render layer
        // (the default; in local multiplayer games, we need a way to hide multiple skyboxes from the players)
        .insert(bevy_flycam::FlyCam); // Marks camera as flycam (specific to bevy_flycam)
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
