mod core;
mod macros;
mod reflect;

pub use crate::core::*;

#[cfg(feature = "bevy_reflect")]
pub use crate::reflect::*;
#[cfg(feature = "bevy_reflect")]
pub use bevy_reflect::Reflect;
#[doc(hidden)]
pub use structstruck::strike as __structstruck_strike;

/// Default generated i18n resource type for plain string leaves.
pub type R = I18NDynamicResource;

/// Shared i18n resource type.
pub type I18NDynamicResource = crate::core::I18NDynamicResourceValue;

/// Shared generated formatter resource type.
pub type I18NDynamicFormatter<A> = crate::core::I18NDynamicFormatter<A>;
