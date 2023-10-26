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
        Extract, Render, RenderApp, RenderSet,
    },
};

use crate::{
    model::{AtmosphereModel, AtmosphereModelMetadata, AtmosphereModelPrecompute},
    settings::AtmosphereSettings,
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

/// The procedural sky optical depths generated by the atmosphere precompute pipeline.
// DISCUSS: configurability
#[derive(Resource, ExtractResource, Debug, Clone)]
pub struct AtmospherePrecomputeImage {
    pub handle: Handle<Image>,
    pub view: Option<TextureView>,
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

#[derive(Resource, Debug, Clone)]
pub struct AtmospherePrecomputeImageBindGroupLayout(pub BindGroupLayout);

impl FromWorld for AtmospherePrecomputeImageBindGroupLayout {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        Self(
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("bevy_atmosphere_precopmute_image_bind_group_layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Rgba32Float,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                }],
            }),
        )
    }
}

#[derive(Resource, Debug, Clone)]
struct AtmosphereBindGroups(pub BindGroup, pub BindGroup);

#[derive(Resource, Default, Clone)]
struct CachedComputeMetadata(pub Option<AtmosphereModelMetadata>);

#[derive(Resource, Default, Clone)]
struct CachedPrecomputeMetadata(pub Option<AtmosphereModelMetadata>);

#[derive(Resource, Default, Clone)]
enum AtmosphereQueue {
    #[default]
    None,
    Precompute(AtmosphereModel, AtmosphereModel),
    Compute(AtmosphereModel),
}

/// A `Plugin` that creates the compute pipeline for rendering a procedural sky cubemap texture.
#[derive(Default, Clone)]
pub struct AtmospherePipelinePlugin {
    pub settings: AtmosphereSettings,
}

impl Plugin for AtmospherePipelinePlugin {
    fn build(&self, app: &mut App) {
        let mut image_assets = app.world.resource_mut::<Assets<Image>>();
        let compute_img = AtmosphereImage {
            handle: {
                let mut image = Image::new_fill(
                    Extent3d {
                        width: self.settings.resolution,
                        height: self.settings.resolution,
                        depth_or_array_layers: 6,
                    },
                    TextureDimension::D2,
                    &[0; 4 * 4],
                    TextureFormat::Rgba16Float,
                );

                image.texture_view_descriptor = Some(ATMOSPHERE_CUBE_TEXTURE_VIEW_DESCRIPTOR);

                image.texture_descriptor =
                    ATMOSPHERE_IMAGE_TEXTURE_DESCRIPTOR(self.settings.resolution);

                image_assets.add(image)
            },
            array_view: None,
        };
        let precompute_img = AtmospherePrecomputeImage {
            handle: {
                let mut image = Image::new_fill(
                    Extent3d {
                        width: 128,
                        height: 512,
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    &[0; 4 * 4],
                    TextureFormat::Rgba32Float,
                );
                image.texture_view_descriptor = Some(TextureViewDescriptor {
                    label: Some("atmosphere_precompute_image_view"),
                    format: Some(TextureFormat::Rgba32Float),
                    dimension: Some(TextureViewDimension::D2),
                    aspect: TextureAspect::All,
                    base_mip_level: 0,
                    mip_level_count: None,
                    base_array_layer: 0,
                    array_layer_count: None,
                });
                image.texture_descriptor = TextureDescriptor {
                    label: Some("atmosphere_precompute_image_texture"),
                    size: Extent3d {
                        width: 128,
                        height: 512,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: TextureFormat::Rgba32Float,
                    usage: TextureUsages::COPY_DST
                        | TextureUsages::STORAGE_BINDING
                        | TextureUsages::TEXTURE_BINDING,
                    view_formats: &[TextureFormat::Rgba32Float],
                };
                image_assets.add(image)
            },
            view: None,
        };
        // TODO: create our own texture_view of precompute and keep a seperate bind group, since wgpu wants to be an asshole
        app.insert_resource(compute_img);
        app.insert_resource(precompute_img);

        app.add_plugins(ExtractResourcePlugin::<AtmosphereImage>::default());
        app.add_plugins(ExtractResourcePlugin::<AtmospherePrecomputeImage>::default());

        app.add_systems(PostUpdate, atmosphere_settings_changed);

        let type_registry = app.world.resource::<AppTypeRegistry>().clone();

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .insert_resource(self.settings.clone())
            .insert_resource(AtmosphereTypeRegistry(type_registry))
            .insert_resource(AtmosphereQueue::default())
            .init_resource::<CachedComputeMetadata>()
            .init_resource::<CachedPrecomputeMetadata>()
            .add_systems(ExtractSchedule, extract_atmosphere_resources)
            .add_systems(
                Render,
                (
                    prepare_atmosphere_assets.in_set(PrepareAssetSet::PostAssetPrepare),
                    queue_atmosphere_bind_group.in_set(RenderSet::Queue),
                    atmosphere_cleanup.in_set(RenderSet::Cleanup),
                ),
            );

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node(NAME, AtmosphereNode::default());
        render_graph.add_node_edge(NAME, bevy::render::main_graph::node::CAMERA_DRIVER);
    }
}

/// Whenever settings are changed, resize the image to the appropriate size.
fn atmosphere_settings_changed(
    mut image_assets: ResMut<Assets<Image>>,
    mut atmosphere_image: ResMut<AtmosphereImage>,
    mut settings_existed: Local<bool>,
    settings: Option<Res<AtmosphereSettings>>,
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
                atmosphere_image.array_view = None; // drop the previous texture view
                #[cfg(feature = "bevy/trace")]
                trace!("Resized image to {:?}", size);
            }
        }
        *settings_existed = false;
    }
}

