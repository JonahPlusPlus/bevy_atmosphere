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
//!         .add_startup_system(setup.system())
//!         .run();
//! }
//!
//! fn setup(mut commands: Commands) {
//!     commands.spawn_bundle(PerspectiveCameraBundle::default());
//! }
//! ```

use std::ops::Deref;
use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy::render::pipeline::PipelineDescriptor;
use bevy::render::render_graph::RenderGraph;

mod material;
pub use material::AtmosphereMat;

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
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<AtmosphereMat>();
        app.add_startup_system(atmosphere_add_sky_sphere.system());
        app.add_system(atmosphere_sky_follow.system());
        if self.dynamic {
            app.add_system(atmosphere_dynamic_sky.system());
        }
    }
}

fn atmosphere_add_sky_sphere(mut commands: Commands,
                             mut meshes: ResMut<Assets<Mesh>>,
                             mut sky_materials: ResMut<Assets<AtmosphereMat>>,
                             pipelines: ResMut<Assets<PipelineDescriptor>>,
                             shaders: ResMut<Assets<Shader>>,
                             render_graph: ResMut<RenderGraph>,
                             config: Option<Res<AtmosphereMat>>
) {

    let render_pipelines = AtmosphereMat::pipeline(pipelines, shaders, render_graph);

    let sky_material = match config {
        None => AtmosphereMat::default(),
        Some(c) => c.deref().clone()
    };

    let sky_material = sky_materials.add(sky_material);

    commands.spawn_bundle(MeshBundle {
        mesh: meshes.add(Mesh::from(shape::Icosphere { radius: -10.0, subdivisions: 2 })),
        render_pipelines: render_pipelines.clone(),
        ..Default::default()
    })
        .insert(sky_material);
}

fn atmosphere_sky_follow(camera_transform_query: Query<&Transform, (With<Camera>, Without<Handle<AtmosphereMat>>)>, mut sky_transform_query: Query<&mut Transform, With<Handle<AtmosphereMat>>>) {
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