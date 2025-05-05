use bevy::prelude::*;
use bevy_atmosphere::prelude::*;

fn main() {
    println!("Demonstrates adding/removing an `AtmosphereCamera`\n\t- Left Mouse Button: Add `AtmosphereCamera`\n\t- Right Mouse Button: Remove `AtmosphereCamera`");
    App::new()
        .add_plugins((DefaultPlugins, AtmospherePlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, update)
        .run();
}

#[derive(Component)]
struct PrimaryCamera;

fn setup(mut commands: Commands) {
    commands.spawn((Camera3d::default(), PrimaryCamera));
}

fn update(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    primary_camera_query: Query<Entity, With<PrimaryCamera>>,
) {
    let Ok(primary_camera) = primary_camera_query.single() else {
        error!("Failed to get a single `PrimaryCamera`");
        return;
    };

    if mouse.just_pressed(MouseButton::Left) {
        commands
            .entity(primary_camera)
            .insert(AtmosphereCamera::default());
        info!("Added `AtmosphereCamera`!");
    } else if mouse.just_pressed(MouseButton::Right) {
        commands.entity(primary_camera).remove::<AtmosphereCamera>();
        info!("Removed `AtmosphereCamera`!");
    }
}
