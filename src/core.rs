use std::sync::Arc;
use std::{collections::HashMap, hash::Hash, str::FromStr};

use bevy_reflect::{Reflect};

/// This type is used to define a simple i18n resource that does not have any dynamic variables
/// that shall be captured by thy.
pub type I18NResource = String;

fn default_dynamic_resource<A>() -> fn(A) -> String {
    |_args: A| "".to_string()
}

#[derive(Reflect, Debug)]
pub struct I18NDynamicResource<A> {
    #[reflect(ignore)]
    #[reflect(default = "default_dynamic_resource")]
    pub caller: fn(A) -> String,
}

/// A trait to define some kind of structure that should be accessed in some way with provided arguments.
trait I18NAccessible<'a, A, R> {
    fn access(&'a self, args: A) -> R;
}

impl<A> I18NDynamicResource<A> {
    pub fn new(caller: fn(A) -> String) -> Self {
        Self { caller }
    }
}

impl<A> I18NAccessible<'_, A, String> for I18NDynamicResource<A> {
    fn access(&self, args: A) -> String {
        (self.caller)(args)
    }
}

pub trait I18NFallback {
    /// This should return the fallback locale (a.k.a: default).
    fn fallback() -> Self;
}

/// This trait groups Key, Value types for a given I18N implementation.
pub trait I18NTrait {
    type Key: Eq + Hash + Copy + Default;
    type Value: I18NFallback;
}

/// The I18NStore wraps a HashMap that maps key value pairs of Locale keys and localized
/// implementations.
#[derive(Debug)]
pub struct I18NStore<L: I18NTrait>(pub HashMap<L::Key, Arc<L::Value>>);

impl<L: I18NTrait, F: Fn() -> L::Value> From<Vec<(L::Key, F)>> for I18NStore<L> {
    fn from(value: Vec<(L::Key, F)>) -> Self {
        Self(value.into_iter().map(|(k, v)| (k, v().into())).collect())
    }
}

/// The I18NWrapper wraps over a I18NStore, acting as the middleman to acquire i18n resources.
#[derive(Debug)]
pub struct I18NWrapper<K: Eq + Hash + Default + Copy, V: I18NFallback> {
    pub store: I18NStore<Self>,
}

impl<K: Eq + Hash + Copy + Default, V: I18NFallback> I18NTrait for I18NWrapper<K, V> {
    type Key = K;
    type Value = V;
}

/// I18NAccess is a wrapper that guards 2 versions of the same resource before accessing it.
/// The fallback version in case the locale is not able to be found otherwise the provided locale for this I18NAccess.
pub struct I18NAccess<L: I18NTrait> {
    pub fallback: Arc<L::Value>,
    pub to: Arc<L::Value>,
}

impl<L: I18NTrait> Clone for I18NAccess<L> {
    fn clone(&self) -> Self {
        Self { fallback: Arc::clone(&self.fallback), to: Arc::clone(&self.to) }
    }
}

impl <'a, L: I18NTrait, R> I18NAccessible<'a, fn(&Arc<L::Value>) -> &Option<R>, &'a R> for I18NAccess<L> {
    /// Returns the required resource, fallbacks to the fallback implementation in case the resource could not be
    /// found for a given locale.
    fn access(&'a self, accessing: fn(&Arc<L::Value>) -> &Option<R>) -> &'a R {
        accessing(&self.to)
            .as_ref()
            .unwrap_or_else(|| accessing(&self.fallback).as_ref().unwrap())
    }
}


// A NewType wrapper for a locale key with extended capabilities.
pub struct LocaleKey<K: Eq + Hash + Copy + Default + FromStr>(pub K);

impl<K: Eq + Hash + Copy + Default + FromStr> From<&str> for LocaleKey<K> {
    fn from(value: &str) -> Self {
        LocaleKey(K::from_str(value).unwrap_or_default())
    }
}

impl<K: Eq + Hash + Copy + Default + FromStr> From<Option<&str>> for LocaleKey<K> {
    fn from(value: Option<&str>) -> Self {
        match value {
            Some(v) => v.into(),
            None => LocaleKey(K::default()),
        }
    }
}

impl<K: Eq + Hash + Copy + Default, V: I18NFallback> I18NWrapper<K, V>
    where
        Self: I18NTrait<Key=K, Value=V>,
{
    pub fn new(store: Vec<(K, fn() -> V)>) -> Self {
        let mut store = I18NStore::from(store);

        store.0.insert(K::default(), V::fallback().into());

        Self { store }
    }

    // Returns the i18n implementation of the provided key if it is found.
    fn ref_opt(&self, locale: &K) -> Option<Arc<V>> {
        self.store.0.get(locale).cloned()
    }

    // Returns the fallback i18n implementation.
    fn ref_default(&self) -> Arc<V> {
        self.ref_opt(&K::default()).unwrap()
    }

    // Returns either the provided key i18n implementation or the fallback one if not found.
    fn ref_any(&self, locale: &K) -> Arc<V> {
        self.ref_opt(locale).unwrap_or_else(|| self.ref_default())
    }

    /// Returns an I18NAccess to a given locale key.
    pub fn get(&self, locale: impl Into<K>) -> I18NAccess<Self> {
        I18NAccess {
            fallback: self.ref_default(),
            to: self.ref_any(&locale.into()),
        }
    }
}