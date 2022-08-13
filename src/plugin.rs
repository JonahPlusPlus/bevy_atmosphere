use bevy::{
    asset::load_internal_asset,
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::{
        camera::{CameraProjection, Projection},
        render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
        view::RenderLayers,
    },
};

use crate::pipeline::{self, *};

/// Sets up the atmosphere and the systems that control it
///
/// Follows the first camera it finds
#[derive(Default)]
pub struct AtmospherePlugin;

/// Label for the startup system that prepares skyboxes
pub const ATMOSPHERE_INIT: &str = "ATMOSPHERE_INIT";

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

        app.add_plugin(crate::pipeline::AtmospherePipelinePlugin);

        #[cfg(feature = "automatic")]
        app.add_startup_system_to_stage(
            StartupStage::PostStartup,
            atmosphere_init.label(ATMOSPHERE_INIT),
        );

        app.add_system(atmosphere_cancel_rotation);
    }
}

/// Camera that receives an atmosphere skybox
#[derive(Component)]
pub struct AtmosphereCamera(pub Option<u8>);

/// Skybox that renders atmosphere
#[derive(Component)]
pub struct AtmosphereSkyBox;

#[cfg(feature = "automatic")]
fn atmosphere_init(
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut atmosphere_cameras: Query<(Entity, &Projection, &AtmosphereCamera)>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
    mut image_assets: ResMut<Assets<Image>>,
) {
    let mut image = Image::new_fill(
        Extent3d {
            width: pipeline::SIZE * 6,
            height: pipeline::SIZE,
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
    let skybox_material_handle = material_assets.add(StandardMaterial {
        base_color_texture: Some(image_handle),
        unlit: true,
        ..default()
    });

    debug!(
        "Found '{}' `AtmosphereCamera`s",
        atmosphere_cameras.iter().len()
    );

    for (camera, projection, atmosphere_camera) in &mut atmosphere_cameras {
        trace!("Adding skybox to camera entity (ID:{:?})", camera);
        commands
            .entity(camera)
            .insert_bundle(VisibilityBundle {
                visibility: Visibility { is_visible: true },
                ..default()
            })
            .with_children(|c| {
                let mut child = c.spawn_bundle(MaterialMeshBundle {
                    mesh: mesh_assets.add(crate::skybox::mesh(projection.far())),
                    material: skybox_material_handle.clone(),
                    ..default()
                });
                
                child
                    .insert(AtmosphereSkyBox)
                    .insert(NotShadowCaster)
                    .insert(NotShadowReceiver);

                if let AtmosphereCamera(Some(render_layer)) = atmosphere_camera {
                    child.insert(RenderLayers::layer(*render_layer));
                }
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
            debug!("Did not get transform of skybox parent");
        }
    }
}
