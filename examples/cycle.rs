use bevy::{pbr::light_consts::lux::AMBIENT_DAYLIGHT, prelude::*};
use bevy_atmosphere::prelude::*;
use bevy_spectator::{Spectator, SpectatorPlugin};

fn main() {
    App::new()
        .insert_resource(AtmosphereModel::default()) // Default Atmosphere material, we can edit it to simulate another planet
        .insert_resource(CycleTimer(Timer::new(
            std::time::Duration::from_millis(50), // Update our atmosphere every 50ms (in a real game, this would be much slower, but for the sake of an example we use a faster update)
            TimerMode::Repeating,
        )))
        .add_plugins((
            DefaultPlugins,
            SpectatorPlugin,  // Simple movement for this example
            AtmospherePlugin, // Default AtmospherePlugin
        ))
        .add_systems(Startup, setup_environment)
        .add_systems(Update, daylight_cycle)
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
    mut atmosphere: AtmosphereMut<Nishita>,
    mut query: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>,
    mut timer: ResMut<CycleTimer>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());

    if timer.0.finished() {
        let t = time.elapsed_secs_wrapped() / 2.0;
        atmosphere.sun_position = Vec3::new(0., t.sin(), t.cos());

        if let Some(Ok((mut light_trans, mut directional))) = query.single_mut().into() {
            light_trans.rotation = Quat::from_rotation_x(-t);
            directional.illuminance = t.sin().max(0.0).powf(2.0) * AMBIENT_DAYLIGHT;
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
    commands.spawn((
        DirectionalLight::default(),
        Sun, // Marks the light as Sun
    ));

    // Simple transform shape just for reference
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(StandardMaterial::from(Color::srgb(0.8, 0.8, 0.8)))),
    ));

    // X axis
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
        MeshMaterial3d(materials.add(StandardMaterial::from(Color::srgb(0.8, 0.0, 0.0)))),
        Transform::from_xyz(1., 0., 0.),
    ));

    // Y axis
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
        MeshMaterial3d(materials.add(StandardMaterial::from(Color::srgb(0.0, 0.8, 0.0)))),
        Transform::from_xyz(0., 1., 0.),
    ));

    // Z axis
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))),
        MeshMaterial3d(materials.add(StandardMaterial::from(Color::srgb(0.0, 0.0, 0.8)))),
        Transform::from_xyz(0., 0., 1.),
    ));

    // Spawn our camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(5., 0., 5.),
        Msaa::Sample4,
        AtmosphereCamera::default(), // Marks camera as having a skybox, by default it doesn't specify the render layers the skybox can be seen on
        Spectator,                   // Marks camera as spectator (specific to bevy_spectator)
    ));
}
