use std::borrow::Cow;

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        render_graph::{self, RenderGraph},
        render_resource::{
            AsBindGroup, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
            CachedComputePipelineId, CachedPipelineState, ComputePassDescriptor,
            ComputePipelineDescriptor, PipelineCache, ShaderStages, StorageTextureAccess,
            TextureFormat, TextureViewDimension,
        },
        renderer::RenderDevice,
        texture::FallbackImage,
        RenderApp, RenderStage,
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

#[derive(Clone, Deref, ExtractResource)]
pub struct AtmosphereImage(pub Handle<Image>);

pub struct AtmosphereBindGroups(pub BindGroup, pub BindGroup);

pub struct AtmospherePipelinePlugin(pub AtmosphereSettings);

impl Plugin for AtmospherePipelinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ExtractResourcePlugin::<Atmosphere>::default());
        app.add_plugin(ExtractResourcePlugin::<AtmosphereImage>::default());

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<AtmospherePipeline>()
            .insert_resource(self.0.clone())
            .add_system_to_stage(RenderStage::Queue, queue_bind_group);

        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node(NAME, AtmosphereNode::default());
        render_graph
            .add_node_edge(NAME, bevy::render::main_graph::node::CAMERA_DRIVER)
            .unwrap();
    }
}

fn queue_bind_group(
    mut commands: Commands,
    pipeline: Res<AtmospherePipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    atmosphere_image: Res<AtmosphereImage>,
    atmosphere: Option<Res<Atmosphere>>,
    render_device: Res<RenderDevice>,
    fallback_image: Res<FallbackImage>,
) {
    let view = &gpu_images[&atmosphere_image.0];

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

    let image_bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: Some("bevy_atmosphere_bind_group"),
        layout: &pipeline.image_bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: BindingResource::TextureView(&view.texture_view),
        }],
    });

    commands.insert_resource(AtmosphereBindGroups(
        atmosphere_bind_group,
        image_bind_group,
    ));
}

pub struct AtmospherePipeline {
    atmosphere_bind_group_layout: BindGroupLayout,
    image_bind_group_layout: BindGroupLayout,
    update_pipeline: CachedComputePipelineId,
}

impl FromWorld for AtmospherePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let atmosphere_bind_group_layout = Atmosphere::bind_group_layout(&render_device);
        let image_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("bevy_atmosphere_bind_group_layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadWrite,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
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
                image_bind_group_layout.clone(),
            ]),
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("main"),
        });

        Self {
            atmosphere_bind_group_layout,
            image_bind_group_layout,
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
            AtmosphereState::Update => if world.is_resource_changed::<Atmosphere>() {
                let bind_groups = world.resource::<AtmosphereBindGroups>();
                let pipeline_cache = world.resource::<PipelineCache>();
                let pipeline = world.resource::<AtmospherePipeline>();
                let settings = world.resource::<AtmosphereSettings>();

                let mut pass = render_context
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
                pass.dispatch_workgroups(settings.size, settings.size, 1);
            }
        }

        Ok(())
    }
}
