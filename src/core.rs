#[cfg(feature = "bevy_reflect")]
use bevy_reflect::Reflect;
use derive_more::derive::{AsRef, Deref, DerefMut, Display as DeriveDisplay, From};
use impl_trait_for_tuples::impl_for_tuples;
use std::{collections::HashMap, fmt::Display, hash::Hash};

/// Errors produced by `rusty18n`.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
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
        let args = args.into_dynamic_resource_args();

        self.render.unwrap_or(|_, _| String::new())(&args, None)
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
#[derive(Debug, Deref, DerefMut, From)]
pub struct I18NStore<K: Eq + Hash + Copy, V>(pub HashMap<K, V>);

impl<K: Eq + Hash + Copy, V> std::iter::FromIterator<(K, V)> for I18NStore<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        iter.into_iter().collect::<HashMap<_, _>>().into()
    }
}

impl<K: Eq + Hash + Default + Copy, V: I18NFallback> I18NStore<K, V> {
    fn ensure_fallback(&mut self) {
        self.entry(K::default()).or_insert_with(V::fallback);
    }

    fn fallback_ref(&self) -> Result<&V> {
        self.get(&K::default()).ok_or(Error::MissingFallbackLocale)
    }

    fn unload_locale(&mut self, locale: K) -> Option<V> {
        if locale == K::default() {
            None
        } else {
            self.remove(&locale)
        }
    }

    fn unload_all_locales(&mut self) {
        let default_locale = K::default();
        self.retain(|locale, _| *locale == default_locale);
        self.ensure_fallback();
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

/// Eagerly loaded i18n resources with fallback support.
#[derive(Debug)]
pub struct I18NWrapper<K: Eq + Hash + Default + Copy, V: I18NFallback> {
    store: I18NStore<K, V>,
}

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
        let mut store: I18NStore<K, V> = store.into_iter().collect();
        store.ensure_fallback();
        Self { store }
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
        &self.store
    }

    /// Returns whether the locale is currently resident in memory.
    ///
    /// For eager wrappers, any registered locale is already loaded.
    #[must_use]
    pub fn is_loaded(&self, locale: K) -> bool {
        self.store.contains_key(&locale)
    }

    /// Creates an access wrapper for the requested locale.
    ///
    /// # Errors
    /// Returns [`Error::MissingFallbackLocale`] when the default locale entry is absent.
    pub fn get(&self, locale: K) -> Result<I18NAccess<'_, Self>> {
        let fallback = self.store.fallback_ref()?;
        let to = self.store.get(&locale).unwrap_or(fallback);

        Ok(I18NAccess { fallback, to })
    }
}

/// Dynamically loaded i18n resources.
#[derive(Debug)]
pub struct I18NDynamicWrapper<K: Eq + Hash + Default + Copy, V: I18NFallback> {
    loaded: I18NStore<K, V>,
    loaders: HashMap<K, I18NLocaleLoader<V>>,
}

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
        Self {
            loaded: HashMap::from([(K::default(), V::fallback())]).into(),
            loaders: loaders.into_iter().collect(),
        }
    }

    /// Returns the currently loaded locale store.
    #[must_use]
    pub const fn store(&self) -> &I18NStore<K, V> {
        &self.loaded
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
        self.loaders.insert(locale, loader)
    }

    /// Unregisters a locale loader and unloads that locale if it is currently loaded.
    ///
    /// # Arguments
    /// * `locale` - The locale key to unregister.
    ///
    /// # Returns
    /// The previously registered loader for the locale, if any.
    pub fn unregister_locale(&mut self, locale: K) -> Option<I18NLocaleLoader<V>> {
        self.loaded.unload_locale(locale);
        self.loaders.remove(&locale)
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
        let Some(load) = self.loaders.get(&locale).copied() else {
            return false;
        };

        self.loaded.insert(locale, load());
        true
    }

    /// Loads all registered locales into memory.
    pub fn load_all(&mut self) {
        let loaders = self
            .loaders
            .iter()
            .map(|(&locale, &load)| (locale, load))
            .collect::<Vec<_>>();

        for (locale, load) in loaders {
            self.loaded.insert(locale, load());
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
        self.loaded.unload_locale(locale)
    }

    /// Unloads all currently loaded locales.
    ///
    pub fn unload_all(&mut self) {
        self.loaded.unload_all_locales();
    }

    /// Returns whether a locale is currently loaded.
    #[must_use]
    pub fn is_loaded(&self, locale: K) -> bool {
        self.loaded.contains_key(&locale)
    }

    /// Returns whether a locale has a registered loader.
    #[must_use]
    pub fn is_registered(&self, locale: K) -> bool {
        self.loaders.contains_key(&locale)
    }

    /// Creates an access wrapper for the requested locale.
    ///
    /// # Errors
    /// Returns [`Error::MissingFallbackLocale`] when the default locale entry is absent.
    pub fn get(&self, locale: K) -> Result<I18NAccess<'_, Self>> {
        let fallback = self.loaded.fallback_ref()?;
        let to = self.loaded.get(&locale).unwrap_or(fallback);

        Ok(I18NAccess { fallback, to })
    }
}
