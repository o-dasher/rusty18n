#[cfg(feature = "bevy_reflect")]
use bevy_reflect::Reflect;
use derive_more::derive::{AsRef, Deref, Display as DeriveDisplay};
use impl_trait_for_tuples::impl_for_tuples;
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    combinator::{iterator, map},
    sequence::delimited,
    IResult,
};
use std::{collections::HashMap, fmt::Display, hash::Hash};

/// Converts user-provided dynamic arguments into positional `String`s.
///
/// This enables ergonomic calls such as:
/// `dynamic.with((1, "name", 3.5))`
/// for resources that internally render templates such as:
/// `"Hello {name}, total {count}"`.
pub trait IntoDynamicResourceArgs {
    /// Converts `self` into the positional arguments expected by the dynamic resource.
    fn into_dynamic_resource_args(self) -> Vec<String>;
}

#[impl_for_tuples(0, 16)]
#[tuple_types_no_default_trait_bound]
impl IntoDynamicResourceArgs for Tuple {
    for_tuples!( where #( Tuple: Display )* );

    fn into_dynamic_resource_args(self) -> Vec<String> {
        let mut args = Vec::new();
        for_tuples!( #( args.extend(std::iter::once(self.Tuple.to_string())); )* );
        args
    }
}

#[derive(Clone, Copy)]
enum TemplatePart<'a> {
    Text(&'a str),
    Escaped(char),
    Placeholder(&'a str),
}

fn template_part(input: &str) -> IResult<&str, TemplatePart<'_>> {
    alt((
        map(tag("{{"), |_| TemplatePart::Escaped('{')),
        map(tag("}}"), |_| TemplatePart::Escaped('}')),
        map(
            delimited(tag("{"), is_not("{}"), tag("}")),
            TemplatePart::Placeholder,
        ),
        map(is_not("{}"), TemplatePart::Text),
    ))(input)
}

fn for_each_template_part<'a>(template: &'a str, mut visit: impl FnMut(TemplatePart<'a>)) {
    let mut parts = iterator(template, template_part);

    for part in &mut parts {
        visit(part);
    }

    let (rest, ()) = parts
        .finish()
        .unwrap_or_else(|_| panic!("invalid template `{template}`"));

    assert!(rest.is_empty(), "invalid template `{template}`");
}

fn normalize_template(template: &str) -> String {
    let mut rendered = String::new();
    for_each_template_part(template, |part| match part {
        TemplatePart::Text(text) => rendered.push_str(text),
        TemplatePart::Escaped(ch) => rendered.push(ch),
        TemplatePart::Placeholder(name) => {
            rendered.push('{');
            rendered.push_str(name);
            rendered.push('}');
        }
    });

    rendered
}

fn render_template(template: &str, args: &[String], display_text: &str) -> String {
    let mut rendered = String::with_capacity(display_text.len());
    let mut placeholders = Vec::<&str>::new();
    for_each_template_part(template, |part| match part {
        TemplatePart::Text(text) => rendered.push_str(text),
        TemplatePart::Escaped(ch) => rendered.push(ch),
        TemplatePart::Placeholder(name) => {
            let index = placeholders
                .iter()
                .position(|candidate| *candidate == name)
                .unwrap_or_else(|| {
                    placeholders.push(name);
                    placeholders.len() - 1
                });

            let value = args.get(index).unwrap_or_else(|| {
                panic!(
                    "expected {} argument(s) for `{display_text}`, got {}",
                    placeholders.len(),
                    args.len()
                )
            });

            rendered.push_str(value);
        }
    });

    assert!(
        args.len() == placeholders.len(),
        "expected {} argument(s) for `{display_text}`, got {}",
        placeholders.len(),
        args.len()
    );

    rendered
}

/// A struct representing an internationalization (i18n) dynamic resource.
#[derive(Debug, PartialEq, Eq, AsRef, Deref, DeriveDisplay)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
#[display("{}", display_text)]
#[doc(hidden)]
pub struct __I18NDynamicResourceValue {
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    template: String,
    /// Template text with escaped braces resolved.
    #[as_ref(forward)]
    #[deref(forward)]
    display_text: String,
}

impl __I18NDynamicResourceValue {
    /// Creates a new resource by parsing a template with `{placeholder}` markers.
    ///
    /// Positional arguments passed to `.with((...))` are matched by first appearance.
    /// Use `{{` and `}}` to render literal braces.
    #[must_use]
    pub fn new(template: &str) -> Self {
        let display_text = normalize_template(template);

        Self {
            template: template.to_string(),
            display_text,
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
    pub fn with<T>(&self, args: T) -> String
    where
        T: IntoDynamicResourceArgs,
    {
        render_template(
            &self.template,
            &args.into_dynamic_resource_args(),
            &self.display_text,
        )
    }
}

impl PartialEq<str> for __I18NDynamicResourceValue {
    fn eq(&self, other: &str) -> bool {
        self.display_text == other
    }
}

/// A trait for defining fallback behavior in internationalization (i18n).
///
/// It should be used when defining the main i18n component, it will be used
/// when a given i18n resource tries to be acquired but isn't present for the
/// given locale at that moment.
pub trait I18NFallback {
    fn fallback() -> Self;
}

/// This trait groups Key, Value types for a given I18N implementation.
pub trait I18NTrait {
    type K: Eq + Hash + Default + Copy;
    type V: I18NFallback;
}

/// The `I18NStore` wraps a `HashMap` that maps key value pairs of Locale keys and localized
/// implementations.
#[derive(Debug)]
pub struct I18NStore<L: I18NTrait>(pub HashMap<L::K, L::V>);

impl<L: I18NTrait, F: Fn() -> L::V> From<Vec<(L::K, F)>> for I18NStore<L> {
    fn from(value: Vec<(L::K, F)>) -> Self {
        Self(value.into_iter().map(|(k, v)| (k, v())).collect())
    }
}

/// A struct representing access to i18n resources, with fallback support.
///
/// This struct holds references to both the fallback and target i18n resources.
/// It allows accessing resources by applying a provided accessor function.
pub struct I18NAccess<'a, L: I18NTrait> {
    pub fallback: &'a L::V,
    pub to: &'a L::V,
}

impl<L: I18NTrait> I18NAccess<'_, L> {
    /// Acquires a resource by applying the provided accessor function.
    ///
    /// This method attempts to access the target resource first and falls back to
    /// the fallback resource if the target resource is not available.
    ///
    /// # Arguments
    /// * `accessing` - A function that takes a reference to an i18n value and returns
    ///   an optional reference to the desired resource.
    ///
    /// # Returns
    /// A reference to the acquired resource.
    ///
    /// # Panics
    /// Panics if neither the target nor the fallback contains the requested resource.
    pub fn acquire<R>(&self, accessing: fn(&L::V) -> Option<&R>) -> &R {
        accessing(self.to).unwrap_or_else(|| accessing(self.fallback).unwrap())
    }
}

/// A wrapper for i18n resources, providing access and fallback support.
#[derive(Debug)]
pub struct I18NWrapper<K: Eq + Hash + Default + Copy, V: I18NFallback> {
    pub store: I18NStore<Self>,
    fallback: V,
}

impl<K: Eq + Hash + Default + Copy, V: I18NFallback> I18NTrait for I18NWrapper<K, V> {
    type K = K;
    type V = V;
}

impl<K: Eq + Hash + Default + Copy, V: I18NFallback> I18NWrapper<K, V>
where
    Self: I18NTrait<K = K, V = V>,
{
    /// Constructs a new `I18NWrapper` with the provided initial i18n resource store.
    ///
    /// # Arguments
    /// * `store` - A vector of key-value pairs representing the initial i18n resource store.
    ///
    /// # Returns
    /// A new `I18NWrapper` instance.
    #[must_use]
    pub fn new(store: Vec<(K, fn() -> V)>) -> Self {
        let mut store = I18NStore::from(store);

        store.0.insert(K::default(), V::fallback());

        Self {
            store,
            fallback: V::fallback(),
        }
    }

    /// Gets a reference to the default i18n resource.
    const fn ref_default(&self) -> &V {
        &self.fallback
    }

    /// Gets a reference to the i18n resource for the specified locale, if available.
    ///
    /// # Arguments
    /// * `locale` - The locale for which to retrieve the i18n resource.
    ///
    /// # Returns
    /// An optional reference to the i18n resource.
    fn ref_opt(&self, locale: K) -> Option<&V> {
        self.store.0.get(&locale)
    }

    /// Gets a reference to the i18n resource for the specified locale or falls back to the default.
    ///
    /// # Arguments
    /// * `locale` - The locale for which to retrieve the i18n resource.
    ///
    /// # Returns
    /// A reference to the i18n resource.
    fn ref_any(&self, locale: K) -> &V {
        self.ref_opt(locale).unwrap_or_else(|| self.ref_default())
    }

    /// Gets an access object for the specified locale.
    ///
    /// # Arguments
    /// * `locale` - The locale for which to retrieve the i18n resource.
    ///
    /// # Returns
    /// An `I18NAccess` object providing access to the i18n resource for the specified locale.
    pub fn get(&self, locale: K) -> I18NAccess<'_, Self> {
        I18NAccess {
            fallback: self.ref_default(),
            to: self.ref_any(locale),
        }
    }
}
