use bevy::prelude::*;
use bevy_atmosphere::prelude::*;
use bevy_spectator::*;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(Atmosphere::default()) // Default Atmosphere material, we can edit it to simulate another planet
        .insert_resource(CycleTimer(Timer::new(
            bevy::utils::Duration::from_millis(50), // Update our atmosphere every 50ms (in a real game, this would be much slower, but for the sake of an example we use a faster update)
            TimerMode::Repeating,
        )))
        .add_plugins(DefaultPlugins)
        .add_plugin(SpectatorPlugin) // Simple movement for this example
        .add_plugin(AtmospherePlugin) // Default AtmospherePlugin
        .add_startup_system(setup_environment)
        .add_system(daylight_cycle)
        .run();
}

// Marker for updating the position of the light, not needed unless we have multiple lights
#[derive(Component)]
struct Sun;

// Timer for updating the daylight cycle (updating the atmosphere every frame is slow, so it's better to do incremental changes)
#[derive(Resource)]
struct CycleTimer(Timer);

// We can edit the Atmosphere resource and it will be updated automatically
fn daylight_cycle(
    mut atmosphere: ResMut<Atmosphere>,
    mut query: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    mut timer: ResMut<CycleTimer>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());

    if timer.0.finished() {
        let t = time.elapsed_seconds_wrapped() as f32 / 2.0;
        atmosphere.sun_position = Vec3::new(0., t.sin(), t.cos());

        if let Some((mut light_trans, mut directional)) = query.single_mut().into() {
            light_trans.rotation = Quat::from_rotation_x(-t.sin().atan2(t.cos()));
            directional.illuminance = t.sin().max(0.0).powf(2.0) * 100000.0;
        }
    }
}

// Simple environment
fn setup_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Our Sun
    commands
        .spawn((
            DirectionalLightBundle {
                ..Default::default()
            },
            Sun // Marks the light as Sun
        ));

    // Simple transform shape just for reference
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(StandardMaterial::from(Color::rgb(0.8, 0.8, 0.8))),
        ..Default::default()
    });

    // X axis
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.5 })),
        material: materials.add(StandardMaterial::from(Color::rgb(0.8, 0.0, 0.0))),
        transform: Transform::from_xyz(1., 0., 0.),
        ..Default::default()
    });

    // Y axis
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.5 })),
        material: materials.add(StandardMaterial::from(Color::rgb(0.0, 0.8, 0.0))),
        transform: Transform::from_xyz(0., 1., 0.),
        ..Default::default()
    });

    // Z axis
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.5 })),
        material: materials.add(StandardMaterial::from(Color::rgb(0.0, 0.0, 0.8))),
        transform: Transform::from_xyz(0., 0., 1.),
        ..Default::default()
    });

    // Spawn our camera
    commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(5., 0., 5.),
                ..default()
            },
            AtmosphereCamera(None),// Marks camera as having an atmosphere that isn't on a specific render layer
            // (the default; in local multiplayer games, we need a way to hide multiple skyboxes from the players)
            Spectator// Marks camera as spectator (specific to bevy_spectator)
        ));
}
