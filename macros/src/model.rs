use bevy_macro_utils::{get_lit_bool, get_lit_str, BevyManifest, Symbol};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Data, DataStruct, Error, Fields, LitInt, LitStr, Meta, Result, Token,
};

const UNIFORM_ATTRIBUTE_NAME: Symbol = Symbol("uniform");
const TEXTURE_ATTRIBUTE_NAME: Symbol = Symbol("texture");
const SAMPLER_ATTRIBUTE_NAME: Symbol = Symbol("sampler");

const EXTERNAL_ATTRIBUTE_NAME: Symbol = Symbol("external");
const INTERNAL_ATTRIBUTE_NAME: Symbol = Symbol("internal");

#[derive(Copy, Clone, Debug)]
enum BindingType {
    Uniform,
    Texture,
    Sampler,
}

#[derive(Clone)]
enum BindingState<'a> {
    Free,
    Occupied {
        binding_type: BindingType,
        ident: &'a Ident,
    },
    OccupiedConvertedUniform,
    OccupiedMergeableUniform {
        uniform_fields: Vec<&'a syn::Field>,
    },
}

#[derive(PartialEq, Clone, Debug)]
enum ShaderPathType {
    None,
    External(String),
    Internal(String),
}

pub fn derive_atmospheric(ast: syn::DeriveInput) -> Result<TokenStream> {
    let manifest = BevyManifest::shared();
    let atmosphere_path = super::bevy_atmosphere_path();
    let render_path = manifest.get_path("bevy_render");
    let asset_path = manifest.get_path("bevy_asset");
    let ecs_path = manifest.get_path("bevy_ecs");

    let id = {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        ast.ident.hash(&mut hasher);
        hasher.finish()
    };

    let mut shader_path = ShaderPathType::None;
    let mut binding_states: Vec<BindingState> = Vec::new();
    let mut binding_impls = Vec::new();
    let mut bind_group_entries = Vec::new();
    let mut binding_layouts = Vec::new();

    // Read struct-level attributes
    for attr in &ast.attrs {
        if let Some(attr_ident) = attr.path().get_ident() {
            if attr_ident == UNIFORM_ATTRIBUTE_NAME {
                let (binding_index, converted_shader_type) = get_uniform_binding_attr(attr)?;

                binding_impls.push(quote! {{
                    use #render_path::render_resource::AsBindGroupShaderType;
                    let mut buffer = #render_path::render_resource::encase::UniformBuffer::new(Vec::new());
                    let converted: #converted_shader_type = self.as_bind_group_shader_type(images);
                    buffer.write(&converted).unwrap();
                    #render_path::render_resource::OwnedBindingResource::Buffer(render_device.create_buffer_with_data(
                        &#render_path::render_resource::BufferInitDescriptor {
                            label: None,
                            usage: #render_path::render_resource::BufferUsages::COPY_DST | #render_path::render_resource::BufferUsages::UNIFORM,
                            contents: buffer.as_ref(),
                        },
                    ))
                }});

                binding_layouts.push(quote!{
                    #render_path::render_resource::BindGroupLayoutEntry {
                        binding: #binding_index,
                        visibility: #render_path::render_resource::ShaderStages::all(),
                        ty: #render_path::render_resource::BindingType::Buffer {
                            ty: #render_path::render_resource::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(<#converted_shader_type as #render_path::render_resource::ShaderType>::min_size()),
                        },
                        count: None,
                    }
                });

                let binding_vec_index = bind_group_entries.len();
                bind_group_entries.push(quote! {
                    #render_path::render_resource::BindGroupEntry {
                        binding: #binding_index,
                        resource: bindings[#binding_vec_index].get_binding(),
                    }
                });

                let required_len = binding_index as usize + 1;
                if required_len > binding_states.len() {
                    binding_states.resize(required_len, BindingState::Free);
                }
                binding_states[binding_index as usize] = BindingState::OccupiedConvertedUniform;
            } else if let Some(attr_ident) = attr.path().get_ident() {
                if attr_ident == EXTERNAL_ATTRIBUTE_NAME {
                    if shader_path != ShaderPathType::None {
                        return Err(Error::new_spanned(attr, "Shader path already set"));
                    }

                    let lit_str = get_shader_path_attr(attr)?;

                    shader_path = ShaderPathType::External(lit_str);
                } else if attr_ident == INTERNAL_ATTRIBUTE_NAME {
                    if shader_path != ShaderPathType::None {
                        return Err(Error::new_spanned(attr, "Shader path already set"));
                    }

                    let lit_str = get_shader_path_attr(attr)?;

                    shader_path = ShaderPathType::Internal(lit_str);
                }
            }
        }
    }

    let shader_path_impl = match shader_path {
        ShaderPathType::None => panic!("Expected `external` or `internal` attribute"),
        ShaderPathType::External(s) => quote! {
            {
                let asset_server = app.world().resource::<AssetServer>();

                asset_server.load(#s)
            }
        },
        ShaderPathType::Internal(s) => quote! {
            {
                let handle: #asset_path::Handle<Shader> = #asset_path::Handle::weak_from_u128(#id as u128);

                let internal_handle = handle.clone();
                #asset_path::load_internal_asset!(
                    app,
                    internal_handle,
                    concat!(env!("CARGO_MANIFEST_DIR"), "/src/", #s),
                    Shader::from_wgsl
                );

                handle
            }
        },
    };

    let fields = match &ast.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => &fields.named,
        _ => {
            return Err(Error::new_spanned(
                ast,
                "Expected a struct with named fields",
            ));
        }
    };

    // Read field-level attributes
    for field in fields.iter() {
        for attr in &field.attrs {
            let Some(attr_ident) = attr.path().get_ident() else {
                continue;
            };

            let binding_type = if attr_ident == UNIFORM_ATTRIBUTE_NAME {
                BindingType::Uniform
            } else if attr_ident == TEXTURE_ATTRIBUTE_NAME {
                BindingType::Texture
            } else if attr_ident == SAMPLER_ATTRIBUTE_NAME {
                BindingType::Sampler
            } else {
                continue;
            };

            let (binding_index, nested_meta_items) = get_binding_nested_attr(attr)?;

            let field_name = field.ident.as_ref().unwrap();
            let required_len = binding_index as usize + 1;
            if required_len > binding_states.len() {
                binding_states.resize(required_len, BindingState::Free);
            }

            match &mut binding_states[binding_index as usize] {
                value @ BindingState::Free => {
                    *value = match binding_type {
                        BindingType::Uniform => BindingState::OccupiedMergeableUniform {
                            uniform_fields: vec![field],
                        },
                        _ => {
                            // only populate bind group entries for non-uniforms
                            // uniform entries are deferred until the end
                            let binding_vec_index = bind_group_entries.len();
                            bind_group_entries.push(quote! {
                                #render_path::render_resource::BindGroupEntry {
                                    binding: #binding_index,
                                    resource: bindings[#binding_vec_index].get_binding(),
                                }
                            });
                            BindingState::Occupied {
                                binding_type,
                                ident: field_name,
                            }
                        }
                    }
                }
                BindingState::Occupied {
                    binding_type,
                    ident: occupied_ident,
                } => {
                    return Err(Error::new_spanned(
                        attr,
                        format!("The '{field_name}' field cannot be assigned to binding {binding_index} because it is already occupied by the field '{occupied_ident}' of type {binding_type:?}.")
                    ));
                }
                BindingState::OccupiedConvertedUniform => {
                    return Err(Error::new_spanned(
                        attr,
                        format!("The '{field_name}' field cannot be assigned to binding {binding_index} because it is already occupied by a struct-level uniform binding at the same index.")
                    ));
                }
                BindingState::OccupiedMergeableUniform { uniform_fields } => match binding_type {
                    BindingType::Uniform => {
                        uniform_fields.push(field);
                    }
                    _ => {
                        return Err(Error::new_spanned(
                                attr,
                                format!("The '{field_name}' field cannot be assigned to binding {binding_index} because it is already occupied by a {:?}.", BindingType::Uniform)
                            ));
                    }
                },
            }

            match binding_type {
                BindingType::Uniform => { /* uniform codegen is deferred to account for combined uniform bindings */
                }
                BindingType::Texture => {
                    let TextureAttrs {
                        dimension,
                        sample_type,
                        multisampled,
                        visibility,
                    } = get_texture_attrs(nested_meta_items)?;

                    let visibility =
                        visibility.hygienic_quote(&quote! { #render_path::render_resource });

                    binding_impls.push(quote! {
                        #render_path::render_resource::OwnedBindingResource::TextureView({
                            let handle: Option<&#asset_path::Handle<#render_path::texture::GpuImage>> = (&self.#field_name).into();
                            if let Some(handle) = handle {
                                images.get(handle).ok_or_else(|| #render_path::render_resource::AsBindGroupError::RetryNextUpdate)?.texture_view.clone()
                            } else {
                                fallback_image.texture_view.clone()
                            }
                        })
                    });

                    binding_layouts.push(quote! {
                        #render_path::render_resource::BindGroupLayoutEntry {
                            binding: #binding_index,
                            visibility: #visibility,
                            ty: #render_path::render_resource::BindingType::Texture {
                                multisampled: #multisampled,
                                sample_type: #render_path::render_resource::#sample_type,
                                view_dimension: #render_path::render_resource::#dimension,
                            },
                            count: None,
                        }
                    });
                }
                BindingType::Sampler => {
                    let SamplerAttrs {
                        sampler_binding_type,
                        visibility,
                    } = get_sampler_attrs(nested_meta_items)?;

                    let visibility =
                        visibility.hygienic_quote(&quote! { #render_path::render_resource });

                    binding_impls.push(quote! {
                        #render_path::render_resource::OwnedBindingResource::Sampler({
                            let handle: Option<&#asset_path::Handle<#render_path::texture::GpuImage>> = (&self.#field_name).into();
                            if let Some(handle) = handle {
                                images.get(handle).ok_or_else(|| #render_path::render_resource::AsBindGroupError::RetryNextUpdate)?.sampler.clone()
                            } else {
                                fallback_image.sampler.clone()
                            }
                        })
                    });

                    binding_layouts.push(quote!{
                        #render_path::render_resource::BindGroupLayoutEntry {
                            binding: #binding_index,
                            visibility: #visibility,
                            ty: #render_path::render_resource::BindingType::Sampler(#render_path::render_resource::#sampler_binding_type),
                            count: None,
                        }
                    });
                }
            }
        }
    }

    // Produce impls for fields with uniform bindings
    let struct_name = &ast.ident;
    let mut field_struct_impls = Vec::new();
    for (binding_index, binding_state) in binding_states.iter().enumerate() {
        let binding_index = binding_index as u32;
        if let BindingState::OccupiedMergeableUniform { uniform_fields } = binding_state {
            let binding_vec_index = bind_group_entries.len();
            bind_group_entries.push(quote! {
                #render_path::render_resource::BindGroupEntry {
                    binding: #binding_index,
                    resource: bindings[#binding_vec_index].get_binding(),
                }
            });
            // single field uniform bindings for a given index can use a straightforward binding
            if uniform_fields.len() == 1 {
                let field = &uniform_fields[0];
                let field_name = field.ident.as_ref().unwrap();
                let field_ty = &field.ty;
                binding_impls.push(quote! {{
                    let mut buffer = #render_path::render_resource::encase::UniformBuffer::new(Vec::new());
                    buffer.write(&self.#field_name).unwrap();
                    #render_path::render_resource::OwnedBindingResource::Buffer(render_device.create_buffer_with_data(
                        &#render_path::render_resource::BufferInitDescriptor {
                            label: None,
                            usage: #render_path::render_resource::BufferUsages::COPY_DST | #render_path::render_resource::BufferUsages::UNIFORM,
                            contents: buffer.as_ref(),
                        },
                    ))
                }});

                binding_layouts.push(quote!{
                    #render_path::render_resource::BindGroupLayoutEntry {
                        binding: #binding_index,
                        visibility: #render_path::render_resource::ShaderStages::all(),
                        ty: #render_path::render_resource::BindingType::Buffer {
                            ty: #render_path::render_resource::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(<#field_ty as #render_path::render_resource::ShaderType>::min_size()),
                        },
                        count: None,
                    }
                });
            // multi-field uniform bindings for a given index require an intermediate struct to derive ShaderType
            } else {
                let uniform_struct_name = Ident::new(
                    &format!("_{struct_name}AsBindGroupUniformStructBindGroup{binding_index}"),
                    Span::call_site(),
                );

                let field_name = uniform_fields.iter().map(|f| f.ident.as_ref().unwrap());
                let field_type = uniform_fields.iter().map(|f| &f.ty);
                field_struct_impls.push(quote! {
                    #[derive(#render_path::render_resource::ShaderType)]
                    struct #uniform_struct_name<'a> {
                        #(#field_name: &'a #field_type,)*
                    }
                });

                let field_name = uniform_fields.iter().map(|f| f.ident.as_ref().unwrap());
                binding_impls.push(quote! {{
                    let mut buffer = #render_path::render_resource::encase::UniformBuffer::new(Vec::new());
                    buffer.write(&#uniform_struct_name {
                        #(#field_name: &self.#field_name,)*
                    }).unwrap();
                    #render_path::render_resource::OwnedBindingResource::Buffer(render_device.create_buffer_with_data(
                        &#render_path::render_resource::BufferInitDescriptor {
                            label: None,
                            usage: #render_path::render_resource::BufferUsages::COPY_DST | #render_path::render_resource::BufferUsages::UNIFORM,
                            contents: buffer.as_ref(),
                        },
                    ))
                }});

                binding_layouts.push(quote!{
                    #render_path::render_resource::BindGroupLayoutEntry {
                        binding: #binding_index,
                        visibility: #render_path::render_resource::ShaderStages::all(),
                        ty: #render_path::render_resource::BindingType::Buffer {
                            ty: #render_path::render_resource::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(<#uniform_struct_name as #render_path::render_resource::ShaderType>::min_size()),
                        },
                        count: None,
                    }
                });
            }
        }
    }

    let generics = ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(TokenStream::from(quote! {
        #(#field_struct_impls)*

        impl #impl_generics #atmosphere_path::model::Atmospheric for #struct_name #ty_generics #where_clause {
            fn as_bind_group(
                &self,
                layout: &#render_path::render_resource::BindGroupLayout,
                render_device: &#render_path::renderer::RenderDevice,
                images: &#render_path::render_asset::RenderAssets<#render_path::texture::GpuImage>,
                fallback_image: &#render_path::texture::FallbackImage,
            ) -> #render_path::render_resource::BindGroup {
                let bindings = vec![#(#binding_impls,)*];

                let bind_group =
                    render_device.create_bind_group(
                        None, &layout, &[#(#bind_group_entries,)*]);
                bind_group
            }

            fn clone_dynamic(&self) -> Box<dyn #atmosphere_path::model::Atmospheric> {
                Box::new((*self).clone())
            }

            fn as_reflect(&self) -> &dyn Reflect {
                self
            }

            fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
                self
            }
        }

        impl #impl_generics #atmosphere_path::model::RegisterAtmosphereModel for #struct_name #ty_generics #where_clause {
            fn register(app: &mut App) {
                use std::borrow::Cow;
                use std::any::TypeId;
                app.register_type::<Self>();

                let handle = #shader_path_impl;

                let render_app = app.sub_app_mut(#render_path::RenderApp);
                let render_device = render_app.world().resource::<#render_path::renderer::RenderDevice>();
                let #atmosphere_path::pipeline::AtmosphereImageBindGroupLayout(image_bind_group_layout) = render_app.world().resource::<#atmosphere_path::pipeline::AtmosphereImageBindGroupLayout>().clone();

                let bind_group_layout = Self::bind_group_layout(render_device);

                let mut pipeline_cache = render_app.world_mut().resource_mut::<#render_path::render_resource::PipelineCache>();

                let pipeline = pipeline_cache.queue_compute_pipeline(#render_path::render_resource::ComputePipelineDescriptor {
                    label: Some(Cow::from("bevy_atmosphere_compute_pipeline")),
                    layout: vec![
                        bind_group_layout.clone(),
                        image_bind_group_layout,
                    ],
                    push_constant_ranges: vec![],
                    shader: handle,
                    shader_defs: vec![],
                    entry_point: Cow::from("main"),
                    zero_initialize_workgroup_memory: true,
                });

                let id = TypeId::of::<Self>();

                let data = #atmosphere_path::model::AtmosphereModelMetadata {
                    id,
                    bind_group_layout,
                    pipeline,
                };

                let type_registry = app.world_mut().resource_mut::<#ecs_path::reflect::AppTypeRegistry>();
                {
                    let mut type_registry = type_registry.write();

                    let mut registration = type_registry.get_mut(std::any::TypeId::of::<Self>()).expect("Type not registered");
                    registration.insert(data);
                }
            }

            fn bind_group_layout(render_device: &#render_path::renderer::RenderDevice) -> #render_path::render_resource::BindGroupLayout {
                render_device.create_bind_group_layout(
                    "atmospheric_bind_group_layout",
                    &[#(#binding_layouts,)*],
                )
            }
        }
    }))
}

