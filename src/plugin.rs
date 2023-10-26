//! Provides a `Plugin` for making skyboxes with procedural sky textures.

use bevy::{asset::load_internal_asset, prelude::*, reflect::TypeUuid, render::RenderApp};

use crate::{
    model::{AddAtmosphereModel, AtmosphereModel, AtmosphereModelPrecompute},
    pipeline::*,
    settings::AtmosphereSettings,
};

#[cfg(feature = "utils")]
pub const UTILS_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 744990950565027692);

/// A `Plugin` that adds the prerequisites for a procedural sky.
#[derive(Default, Clone)]
pub struct AtmospherePlugin {
    pub settings: AtmosphereSettings,
}

impl Plugin for AtmospherePlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "utils")]
        load_internal_asset!(
            app,
            UTILS_SHADER_HANDLE,
            "shaders/utils.wgsl",
            Shader::from_wgsl
        );

        #[cfg(feature = "procedural")]
        app.add_plugins(AtmospherePipelinePlugin {
            settings: self.settings.clone(),
        });

        app.add_systems(PostUpdate, atmosphere_update_precompute);
    }

    fn finish(&self, app: &mut App) {
        app.world.get_resource::<AtmosphereImage>().expect("`AtmosphereImage` missing! If the `procedural` feature is disabled, add the resource manually.");
        app.world.get_resource::<AtmospherePrecomputeImage>().expect("`AtmospherePrecomputeImage` missing! If the `procedural` feature is disabled, add the resource manually.");

        let render_app = app.sub_app_mut(RenderApp);

        render_app.init_resource::<AtmosphereImageBindGroupLayout>();
        render_app.init_resource::<AtmospherePrecomputeImageBindGroupLayout>();

        #[cfg(feature = "gradient")]
        app.add_atmosphere_model::<crate::collection::gradient::Gradient>();

        #[cfg(any(feature = "nishita", feature = "applesky"))]
        app.add_atmosphere_model::<crate::collection::nishita_precompute::NishitaPrecompute>();

        #[cfg(feature = "nishita")]
        app.add_atmosphere_model::<crate::collection::nishita::Nishita>();

        #[cfg(feature = "applesky")]
        app.add_atmosphere_model::<crate::collection::applesky::Applesky>();
    }
}

fn atmosphere_update_precompute(
    mut commands: Commands,
    atmosphere: Option<Res<AtmosphereModel>>,
    mut atmosphere_precompute: Option<ResMut<AtmosphereModelPrecompute>>,
) {
    if let Some(atmosphere) = atmosphere {
        atmosphere
            .model()
            .update_precompute(&mut commands, atmosphere_precompute.as_mut());
    }
}
