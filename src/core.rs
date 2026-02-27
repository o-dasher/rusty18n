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

/// Errors produced by `rusty18n`.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The translation template contains invalid brace syntax.
    #[error("invalid template `{template}`")]
    InvalidTemplate { template: String },
    /// A dynamic resource was rendered with the wrong number of arguments.
    #[error("expected {expected} argument(s) for `{template}`, got {got}")]
    InvalidArgumentCount {
        template: String,
        expected: usize,
        got: usize,
    },
    /// The default locale entry is missing from the locale store.
    #[error("missing fallback locale in store")]
    MissingFallbackLocale,
    /// The requested resource is missing from both the target and fallback locale values.
    #[error("resource is missing from both target and fallback locales")]
    MissingResource,
}

/// A crate-local `Result` alias.
pub type Result<T> = std::result::Result<T, Error>;

/// A locale constructor used by the dynamic wrapper.
pub type I18NLocaleLoader<V> = fn() -> V;

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

fn parse_template(template: &str) -> Result<(String, Vec<String>)> {
    let mut rendered = String::new();
    let mut placeholders = Vec::new();

    for_each_template_part(template, |part| {
        match part {
            TemplatePart::Text(text) => rendered.push_str(text),
            TemplatePart::Escaped(ch) => rendered.push(ch),
            TemplatePart::Placeholder(name) => {
                rendered.push('{');
                rendered.push_str(name);
                rendered.push('}');

                if !placeholders
                    .iter()
                    .any(|candidate: &String| candidate == name)
                {
                    placeholders.push(name.to_string());
                }
            }
        }
        Ok(())
    })?;

    Ok((rendered, placeholders))
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

fn for_each_template_part<'a>(
    template: &'a str,
    mut visit: impl FnMut(TemplatePart<'a>) -> Result<()>,
) -> Result<()> {
    let mut parts = iterator(template, template_part);

    for part in &mut parts {
        visit(part)?;
    }

    let (rest, ()) = parts.finish().map_err(|_| Error::InvalidTemplate {
        template: template.to_string(),
    })?;

    if rest.is_empty() {
        Ok(())
    } else {
        Err(Error::InvalidTemplate {
            template: template.to_string(),
        })
    }
}

fn build_static_template(template: &'static str) -> (String, Vec<String>) {
    let bytes = template.as_bytes();
    let mut display_text = String::with_capacity(template.len());
    let mut placeholders = Vec::new();
    let mut index = 0;
    let mut text_start = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'{' if index + 1 < bytes.len() && bytes[index + 1] == b'{' => {
                display_text.push_str(&template[text_start..index]);
                display_text.push('{');
                index += 2;
                text_start = index;
            }
            b'}' if index + 1 < bytes.len() && bytes[index + 1] == b'}' => {
                display_text.push_str(&template[text_start..index]);
                display_text.push('}');
                index += 2;
                text_start = index;
            }
            b'{' => {
                let mut placeholder_end = index + 1;
                let mut is_placeholder = placeholder_end < bytes.len();

                while placeholder_end < bytes.len() && bytes[placeholder_end] != b'}' {
                    if bytes[placeholder_end] == b'{' {
                        is_placeholder = false;
                        break;
                    }
                    placeholder_end += 1;
                }

                if is_placeholder && placeholder_end < bytes.len() && placeholder_end > index + 1 {
                    display_text.push_str(&template[text_start..index]);

                    let name = &template[index + 1..placeholder_end];
                    display_text.push('{');
                    display_text.push_str(name);
                    display_text.push('}');

                    if !placeholders
                        .iter()
                        .any(|candidate: &String| candidate == name)
                    {
                        placeholders.push(name.to_string());
                    }

                    index = placeholder_end + 1;
                    text_start = index;
                } else {
                    index += 1;
                }
            }
            _ => index += 1,
        }
    }

    display_text.push_str(&template[text_start..]);

    (display_text, placeholders)
}

#[doc(hidden)]
pub const fn __assert_valid_template(template: &str) {
    let bytes = template.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'{' => {
                if index + 1 < bytes.len() && bytes[index + 1] == b'{' {
                    index += 2;
                    continue;
                }

                index += 1;
                let start = index;

                while index < bytes.len() && bytes[index] != b'}' {
                    assert!(bytes[index] != b'{', "invalid template literal");
                    index += 1;
                }

                assert!(
                    !(index == start || index >= bytes.len()),
                    "invalid template literal"
                );

                index += 1;
            }
            b'}' => {
                assert!(
                    index + 1 < bytes.len() && bytes[index + 1] == b'}',
                    "invalid template literal"
                );
                index += 2;
            }
            _ => index += 1,
        }
    }
}