/// Represents the arguments for the `uniform` binding attribute.
///
/// If parsed, represents an attribute
/// like `#[uniform(LitInt, Ident)]`
struct UniformBindingMeta {
    lit_int: LitInt,
    _comma: Token![,],
    ident: Ident,
}

/// Represents the arguments for any general binding attribute.
///
/// If parsed, represents an attribute
/// like `#[foo(LitInt, ...)]` where the rest is optional `NestedMeta`.
enum BindingMeta {
    IndexOnly(LitInt),
    IndexWithOptions(BindingIndexOptions),
}

/// Represents the arguments for an attribute with a list of arguments.
///
/// This represents, for example, `#[texture(0, dimension = "2d_array")]`.
struct BindingIndexOptions {
    lit_int: LitInt,
    _comma: Token![,],
    meta_list: Punctuated<syn::Meta, Token![,]>,
}

/// Represents the arguments for the `external` and `internal` binding attributes
struct ShaderPathMeta {
    lit_str: syn::LitStr,
}

impl Parse for BindingMeta {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek2(Token![,]) {
            input.parse().map(Self::IndexWithOptions)
        } else {
            input.parse().map(Self::IndexOnly)
        }
    }
}

impl Parse for BindingIndexOptions {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            lit_int: input.parse()?,
            _comma: input.parse()?,
            meta_list: input.parse_terminated(Meta::parse, Token![,])?,
        })
    }
}

