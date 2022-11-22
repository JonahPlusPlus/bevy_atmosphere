//! Provides a [`Plugin`] for making skyboxes with procedural sky textures.

use bevy::{
    asset::load_internal_asset,
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::{
        camera::{CameraProjection, Projection},
        view::RenderLayers,
    },
};

use crate::{
    model::AddAtmosphereModel,
    pipeline::*,
    skybox::{AtmosphereSkyBoxMaterial, SkyBoxMaterial, ATMOSPHERE_SKYBOX_SHADER_HANDLE},
};

/// A [`Plugin`] that adds the prerequisites for a procedural sky.
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

        #[cfg(feature = "nishita")]
        app.add_atmosphere_model::<crate::model::nishita::Nishita>();

        app.add_plugin(MaterialPlugin::<SkyBoxMaterial>::default());

        #[cfg(feature = "procedural")]
        app.add_plugin(AtmospherePipelinePlugin);

        {
            let image_handle = {
                let image = app.world.get_resource::<AtmosphereImage>().expect("`AtmosphereImage` missing! If the `procedural` feature is disabled, add the resource before `AtmospherePlugin`");
                image.handle.clone()
            };
            let mut material_assets = app.world.resource_mut::<Assets<SkyBoxMaterial>>();
            let material = material_assets.add(SkyBoxMaterial {
                sky_texture: image_handle,
            });

            app.insert_resource(AtmosphereSkyBoxMaterial(material));
        }

        #[cfg(feature = "detection")]
        {
            app.add_system_set_to_stage(
                CoreStage::PostUpdate,
                SystemSet::new()
                    .with_system(atmosphere_insert)
                    .with_system(atmosphere_remove),
            );
        }

        app.add_system(atmosphere_cancel_rotation);
    }
}

/// Marker for a [`Camera`] that receives a skybox.
///
/// When added, a skybox will be created as a child.
/// This behaviour can be disabled by turning off the "detection" feature.
///
/// `Some(u8)` specifies the [`RenderLayers`] for the skybox to be on.
/// `None` doesn't add the [`RenderLayers`] component.
#[derive(Component, Debug, Clone, Copy)]
pub struct AtmosphereCamera(pub Option<u8>);

/// Marker for skybox entities.
///
/// Added for skybox generated when a [`AtmosphereCamera`] is detected by the "detection" feature.
#[derive(Component, Debug, Clone, Copy)]
pub struct AtmosphereSkyBox;

/// Inserts a skybox when the [`AtmosphereCamera`] component is added.
#[cfg(feature = "detection")]
fn atmosphere_insert(
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    material: Res<AtmosphereSkyBoxMaterial>,
    atmosphere_cameras: Query<(Entity, &Projection, &AtmosphereCamera), Added<AtmosphereCamera>>,
) {
    for (camera, projection, atmosphere_camera) in &atmosphere_cameras {
        #[cfg(feature = "bevy/trace")]
        trace!("Adding skybox to camera entity (ID:{:?})", camera);
        commands
            .entity(camera)
            .insert(VisibilityBundle {
                visibility: Visibility { is_visible: true },
                ..default()
            })
            .with_children(|c| {
                let mut child = c.spawn((
                    MaterialMeshBundle {
                        mesh: mesh_assets.add(crate::skybox::mesh(projection.far())),
                        material: material.0.clone(),
                        ..default()
                    },
                    AtmosphereSkyBox,
                    NotShadowCaster,
                    NotShadowReceiver,
                ));

                if let AtmosphereCamera(Some(render_layer)) = atmosphere_camera {
                    child.insert(RenderLayers::layer(*render_layer));
                }
            });
    }
}

/// Removes the skybox when the [`AtmosphereCamera`] component is removed.
#[cfg(feature = "detection")]
fn atmosphere_remove(
    mut commands: Commands,
    parents: Query<&Children>,
    atmosphere_skyboxes: Query<Entity, With<AtmosphereSkyBox>>,
    atmosphere_cameras: RemovedComponents<AtmosphereCamera>,
) {
    for camera in &atmosphere_cameras {
        #[cfg(feature = "bevy/trace")]
        trace!("Removing skybox from camera entity (ID:{:?})", camera);
        let Ok(children) = parents.get(camera) else {
            error!("Failed to get skybox children entities from camera entity.");
            continue;
        };

        for child in children {
            let Ok(skybox_entity) = atmosphere_skyboxes.get(*child) else {
                #[cfg(feature = "bevy/trace")]
                trace!("Child wasn't found in skybox entities.");
                continue;
            };

            commands.entity(skybox_entity).despawn();
        }
    }
}

/// Cancels the rotation of the camera.
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
