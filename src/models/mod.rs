//! A collection of atmospheric models.
//! 
//! To create a new atmospheric model, refer to [crate::model].

#[cfg(any(doc, feature = "nishita"))]
pub mod nishita;

#[cfg(any(doc, feature = "gradient"))]
pub mod gradient;

