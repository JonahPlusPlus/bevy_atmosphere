use bevy::{asset::load_internal_asset, prelude::*, render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages}};

use crate::{pipeline::*, settings::*};

/// Sets up the atmosphere and the systems that control it
///
/// Follows the first camera it finds
#[derive(Default)]
pub struct AtmospherePlugin(pub Option<AtmosphereSettings>);

/// Label for startup system that prepares skyboxes
pub const ATMOSPHERE_INIT: &'static str = "ATMOSPHERE_INIT";

impl Plugin for AtmospherePlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            ATMOSPHERE_TYPES_SHADER_HANDLE,
            "shaders/types.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            ATMOSPHERE_MATH_SHADER_HANDLE,
            "shaders/math.wgsl",
            Shader::from_wgsl
        );

        load_internal_asset!(
            app,
            ATMOSPHERE_MAIN_SHADER_HANDLE,
            "shaders/main.wgsl",
            Shader::from_wgsl
        );

        let settings = match &self.0 {
            Some(s) => s.clone(),
            None => default(),
        };

        app.insert_resource(settings.clone());

        app.add_plugin(crate::pipeline::AtmospherePipelinePlugin(settings));

        app.add_startup_system_to_stage(
            StartupStage::PostStartup,
            atmosphere_init.label(ATMOSPHERE_INIT),
        );

        app.add_system(atmosphere_cancel_rotation);
    }
}

/// Camera that receives an atmosphere skybox
#[derive(Component)]
pub struct AtmosphereCamera;

/// Skybox that renders atmosphere
#[derive(Component)]
pub struct AtmosphereSkyBox;

/// Plane that the atmosphere texture is rendered on
#[derive(Component)]
pub struct AtmosphereRenderPlane;

/// Camera that handles rendering the skybox texture
#[derive(Component)]
pub struct AtmosphereRenderCamera;

fn atmosphere_init(
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut atmosphere_cameras: Query<Entity, With<AtmosphereCamera>>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
    mut image_assets: ResMut<Assets<Image>>,
    settings: Res<AtmosphereSettings>,
) {
    let mut image = Image::new_fill(
        Extent3d {
            width: settings.size * 8 * 6 + 12,
            height: settings.size * 8 + 2,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8Unorm,
    );

    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;

    let image_handle = image_assets.add(image);

    commands.insert_resource(AtmosphereImage(image_handle.clone()));

    // Spawn atmosphere skyboxes
    let image_material_handle = material_assets.add(StandardMaterial {
        base_color_texture: Some(image_handle),
        unlit: true,
        ..default()
    });

    debug!(
        "Found '{}' `AtmosphereCamera`s",
        atmosphere_cameras.iter().len()
    );

    commands.spawn_bundle(PbrBundle {
        mesh: mesh_assets.add(crate::mesh::skybox_mesh(settings.size as f32)),
        material: image_material_handle.clone(),
        ..default()
    })
    .insert(AtmosphereSkyBox);

    for camera in &mut atmosphere_cameras {
        trace!("Adding skybox to camera entity (ID:{:?})", camera);
        commands
            .entity(camera)
            .insert_bundle(VisibilityBundle {
                visibility: Visibility { is_visible: true },
                ..default()
            })
            .with_children(|p| {
                // p.spawn_bundle(PbrBundle {
                //     mesh: mesh_assets.add(crate::mesh::skybox_mesh(settings.resolution as f32)),
                //     material: image_material_handle.clone(),
                //     ..default()
                // })
                // .insert(AtmosphereSkyBox);
            });
    }
}

// Cancels the rotation of the camera
fn atmosphere_cancel_rotation(
    mut atmosphere_sky_boxes: Query<(&mut Transform, &Parent), With<AtmosphereSkyBox>>,
    atmosphere_cameras: Query<&GlobalTransform, With<AtmosphereCamera>>,
) {
    for (mut transform, parent) in &mut atmosphere_sky_boxes {
        if let Ok(parent_transform) = atmosphere_cameras.get(parent.get()) {
            let (_, parent_rotation, _) = parent_transform.to_scale_rotation_translation();
            transform.rotation = parent_rotation.inverse();
        } else {
            warn!("Failed to get transform of skybox parent");
        }
    }
}