impl Parse for UniformBindingMeta {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            lit_int: input.parse()?,
            _comma: input.parse()?,
            ident: input.parse()?,
        })
    }
}

impl Parse for ShaderPathMeta {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            lit_str: input.parse()?,
        })
    }
}

fn get_uniform_binding_attr(attr: &syn::Attribute) -> Result<(u32, Ident)> {
    let uniform_binding_meta = attr.parse_args_with(UniformBindingMeta::parse)?;

    let binding_index = uniform_binding_meta.lit_int.base10_parse()?;
    let ident = uniform_binding_meta.ident;

    Ok((binding_index, ident))
}

fn get_binding_nested_attr(attr: &syn::Attribute) -> Result<(u32, Vec<syn::Meta>)> {
    let binding_meta = attr.parse_args_with(BindingMeta::parse)?;

    match binding_meta {
        BindingMeta::IndexOnly(lit_int) => Ok((lit_int.base10_parse()?, Vec::new())),
        BindingMeta::IndexWithOptions(BindingIndexOptions {
            lit_int,
            _comma: _,
            meta_list,
        }) => Ok((lit_int.base10_parse()?, meta_list.into_iter().collect())),
    }
}

fn get_shader_path_attr(attr: &syn::Attribute) -> Result<String> {
    let shader_path_meta = attr.parse_args_with(ShaderPathMeta::parse)?;

    let lit_str = shader_path_meta.lit_str.value();

    Ok(lit_str)
}

