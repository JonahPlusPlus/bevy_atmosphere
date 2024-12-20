//! Provides a `Plugin` for making skyboxes with procedural sky textures.

use bevy::{
    asset::load_internal_asset,
    prelude::*,
    render::{view::RenderLayers, RenderApp},
};

use crate::{
    pipeline::*,
    skybox::{AtmosphereSkyBoxMaterial, SkyBoxMaterial, ATMOSPHERE_SKYBOX_SHADER_HANDLE},
};

#[cfg(feature = "detection")]
use crate::settings::{AtmosphereSettings, SkyboxCreationMode};
#[cfg(feature = "detection")]
use bevy::{
    pbr::{NotShadowCaster, NotShadowReceiver},
    render::camera::CameraProjection as _,
};

#[cfg(any(feature = "gradient", feature = "nishita"))]
use crate::model::AddAtmosphereModel as _;

/// A `Plugin` that adds the prerequisites for a procedural sky.
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

        app.add_plugins(MaterialPlugin::<SkyBoxMaterial>::default());

        #[cfg(feature = "procedural")]
        app.add_plugins(AtmospherePipelinePlugin);

        {
            let image_handle = {
                let image = app.world().get_resource::<AtmosphereImage>().expect("`AtmosphereImage` missing! If the `procedural` feature is disabled, add the resource before `AtmospherePlugin`");
                image.handle.clone()
            };

            #[cfg(feature = "dithering")]
            let settings = {
                let settings = app
                    .world()
                    .get_resource::<crate::settings::AtmosphereSettings>();
                settings.copied().unwrap_or_default()
            };

            let mut material_assets = app.world_mut().resource_mut::<Assets<SkyBoxMaterial>>();
            let material = material_assets.add(SkyBoxMaterial {
                sky_texture: image_handle,
                #[cfg(feature = "dithering")]
                dithering: settings.dithering,
            });

            app.insert_resource(AtmosphereSkyBoxMaterial(material));
        }

        #[cfg(feature = "detection")]
        {
            app.add_systems(PostUpdate, (atmosphere_insert, atmosphere_remove));
        }

        app.add_systems(Update, atmosphere_cancel_rotation);
    }

    fn finish(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);

        render_app.init_resource::<AtmosphereImageBindGroupLayout>();

        #[cfg(feature = "gradient")]
        app.add_atmosphere_model::<crate::collection::gradient::Gradient>();

        #[cfg(feature = "nishita")]
        app.add_atmosphere_model::<crate::collection::nishita::Nishita>();
    }
}

/// A marker `Component` for a `Camera` that receives a skybox.
///
/// When added, a skybox will be created as a child.
/// When removed, that skybox will also be removed.
/// This behaviour can be disabled by turning off the "detection" feature.
#[derive(Component, Default, Debug, Clone)]
pub struct AtmosphereCamera {
    /// Controls whether or not the skybox will be seen only on certain render layers.
    pub render_layers: Option<RenderLayers>,
}

/// A marker `Component` for skybox entities.
///
/// Added for the skybox generated when an `AtmosphereCamera` is detected by the "detection" feature.
#[derive(Component, Debug, Clone, Copy)]
pub struct AtmosphereSkyBox;

/// Inserts a skybox when the `AtmosphereCamera` component is added.
#[cfg(feature = "detection")]
fn atmosphere_insert(
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    settings: Option<Res<AtmosphereSettings>>,
    material: Res<AtmosphereSkyBoxMaterial>,
    atmosphere_cameras: Query<(Entity, &Projection, &AtmosphereCamera), Added<AtmosphereCamera>>,
) {
    let skybox_creation_mode = match settings {
        Some(settings) => settings.skybox_creation_mode,
        None => SkyboxCreationMode::default(),
    };

    for (camera, projection, atmosphere_camera) in &atmosphere_cameras {
        let far_size = match skybox_creation_mode {
            // TODO: Use `unwrap_or(fallback)` when `projection.far()` becomes an `Option<f32>`
            SkyboxCreationMode::FromProjectionFarWithFallback(_fallback) => projection.far(),
            SkyboxCreationMode::FromSpecifiedFar(specified) => specified,
        };

        trace!(
            "Adding skybox to camera entity (ID:{:?}) with far_size {}",
            camera,
            far_size
        );

        commands
            .entity(camera)
            .insert(Visibility::Visible)
            .with_children(|c| {
                let mut child = c.spawn((
                    Mesh3d(mesh_assets.add(crate::skybox::mesh(far_size))),
                    MeshMaterial3d(material.0.clone()),
                    AtmosphereSkyBox,
                    NotShadowCaster,
                    NotShadowReceiver,
                ));

                if let AtmosphereCamera {
                    render_layers: Some(render_layers),
                } = atmosphere_camera
                {
                    child.insert(render_layers.clone());
                }
            });
    }
}

/// Removes the skybox when the `AtmosphereCamera` component is removed.
#[cfg(feature = "detection")]
fn atmosphere_remove(
    mut commands: Commands,
    parents: Query<&Children>,
    atmosphere_skyboxes: Query<Entity, With<AtmosphereSkyBox>>,
    mut atmosphere_cameras: RemovedComponents<AtmosphereCamera>,
) {
    for camera in &mut atmosphere_cameras.read() {
        trace!("Removing skybox from camera entity (ID:{:?})", camera);
        let Ok(children) = parents.get(camera) else {
            error!("Failed to get skybox children entities from camera entity.");
            continue;
        };

        for child in children {
            let Ok(skybox_entity) = atmosphere_skyboxes.get(*child) else {
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
