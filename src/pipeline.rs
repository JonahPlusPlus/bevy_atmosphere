//! Provides types and logic for a compute pipeline that renders the procedural sky texture.
//!
//! It's possible to use [`AtmospherePipelinePlugin`] with your own custom code to render to custom targets.

use std::ops::Deref;

use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::{PrepareAssetSet, RenderAssets},
        render_graph::{self, RenderGraph},
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
            CachedPipelineState, ComputePassDescriptor, Extent3d, PipelineCache, ShaderStages,
            StorageTextureAccess, TextureAspect, TextureDescriptor, TextureDimension,
            TextureFormat, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
        },
        renderer::RenderDevice,
        texture::FallbackImage,
        Extract, RenderApp, RenderSet, Render,
    },
};

use crate::{
    model::{AtmosphereModel, AtmosphereModelMetadata},
    settings::AtmosphereSettings,
    skybox::{AtmosphereSkyBoxMaterial, SkyBoxMaterial},
};

/// Name of the compute pipeline `render_graph::Node`.
pub const NAME: &str = "bevy_atmosphere";
/// Size of the compute workgroups in the x and y axis.
///
/// Complete workgroup size is (8, 8, 6);
pub const WORKGROUP_SIZE: u32 = 8;

/// The procedural sky `Image` generated by the atmosphere compute pipeline.
///
/// It can be used in a material for a skybox mesh.
#[derive(Resource, ExtractResource, Debug, Clone)]
pub struct AtmosphereImage {
    /// `Handle` to a procedural sky `Image`.
    ///
    /// The `TextureView` associated with this handle is `TextureViewDimension::Cube`.
    pub handle: Handle<Image>,
    /// `TextureView` of the image with `TextureViewDimension::D2Array`.
    pub array_view: Option<TextureView>,
}

/// The `BindGroupLayout` for binding [`AtmosphereImage`] to the compute shader.
#[derive(Resource, Debug, Clone)]
pub struct AtmosphereImageBindGroupLayout(pub BindGroupLayout);

impl FromWorld for AtmosphereImageBindGroupLayout {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        Self(
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("bevy_atmosphere_image_bind_group_layout"),
                entries: &[BindGroupLayoutEntry {
                    // AtmosphereImage
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba16Float,
                        view_dimension: TextureViewDimension::D2Array,
                    },
                    count: None,
                }],
            }),
        )
    }
}

/// Signals the pipeline (inside `RenderApp`) to render the atmosphere.
#[derive(Debug, Clone, Copy, Event)]
pub struct AtmosphereUpdateEvent;

#[derive(Resource, Debug, Clone)]
struct AtmosphereBindGroups(pub BindGroup, pub BindGroup);

#[derive(Resource, Default, Clone)]
struct CachedAtmosphereModelMetadata(pub Option<AtmosphereModelMetadata>);

/// A `Plugin` that creates the compute pipeline for rendering a procedural sky cubemap texture.
#[derive(Debug, Clone, Copy)]
pub struct AtmospherePipelinePlugin;

impl Plugin for AtmospherePipelinePlugin {
    fn build(&self, app: &mut App) {
        let settings = match app.world.get_resource::<AtmosphereSettings>() {
            Some(s) => *s,
            None => default(),
        };

        let atmosphere = match app.world.get_resource::<AtmosphereModel>() {
            Some(a) => a.clone(),
            None => default(),
        };

        let mut image = Image::new_fill(
            Extent3d {
                width: settings.resolution,
                height: settings.resolution,
                depth_or_array_layers: 6,
            },
            TextureDimension::D2,
            &[0; 4 * 4],
            TextureFormat::Rgba16Float,
        );

        image.texture_view_descriptor = Some(ATMOSPHERE_CUBE_TEXTURE_VIEW_DESCRIPTOR);

        image.texture_descriptor = ATMOSPHERE_IMAGE_TEXTURE_DESCRIPTOR(settings.resolution);

        let mut image_assets = app.world.resource_mut::<Assets<Image>>();
        let handle = image_assets.add(image);

        app.insert_resource(AtmosphereImage {
            handle,
            array_view: None,
        });

        app.add_plugins(ExtractResourcePlugin::<AtmosphereImage>::default());

        app.add_systems(Update, atmosphere_settings_changed);

        let type_registry = app.world.resource::<AppTypeRegistry>().clone();

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .insert_resource(atmosphere)
            .insert_resource(settings)
            .insert_resource(AtmosphereTypeRegistry(type_registry))
            .init_resource::<CachedAtmosphereModelMetadata>()
            .init_resource::<AtmosphereImageBindGroupLayout>()
            .init_resource::<Events<AtmosphereUpdateEvent>>()
            .add_systems(ExtractSchedule, extract_atmosphere_resources)
            .add_systems(Render, Events::<AtmosphereUpdateEvent>::update_system.in_set(RenderSet::Prepare))
            .add_systems(PostStartup, prepare_atmosphere_assets.in_set(PrepareAssetSet::PostAssetPrepare))
            .add_systems(Render, queue_atmosphere_bind_group.in_set(RenderSet::Queue)
        );

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node(NAME, AtmosphereNode::default());
        render_graph.add_node_edge(NAME, bevy::render::main_graph::node::CAMERA_DRIVER);
    }
}

