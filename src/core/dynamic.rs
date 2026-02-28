use super::{I18NFallback, I18NStore, I18NTrait};
use std::{collections::HashMap, hash::Hash};

/// A locale constructor used by the dynamic wrapper.
pub type I18NLocaleLoader<V> = fn() -> V;

/// Dynamically loaded i18n resources.
#[derive(Debug)]
pub struct I18NDynamicWrapper<K: Eq + Hash + Default + Copy, V: I18NFallback> {
    pub loaded: I18NStore<K, V>,
    pub loaders: HashMap<K, I18NLocaleLoader<V>>,
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
        let default_locale = K::default();

        Self {
            loaded: I18NStore::new(std::iter::empty::<(K, V)>()),
            loaders: loaders
                .into_iter()
                .filter(|(locale, _)| *locale != default_locale)
                .collect(),
        }
    }

    /// Unregisters a locale loader and unloads that locale if it is currently loaded.
    ///
    /// # Arguments
    /// * `locale` - The locale key to unregister.
    ///
    /// # Returns
    /// The previously registered loader for the locale, if any.
    pub fn unregister_locale(&mut self, locale: K) -> Option<I18NLocaleLoader<V>> {
        self.loaders.remove(&locale).inspect(|_| {
            self.loaded.unload(locale);
        })
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
        self.loaders
            .get(&locale)
            .copied()
            .map(|load| self.loaded.locales.insert(locale, load()))
            .is_some()
    }

    /// Loads all registered locales into memory.
    pub fn load_all(&mut self) {
        self.loaded
            .locales
            .extend(self.loaders.iter().map(|(&locale, &load)| (locale, load())));
    }

    /// Returns the resolved locale value for the requested key.
    #[must_use]
    pub fn get(&self, locale: K) -> &V {
        self.loaded.get(locale)
    }
}
