use std::{collections::HashMap, hash::Hash};

use bevy_reflect::Reflect;

/// A trait to define some kind of structure that should be accessed in some way with provided arguments.
pub trait I18NAccessible<'a, A, R> {
    fn access(&'a self, args: A) -> R;
}

fn default_dynamic_resource<A>() -> fn(A) -> String {
    |_args: A| String::new()
}

#[derive(Reflect, Debug)]
pub struct I18NDynamicResource<A> {
    #[reflect(ignore)]
    #[reflect(default = "default_dynamic_resource")]
    pub caller: fn(A) -> String,
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

pub trait I18NKey = Eq + Hash + Default + Copy;

/// This trait groups Key, Value types for a given I18N implementation.
pub trait I18NTrait {
    type Key: I18NKey;
    type Value: I18NFallback;
}

/// The I18NStore wraps a HashMap that maps key value pairs of Locale keys and localized
/// implementations.
#[derive(Debug)]
pub struct I18NStore<L: I18NTrait>(pub HashMap<L::Key, L::Value>);

impl<L: I18NTrait, F: Fn() -> L::Value> From<Vec<(L::Key, F)>> for I18NStore<L> {
    fn from(value: Vec<(L::Key, F)>) -> Self {
        Self(value.into_iter().map(|(k, v)| (k, v())).collect())
    }
}

/// The I18NWrapper wraps over a I18NStore, acting as the middleman to acquire i18n resources.
#[derive(Debug)]
pub struct I18NWrapper<K: I18NKey, V: I18NFallback> {
    pub store: I18NStore<Self>,
}

impl<K: I18NKey, V: I18NFallback> I18NTrait for I18NWrapper<K, V> {
    type Key = K;
    type Value = V;
}

/// I18NAccess is a wrapper that guards 2 versions of the same resource before accessing it.
/// The fallback version in case the locale is not able to be found otherwise the provided locale for this I18NAccess.
pub struct I18NAccess<'a, L: I18NTrait> {
    pub fallback: &'a L::Value,
    pub to: &'a L::Value,
}

impl<L: I18NTrait> Clone for I18NAccess<'_, L> {
    fn clone(&self) -> Self {
        Self {
            fallback: self.fallback,
            to: self.to,
        }
    }
}

impl<'a, L: I18NTrait, Resource>
    I18NAccessible<'a, fn(&L::Value) -> Option<&Resource>, &'a Resource> for I18NAccess<'_, L>
{
    /// Returns the required resource, fallbacks to the fallback implementation in case the resource could not be
    /// found for a given locale.
    fn access(&'a self, accessing: fn(&L::Value) -> Option<&Resource>) -> &'a Resource {
        accessing(self.to).unwrap_or_else(|| accessing(self.fallback).unwrap())
    }
}

impl<K: I18NKey, V: I18NFallback> I18NWrapper<K, V>
where
    Self: I18NTrait<Key = K, Value = V>,
{
    pub fn new(store: Vec<(K, fn() -> V)>) -> Self {
        let mut store = I18NStore::from(store);

        store.0.insert(K::default(), V::fallback());

        Self { store }
    }

    // Returns the fallback i18n implementation.
    fn ref_default(&self) -> &V {
        self.ref_opt(K::default()).unwrap()
    }

    // Returns the i18n implementation of the provided key if it is found.
    fn ref_opt(&self, locale: K) -> Option<&V> {
        self.store.0.get(&locale)
    }

    // Returns either the provided key i18n implementation or the fallback one if not found.
    fn ref_any(&self, locale: K) -> &V {
        self.ref_opt(locale).unwrap_or_else(|| self.ref_default())
    }

    /// Returns an I18NAccess to a given locale key.
    pub fn get(&self, locale: K) -> I18NAccess<Self> {
        I18NAccess {
            fallback: self.ref_default(),
            to: self.ref_any(locale),
        }
    }
}
