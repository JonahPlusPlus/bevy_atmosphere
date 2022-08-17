use bevy::{
    asset::load_internal_asset,
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::{
        camera::{CameraProjection, Projection},
        render_resource::Extent3d,
        view::RenderLayers,
    },
};

use crate::{
    pipeline::*,
    settings::AtmosphereSettings,
    skybox::{AtmosphereSkyBoxMaterial, SkyBoxMaterial, ATMOSPHERE_SKYBOX_SHADER_HANDLE},
};

/// Label for the startup system that prepares skyboxes
pub const ATMOSPHERE_INIT: &str = "ATMOSPHERE_INIT";

/// A [Plugin] that adds the prerequisites for a procedural sky
#[derive(Debug, Clone, Copy)]
pub struct AtmospherePlugin;

impl Plugin for AtmospherePlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            ATMOSPHERE_SKYBOX_SHADER_HANDLE,
            "shaders/skybox.wgsl",
            Shader::from_wgsl
        );

        app.add_plugin(MaterialPlugin::<SkyBoxMaterial>::default());

        app.add_plugin(AtmospherePipelinePlugin);

        {
            let image_handle = {
                let image = app.world.resource::<AtmosphereImage>();
                image.handle.clone()
            };
            let mut material_assets = app.world.resource_mut::<Assets<SkyBoxMaterial>>();
            let material = material_assets.add(SkyBoxMaterial {
                sky_texture: image_handle,
            });
            app.insert_resource(AtmosphereSkyBoxMaterial(material));
        }

        #[cfg(feature = "init")]
        app.add_startup_system_to_stage(
            StartupStage::PostStartup,
            atmosphere_init.label(ATMOSPHERE_INIT),
        );

        app.add_system(atmosphere_cancel_rotation)
            .add_system(atmosphere_settings_changed);
    }
}

/// Marker for a `Camera` that receives a skybox
///
/// When added before the `ATMOSPHERE_INIT` stage, a skybox will be added
/// This behaviour can be disabled by turning off the "automatic" feature
///
/// `Some(u8)` specifies the `RenderLayers` for the skybox to be on
/// `None` doesn't add the `RenderLayers` component
#[derive(Component, Debug, Clone, Copy)]
pub struct AtmosphereCamera(pub Option<u8>);

/// Marker for skyboxes
///
/// Automatically added to skyboxes generated in the `ATMOSPHERE_INIT` stage
#[derive(Component, Debug, Clone, Copy)]
pub struct AtmosphereSkyBox;

#[cfg(feature = "init")]
fn atmosphere_init(
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    material: Res<AtmosphereSkyBoxMaterial>,
    atmosphere_cameras: Query<(Entity, &Projection, &AtmosphereCamera)>,
) {
    // Spawn atmosphere skyboxes
    debug!(
        "Found '{}' `AtmosphereCamera`s",
        atmosphere_cameras.iter().len()
    );

    for (camera, projection, atmosphere_camera) in &atmosphere_cameras {
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
                    material: material.0.clone(),
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

// Whenever settings are changed, resize the image to the appropriate size
fn atmosphere_settings_changed(
    mut image_assets: ResMut<Assets<Image>>,
    mut material_assets: ResMut<Assets<SkyBoxMaterial>>,
    atmosphere_image: Res<AtmosphereImage>,
    settings: Option<Res<AtmosphereSettings>>,
    material: Res<AtmosphereSkyBoxMaterial>,
) {
    if let Some(settings) = settings {
        if settings.is_changed() {
            if let Some(image) = image_assets.get_mut(&atmosphere_image.handle) {
                let size = Extent3d {
                    width: settings.resolution,
                    height: settings.resolution,
                    depth_or_array_layers: 6,
                };
                image.resize(size);
                let _ = material_assets.get_mut(&material.0);
            }
        }
    }
}
