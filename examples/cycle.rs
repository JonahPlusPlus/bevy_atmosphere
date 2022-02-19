use bevy::prelude::*;
use bevy_atmosphere::*;
use bevy_flycam::PlayerPlugin;


fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(AtmosphereMat::default())// Default AtmosphereMat, we can edit it to simulate another planet
        .add_plugins(DefaultPlugins)
        .add_plugin(PlayerPlugin)// Simple movement for this example
        .add_plugin(AtmospherePlugin { dynamic: true }) // Dynamic is set to true so that the material is updated when the SkyMaterial resource is edited. If it was not set to true, we would have to update ourselves.
        .add_startup_system(setup_environment)
        .add_system(daylight_cycle)
        .run();
}

// Marker for updating the position of the light, not needed unless we have multiple lights
#[derive(Component)]
struct Sun;

// We can edit the SkyMaterial resource and it will be updated automatically, as long as ZephyrPlugin.dynamic is true
fn daylight_cycle(mut sky_mat: ResMut<AtmosphereMat>, mut query: Query<(&mut Transform, &mut DirectionalLight), With<Sun>>, time: Res<Time>) {
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
fn setup_environment(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // Our Sun
    commands.spawn_bundle(DirectionalLightBundle {
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
