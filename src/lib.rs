#![feature(trait_alias)]

mod core;
mod macros;
mod reflect;

pub use crate::core::*;

#[cfg(feature = "bevy_reflect")]
pub use crate::reflect::*;

/// This type is used to define a simple i18n resource that does not have any dynamic variables
/// that shall be captured by thy. R stands for Resource.
pub type R = String;

/// This type is used to define a dynamic i18n resource that uses argument variables.
/// DR stands for Dynamic Resource.
pub type DR<A> = I18NDynamicResource<A>;
