use super::{I18NFallback, I18NTrait};
use std::{collections::HashMap, hash::Hash};

/// The `I18NStore` always keeps the fallback locale resident.
///
/// Non-default locales are stored in `locales`; the fallback value is kept in its
/// own slot so the missing-fallback state is not representable.
#[derive(Debug)]
pub struct I18NStore<K: Eq + Hash + Copy, V> {
    fallback: V,
    pub(crate) locales: HashMap<K, V>,
}

impl<K: Eq + Hash + Default + Copy, V: I18NFallback> std::iter::FromIterator<(K, V)>
    for I18NStore<K, V>
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let default_locale = K::default();
        let mut fallback = None;
        let mut locales = HashMap::new();

        for (locale, value) in iter {
            if locale == default_locale {
                fallback = Some(value);
            } else {
                locales.extend(std::iter::once((locale, value)));
            }
        }

        Self {
            fallback: fallback.unwrap_or_else(V::fallback),
            locales,
        }
    }
}

impl<K: Eq + Hash + Default + Copy, V: I18NFallback> I18NTrait for I18NStore<K, V> {
    type K = K;
    type V = V;
}

impl<K: Eq + Hash + Default + Copy, V: I18NFallback> I18NStore<K, V> {
    /// Constructs a locale store and ensures the fallback locale is present.
    pub fn new<T>(locales: T) -> Self
    where
        T: IntoIterator<Item = (K, V)>,
    {
        locales.into_iter().collect()
    }

    /// Returns the resolved locale value for the requested key.
    pub fn get(&self, locale: K) -> &V {
        self.locales.get(&locale).unwrap_or(&self.fallback)
    }

    /// Returns whether the locale is currently present in the store.
    pub fn contains_key(&self, locale: &K) -> bool {
        self.locales.contains_key(locale) || *locale == K::default()
    }

    /// Unloads a single locale value from the store.
    ///
    /// The default fallback locale is never stored in the loaded map, so removing
    /// it naturally returns `None`.
    pub fn unload(&mut self, locale: K) -> Option<V> {
        self.locales.remove(&locale)
    }

    /// Unloads every non-fallback locale from the store.
    pub fn unload_all(&mut self) {
        self.locales.clear();
    }
}
