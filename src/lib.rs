mod core;
mod macros;
mod reflect;

pub use crate::core::*;

#[cfg(feature = "bevy_reflect")]
pub use crate::reflect::*;
#[cfg(feature = "bevy_reflect")]
pub use bevy_reflect::Reflect;

/// Default generated i18n resource type.
///
/// It stores the original template text and supports positional rendering through `.with((...))`.
/// Plain strings and strings with `{placeholders}` both use this type.
pub type R = I18NDynamicResource;

/// Dynamic i18n resource type.
pub type I18NDynamicResource = crate::core::__I18NDynamicResourceValue;
