#[cfg(feature = "bevy_reflect")]
use bevy_reflect::{FromReflect, Reflect};
use derive_more::derive::{AsRef, Deref, Display as DeriveDisplay, From};
use impl_trait_for_tuples::impl_for_tuples;

/// Shared i18n resource value used for plain string leaves.
#[derive(Debug, Default, Clone, Copy, AsRef, Deref, DeriveDisplay, PartialEq, Eq, From)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
#[cfg_attr(feature = "bevy_reflect", reflect(from_reflect = false))]
#[display("{}", _0)]
#[doc(hidden)]
pub struct I18NDynamicResourceValue(
    #[as_ref(forward)]
    #[deref(forward)]
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    &'static str,
);

impl PartialEq<str> for I18NDynamicResourceValue {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

#[cfg(feature = "bevy_reflect")]
impl FromReflect for I18NDynamicResourceValue {
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        reflect.downcast_ref::<Self>().copied()
    }
}

#[doc(hidden)]
pub trait IntoFormatterArgs {
    type Args;

    fn with_formatter_args<R, F>(self, f: F) -> R
    where
        F: FnOnce(Self::Args) -> R;
}

#[impl_for_tuples(0, 16)]
#[tuple_types_custom_trait_bound(::core::fmt::Display)]
impl IntoFormatterArgs for Tuple {
    for_tuples!( type Args = ( #( ::std::string::String ),* ); );
    fn with_formatter_args<R, F>(self, f: F) -> R
    where
        F: FnOnce(Self::Args) -> R,
    {
        f(for_tuples!( ( #( self.Tuple.to_string() ),* ) ))
    }
}

/// Shared formatter resource used for dynamic string leaves.
#[derive(Debug)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
#[cfg_attr(feature = "bevy_reflect", reflect(from_reflect = false))]
#[doc(hidden)]
pub struct I18NDynamicFormatter<A>(
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))] fn(A) -> String,
);

impl<A> I18NDynamicFormatter<A> {
    #[doc(hidden)]
    #[must_use]
    pub const fn new(render: fn(A) -> String) -> Self {
        Self(render)
    }

    #[must_use]
    pub fn with<T>(&self, args: T) -> String
    where
        T: IntoFormatterArgs<Args = A>,
    {
        args.with_formatter_args(self.0)
    }
}

impl<A> Default for I18NDynamicFormatter<A> {
    fn default() -> Self {
        Self(|_| String::new())
    }
}

#[cfg(feature = "bevy_reflect")]
impl<A> FromReflect for I18NDynamicFormatter<A>
where
    Self: Reflect,
{
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
        reflect.downcast_ref::<Self>().map(|value| Self(value.0))
    }
}
