//! A procedural sky plugin for bevy
//!
//! ## Example
//! ```
//! use bevy::prelude::*;
//! use bevy_atmosphere::*;
//!
//! fn main() {
//!     App::build()
//!         .insert_resource(bevy_atmosphere::AtmosphereMat::default()) // Default Earth sky
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(bevy_atmosphere::AtmospherePlugin { dynamic: false }) // Set to false since we aren't changing the sky's appearance
//!         .add_startup_system(setup)
//!         .run();
//! }
//!
//! fn setup(mut commands: Commands) {
//!     commands.spawn_bundle(PerspectiveCameraBundle::default());
//! }
//! ```

use std::ops::Deref;
use bevy::{prelude::*, pbr::NotShadowCaster};
use bevy::render::camera::Camera;

mod material;
pub use material::AtmosphereMat;
use material::{SKY_VERTEX_SHADER_HANDLE, SKY_FRAGMENT_SHADER_HANDLE};
use naga::ShaderStage;

const SKY_VERTEX_SHADER: &str = include_str!("shaders/sky.vert");
const SKY_FRAGMENT_SHADER: &str = include_str!("shaders/sky.frag");

/// Sets up the atmosphere and the systems that control it
///
/// Follows the first camera it finds
#[derive(Default)]
pub struct AtmospherePlugin {
    /// If set to `true`, whenever the [`AtmosphereMat`](crate::AtmosphereMat) resource (if it exists) is changed, the sky is updated
    ///
    /// If set to `false`, whenever the sky needs to be updated, it will have to be done manually through a system
    ///
    /// To update the sky manually in a system, you will need the [`AtmosphereMat`](crate::AtmosphereMat) resource, a [`Handle`](bevy::asset::Handle) to the [`AtmosphereMat`](crate::AtmosphereMat) used and the [`Assets`](bevy::asset::Assets) that stores the [`AtmosphereMat`](crate::AtmosphereMat)
    /// ### Example
    /// ```
    /// use bevy::prelude::*;
    /// use bevy_atmosphere::AtmosphereMat;
    /// use std::ops::Deref;
    ///
    /// fn atmosphere_dynamic_sky(config: Res<AtmosphereMat>, sky_mat_query: Query<&Handle<AtmosphereMat>>, mut sky_materials: ResMut<Assets<AtmosphereMat>>) {
    ///     if config.is_changed() {
    ///         if let Some(sky_mat_handle) = sky_mat_query.iter().next() {
    ///             if let Some(sky_mat) = sky_materials.get_mut(sky_mat_handle) {
    ///                 *sky_mat = config.deref().clone();
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    pub dynamic: bool
}

impl Plugin for AtmospherePlugin {
    fn build(&self, app: &mut App) {
        let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();
        shaders.set_untracked(
            SKY_VERTEX_SHADER_HANDLE,
            Shader::from_glsl(SKY_VERTEX_SHADER, ShaderStage::Vertex),
        );
        shaders.set_untracked(
            SKY_FRAGMENT_SHADER_HANDLE,
            Shader::from_glsl(SKY_FRAGMENT_SHADER, ShaderStage::Fragment),
        );

        app.add_plugin(MaterialPlugin::<AtmosphereMat>::default());
        app.add_startup_system(atmosphere_add_sky_sphere);
        app.add_system_to_stage(CoreStage::Last, // Should run after transform_propagate_system
                                atmosphere_sky_follow);
        if self.dynamic {
            app.add_system(atmosphere_dynamic_sky);
        }
    }
}

fn atmosphere_add_sky_sphere(mut commands: Commands,
                             mut meshes: ResMut<Assets<Mesh>>,
                             mut sky_materials: ResMut<Assets<AtmosphereMat>>,
                             config: Option<Res<AtmosphereMat>>
) {
    let sky_material = match config {
        None => AtmosphereMat::default(),
        Some(c) => c.deref().clone()
    };

    let sky_material = sky_materials.add(sky_material);

    commands.spawn_bundle(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(shape::Icosphere { radius: -10.0, subdivisions: 2 })),
        material: sky_material,
        ..Default::default()
    }).insert(NotShadowCaster);
}

fn atmosphere_sky_follow(camera_transform_query: Query<&GlobalTransform, (With<Camera>, Without<Handle<AtmosphereMat>>)>, mut sky_transform_query: Query<&mut GlobalTransform, With<Handle<AtmosphereMat>>>) {
    if let Some(camera_transform) = camera_transform_query.iter().next() {
        if let Some(mut sky_transform) = sky_transform_query.iter_mut().next() {
            sky_transform.translation = camera_transform.translation;
        }
    }
}

fn atmosphere_dynamic_sky(config: Res<AtmosphereMat>, sky_mat_query: Query<&Handle<AtmosphereMat>>, mut sky_materials: ResMut<Assets<AtmosphereMat>>) {
    if config.is_changed() {
        if let Some(sky_mat_handle) = sky_mat_query.iter().next() {
            if let Some(sky_mat) = sky_materials.get_mut(sky_mat_handle) {
                *sky_mat = config.deref().clone();
            }
        }
    }
}
