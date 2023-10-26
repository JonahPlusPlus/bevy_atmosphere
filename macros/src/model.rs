use ::serde::de;
use bevy_macro_utils::BevyManifest;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use serde::Deserialize;
use serde_tokenstream::{from_tokenstream, ParseWrapper};
use strum_macros::{AsRefStr, EnumDiscriminants};
use syn::{Error, Meta, Result, TypePath};

#[derive(PartialEq, Clone, Debug)]
enum ShaderPathType {
    None,
    External(String),
    Internal(String),
}

pub fn derive_atmospheric(ast: syn::DeriveInput) -> Result<TokenStream> {
    let manifest = BevyManifest::default();
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

    let mut is_precompute = false;
    let mut shader_path = ShaderPathType::None;
    let mut precompute = None::<TypePath>;
    let mut defs = Vec::<ShaderDefVal>::new();

    // Read struct-level attributes
    for attr in &ast.attrs {
        let meta: StructMeta = match &attr.meta {
            Meta::List(attr) => match from_tokenstream::<Unbraced<StructMetaDiscriminants>>(
                &attr.path.to_token_stream(),
            ) {
                Ok(_) => match from_tokenstream::<Unbraced<_>>(&attr.to_token_stream()) {
                    Ok(meta) => meta.0,
                    Err(e) => return Err(Error::new_spanned(attr, e)),
                },
                Err(_) => continue,
            },
            _ => continue,
        };
        match meta {
            StructMeta::external(path) => {
                if shader_path != ShaderPathType::None {
                    return Err(Error::new_spanned(attr, "Shader path already set"));
                }
                shader_path = ShaderPathType::External(path);
            }
            StructMeta::internal(path) => {
                if shader_path != ShaderPathType::None {
                    return Err(Error::new_spanned(attr, "Shader path already set"));
                }
                shader_path = ShaderPathType::Internal(path);
            }
            StructMeta::after(precompute_type) => {
                let ty = precompute_type.into_inner();
                if precompute.is_some() {
                    return Err(Error::new_spanned(
                        attr,
                        "Atmospheric dependency already set",
                    ));
                }
                precompute = Some(ty);
            }
            StructMeta::defines(defines) => {
                if !defs.is_empty() {
                    return Err(Error::new_spanned(attr, "Defines already set"));
                }
                defs = defines;
            }
            StructMeta::precompute() => {
                is_precompute = true;
            }
        }
    }

    let shader_path_impl = match shader_path {
        ShaderPathType::None => panic!("Expected `external` or `internal` attribute"),
        ShaderPathType::External(s) => quote! {
            {
                let asset_server = app.world.resource::<AssetServer>();

                asset_server.load(#s)
            }
        },
        ShaderPathType::Internal(s) => quote! {
            {
                use bevy::reflect::TypeUuid;
                let handle = #asset_path::HandleUntyped::weak_from_u64(#render_path::render_resource::Shader::TYPE_UUID, #id);

                let internal_handle = handle.clone();
                #asset_path::load_internal_asset!(
                    app,
                    internal_handle,
                    concat!(env!("CARGO_MANIFEST_DIR"), "/src/", #s),
                    Shader::from_wgsl
                );

                handle.typed()
            }
        },
    };

    // Produce impls for fields with uniform bindings
    let struct_name = &ast.ident;
    let generics = ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let slayout = match is_precompute {
        true => quote! { vec![
            bind_group_layout.clone(),
            precompute_image_bind_group_layout,
        ] },
        false => quote! { vec![
            bind_group_layout.clone(),
            image_bind_group_layout,
        ] },
    };
    let sprecompute = precompute
        .map(|t| quote! {
            let pre = #t::from(self);
            if let Some(precompute) = precompute {
                if let Some(precompute) = <_ as std::ops::Deref>::deref(precompute).0.to_ref::<#t>() {
                    if let Some(true) = <#t as Reflect>::reflect_partial_eq(precompute, &pre) {
                        return;
                    }
                }
            }
            commands.insert_resource(#atmosphere_path::model::AtmosphereModelPrecompute(#atmosphere_path::model::AtmosphereModel::new(pre)));
        })
        .unwrap_or(quote! {
            if let Some(_) = precompute {
                commands.remove_resource::<#atmosphere_path::model::AtmosphereModelPrecompute>();
            }
         });

    let def_iter = defs.into_iter();
    Ok(TokenStream::from(quote! {
        impl #impl_generics #atmosphere_path::model::Atmospheric for #struct_name #ty_generics #where_clause {
            fn as_bind_group(
                &self,
                layout: &#render_path::render_resource::BindGroupLayout,
                render_device: &#render_path::renderer::RenderDevice,
                images: &#render_path::render_asset::RenderAssets<#render_path::texture::Image>,
                fallback_image: &#render_path::texture::FallbackImage,
            ) -> Result<#atmosphere_path::model::PreparedBindGroup, #render_path::render_resource::AsBindGroupError> {
                Ok(#atmosphere_path::model::PreparedBindGroup::from(<Self as #render_path::render_resource::AsBindGroup>::as_bind_group(
                    self,
                    layout,
                    render_device,
                    images,
                    fallback_image,
                )?))
            }

            fn update_precompute<'a>(&'a self, commands: &'a mut #ecs_path::system::Commands, precompute: Option<&'a mut #ecs_path::change_detection::ResMut<#atmosphere_path::model::AtmosphereModelPrecompute>>) {
                #sprecompute
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
                let render_device = render_app.world.resource::<#render_path::renderer::RenderDevice>();
                let #atmosphere_path::pipeline::AtmosphereImageBindGroupLayout(image_bind_group_layout) = render_app.world.resource::<#atmosphere_path::pipeline::AtmosphereImageBindGroupLayout>().clone();
                let #atmosphere_path::pipeline::AtmospherePrecomputeImageBindGroupLayout(precompute_image_bind_group_layout) = render_app.world.resource::<#atmosphere_path::pipeline::AtmospherePrecomputeImageBindGroupLayout>().clone();

                let bind_group_layout = Self::bind_group_layout(render_device);

                let mut pipeline_cache = render_app.world.resource_mut::<#render_path::render_resource::PipelineCache>();

                let pipeline = pipeline_cache.queue_compute_pipeline(#render_path::render_resource::ComputePipelineDescriptor {
                    label: Some(Cow::from("bevy_atmosphere_compute_pipeline")),
                    layout: #slayout,
                    push_constant_ranges: vec![],
                    shader: handle,
                    shader_defs: vec![#(#render_path::render_resource::#def_iter,)*],
                    entry_point: Cow::from("main"),
                });

                let id = TypeId::of::<Self>();

                let data = #atmosphere_path::model::AtmosphereModelMetadata {
                    id,
                    bind_group_layout,
                    pipeline,
                };

                let type_registry = app.world.resource_mut::<#ecs_path::reflect::AppTypeRegistry>();
                {
                    let mut type_registry = type_registry.write();

                    let mut registration = type_registry.get_mut(std::any::TypeId::of::<Self>()).expect("Type not registered");
                    registration.insert(data);
                }
            }
        }
    }))
}

