mod core;
mod macros;
mod reflect;

pub use crate::core::*;

#[cfg(feature = "bevy_reflect")]
pub use crate::reflect::*;
#[cfg(feature = "bevy_reflect")]
pub use bevy_reflect::Reflect;
#[doc(hidden)]
pub use deep_struct_update::update as __deep_update;
#[doc(hidden)]
pub use structstruck::strike as __structstruck_strike;

/// Default generated i18n resource type.
///
/// It stores a compile-time validated template and supports inferred positional
/// rendering through `.with(&[...])`.
/// Plain strings and strings with `{placeholders}` both use this type.
pub type R<const N: usize = 0> = I18NDynamicResource<N>;

/// Dynamic i18n resource type.
pub type I18NDynamicResource<const N: usize = 0> = crate::core::I18NDynamicResourceValue<N>;