/// Whenever settings are changed, resize the image to the appropriate size.
fn atmosphere_settings_changed(
    mut image_assets: ResMut<Assets<Image>>,
    mut material_assets: ResMut<Assets<SkyBoxMaterial>>,
    mut atmosphere_image: ResMut<AtmosphereImage>,
    mut settings_existed: Local<bool>,
    settings: Option<Res<AtmosphereSettings>>,
    material: Res<AtmosphereSkyBoxMaterial>,
) {
    if let Some(settings) = settings {
        if settings.is_changed() {
            #[cfg(feature = "bevy/trace")]
            let _atmosphere_settings_changed_executed_span = info_span!(
                "executed",
                name = "bevy_atmosphere::pipeline::atmosphere_settings_changed"
            )
            .entered();
            if let Some(image) = image_assets.get_mut(&atmosphere_image.handle) {
                if settings.resolution % 8 != 0 {
                    warn!("Resolution is not a multiple of 8, issues may be encountered");
                }
                let size = Extent3d {
                    width: settings.resolution,
                    height: settings.resolution,
                    depth_or_array_layers: 6,
                };
                image.resize(size);
                if let Some(mut skybox_material) = material_assets.get_mut(&material.0) {
                    // `get_mut` tells the material to update, so it's needed anyways
                    skybox_material.dithering = settings.dithering;
                }
                atmosphere_image.array_view = None; // drop the previous texture view
                #[cfg(feature = "bevy/trace")]
                trace!("Resized image to {:?}", size);
            }
        }
        *settings_existed = true;
    } else {
        if *settings_existed {
            #[cfg(feature = "bevy/trace")]
            let _atmosphere_settings_changed_executed_span = info_span!(
                "executed",
                name = "bevy_atmosphere::pipeline::atmosphere_settings_changed"
            )
            .entered();
            if let Some(image) = image_assets.get_mut(&atmosphere_image.handle) {
                let resolution = AtmosphereSettings::default().resolution;
                if resolution % 8 != 0 {
                    warn!("Resolution is not a multiple of 8, issues may be encountered");
                }
                let size = Extent3d {
                    width: resolution,
                    height: resolution,
                    depth_or_array_layers: 6,
                };
                image.resize(size);
                let _ = material_assets.get_mut(&material.0); // `get_mut` tells the material to update
                atmosphere_image.array_view = None; // drop the previous texture view
                #[cfg(feature = "bevy/trace")]
                trace!("Resized image to {:?}", size);
            }
        }
        *settings_existed = false;
    }
}

/// Extracts [`AtmosphereModel`] and [`AtmosphereSettings`] from main world.
#[allow(clippy::too_many_arguments)]
fn extract_atmosphere_resources(
    type_registry: Res<AtmosphereTypeRegistry>,
    mut cached_metadata: ResMut<CachedAtmosphereModelMetadata>,
    main_atmosphere: Extract<Option<Res<AtmosphereModel>>>,
    mut render_atmosphere: ResMut<AtmosphereModel>,
    mut atmosphere_existed: Local<bool>,
    main_settings: Extract<Option<Res<AtmosphereSettings>>>,
    mut render_settings: ResMut<AtmosphereSettings>,
    mut settings_existed: Local<bool>,
) {
    macro_rules! cache_metadata {
        ($id:ident) => {
            *cached_metadata = CachedAtmosphereModelMetadata(Some({
                let type_registry = type_registry.read();
                type_registry
                    .get_type_data::<AtmosphereModelMetadata>($id)
                    .expect("Failed to get type data")
                    .clone()
            }));
        };
    }

    if let Some(atmosphere) = &*main_atmosphere {
        if atmosphere.is_changed() {
            *render_atmosphere = AtmosphereModel::extract_resource(atmosphere);
            let id = render_atmosphere.model().type_id();
            if let CachedAtmosphereModelMetadata(Some(metadata)) = cached_metadata.as_ref() {
                if metadata.id != id {
                    cache_metadata!(id);
                }
            }
        }
        *atmosphere_existed = true;
    } else {
        if *atmosphere_existed {
            *render_atmosphere = AtmosphereModel::extract_resource(&AtmosphereModel::default());
            let id = render_atmosphere.model().type_id();
            cache_metadata!(id);
        }
        *atmosphere_existed = false;
    }

    if let Some(settings) = &*main_settings {
        if settings.is_changed() {
            *render_settings = AtmosphereSettings::extract_resource(settings);
        }
        *settings_existed = true;
    } else {
        if *settings_existed {
            *render_settings = AtmosphereSettings::extract_resource(&AtmosphereSettings::default());
        }
        *settings_existed = false;
    }
}

