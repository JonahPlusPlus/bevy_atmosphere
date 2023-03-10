//! Based off the Bevy "Split Screen" example
//! Used to demonstrate how multiple skyboxes could be made for a local multiplayer game

use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    render::{camera::Viewport, view::RenderLayers},
    window::WindowResized,
};
use bevy_atmosphere::prelude::*;
use bevy_spectator::*;

fn main() {
    println!("Demonstrates using `AtmosphereCamera.render_layers` to have multiple skyboxes in the scene at once\n\t- E: Switch camera");
    App::new()
        .insert_resource(Msaa::Sample4)
        .insert_resource(AtmosphereModel::new(Nishita {
            rayleigh_coefficient: Vec3::new(22.4e-6, 5.5e-6, 13.0e-6), // Change rayleigh coefficient to change color
            ..default()
        }))
        .add_plugins(DefaultPlugins)
        .add_plugin(AtmospherePlugin)
        .add_plugin(SpectatorPlugin)
        .add_startup_system(setup)
        .add_system(set_camera_viewports)
        .add_system(switch_camera)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane {
            size: 100.0,
            subdivisions: 0,
        })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });

    // Light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            1.0,
            -std::f32::consts::FRAC_PI_4,
        )),
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });

    // Spawn left screen camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 25.0, -100.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        RenderLayers::from_layers(&[0, 1]), // To prevent each player from seeing the other skybox, we put each one on a separate render layer (you could also use this render layer for other player specific effects)
        AtmosphereCamera {
            render_layers: Some(RenderLayers::layer(1)),
        },
        LeftCamera,
        Spectator,
    ));

    // Spawn right screen camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(100.0, 50.0, 150.0).looking_at(Vec3::ZERO, Vec3::Y),
            camera: Camera {
                // Renders the right camera after the left camera, which has a default priority of 0
                order: 1,
                ..default()
            },
            camera_3d: Camera3d {
                // Don't clear on the second camera because the first camera already cleared the window
                clear_color: ClearColorConfig::None,
                ..default()
            },
            ..default()
        },
        RenderLayers::from_layers(&[0, 2]),
        AtmosphereCamera {
            render_layers: Some(RenderLayers::layer(2)),
        },
        RightCamera,
        Spectator,
    ));
}

#[derive(Component)]
struct LeftCamera;

#[derive(Component)]
struct RightCamera;

fn set_camera_viewports(
    windows: Query<&Window>,
    mut resize_events: EventReader<WindowResized>,
    mut left_camera: Query<&mut Camera, (With<LeftCamera>, Without<RightCamera>)>,
    mut right_camera: Query<&mut Camera, With<RightCamera>>,
) {
    // We need to dynamically resize the camera's viewports whenever the window size changes
    // so then each camera always takes up half the screen.
    // A resize_event is sent when the window is first created, allowing us to reuse this system for initial setup.
    for resize_event in resize_events.iter() {
        let window = windows.get(resize_event.window).unwrap();
        let mut left_camera = left_camera.single_mut();
        left_camera.viewport = Some(Viewport {
            physical_position: UVec2::new(0, 0),
            physical_size: UVec2::new(window.physical_width() / 2, window.physical_height()),
            ..default()
        });

        let mut right_camera = right_camera.single_mut();
        right_camera.viewport = Some(Viewport {
            physical_position: UVec2::new(window.physical_width() / 2, 0),
            physical_size: UVec2::new(window.physical_width() / 2, window.physical_height()),
            ..default()
        });
    }
}

fn switch_camera(
    mut settings: ResMut<SpectatorSettings>,
    keys: Res<Input<KeyCode>>,
    left_camera: Query<Entity, (With<LeftCamera>, Without<RightCamera>)>,
    right_camera: Query<Entity, With<RightCamera>>,
) {
    let left_camera = left_camera.single();
    let right_camera = right_camera.single();

    if keys.just_pressed(KeyCode::E) {
        if let Some(spectator) = settings.active_spectator {
            if spectator == left_camera {
                settings.active_spectator = Some(right_camera);
            } else {
                settings.active_spectator = Some(left_camera);
            }
        } else {
            settings.active_spectator = Some(left_camera);
        }
        println!("Switched camera");
    }
}
