#[cfg(feature = "bevy_reflect")]
use bevy_reflect::Reflect;
use impl_trait_for_tuples::impl_for_tuples;
use std::fmt::Display;
use std::{collections::HashMap, hash::Hash};

#[cfg(feature = "bevy_reflect")]
fn default_dynamic_resource<A>() -> fn(A) -> String {
    |_args: A| String::new()
}

/// Converts user-provided dynamic arguments into an internal tuple of `String`s.
///
/// This enables ergonomic calls such as:
/// `dynamic.with((1, "name", 3.5))`
/// for resources that internally use `(String, String, String)`.
pub trait IntoDynamicResourceArgs {
    /// Internal dynamic argument tuple used by `I18NDynamicResource`.
    type Output;

    /// Converts `self` into the internal argument tuple expected by the dynamic resource.
    fn into_dynamic_resource_args(self) -> Self::Output;
}

#[impl_for_tuples(1, 16)]
#[tuple_types_no_default_trait_bound]
impl IntoDynamicResourceArgs for Tuple {
    for_tuples!( where #( Tuple: Display )* );
    for_tuples!( type Output = ( #( String ),* ); );
    fn into_dynamic_resource_args(self) -> Self::Output {
        for_tuples!( ( #( self.Tuple.to_string() ),* ) )
    }
}

/// A struct representing an internationalization (i18n) dynamic resource.
#[derive(Debug)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct I18NDynamicResource<A> {
    #[cfg_attr(
        feature = "bevy_reflect",
        reflect(ignore, default = "default_dynamic_resource")
    )]
    /// A function that takes arguments of type `A` and returns a string representing
    /// the localized resource.
    caller: fn(A) -> String,
}

impl<A> I18NDynamicResource<A> {
    /// Creates a new dynamic resource with the provided renderer function.
    pub fn new(caller: fn(A) -> String) -> Self {
        Self { caller }
    }

    /// Invokes the dynamic resource with user-provided arguments.
    ///
    /// # Arguments
    /// * `args` - Arguments that can be converted into the internal tuple type `A`.
    ///   Each tuple item must implement `Display`.
    ///
    /// # Returns
    /// A string representing the localized resource.
    pub fn with<T>(&self, args: T) -> String
    where
        T: IntoDynamicResourceArgs<Output = A>,
    {
        (self.caller)(args.into_dynamic_resource_args())
    }
}

/// A trait for defining fallback behavior in internationalization (i18n).
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

/// The I18NStore wraps a HashMap that maps key value pairs of Locale keys and localized
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
    pub fn new(store: Vec<(K, fn() -> V)>) -> Self {
        let mut store = I18NStore::from(store);

        store.0.insert(K::default(), V::fallback());

        Self {
            store,
            fallback: V::fallback(),
        }
    }

    /// Gets a reference to the default i18n resource.
    fn ref_default(&self) -> &V {
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
