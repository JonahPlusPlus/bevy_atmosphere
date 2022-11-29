use crate::model::AtmosphereModel;
use bevy::{prelude::*, render::render_resource::ShaderType};

/// Gradient sky model.
#[derive(AtmosphereModel, ShaderType, Reflect, Debug, Clone)]
#[uniform(0, Gradient)]
#[internal("shaders/gradient.wgsl")]
pub struct Gradient {
    /// Sky Color (Default: `Color::rgb(0.29, 0.41, 0.50)`).
    /// <div style="background-color:rgb(29%, 41%, 50%); width: 10px; padding: 10px; border: 1px solid;"></div>
    ///
    ///
    /// The color of the top.
    pub sky: Color,
    /// Horizon Color (Default: `Color::rgb(0.48, 0.62, 0.69)`).
    /// <div style="background-color:rgb(48%, 62%, 69%); width: 10px; padding: 10px; border: 1px solid;"></div>
    ///
    ///
    /// The color of the sides.
    pub horizon: Color,
    /// Ground Color (Default: `Color::rgb(0.71, 0.69, 0.57)`).
    /// <div style="background-color:rgb(71%, 69%, 57%); width: 10px; padding: 10px; border: 1px solid;"></div>
    ///
    ///
    /// The color of the bottom.
    pub ground: Color,
}

impl Default for Gradient {
    fn default() -> Self {
        Self {
            sky: Color::rgb(0.29, 0.41, 0.50),
            horizon: Color::rgb(0.48, 0.62, 0.69),
            ground: Color::rgb(0.71, 0.69, 0.57),
        }
    }
}

impl From<&Gradient> for Gradient {
    fn from(gradient: &Gradient) -> Self {
        gradient.clone()
    }
}