#[derive(Default)]
enum ShaderStageVisibility {
    #[default]
    All,
    None,
    Flags(VisibilityFlags),
}

#[derive(Default)]
struct VisibilityFlags {
    vertex: bool,
    fragment: bool,
    compute: bool,
}

impl ShaderStageVisibility {
    fn vertex_fragment() -> Self {
        Self::Flags(VisibilityFlags::vertex_fragment())
    }
}

impl VisibilityFlags {
    fn vertex_fragment() -> Self {
        Self {
            vertex: true,
            fragment: true,
            ..Default::default()
        }
    }
}

impl ShaderStageVisibility {
    fn hygienic_quote(&self, path: &proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        match self {
            ShaderStageVisibility::All => quote! { #path::ShaderStages::all() },
            ShaderStageVisibility::None => quote! { #path::ShaderStages::NONE },
            ShaderStageVisibility::Flags(flags) => {
                let mut quoted = Vec::new();

                if flags.vertex {
                    quoted.push(quote! { #path::ShaderStages::VERTEX });
                }
                if flags.fragment {
                    quoted.push(quote! { #path::ShaderStages::FRAGMENT });
                }
                if flags.compute {
                    quoted.push(quote! { #path::ShaderStages::COMPUTE });
                }

                quote! { #(#quoted)|* }
            }
        }
    }
}

const VISIBILITY: Symbol = Symbol("visibility");
const VISIBILITY_VERTEX: Symbol = Symbol("vertex");
const VISIBILITY_FRAGMENT: Symbol = Symbol("fragment");
const VISIBILITY_COMPUTE: Symbol = Symbol("compute");
const VISIBILITY_ALL: Symbol = Symbol("all");
const VISIBILITY_NONE: Symbol = Symbol("none");

fn get_visibility_flag_value(metas: &Punctuated<Meta, Token![,]>) -> Result<ShaderStageVisibility> {
    let mut visibility = VisibilityFlags::vertex_fragment();

    for meta in metas {
        use syn::Meta::Path;
        match meta {
            // Parse `visibility(all)]`.
            Path(path) if path == VISIBILITY_ALL => {
                return Ok(ShaderStageVisibility::All)
            }
            // Parse `visibility(none)]`.
            Path(path) if path == VISIBILITY_NONE => {
                return Ok(ShaderStageVisibility::None)
            }
            // Parse `visibility(vertex, ...)]`.
            Path(path) if path == VISIBILITY_VERTEX => {
                visibility.vertex = true;
            }
            // Parse `visibility(fragment, ...)]`.
            Path(path) if path == VISIBILITY_FRAGMENT => {
                visibility.fragment = true;
            }
            // Parse `visibility(compute, ...)]`.
            Path(path) if path == VISIBILITY_COMPUTE => {
                visibility.compute = true;
            }
            Path(path) => return Err(Error::new_spanned(
                path,
                "Not a valid visibility flag. Must be `all`, `none`, or a list-combination of `vertex`, `fragment` and/or `compute`."
            )),
            _ => return Err(Error::new_spanned(
                meta,
                "Invalid visibility format: `visibility(...)`.",
            )),
        }
    }

    Ok(ShaderStageVisibility::Flags(visibility))
}

#[derive(Default)]
enum BindingTextureDimension {
    D1,
    #[default]
    D2,
    D2Array,
    Cube,
    CubeArray,
    D3,
}

enum BindingTextureSampleType {
    Float { filterable: bool },
    Depth,
    Sint,
    Uint,
}

impl ToTokens for BindingTextureDimension {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            BindingTextureDimension::D1 => quote! { TextureViewDimension::D1 },
            BindingTextureDimension::D2 => quote! { TextureViewDimension::D2 },
            BindingTextureDimension::D2Array => quote! { TextureViewDimension::D2Array },
            BindingTextureDimension::Cube => quote! { TextureViewDimension::Cube },
            BindingTextureDimension::CubeArray => quote! { TextureViewDimension::CubeArray },
            BindingTextureDimension::D3 => quote! { TextureViewDimension::D3 },
        });
    }
}

impl ToTokens for BindingTextureSampleType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            BindingTextureSampleType::Float { filterable } => {
                quote! { TextureSampleType::Float { filterable: #filterable } }
            }
            BindingTextureSampleType::Depth => quote! { TextureSampleType::Depth },
            BindingTextureSampleType::Sint => quote! { TextureSampleType::Sint },
            BindingTextureSampleType::Uint => quote! { TextureSampleType::Uint },
        });
    }
}

struct TextureAttrs {
    dimension: BindingTextureDimension,
    sample_type: BindingTextureSampleType,
    multisampled: bool,
    visibility: ShaderStageVisibility,
}

impl Default for BindingTextureSampleType {
    fn default() -> Self {
        BindingTextureSampleType::Float { filterable: true }
    }
}

impl Default for TextureAttrs {
    fn default() -> Self {
        Self {
            dimension: Default::default(),
            sample_type: Default::default(),
            multisampled: true,
            visibility: Default::default(),
        }
    }
}

const DIMENSION: Symbol = Symbol("dimension");
const SAMPLE_TYPE: Symbol = Symbol("sample_type");
const FILTERABLE: Symbol = Symbol("filterable");
const MULTISAMPLED: Symbol = Symbol("multisampled");

// Values for `dimension` attribute.
const DIM_1D: &str = "1d";
const DIM_2D: &str = "2d";
const DIM_3D: &str = "3d";
const DIM_2D_ARRAY: &str = "2d_array";
const DIM_CUBE: &str = "cube";
const DIM_CUBE_ARRAY: &str = "cube_array";

// Values for sample `type` attribute.
const FLOAT: &str = "float";
const DEPTH: &str = "depth";
const S_INT: &str = "s_int";
const U_INT: &str = "u_int";

fn get_texture_attrs(metas: Vec<Meta>) -> Result<TextureAttrs> {
    let mut dimension = Default::default();
    let mut sample_type = Default::default();
    let mut multisampled = Default::default();
    let mut filterable = None;
    let mut filterable_ident = None;

    let mut visibility = ShaderStageVisibility::vertex_fragment();

    for meta in metas {
        use syn::Meta::{List, NameValue};
        match meta {
            // Parse #[texture(0, dimension = "...")].
            NameValue(m) if m.path == DIMENSION => {
                let value = get_lit_str(DIMENSION, &m.value)?;
                dimension = get_texture_dimension_value(value)?;
            }
            // Parse #[texture(0, sample_type = "...")].
            NameValue(m) if m.path == SAMPLE_TYPE => {
                let value = get_lit_str(SAMPLE_TYPE, &m.value)?;
                sample_type = get_texture_sample_type_value(value)?;
            }
            // Parse #[texture(0, multisampled = "...")].
            NameValue(m) if m.path == MULTISAMPLED => {
                multisampled = get_lit_bool(MULTISAMPLED, &m.value)?;
            }
            // Parse #[texture(0, filterable = "...")].
            NameValue(m) if m.path == FILTERABLE => {
                filterable = get_lit_bool(FILTERABLE, &m.value)?.into();
                filterable_ident = m.path.into();
            }
            // Parse #[texture(0, visibility(...))].
            List(m) if m.path == VISIBILITY => {
                let metas = m.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
                visibility = get_visibility_flag_value(&metas)?;
            }
            NameValue(m) => {
                return Err(Error::new_spanned(
                    m.path,
                    "Not a valid name. Available attributes: `dimension`, `sample_type`, `multisampled`, or `filterable`."
                ));
            }
            _ => {
                return Err(Error::new_spanned(
                    meta,
                    "Not a name value pair: `foo = \"...\"`",
                ));
            }
        }
    }

    // Resolve `filterable` since the float
    // sample type is the one that contains the value.
    if let Some(filterable) = filterable {
        let path = filterable_ident.unwrap();
        match sample_type {
            BindingTextureSampleType::Float { filterable: _ } => {
                sample_type = BindingTextureSampleType::Float { filterable }
            }
            _ => {
                return Err(Error::new_spanned(
                    path,
                    "Type must be `float` to use the `filterable` attribute.",
                ));
            }
        };
    }

    Ok(TextureAttrs {
        dimension,
        sample_type,
        multisampled,
        visibility,
    })
}

fn get_texture_dimension_value(lit_str: &LitStr) -> Result<BindingTextureDimension> {
    match lit_str.value().as_str() {
        DIM_1D => Ok(BindingTextureDimension::D1),
        DIM_2D => Ok(BindingTextureDimension::D2),
        DIM_2D_ARRAY => Ok(BindingTextureDimension::D2Array),
        DIM_3D => Ok(BindingTextureDimension::D3),
        DIM_CUBE => Ok(BindingTextureDimension::Cube),
        DIM_CUBE_ARRAY => Ok(BindingTextureDimension::CubeArray),

        _ => Err(Error::new_spanned(
            lit_str,
            "Not a valid dimension. Must be `1d`, `2d`, `2d_array`, `3d`, `cube` or `cube_array`.",
        )),
    }
}

fn get_texture_sample_type_value(lit_str: &LitStr) -> Result<BindingTextureSampleType> {
    match lit_str.value().as_str() {
        FLOAT => Ok(BindingTextureSampleType::Float { filterable: true }),
        DEPTH => Ok(BindingTextureSampleType::Depth),
        S_INT => Ok(BindingTextureSampleType::Sint),
        U_INT => Ok(BindingTextureSampleType::Uint),

        _ => Err(Error::new_spanned(
            lit_str,
            "Not a valid sample type. Must be `float`, `depth`, `s_int` or `u_int`.",
        )),
    }
}

#[derive(Default)]
struct SamplerAttrs {
    sampler_binding_type: SamplerBindingType,
    visibility: ShaderStageVisibility,
}

#[derive(Default)]
enum SamplerBindingType {
    #[default]
    Filtering,
    NonFiltering,
    Comparison,
}

impl ToTokens for SamplerBindingType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            SamplerBindingType::Filtering => quote! { SamplerBindingType::Filtering },
            SamplerBindingType::NonFiltering => quote! { SamplerBindingType::NonFiltering },
            SamplerBindingType::Comparison => quote! { SamplerBindingType::Comparison },
        });
    }
}

