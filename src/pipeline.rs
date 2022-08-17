use std::{borrow::Cow, num::NonZeroU32};

use bevy::{
    asset::load_internal_asset,
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::{RenderAssets, PrepareAssetLabel},
        render_graph::{self, RenderGraph},
        render_resource::{
            AsBindGroup, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
            CachedComputePipelineId, CachedPipelineState, ComputePassDescriptor,
            ComputePipelineDescriptor, Extent3d, PipelineCache, ShaderStages, StorageTextureAccess,
            TextureAspect, TextureDimension, TextureFormat, TextureUsages, TextureView,
            TextureViewDescriptor, TextureViewDimension, TextureDescriptor,
        },
        renderer::RenderDevice,
        texture::FallbackImage,
        Extract, RenderApp, RenderStage,
    },
};

use crate::{resource::Atmosphere, settings::AtmosphereSettings};

pub const ATMOSPHERE_MAIN_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 05132991701789555342);
pub const ATMOSPHERE_MATH_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 07843425155352921761);
pub const ATMOSPHERE_TYPES_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 09615256157423613453);

pub const NAME: &str = "bevy_atmosphere";
pub const WORKGROUP_SIZE: u32 = 8;

/// [Handle] to a procedural sky [Image]
///
/// It can be used in a material for a skybox mesh.
///
/// The image is (6*512)px by 512px.
/// Each 512x512 area corresponds to a face on the mesh.
/// The order of faces is [X, Y, Z, -X, -Y, -Z].
#[derive(ExtractResource, Debug, Clone)]
pub struct AtmosphereImage {
    pub handle: Handle<Image>,
    pub array_view: Option<TextureView>,
}

#[derive(Clone, Debug)]
struct AtmosphereBindGroups(pub BindGroup, pub BindGroup);

/// A [Plugin] that creates the compute pipeline for rendering a procedural sky cubemap texture.
#[derive(Debug, Clone, Copy)]
pub struct AtmospherePipelinePlugin;

impl Plugin for AtmospherePipelinePlugin {
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

        let settings = match app.world.get_resource::<AtmosphereSettings>() {
            Some(s) => *s,
            None => default(),
        };

        let atmosphere = match app.world.get_resource::<Atmosphere>() {
            Some(s) => *s,
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

        app.add_plugin(ExtractResourcePlugin::<AtmosphereImage>::default());

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<AtmospherePipeline>()
            .insert_resource(atmosphere)
            .insert_resource(settings)
            .add_system_to_stage(RenderStage::Extract, extract_atmosphere_resources)
            .add_system_to_stage(RenderStage::Prepare, prepare_changed_settings.after(PrepareAssetLabel::AssetPrepare))
            .add_system_to_stage(RenderStage::Queue, queue_bind_group);

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node(NAME, AtmosphereNode::default());
        render_graph
            .add_node_edge(NAME, bevy::render::main_graph::node::CAMERA_DRIVER)
            .unwrap();
    }
}

fn extract_atmosphere_resources(
    main_atmosphere: Extract<Option<Res<Atmosphere>>>,
    mut render_atmosphere: ResMut<Atmosphere>,
    main_settings: Extract<Option<Res<AtmosphereSettings>>>,
    mut render_settings: ResMut<AtmosphereSettings>,
) {
    if let Some(atmosphere) = &*main_atmosphere {
        if atmosphere.is_changed() {
            *render_atmosphere = Atmosphere::extract_resource(&*atmosphere);
        }
    }

    if let Some(settings) = &*main_settings {
        if settings.is_changed() {
            *render_settings = AtmosphereSettings::extract_resource(&*settings);
        }
    }
}

const ATMOSPHERE_CUBE_TEXTURE_VIEW_DESCRIPTOR: TextureViewDescriptor = TextureViewDescriptor {
    label: Some("atmosphere_image_array_view"),
    format: Some(TextureFormat::Rgba16Float),
    dimension: Some(TextureViewDimension::Cube),
    aspect: TextureAspect::All,
    base_mip_level: 0,
    mip_level_count: None,
    base_array_layer: 0,
    array_layer_count: NonZeroU32::new(6),
};

const ATMOSPHERE_ARRAY_TEXTURE_VIEW_DESCRIPTOR: TextureViewDescriptor = TextureViewDescriptor {
    label: Some("atmosphere_image_cube_view"),
    format: Some(TextureFormat::Rgba16Float),
    dimension: Some(TextureViewDimension::D2Array),
    aspect: TextureAspect::All,
    base_mip_level: 0,
    mip_level_count: None,
    base_array_layer: 0,
    array_layer_count: NonZeroU32::new(6),
};

