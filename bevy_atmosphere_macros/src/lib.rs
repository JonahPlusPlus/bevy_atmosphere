use bevy_macro_utils::BevyManifest;
use proc_macro::TokenStream;
use proc_macro_crate::{crate_name, FoundCrate};
use syn::{parse_macro_input, DeriveInput};

mod model;

pub(crate) fn bevy_atmosphere_path() -> syn::Path {
    let found_crate = crate_name("bevy_atmosphere").expect("Failed to find bevy_atmosphere in `Cargo.toml`");

    match found_crate {
        FoundCrate::Itself => BevyManifest::parse_str("crate"),
        FoundCrate::Name(name) => BevyManifest::parse_str(name.as_str())
    }
}

#[proc_macro_derive(AtmosphereModel, attributes(external, internal, uniform, texture, sampler))]
pub fn derive_atmosphere_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    model::derive_atmosphere_model(input).unwrap_or_else(|err| err.to_compile_error().into())
}