/// For creating a `TextureView` with `TextureViewDimension::Cube`.
pub const ATMOSPHERE_CUBE_TEXTURE_VIEW_DESCRIPTOR: TextureViewDescriptor = TextureViewDescriptor {
    label: Some("atmosphere_image_array_view"),
    format: Some(TextureFormat::Rgba16Float),
    dimension: Some(TextureViewDimension::Cube),
    aspect: TextureAspect::All,
    base_mip_level: 0,
    mip_level_count: None,
    base_array_layer: 0,
    array_layer_count: Some(6),
};

/// For creating a `TextureView` with `TextureViewDimension::D2Array`.
pub const ATMOSPHERE_ARRAY_TEXTURE_VIEW_DESCRIPTOR: TextureViewDescriptor = TextureViewDescriptor {
    label: Some("atmosphere_image_cube_view"),
    format: Some(TextureFormat::Rgba16Float),
    dimension: Some(TextureViewDimension::D2Array),
    aspect: TextureAspect::All,
    base_mip_level: 0,
    mip_level_count: None,
    base_array_layer: 0,
    array_layer_count: Some(6),
};

/// For creating a `Texture` with 6 layers.
pub const ATMOSPHERE_IMAGE_TEXTURE_DESCRIPTOR: fn(u32) -> TextureDescriptor<'static> =
    |res| TextureDescriptor {
        label: Some("atmosphere_image_texture"),
        size: Extent3d {
            width: res,
            height: res,
            depth_or_array_layers: 6,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba16Float,
        usage: TextureUsages::COPY_DST
            | TextureUsages::STORAGE_BINDING
            | TextureUsages::TEXTURE_BINDING,
        view_formats: &[TextureFormat::Rgba16Float],
    };

/// Whenever settings changed, the texture view needs to be updated to use the new texture.
fn prepare_atmosphere_assets(
    mut update_events: ResMut<Events<AtmosphereUpdateEvent>>,
    mut atmosphere_image: ResMut<AtmosphereImage>,
    gpu_images: Res<RenderAssets<Image>>,
    atmosphere: Res<AtmosphereModel>,
) {
    let mut update = || update_events.send(AtmosphereUpdateEvent);

    if atmosphere_image.array_view.is_none() {
        #[cfg(feature = "bevy/trace")]
        let _prepare_atmosphere_assets_executed_span = info_span!(
            "executed",
            name = "bevy_atmosphere::pipeline::prepare_atmosphere_assets"
        )
        .entered();
        let texture = &gpu_images[&atmosphere_image.handle].texture;
        let view = texture.create_view(&ATMOSPHERE_ARRAY_TEXTURE_VIEW_DESCRIPTOR);
        atmosphere_image.array_view = Some(view);
        update();
        #[cfg(feature = "bevy/trace")]
        trace!(
            "Created new 2D array texture view from atmosphere texture of size {:?}",
            &gpu_images[&atmosphere_image.handle].size
        );
    }

    if atmosphere.is_changed() {
        update();
    }
}

/// Queue the generated bind groups for the compute pipeline.
#[allow(clippy::too_many_arguments)]
fn queue_atmosphere_bind_group(
    mut commands: Commands,
    mut cached_metadata: ResMut<CachedAtmosphereModelMetadata>,
    gpu_images: Res<RenderAssets<Image>>,
    atmosphere_image: Res<AtmosphereImage>,
    render_device: Res<RenderDevice>,
    fallback_image: Res<FallbackImage>,
    type_registry: Res<AtmosphereTypeRegistry>,
    image_bind_group_layout: Res<AtmosphereImageBindGroupLayout>,
    atmosphere: Option<Res<AtmosphereModel>>,
) {
    let view = atmosphere_image.array_view.as_ref().expect("prepare_changed_settings should have took care of making AtmosphereImage.array_value Some(TextureView)");

    let atmosphere = match atmosphere {
        Some(a) => a.clone(),
        None => default(),
    };

    let bind_group_layout = {
        let data = cached_metadata.clone().0.unwrap_or_else(|| {
            let data = {
                let type_registry = type_registry.read();
                type_registry
                    .get_type_data::<AtmosphereModelMetadata>(atmosphere.model().type_id())
                    .expect("Failed to get type data")
                    .clone()
            };
            *cached_metadata = CachedAtmosphereModelMetadata(Some(data.clone()));
            data
        });
        data.bind_group_layout
    };

    let atmosphere_bind_group = atmosphere.model().as_bind_group(
        &bind_group_layout,
        &render_device,
        &gpu_images,
        &fallback_image,
    );

    let image_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: Some("bevy_atmosphere_image_bind_group"),
        layout: &image_bind_group_layout.0,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: BindingResource::TextureView(view),
        }],
    });

    commands.insert_resource(AtmosphereBindGroups(
        atmosphere_bind_group,
        image_bind_group,
    ));
}

