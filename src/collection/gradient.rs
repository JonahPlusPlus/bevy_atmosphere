use crate::model::{AtmosphereModelMetadata, Atmospheric, RegisterAtmosphereModel};
use bevy::{asset::uuid_handle, ecs::reflect::AppTypeRegistry, prelude::*, render::render_resource::ShaderType};

/// Handle for the gradient shader
pub const GRADIENT_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("3b5c8d7f-1a2e-4f6c-9b0d-1e2a3c4f5a6b");

/// The Gradient sky model.
///
/// A simple gradient for creating a stylized environment.
#[derive(ShaderType, Reflect, Debug, Clone)]
pub struct Gradient {
    /// Sky Color (Default: `Color::srgb(0.29, 0.41, 0.50)`).
    /// <div style="background-color:rgb(29%, 41%, 50%); width: 10px; padding: 10px; border: 1px solid;"></div>
    ///
    ///
    /// The color of the top.
    pub sky: LinearRgba,
    /// Horizon Color (Default: `Color::srgb(0.48, 0.62, 0.69)`).
    /// <div style="background-color:rgb(48%, 62%, 69%); width: 10px; padding: 10px; border: 1px solid;"></div>
    ///
    ///
    /// The color of the sides.
    pub horizon: LinearRgba,
    /// Ground Color (Default: `Color::srgb(0.71, 0.69, 0.57)`).
    /// <div style="background-color:rgb(71%, 69%, 57%); width: 10px; padding: 10px; border: 1px solid;"></div>
    ///
    ///
    /// The color of the bottom.
    pub ground: LinearRgba,
}

impl Default for Gradient {
    fn default() -> Self {
        Self {
            sky: Color::srgb(0.29, 0.41, 0.50).into(),
            horizon: Color::srgb(0.48, 0.62, 0.69).into(),
            ground: Color::srgb(0.71, 0.69, 0.57).into(),
        }
    }
}

impl From<&Gradient> for Gradient {
    fn from(gradient: &Gradient) -> Self {
        gradient.clone()
    }
}

impl Atmospheric for Gradient {
    fn as_bind_group(
        &self,
        layout: &bevy::render::render_resource::BindGroupLayout,
        render_device: &bevy::render::renderer::RenderDevice,
        _images: &bevy::render::render_asset::RenderAssets<bevy::render::texture::GpuImage>,
        _fallback_image: &bevy::render::texture::FallbackImage,
    ) -> bevy::render::render_resource::BindGroup {
        use bevy::render::render_resource::*;
        let mut buffer = encase::UniformBuffer::new(Vec::new());
        buffer.write(self).unwrap();
        render_device.create_bind_group(
            None,
            layout,
            &[BindGroupEntry {
                binding: 0,
                resource: render_device.create_buffer_with_data(
                    &BufferInitDescriptor {
                        label: None,
                        usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
                        contents: buffer.as_ref(),
                    }
                ).as_entire_binding(),
            }],
        )
    }

    fn clone_dynamic(&self) -> Box<dyn Atmospheric> {
        Box::new(self.clone())
    }

    fn as_reflect(&self) -> &dyn bevy::reflect::Reflect {
        self
    }

    fn as_reflect_mut(&mut self) -> &mut dyn bevy::reflect::Reflect {
        self
    }
}

impl RegisterAtmosphereModel for Gradient {
    fn register(app: &mut bevy::prelude::App) {
        use bevy::render::{render_resource::ComputePipelineDescriptor, renderer::RenderDevice, RenderApp};
        use std::{any::TypeId, borrow::Cow};

        app.register_type::<Self>();

        let handle = GRADIENT_SHADER_HANDLE;

        let render_app = app.sub_app_mut(RenderApp);
        let render_device = render_app.world().resource::<RenderDevice>();
        let crate::pipeline::AtmosphereImageBindGroupLayout(image_bind_group_layout) =
            render_app
                .world()
                .resource::<crate::pipeline::AtmosphereImageBindGroupLayout>()
                .clone();

        let bind_group_layout = Self::bind_group_layout(render_device);

        let pipeline_cache = render_app
            .world_mut()
            .resource_mut::<bevy::render::render_resource::PipelineCache>();

        let pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("bevy_atmosphere_compute_pipeline")),
            layout: vec![bind_group_layout.clone(), image_bind_group_layout],
            push_constant_ranges: vec![],
            shader: handle,
            shader_defs: vec![],
            entry_point: Some(Cow::from("main")),
            zero_initialize_workgroup_memory: true,
        });

        let id = TypeId::of::<Self>();
        let data = AtmosphereModelMetadata {
            id,
            bind_group_layout,
            pipeline,
        };

        let type_registry = app.world_mut().resource_mut::<AppTypeRegistry>();
        {
            let mut type_registry = type_registry.write();
            let registration = type_registry
                .get_mut(std::any::TypeId::of::<Self>())
                .expect("Type not registered");
            registration.insert(data);
        }
    }
    
    fn bind_group_layout(render_device: &bevy::render::renderer::RenderDevice) -> bevy::render::render_resource::BindGroupLayout {
        use bevy::render::render_resource::*;
        render_device.create_bind_group_layout(
            None,
            &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::COMPUTE,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: Some(std::num::NonZero::new(<Self as ShaderType>::min_size().get()).unwrap()),
                },
                count: None,
            }],
        )
    }
}
