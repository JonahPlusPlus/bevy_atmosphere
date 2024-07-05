use crate::model::Atmospheric;
use bevy::{prelude::*, render::render_resource::ShaderType};

/// The Gradient sky model.
///
/// A simple gradient for creating a stylized environment.
#[derive(Atmospheric, ShaderType, Reflect, Debug, Clone)]
#[uniform(0, Gradient)]
#[internal("shaders/gradient.wgsl")]
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