const SAMPLER_TYPE: Symbol = Symbol("sampler_type");

const FILTERING: &str = "filtering";
const NON_FILTERING: &str = "non_filtering";
const COMPARISON: &str = "comparison";

fn get_sampler_attrs(metas: Vec<Meta>) -> Result<SamplerAttrs> {
    let mut sampler_binding_type = Default::default();
    let mut visibility = ShaderStageVisibility::vertex_fragment();

    for meta in metas {
        use syn::Meta::{List, NameValue};
        match meta {
            // Parse #[sampler(0, sampler_type = "..."))].
            NameValue(m) if m.path == SAMPLER_TYPE => {
                let value = get_lit_str(DIMENSION, &m.value)?;
                sampler_binding_type = get_sampler_binding_type_value(value)?;
            }
            // Parse #[sampler(0, visibility(...))].
            List(m) if m.path == VISIBILITY => {
                let metas = m.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
                visibility = get_visibility_flag_value(&metas)?;
            }
            NameValue(m) => {
                return Err(Error::new_spanned(
                    m.path,
                    "Not a valid name. Available attributes: `sampler_type`.",
                ));
            }
            _ => {
                return Err(Error::new_spanned(
                    meta,
                    "Not a name value pair: `foo = \"...\"`",
                ));
            }
        }
    }

    Ok(SamplerAttrs {
        sampler_binding_type,
        visibility,
    })
}

fn get_sampler_binding_type_value(lit_str: &LitStr) -> Result<SamplerBindingType> {
    match lit_str.value().as_str() {
        FILTERING => Ok(SamplerBindingType::Filtering),
        NON_FILTERING => Ok(SamplerBindingType::NonFiltering),
        COMPARISON => Ok(SamplerBindingType::Comparison),

        _ => Err(Error::new_spanned(
            lit_str,
            "Not a valid dimension. Must be `filtering`, `non_filtering`, or `comparison`.",
        )),
    }
}