/// Represents the arguments for any struct attribute.
#[derive(Debug, Deserialize, EnumDiscriminants)]
#[strum_discriminants(derive(Deserialize))]
enum StructMeta {
    external(String),
    internal(String),
    after(ParseWrapper<TypePath>),
    defines(Vec<ShaderDefVal>),
    precompute(),
}

#[derive(Debug, Deserialize, EnumDiscriminants)]
#[strum_discriminants(derive(Deserialize, AsRefStr))]
enum ShaderDefVal {
    Bool(String, bool),
    Int(String, i32),
    UInt(String, u32),
    Generic(ShaderDefValDiscriminants, String, ParseWrapper<Ident>),
}

impl ToTokens for ShaderDefVal {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            ShaderDefVal::Bool(name, val) => quote! { ShaderDefVal::Bool(#name.into(), #val) },
            ShaderDefVal::Int(name, val) => quote! { ShaderDefVal::Int(#name.into(), #val) },
            ShaderDefVal::UInt(name, val) => quote! { ShaderDefVal::UInt(#name.into(), #val) },
            ShaderDefVal::Generic(dis, name, val) => {
                let v: &Ident = <ParseWrapper<Ident> as std::ops::Deref>::deref(val);
                quote! { ShaderDefVal::#dis(#name.into(), #v) }
            }
        })
    }
}
impl ToTokens for ShaderDefValDiscriminants {
    fn to_tokens(&self, tokens: &mut ::proc_macro2::TokenStream) {
        let ident = ::proc_macro2::Ident::new(self.as_ref(), ::proc_macro2::Span::call_site());
        tokens.extend(quote! { #ident });
    }
}

#[derive(Debug)]
struct Unbraced<T>(T);

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Unbraced<T> {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct UnbracedVisitor<T>(std::marker::PhantomData<T>);
        impl<'de, T: Deserialize<'de>> de::Visitor<'de> for UnbracedVisitor<T> {
            type Value = Unbraced<T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a braced struct")
            }

            fn visit_map<A>(self, mut map: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let value = map.next_value()?;
                Ok(Unbraced(value))
            }
        }
        deserializer.deserialize_any(UnbracedVisitor(std::marker::PhantomData))
    }
}

#[derive(Debug)]
struct Unbracketed<T>(T);

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Unbracketed<T> {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct UnbracketedVisitor<T>(std::marker::PhantomData<T>);
        impl<'de, T: Deserialize<'de>> de::Visitor<'de> for UnbracketedVisitor<T> {
            type Value = Unbracketed<T>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a braced struct")
            }

            fn visit_seq<A>(self, seq: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let value = T::deserialize(de::value::SeqAccessDeserializer::new(seq))?;
                Ok(Unbracketed(value))
            }
        }
        deserializer.deserialize_any(UnbracketedVisitor(std::marker::PhantomData))
    }
}