#[derive(Resource)]
struct AtmosphereTypeRegistry(AppTypeRegistry);

impl Deref for AtmosphereTypeRegistry {
    type Target = AppTypeRegistry;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

enum AtmosphereState {
    Loading,
    Update,
}

struct AtmosphereNode {
    state: AtmosphereState,
}

impl Default for AtmosphereNode {
    fn default() -> Self {
        Self {
            state: AtmosphereState::Loading,
        }
    }
}

impl render_graph::Node for AtmosphereNode {
    fn update(&mut self, world: &mut World) {
        match self.state {
            AtmosphereState::Loading => {
                let cached_metadata = world.resource::<CachedAtmosphereModelMetadata>();
                let pipeline = {
                    let data = cached_metadata.clone().0.unwrap_or_else(|| {
                        let atmosphere = world.resource::<AtmosphereModel>();
                        let type_registry = world.resource::<AtmosphereTypeRegistry>();
                        let data = {
                            let type_registry = type_registry.read();
                            type_registry
                                .get_type_data::<AtmosphereModelMetadata>(
                                    atmosphere.model().type_id(),
                                )
                                .expect("Failed to get type data")
                                .clone()
                        };
                        let mut cached_metadata =
                            world.resource_mut::<CachedAtmosphereModelMetadata>();
                        *cached_metadata = CachedAtmosphereModelMetadata(Some(data.clone()));
                        data
                    });
                    data.pipeline
                };

                let pipeline_cache = world.resource::<PipelineCache>();

                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline)
                {
                    let mut event_writer = world.resource_mut::<Events<AtmosphereUpdateEvent>>();
                    event_writer.send(AtmosphereUpdateEvent);
                    self.state = AtmosphereState::Update;
                }
            }
            AtmosphereState::Update => {}
        }
    }

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let update_events = world.resource::<Events<AtmosphereUpdateEvent>>();
        match self.state {
            AtmosphereState::Loading => {}
            AtmosphereState::Update => {
                if !update_events.is_empty() {
                    // only run when there are update events available
                    let bind_groups = world.resource::<AtmosphereBindGroups>();
                    let pipeline_cache = world.resource::<PipelineCache>();
                    let cached_metadata = world.resource::<CachedAtmosphereModelMetadata>();
                    let settings = world.resource::<AtmosphereSettings>();

                    let pipeline = {
                        let data = cached_metadata.0.clone().expect("Failed to get type data!");
                        data.pipeline
                    };

                    let mut pass = render_context.command_encoder().begin_compute_pass(
                        &ComputePassDescriptor {
                            label: Some("atmosphere_pass"),
                        },
                    );

                    pass.set_bind_group(0, &bind_groups.0, &[]);
                    pass.set_bind_group(1, &bind_groups.1, &[]);

                    let update_pipeline = pipeline_cache.get_compute_pipeline(pipeline).unwrap();
                    pass.set_pipeline(update_pipeline);
                    pass.dispatch_workgroups(
                        settings.resolution / WORKGROUP_SIZE,
                        settings.resolution / WORKGROUP_SIZE,
                        6,
                    );
                }
            }
        }

        Ok(())
    }
}