macro_rules! cache_metadata {
    ($type_registry:expr, $id:expr) => {{
        let type_registry = $type_registry.read();
        type_registry
            .get_type_data::<AtmosphereModelMetadata>($id)
            .expect("Failed to get type data, perhaps you forgot to register the atmospheric model")
            .clone()
    }};
}

/// Extracts [`AtmosphereModel`] and [`AtmosphereSettings`] from main world.
#[allow(clippy::too_many_arguments)]
fn extract_atmosphere_resources(
    type_registry: Res<AtmosphereTypeRegistry>,
    mut cached_compute: ResMut<CachedComputeMetadata>,
    mut cached_precompute: ResMut<CachedPrecomputeMetadata>,
    main_atmosphere: Extract<Option<Res<AtmosphereModel>>>,
    pre_atmosphere: Extract<Option<Res<AtmosphereModelPrecompute>>>,
    mut queue: ResMut<AtmosphereQueue>,
    main_settings: Extract<Option<Res<AtmosphereSettings>>>,
    mut render_settings: ResMut<AtmosphereSettings>,
    mut settings_existed: Local<bool>,
) {
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

    let Some(atmosphere) = &*main_atmosphere else {
        *queue = AtmosphereQueue::None;
        return;
    };

    if atmosphere.is_changed() || atmosphere.is_added() {
        let compute = AtmosphereModel::extract_resource(atmosphere);
        if let Some(metadata) = &mut cached_compute.0 {
            let id = compute.model().type_id();
            if metadata.id != id {
                *metadata = cache_metadata!(type_registry, id);
            }
        }
        *queue = match &*pre_atmosphere {
            Some(pre) if pre.is_changed() || pre.is_added() => {
                let precompute = AtmosphereModel::extract_resource(&pre.0);
                if let Some(metadata) = &mut cached_precompute.0 {
                    let id = precompute.model().type_id();
                    if metadata.id != id {
                        *metadata = cache_metadata!(type_registry, id);
                    }
                }
                AtmosphereQueue::Precompute(precompute, compute)
            }
            _ => AtmosphereQueue::Compute(compute),
        }
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
    mut atmosphere_image: ResMut<AtmosphereImage>,
    mut precompute_image: ResMut<AtmospherePrecomputeImage>,
    gpu_images: Res<RenderAssets<Image>>,
) {
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
        #[cfg(feature = "bevy/trace")]
        trace!(
            "Created new 2D array texture view from atmosphere texture of size {:?}",
            &gpu_images[&atmosphere_image.handle].size
        );
    }
    if precompute_image.view.is_none() {
        let texture = &gpu_images[&precompute_image.handle].texture;
        let view = texture.create_view(&TextureViewDescriptor {
            label: Some("atmosphere_precompute_image_view"),
            format: Some(TextureFormat::Rgba32Float),
            dimension: Some(TextureViewDimension::D2),
            aspect: TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });
        precompute_image.view = Some(view);
    }
}

