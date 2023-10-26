//! Provides a collection of atmospheric models.
//!
//! To create a new atmospheric model, refer to [crate::model].

/// [`Applesky`](crate::collection::applesky::Applesky) sky model.
#[cfg(any(doc, feature = "applesky"))]
pub mod applesky;

#[cfg(any(doc, feature = "precompute"))]
pub mod nishita_precompute;

/// [`Nishita`](crate::collection::nishita::Nishita) sky model.
#[cfg(any(doc, feature = "nishita"))]
pub mod nishita;

/// [`Gradient`](crate::collection::gradient::Gradient) sky model.
#[cfg(any(doc, feature = "gradient"))]
pub mod gradient;
