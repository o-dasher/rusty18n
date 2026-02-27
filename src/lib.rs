mod core;
mod macros;
mod reflect;

pub use crate::core::*;

#[cfg(feature = "bevy_reflect")]
pub use crate::reflect::*;
#[cfg(feature = "bevy_reflect")]
pub use bevy_reflect::Reflect;
#[doc(hidden)]
pub use rusty18n_macros::__i18n_build_resource;

/// Default generated i18n resource type.
///
/// It stores a compile-time generated formatter and supports positional rendering through `.with((...))`.
/// Plain strings and strings with `{placeholders}` both use this type.
pub type R = I18NDynamicResource;

/// Dynamic i18n resource type.
pub type I18NDynamicResource = crate::core::__I18NDynamicResourceValue;
