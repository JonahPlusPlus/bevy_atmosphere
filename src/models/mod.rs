//! A collection of atmospheric models.
//! 
//! To create a new atmospheric model, refer to [crate::model].

/// [Nishita](crate::models::nishita::Nishita) sky model.
#[cfg(any(doc, feature = "nishita"))]
pub mod nishita;

/// [Gradient](crate::models::gradient::Gradient) sky model.
#[cfg(any(doc, feature = "gradient"))]
pub mod gradient;

