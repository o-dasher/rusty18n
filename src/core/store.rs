use super::{I18NFallback, I18NResolved, I18NTrait};
use std::{collections::HashMap, hash::Hash};

/// The `I18NStore` always keeps the fallback locale resident.
///
/// Non-default locales are stored in `locales`; the fallback value is kept in its
/// own slot so the missing-fallback state is not representable.
#[derive(Debug)]
pub struct I18NStore<K: Eq + Hash + Copy, V, O> {
    fallback: V,
    pub(crate) locales: HashMap<K, O>,
}

impl<K: Eq + Hash + Default + Copy, V: I18NFallback, O> I18NTrait for I18NStore<K, V, O> {
    type K = K;
    type V = V;
}

impl<K: Eq + Hash + Default + Copy, V, O> I18NStore<K, V, O> {
    /// Constructs a locale store from a fallback value and sparse locale overrides.
    pub fn new<T>(fallback: V, locales: T) -> Self
    where
        T: IntoIterator<Item = (K, O)>,
    {
        let default_locale = K::default();

        Self {
            fallback,
            locales: locales
                .into_iter()
                .filter(|(locale, _)| *locale != default_locale)
                .collect(),
        }
    }

    /// Returns the resolved locale view for the requested key.
    #[must_use]
    pub fn get(&self, locale: K) -> I18NResolved<'_, V, O> {
        I18NResolved {
            fallback: &self.fallback,
            override_locale: self.locales.get(&locale),
        }
    }

    /// Returns whether the locale is currently present in the store.
    pub fn contains_key(&self, locale: &K) -> bool {
        self.locales.contains_key(locale) || *locale == K::default()
    }

    /// Unloads a single locale value from the store.
    ///
    /// The default fallback locale is never stored in the loaded map, so removing
    /// it naturally returns `None`.
    pub fn unload(&mut self, locale: K) -> Option<O> {
        self.locales.remove(&locale)
    }

    /// Unloads every non-fallback locale from the store.
    pub fn unload_all(&mut self) {
        self.locales.clear();
    }
}
