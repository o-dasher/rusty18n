#[cfg(feature = "bevy_reflect")]
use bevy_reflect::Reflect;
use derive_more::derive::{AsRef, Deref, Display as DeriveDisplay};
use impl_trait_for_tuples::impl_for_tuples;
use std::fmt::Display;

type I18NRenderFn<M> = fn(&[String], Option<fn() -> M>) -> String;

/// Converts user-provided dynamic arguments into positional `String`s.
///
/// This enables ergonomic calls such as:
/// `dynamic.with((1, "name", 3.5))`
/// for resources that internally render templates such as:
/// `"Hello {name}, total {count}"`.
pub trait IntoDynamicResourceArgs {
    type Marker;

    /// Converts `self` into the positional arguments expected by the dynamic resource.
    fn into_dynamic_resource_args(self) -> Vec<String>;
}

#[impl_for_tuples(0, 16)]
#[tuple_types_no_default_trait_bound]
impl IntoDynamicResourceArgs for Tuple {
    for_tuples!( where #( Tuple: Display )* );
    for_tuples!( type Marker = ( #( () ),* ); );

    fn into_dynamic_resource_args(self) -> Vec<String> {
        let mut args = Vec::new();
        for_tuples!( #( args.extend(std::iter::once(self.Tuple.to_string())); )* );
        args
    }
}

/// A struct representing an internationalization (i18n) dynamic resource.
#[derive(Debug, Default, AsRef, Deref, DeriveDisplay)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
#[display("{}", display_text)]
#[doc(hidden)]
pub struct I18NDynamicResourceValue<M = ()> {
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    template: &'static str,
    /// Template text with escaped braces resolved.
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    #[as_ref(forward)]
    #[deref(forward)]
    display_text: &'static str,
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    render: Option<I18NRenderFn<M>>,
}

impl<M> I18NDynamicResourceValue<M> {
    #[doc(hidden)]
    #[must_use]
    pub const fn new_static(
        display_text: &'static str,
        template: &'static str,
        render: I18NRenderFn<M>,
    ) -> Self {
        Self {
            template,
            display_text,
            render: Some(render),
        }
    }

    /// Invokes the dynamic resource with user-provided arguments.
    ///
    /// # Arguments
    /// * `args` - Arguments that can be converted into positional strings.
    ///   Each tuple item must implement `Display`.
    ///
    /// # Returns
    /// A string representing the localized resource.
    ///
    #[must_use]
    pub fn with<T>(&self, args: T) -> String
    where
        T: IntoDynamicResourceArgs<Marker = M>,
    {
        self.render.map_or_else(String::new, |render| {
            render(&args.into_dynamic_resource_args(), None)
        })
    }
}

impl<M> PartialEq for I18NDynamicResourceValue<M> {
    fn eq(&self, other: &Self) -> bool {
        self.template == other.template
    }
}

impl<M> Eq for I18NDynamicResourceValue<M> {}

impl<M> PartialEq<str> for I18NDynamicResourceValue<M> {
    fn eq(&self, other: &str) -> bool {
        self.display_text == other
    }
}