fn render_template(template: &str, args: &[String], placeholders: &[String]) -> Result<String> {
    let mut rendered = String::with_capacity(template.len());

    if args.len() != placeholders.len() {
        return Err(Error::InvalidArgumentCount {
            template: template.to_string(),
            expected: placeholders.len(),
            got: args.len(),
        });
    }

    for_each_template_part(template, |part| {
        match part {
            TemplatePart::Text(text) => rendered.push_str(text),
            TemplatePart::Escaped(ch) => rendered.push(ch),
            TemplatePart::Placeholder(name) => {
                let index = placeholders
                    .iter()
                    .position(|candidate| candidate == name)
                    .ok_or_else(|| Error::InvalidTemplate {
                        template: template.to_string(),
                    })?;

                let value = args.get(index).ok_or_else(|| Error::InvalidArgumentCount {
                    template: template.to_string(),
                    expected: placeholders.len(),
                    got: args.len(),
                })?;

                rendered.push_str(value);
            }
        }
        Ok(())
    })?;

    Ok(rendered)
}

/// A struct representing an internationalization (i18n) dynamic resource.
#[derive(Debug, PartialEq, Eq, AsRef, Deref, DeriveDisplay)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
#[display("{}", display_text)]
#[doc(hidden)]
pub struct __I18NDynamicResourceValue {
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    template: String,
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    placeholders: Vec<String>,
    /// Template text with escaped braces resolved.
    #[as_ref(forward)]
    #[deref(forward)]
    display_text: String,
}

impl __I18NDynamicResourceValue {
    #[must_use]
    fn new_static(template: &'static str) -> Self {
        let (display_text, placeholders) = build_static_template(template);

        Self {
            template: template.to_string(),
            placeholders,
            display_text,
        }
    }

    /// Creates a new resource by parsing a template with `{placeholder}` markers.
    ///
    /// Positional arguments passed to `.with((...))` are matched by first appearance.
    /// Use `{{` and `}}` to render literal braces.
    ///
    /// # Errors
    /// Returns [`Error::InvalidTemplate`] when the template contains invalid brace syntax.
    pub fn new(template: &str) -> Result<Self> {
        let (display_text, placeholders) = parse_template(template)?;

        Ok(Self {
            template: template.to_string(),
            placeholders,
            display_text,
        })
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
    /// # Errors
    /// Returns [`Error::InvalidTemplate`] when the template is malformed, or
    /// [`Error::InvalidArgumentCount`] when the provided arguments do not match
    /// the inferred placeholder count.
    pub fn with<T>(&self, args: T) -> Result<String>
    where
        T: IntoDynamicResourceArgs,
    {
        render_template(
            &self.template,
            &args.into_dynamic_resource_args(),
            &self.placeholders,
        )
    }
}

#[doc(hidden)]
pub mod __private {
    use super::__I18NDynamicResourceValue;

