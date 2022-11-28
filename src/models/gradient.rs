use bevy::{prelude::*, render::render_resource::ShaderType};
use bevy_atmosphere_macros::AtmosphereModel;

use crate::model::AtmosphereModel;

#[derive(AtmosphereModel, ShaderType, Reflect, Debug, Clone)]
#[uniform(0, Gradient)]
#[internal("shaders/gradient.wgsl")]
pub struct Gradient {
    pub ground: Color,
    pub horizon: Color,
    pub sky: Color,
}

impl From<&Gradient> for Gradient {
    fn from(gradient: &Gradient) -> Self {
        gradient.clone()
    }
}