/// Queue the generated bind groups for the compute pipeline.
#[allow(clippy::too_many_arguments)]
fn queue_atmosphere_bind_group(
    mut commands: Commands,
    mut cached_compute: ResMut<CachedComputeMetadata>,
    mut cached_precompute: ResMut<CachedPrecomputeMetadata>,
    gpu_images: Res<RenderAssets<Image>>,
    atmosphere_image: Res<AtmosphereImage>,
    precompute_image: Res<AtmospherePrecomputeImage>,
    render_device: Res<RenderDevice>,
    fallback_image: Res<FallbackImage>,
    type_registry: Res<AtmosphereTypeRegistry>,
    image_bind_group_layout: Res<AtmosphereImageBindGroupLayout>,
    precompute_image_bind_group_layout: Res<AtmospherePrecomputeImageBindGroupLayout>,
    mut queue: ResMut<AtmosphereQueue>,
) {
    #[cfg(feature = "bevy/trace")]
    let _queue_atmosphere_bind_group = info_span!(
        "executed",
        name = "bevy_atmosphere::pipeline::queue_atmosphere_bind_group"
    )
    .entered();

    let get_bind_group_layout = |cache: &mut Option<AtmosphereModelMetadata>,
                                 model: &AtmosphereModel| {
        let data = cache.clone().unwrap_or_else(|| {
            let data = cache_metadata!(type_registry, model.model().type_id());
            *cache = Some(data.clone());
            data
        });
        debug_assert!(
            data.id == model.model().type_id(),
            "CachedAtmosphereModelMetadata should have been updated, but wasn't"
        );
        data.bind_group_layout
    };

    macro_rules! get_atmosphere_bind_group {
        ($layout:expr, $model:expr) => {
            match $model.model().as_bind_group(
                &$layout,
                &render_device,
                &gpu_images,
                &fallback_image,
            ) {
                Ok(bind_group) => bind_group,
                Err(_) => {
                    error!("Failed to create atmosphere model bind group");
                    *queue = AtmosphereQueue::None;
                    return;
                }
            }
        };
    }

    match &*queue {
        AtmosphereQueue::None => return,
        AtmosphereQueue::Precompute(model, _) => {
            let bind_group = get_atmosphere_bind_group!(
                get_bind_group_layout(&mut cached_precompute.0, model),
                model
            );
            commands.insert_resource(AtmosphereBindGroups(
                bind_group.bind_group,
                render_device.create_bind_group(&BindGroupDescriptor {
                    label: Some("bevy_precompute_atmosphere_image_bind_group"),
                    layout: &precompute_image_bind_group_layout.0,
                    entries: &[BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(precompute_image.view.as_ref().expect("prepare_changed_settings should have took care of making AtmospherePrecomputeImage.array_value Some(TextureView)")),
                    }],
                }),
            ));
        }
        AtmosphereQueue::Compute(model) => {
            let bind_group = get_atmosphere_bind_group!(
                get_bind_group_layout(&mut cached_compute.0, model),
                model
            );
            commands.insert_resource(AtmosphereBindGroups(
                bind_group.bind_group,
                render_device.create_bind_group(&BindGroupDescriptor {
                    label: Some("bevy_atmosphere_image_bind_group"),
                    layout: &image_bind_group_layout.0,
                    entries: &[BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(atmosphere_image.array_view.as_ref().expect("prepare_changed_settings should have took care of making AtmosphereImage.array_value Some(TextureView)")),
                    }],
                }),
            ));
        }
    }
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
                let pipeline = {
                    let data = match world.resource::<AtmosphereQueue>().clone() {
                        AtmosphereQueue::None => return,
                        AtmosphereQueue::Precompute(a, _) => {
                            let cache = world.resource::<CachedPrecomputeMetadata>().0.clone();
                            cache.unwrap_or_else(|| {
                                let data = cache_metadata!(
                                    world.resource::<AtmosphereTypeRegistry>(),
                                    a.model().type_id()
                                );
                                let cache = &mut world.resource_mut::<CachedPrecomputeMetadata>().0;
                                *cache = Some(data.clone());
                                data
                            })
                        }
                        AtmosphereQueue::Compute(a) => {
                            let cache = world.resource::<CachedComputeMetadata>().0.clone();
                            cache.unwrap_or_else(|| {
                                let data = cache_metadata!(
                                    world.resource::<AtmosphereTypeRegistry>(),
                                    a.model().type_id()
                                );
                                let cache = &mut world.resource_mut::<CachedComputeMetadata>().0;
                                *cache = Some(data.clone());
                                data
                            })
                        }
                    };
                    data.pipeline
                };

                let pipeline_cache = world.resource::<PipelineCache>();

                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline)
                {
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
        match self.state {
            AtmosphereState::Loading => {}
            AtmosphereState::Update => {
                let (cache, workgroups) = match world.resource::<AtmosphereQueue>() {
                    AtmosphereQueue::None => return Ok(()),
                    AtmosphereQueue::Precompute(_, _) => (
                        &world.resource::<CachedPrecomputeMetadata>().0,
                        (128 / WORKGROUP_SIZE, 512 / WORKGROUP_SIZE, 1),
                    ),
                    AtmosphereQueue::Compute(_) => {
                        let settings = world.resource::<AtmosphereSettings>();
                        (
                            &world.resource::<CachedComputeMetadata>().0,
                            (
                                settings.resolution / WORKGROUP_SIZE,
                                settings.resolution / WORKGROUP_SIZE,
                                6,
                            ),
                        )
                    }
                };

                let cache = cache.clone().expect("Failed to get type data!");

                let bind_groups = world.resource::<AtmosphereBindGroups>();
                let pipeline_cache = world.resource::<PipelineCache>();
                let pipeline = cache.pipeline;
                let mut pass =
                    render_context
                        .command_encoder()
                        .begin_compute_pass(&ComputePassDescriptor {
                            label: Some("atmosphere_pass"),
                        });

                pass.set_bind_group(0, &bind_groups.0, &[]);
                pass.set_bind_group(1, &bind_groups.1, &[]);

                let update_pipeline = pipeline_cache.get_compute_pipeline(pipeline).unwrap();

                pass.set_pipeline(update_pipeline);
                pass.dispatch_workgroups(workgroups.0, workgroups.1, workgroups.2);
            }
        }

        Ok(())
    }
}

fn atmosphere_cleanup(mut queue: ResMut<AtmosphereQueue>) {
    let q = std::mem::take(&mut *queue);
    *queue = match q {
        AtmosphereQueue::None => return,
        AtmosphereQueue::Precompute(_, a) => AtmosphereQueue::Compute(a),
        AtmosphereQueue::Compute(_) => AtmosphereQueue::None,
    }
}
