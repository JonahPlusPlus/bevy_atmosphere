use bevy::prelude::*;
use bevy_atmosphere::*;
use bevy_flycam::PlayerPlugin;


fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(AtmosphereMat::default())// Default AtmosphereMat, we can edit it to simulate another planet
        .add_plugins(DefaultPlugins)
        .add_plugin(PlayerPlugin)// Simple movement for this example
        .add_plugin(AtmospherePlugin { dynamic: true }) // Dynamic is set to true so that the material is updated when the SkyMaterial resource is edited. If it was not set to true, we would have to update ourselves.
        .add_startup_system(setup_environment.system())
        .add_system(daylight_cycle.system())
        .run();
}

// Marker for updating the position of the light, not needed unless we have multiple lights
struct Sun;

// We can edit the SkyMaterial resource and it will be updated automatically, as long as ZephyrPlugin.dynamic is true
fn daylight_cycle(mut sky_mat: ResMut<AtmosphereMat>, mut query: Query<&mut Transform, With<Sun>>, time: Res<Time>) {
    let mut pos = sky_mat.sun_position;
    let t = time.time_since_startup().as_millis() as f32 / 2000.0;
    pos.y = t.sin();
    pos.z = t.cos();
    sky_mat.sun_position = pos;

    // Since Bevy doesn't have directional lights, so we are just moving a point light around the cube.
    if let Some(mut light_trans) = query.iter_mut().next() {
        light_trans.translation = pos * 5.0;
    }
}

// Simple environment
fn setup_environment(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // Our Sun
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_translation(Vec3::new(2.0, 1.0, 2.5)),
        ..Default::default()
    })
        .insert(Sun); // Marks the light as Sun

    // Simple cube just for reference
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(StandardMaterial::from(Color::rgb(1.0, 0.0, 0.5))),
        ..Default::default()
    });
}