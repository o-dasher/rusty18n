use super::{I18NAccess, I18NFallback, I18NTrait, Result};
use derive_more::derive::{Deref, DerefMut, From};
use std::{collections::HashMap, hash::Hash};

/// The `I18NStore` wraps a `HashMap` that maps key value pairs of `Locale` keys and localized
/// implementations.
#[derive(Debug, Deref, DerefMut, From)]
pub struct I18NStore<K: Eq + Hash + Copy, V>(pub HashMap<K, V>);

impl<K: Eq + Hash + Copy, V> std::iter::FromIterator<(K, V)> for I18NStore<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        iter.into_iter().collect::<HashMap<_, _>>().into()
    }
}

impl<K: Eq + Hash + Default + Copy, V: I18NFallback> I18NTrait for I18NStore<K, V> {
    type K = K;
    type V = V;
}

impl<K: Eq + Hash + Default + Copy, V: I18NFallback> I18NStore<K, V> {
    /// Constructs a locale store and ensures the fallback locale is present.
    #[must_use]
    pub fn new<T>(locales: T) -> Self
    where
        T: IntoIterator<Item = (K, V)>,
    {
        let mut store: Self = locales.into_iter().collect();
        store.entry(K::default()).or_insert_with(V::fallback);
        store
    }

    fn resolve(&self, locale: K) -> Result<(&V, &V)> {
        self.0
            .get(&K::default())
            .ok_or(crate::Error::MissingFallbackLocale)
            .map(|fallback| (fallback, self.0.get(&locale).unwrap_or(fallback)))
    }

    pub(crate) fn access<L>(&self, locale: K) -> Result<I18NAccess<'_, L>>
    where
        L: I18NTrait<K = K, V = V>,
    {
        self.resolve(locale)
            .map(|(fallback, to)| I18NAccess { fallback, to })
    }

    /// Creates an access wrapper for the requested locale.
    ///
    /// # Errors
    /// Returns [`crate::Error::MissingFallbackLocale`] when the default locale entry is absent.
    pub fn get(&self, locale: K) -> Result<I18NAccess<'_, Self>> {
        self.access(locale)
    }

    /// Unloads a single locale value from the store.
    ///
    /// The default fallback locale is kept resident.
    pub fn unload(&mut self, locale: K) -> Option<V> {
        if locale == K::default() {
            None
        } else {
            self.remove(&locale)
        }
    }

    /// Unloads every non-fallback locale from the store.
    pub fn unload_all(&mut self) {
        let default_locale = K::default();
        self.retain(|locale, _| *locale == default_locale);
    }
}
