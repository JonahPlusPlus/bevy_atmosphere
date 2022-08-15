use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::{
        camera::{CameraProjection, Projection},
        view::RenderLayers,
    },
};

use crate::pipeline::*;

/// Label for the startup system that prepares skyboxes
pub const ATMOSPHERE_INIT: &str = "ATMOSPHERE_INIT";

/// A [Plugin] that adds the prerequisites for a procedural sky
pub struct AtmospherePlugin<const SIZE: u32>;

impl Default for AtmospherePlugin<1024> {
    fn default() -> Self {
        Self
    }
}

impl<const SIZE: u32> Plugin for AtmospherePlugin<SIZE> {
    fn build(&self, app: &mut App) {
        app.add_plugin(AtmospherePipelinePlugin::<SIZE>);

        #[cfg(feature = "init")]
        app.add_startup_system_to_stage(
            StartupStage::PostStartup,
            atmosphere_init::<SIZE>.label(ATMOSPHERE_INIT),
        );

        app.add_system(atmosphere_cancel_rotation);
    }
}

/// Marker for a `Camera` that receives a skybox
/// 
/// When added before the `ATMOSPHERE_INIT` stage, a skybox will be added
/// This behaviour can be disabled by turning off the "automatic" feature
/// 
/// `Some(u8)` specifies the `RenderLayers` for the skybox to be on
/// `None` doesn't add the `RenderLayers` component
#[derive(Component)]
pub struct AtmosphereCamera(pub Option<u8>);

/// Marker for skyboxes
/// 
/// Automatically added to skyboxes generated in the `ATMOSPHERE_INIT` stage
#[derive(Component)]
pub struct AtmosphereSkyBox;

#[cfg(feature = "init")]
fn atmosphere_init<const SIZE: u32>(
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut atmosphere_cameras: Query<(Entity, &Projection, &AtmosphereCamera)>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
    image: Res<AtmosphereImage>,
) {
    // Spawn atmosphere skyboxes
    let skybox_material_handle = material_assets.add(StandardMaterial {
        base_color_texture: Some(image.0.clone()),
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
                    mesh: mesh_assets.add(crate::skybox::mesh(projection.far(), SIZE as f32)),
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