    #[must_use]
    pub fn new_static_resource(template: &'static str) -> __I18NDynamicResourceValue {
        __I18NDynamicResourceValue::new_static(template)
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
pub trait I18NFallback: Sized {
    /// Constructs the canonical fallback value.
    fn fallback() -> Self;
}

/// This trait groups Key, Value types for a given I18N implementation.
pub trait I18NTrait {
    type K: Eq + Hash + Default + Copy;
    type V: I18NFallback;
}

/// The `I18NStore` wraps a `HashMap` that maps key value pairs of `Locale` keys and localized
/// implementations.
#[derive(Debug)]
pub struct I18NStore<K: Eq + Hash + Copy, V>(pub HashMap<K, V>);

impl<K: Eq + Hash + Copy, V> From<Vec<(K, V)>> for I18NStore<K, V> {
    fn from(value: Vec<(K, V)>) -> Self {
        Self(value.into_iter().collect())
    }
}

#[derive(Debug)]
struct I18NCore<K: Eq + Hash + Default + Copy, V: I18NFallback> {
    store: I18NStore<K, V>,
}

impl<K: Eq + Hash + Default + Copy, V: I18NFallback> I18NCore<K, V> {
    fn new_empty() -> Self {
        let mut store = HashMap::new();
        store.insert(K::default(), V::fallback());

        Self {
            store: I18NStore(store),
        }
    }

    fn new_loaded(store: Vec<(K, V)>) -> Self {
        let mut store = I18NStore::from(store);

        if let std::collections::hash_map::Entry::Vacant(entry) = store.0.entry(K::default()) {
            entry.insert(V::fallback());
        }

        Self { store }
    }

    fn fallback_ref(&self) -> Result<&V> {
        self.store
            .0
            .get(&K::default())
            .ok_or(Error::MissingFallbackLocale)
    }

    fn loaded_ref(&self, locale: K) -> Option<&V> {
        self.store.0.get(&locale)
    }

    const fn store_ref(&self) -> &I18NStore<K, V> {
        &self.store
    }

    fn insert_loaded(&mut self, locale: K, value: V) -> Option<V> {
        self.store.0.insert(locale, value)
    }

    fn unload(&mut self, locale: K) -> Option<V> {
        if locale == K::default() {
            None
        } else {
            self.store.0.remove(&locale)
        }
    }

    fn unload_all(&mut self) {
        let default_locale = K::default();
        self.store.0.retain(|locale, _| *locale == default_locale);
        if let std::collections::hash_map::Entry::Vacant(entry) = self.store.0.entry(default_locale)
        {
            entry.insert(V::fallback());
        }
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
    /// # Errors
    /// Returns [`Error::MissingResource`] when neither the target nor the fallback
    /// contains the requested resource.
    pub fn acquire<R>(&self, accessing: fn(&L::V) -> Option<&R>) -> Result<&R> {
        accessing(self.to)
            .or_else(|| accessing(self.fallback))
            .ok_or(Error::MissingResource)
    }
}

/// A wrapper for eagerly loaded i18n resources, providing access and fallback support.
#[derive(Debug)]
pub struct I18NWrapper<K: Eq + Hash + Default + Copy, V: I18NFallback>(I18NCore<K, V>);

impl<K: Eq + Hash + Default + Copy, V: I18NFallback> I18NTrait for I18NWrapper<K, V> {
    type K = K;
    type V = V;
}

impl<K: Eq + Hash + Default + Copy, V: I18NFallback> I18NWrapper<K, V>
where
    Self: I18NTrait<K = K, V = V>,
{
    /// Constructs a new eagerly loaded `I18NWrapper`.
    ///
    /// # Arguments
    /// * `store` - A vector of key-value pairs representing the initial i18n resource store.
    ///
    /// # Returns
    /// A new `I18NWrapper` instance.
    ///
    #[must_use]
    pub fn new(store: Vec<(K, V)>) -> Self {
        Self(I18NCore::new_loaded(store))
    }

    /// Constructs a new `I18NDynamicWrapper`.
    ///
    /// This provides the previous dynamic-loading constructor entry point without
    /// forcing eager wrappers to carry loader state.
    ///
    /// # Arguments
    /// * `loaders` - A vector of locale keys and locale constructor functions.
    ///
    /// # Returns
    /// A new `I18NDynamicWrapper` instance.
    ///
    #[must_use]
    pub fn new_dynamic(loaders: Vec<(K, I18NLocaleLoader<V>)>) -> I18NDynamicWrapper<K, V> {
        I18NDynamicWrapper::new(loaders)
    }

    /// Returns the eagerly loaded store.
    #[must_use]
    pub const fn store(&self) -> &I18NStore<K, V> {
        self.0.store_ref()
    }

    /// Returns whether the locale is currently resident in memory.
    ///
    /// For eager wrappers, any registered locale is already loaded.
    #[must_use]
    pub fn is_loaded(&self, locale: K) -> bool {
        self.0.loaded_ref(locale).is_some()
    }

    /// Creates an access wrapper for the requested locale.
    ///
    /// # Errors
    /// Returns [`Error::MissingFallbackLocale`] when the default locale entry is absent.
    pub fn get(&self, locale: K) -> Result<I18NAccess<'_, Self>> {
        let fallback = self.0.fallback_ref()?;
        let to = self.0.loaded_ref(locale).unwrap_or(fallback);

        Ok(I18NAccess { fallback, to })
    }
}

#[derive(Debug)]
struct I18NDynamicCore<K: Eq + Hash + Default + Copy, V: I18NFallback> {
    loaded: I18NCore<K, V>,
    loaders: HashMap<K, I18NLocaleLoader<V>>,
}

/// A wrapper for dynamically loaded i18n resources.
#[derive(Debug)]
pub struct I18NDynamicWrapper<K: Eq + Hash + Default + Copy, V: I18NFallback>(
    I18NDynamicCore<K, V>,
);

impl<K: Eq + Hash + Default + Copy, V: I18NFallback> I18NTrait for I18NDynamicWrapper<K, V> {
    type K = K;
    type V = V;
}

impl<K: Eq + Hash + Default + Copy, V: I18NFallback> I18NDynamicWrapper<K, V>
where
    Self: I18NTrait<K = K, V = V>,
{
    /// Constructs a new `I18NDynamicWrapper`.
    ///
    /// # Arguments
    /// * `loaders` - A vector of locale keys and locale constructor functions.
    ///
    /// # Returns
    /// A new `I18NDynamicWrapper` instance with the fallback locale preloaded.
    ///
    #[must_use]
    pub fn new(loaders: Vec<(K, I18NLocaleLoader<V>)>) -> Self {
        Self(I18NDynamicCore {
            loaded: I18NCore::new_empty(),
            loaders: loaders.into_iter().collect(),
        })
    }

    /// Returns the currently loaded locale store.
    #[must_use]
    pub const fn store(&self) -> &I18NStore<K, V> {
        self.0.loaded.store_ref()
    }

    /// Registers or replaces a locale loader.
    ///
    /// # Arguments
    /// * `locale` - The locale key to register.
    /// * `loader` - A constructor function for the locale value.
    ///
    /// # Returns
    /// The previously registered loader for the locale, if any.
    pub fn register_locale(
        &mut self,
        locale: K,
        loader: I18NLocaleLoader<V>,
    ) -> Option<I18NLocaleLoader<V>> {
        self.0.loaders.insert(locale, loader)
    }

    /// Unregisters a locale loader and unloads that locale if it is currently loaded.
    ///
    /// # Arguments
    /// * `locale` - The locale key to unregister.
    ///
    /// # Returns
    /// The previously registered loader for the locale, if any.
    pub fn unregister_locale(&mut self, locale: K) -> Option<I18NLocaleLoader<V>> {
        self.0.loaded.unload(locale);
        self.0.loaders.remove(&locale)
    }

    /// Loads a single locale into memory using its registered loader.
    ///
    /// # Arguments
    /// * `locale` - The locale key to load.
    ///
    /// # Returns
    /// `true` if the locale had a registered loader and is now loaded.
    ///
    #[must_use]
    pub fn load(&mut self, locale: K) -> bool {
        self.0.loaders.get(&locale).copied().is_some_and(|load| {
            self.0.loaded.insert_loaded(locale, load());
            true
        })
    }

    /// Loads all registered locales into memory.
    pub fn load_all(&mut self) {
        let loaders = self
            .0
            .loaders
            .iter()
            .map(|(&locale, &load)| (locale, load))
            .collect::<Vec<_>>();

        for (locale, load) in loaders {
            self.0.loaded.insert_loaded(locale, load());
        }
    }

    /// Unloads a single locale from memory.
    ///
    /// # Arguments
    /// * `locale` - The locale key to unload.
    ///
    /// # Returns
    /// The previously loaded locale value, if it was loaded.
    pub fn unload(&mut self, locale: K) -> Option<V> {
        self.0.loaded.unload(locale)
    }

    /// Unloads all currently loaded locales.
    ///
    pub fn unload_all(&mut self) {
        self.0.loaded.unload_all();
    }

    /// Returns whether a locale is currently loaded.
    #[must_use]
    pub fn is_loaded(&self, locale: K) -> bool {
        self.0.loaded.loaded_ref(locale).is_some()
    }

    /// Returns whether a locale has a registered loader.
    #[must_use]
    pub fn is_registered(&self, locale: K) -> bool {
        self.0.loaders.contains_key(&locale)
    }

    /// Creates an access wrapper for the requested locale.
    ///
    /// # Errors
    /// Returns [`Error::MissingFallbackLocale`] when the default locale entry is absent.
    pub fn get(&self, locale: K) -> Result<I18NAccess<'_, Self>> {
        let fallback = self.0.loaded.fallback_ref()?;
        let to = self.0.loaded.loaded_ref(locale).unwrap_or(fallback);

        Ok(I18NAccess { fallback, to })
    }
}
