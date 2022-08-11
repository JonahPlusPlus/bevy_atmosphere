use bevy::prelude::*;
use bevy_atmosphere::prelude::*;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(Atmosphere::default()) // Default Atmosphere material, we can edit it to simulate another planet
        .insert_resource(WindowDescriptor {
            // uncomment for unthrottled FPS
            present_mode: bevy::window::PresentMode::AutoNoVsync,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_flycam::NoCameraPlayerPlugin) // Simple movement for this example
        .add_plugin(AtmospherePlugin(Some(AtmosphereSettings {
            size: 64,
            ..default()
        }))) // Default AtmospherePlugin
        .add_startup_system(setup_environment)
        .add_system(daylight_cycle)
        .run();
}

// Marker for updating the position of the light, not needed unless we have multiple lights
#[derive(Component)]
struct Sun;

// We can edit the SkyMaterial resource and it will be updated automatically, as long as AtmospherePlugin.dynamic is true
fn daylight_cycle(
    mut sky_mat: ResMut<Atmosphere>,
    mut query: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    time: Res<Time>,
) {
    let mut pos = sky_mat.sun_position;
    let t = time.time_since_startup().as_millis() as f32 / 2000.0;
    pos.y = t.sin();
    pos.z = t.cos();
    sky_mat.sun_position = pos;

    if let Some((mut light_trans, mut directional)) = query.single_mut().into() {
        light_trans.rotation = Quat::from_rotation_x(-pos.y.atan2(pos.z));
        directional.illuminance = t.sin().max(0.0).powf(2.0) * 100000.0;
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
            transform: Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(AtmosphereCamera) // Marks camera as having an atmosphere
        .insert(bevy_flycam::FlyCam); // Marks camera as flycam (specific to bevy_flycam)
}