const ATMOSPHERE_IMAGE_TEXTURE_DESCRIPTOR:  fn(u32) -> TextureDescriptor<'static> = |res| TextureDescriptor {
    label: Some("atmosphere_image_texture"),
    size: Extent3d {
        width: res.clone(),
        height: res.clone(),
        depth_or_array_layers: 6,
    },
    mip_level_count: 1,
    sample_count: 1,
    dimension: TextureDimension::D2,
    format: TextureFormat::Rgba16Float,
    usage: TextureUsages::COPY_DST
    | TextureUsages::STORAGE_BINDING
    | TextureUsages::TEXTURE_BINDING
};

fn prepare_changed_settings(
    mut atmosphere_image: ResMut<AtmosphereImage>,
    gpu_images: Res<RenderAssets<Image>>,
    settings: Res<AtmosphereSettings>,
) {
    if settings.is_changed() | atmosphere_image.array_view.is_none() {
        let texture = &gpu_images[&atmosphere_image.handle].texture;
        let view = texture.create_view(&ATMOSPHERE_ARRAY_TEXTURE_VIEW_DESCRIPTOR);
        atmosphere_image.array_view = Some(view.clone());
    }
}

fn queue_bind_group(
    mut commands: Commands,
    gpu_images: Res<RenderAssets<Image>>,
    atmosphere_image: Res<AtmosphereImage>,
    render_device: Res<RenderDevice>,
    fallback_image: Res<FallbackImage>,
    pipeline: Res<AtmospherePipeline>,
    atmosphere: Option<Res<Atmosphere>>,
) {
    let view = atmosphere_image.array_view.as_ref().expect("prepare_changed_settings should have took care of making AtmosphereImage.array_value Some(TextureView)");

    let atmosphere = match atmosphere {
        Some(a) => *a,
        None => default(),
    };

    let atmosphere_bind_group = atmosphere
        .as_bind_group(
            &pipeline.atmosphere_bind_group_layout,
            &render_device,
            &gpu_images,
            &fallback_image,
        )
        .unwrap_or_else(|_| {
            panic!("Failed to get as bind group");
        })
        .bind_group;

    let associated_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: Some("bevy_atmosphere_associated_bind_group"),
        layout: &pipeline.associated_bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: BindingResource::TextureView(&view),
        }],
    });

    commands.insert_resource(AtmosphereBindGroups(
        atmosphere_bind_group,
        associated_bind_group,
    ));
}

struct AtmospherePipeline {
    atmosphere_bind_group_layout: BindGroupLayout,
    associated_bind_group_layout: BindGroupLayout,
    update_pipeline: CachedComputePipelineId,
}

impl FromWorld for AtmospherePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let atmosphere_bind_group_layout = Atmosphere::bind_group_layout(&render_device);
        let associated_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("bevy_atmosphere_associated_bind_group_layout"),
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
            });
        let shader = ATMOSPHERE_MAIN_SHADER_HANDLE.typed();
        let mut pipeline_cache = world.resource_mut::<PipelineCache>();

        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("bevy_atmosphere_compute_pipeline")),
            layout: Some(vec![
                atmosphere_bind_group_layout.clone(),
                associated_bind_group_layout.clone(),
            ]),
            shader,
            shader_defs: vec![
                #[cfg(feature = "dither")]
                String::from("DITHER"),
            ],
            entry_point: Cow::from("main"),
        });

        Self {
            atmosphere_bind_group_layout,
            associated_bind_group_layout,
            update_pipeline,
        }
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
        let pipeline = world.resource::<AtmospherePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        match self.state {
            AtmosphereState::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_compute_pipeline_state(pipeline.update_pipeline)
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
                if world.is_resource_changed::<Atmosphere>()
                    || world.is_resource_changed::<AtmosphereSettings>()
                {
                    let bind_groups = world.resource::<AtmosphereBindGroups>();
                    let pipeline_cache = world.resource::<PipelineCache>();
                    let pipeline = world.resource::<AtmospherePipeline>();
                    let settings = world.resource::<AtmosphereSettings>();

                    let mut pass =
                        render_context
                            .command_encoder
                            .begin_compute_pass(&ComputePassDescriptor {
                                label: Some("atmosphere_pass"),
                            });

                    pass.set_bind_group(0, &bind_groups.0, &[]);
                    pass.set_bind_group(1, &bind_groups.1, &[]);

                    let update_pipeline = pipeline_cache
                        .get_compute_pipeline(pipeline.update_pipeline)
                        .unwrap();
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
