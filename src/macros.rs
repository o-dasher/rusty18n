/// Expands the DSL into the generated value and override struct modules.
#[doc(hidden)]
#[macro_export]
macro_rules! __i18n_define_types {
    ($type_name:ident { $($body:tt)* }) => {
        ::paste::paste! {
            pub mod [<$type_name:snake>] {
                $crate::__i18n_define_struct_tree!(value $type_name { $($body)* });
                $crate::__i18n_define_struct_tree!(option [<$type_name Overrides>] { $($body)* });
            }
        }
    };
}

/// Emits one generated struct with the derives needed when `bevy_reflect` is enabled.
#[doc(hidden)]
#[macro_export]
#[cfg(feature = "bevy_reflect")]
macro_rules! __i18n_finish_struct {
    ($type_name:ident { $($fields:tt)* }) => {
        #[derive(Debug, Default, $crate::Reflect)]
        #[reflect(from_reflect = false)]
        pub struct $type_name {
            $($fields)*
        }
    };
}

/// Emits one generated struct with the standard derives in non-reflect builds.
#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "bevy_reflect"))]
macro_rules! __i18n_finish_struct {
    ($type_name:ident { $($fields:tt)* }) => {
        #[derive(Debug, Default)]
        pub struct $type_name {
            $($fields)*
        }
    };
}

/// Maps each dynamic formatter parameter name to its stored argument type.
#[doc(hidden)]
#[macro_export]
macro_rules! __i18n_string_type {
    ($name:ident) => {
        ::std::string::String
    };
}

/// Wraps a generated leaf field type for either concrete values or sparse overrides.
#[doc(hidden)]
#[macro_export]
macro_rules! __i18n_wrap_type {
    (value $ty:ty) => {
        $ty
    };

    (option $ty:ty) => {
        Option<$ty>
    };
}

/// Wraps a generated leaf value for either concrete values or sparse overrides.
#[doc(hidden)]
#[macro_export]
macro_rules! __i18n_wrap_expr {
    (value $expr:expr) => {
        $expr
    };

    (option $expr:expr) => {
        Some($expr)
    };
}

/// Recursively parses the schema DSL into concrete nested struct definitions.
///
/// `value` mode emits concrete leaf types, while `option` mode wraps leaves in
/// `Option` for sparse locale overrides.
#[doc(hidden)]
#[macro_export]
macro_rules! __i18n_define_struct_tree {
    ($mode:ident $name:ident { $($body:tt)* }) => {
        $crate::__i18n_define_struct_tree!(@munch $mode $name [] $($body)*);
    };

    (@munch $mode:ident $name:ident [$($f:tt)*]) => {
        $crate::__i18n_finish_struct!($name { $($f)* });
    };

    (@munch $mode:ident $name:ident [$($f:tt)*] $field:ident : { $($inner:tt)* } $(, $($rest:tt)*)?) => {
        ::paste::paste! {
            $crate::__i18n_define_struct_tree!($mode [<$name $field:camel>] { $($inner)* });
            $crate::__i18n_define_struct_tree!(@munch $mode $name [$($f)* pub $field: [<$name $field:camel>],] $($($rest)*)?);
        }
    };

    (@munch $mode:ident $name:ident [$($f:tt)*] $field:ident : | $($arg:ident),* $(,)? | $lit:literal $(, $($rest:tt)*)?) => {
        $crate::__i18n_define_struct_tree!(
            @munch $mode $name 
            [$($f)* pub $field: $crate::__i18n_wrap_type!($mode $crate::I18NDynamicFormatter<($( $crate::__i18n_string_type!($arg), )*)>),]
            $($($rest)*)?
        );
    };

    (@munch $mode:ident $name:ident [$($f:tt)*] $field:ident : $lit:literal $(, $($rest:tt)*)?) => {
        $crate::__i18n_define_struct_tree!(
            @munch $mode $name 
            [$($f)* pub $field: $crate::__i18n_wrap_type!($mode $crate::R),]
            $($($rest)*)?
        );
    };
}

/// Recursively applies the locale DSL onto a concrete value or sparse override.
#[doc(hidden)]
#[macro_export]
macro_rules! __i18n_build {
    ($mode:ident $base:expr; $($body:tt)*) => {
        {
            let mut __v = $base;
            $crate::__i18n_build!(@munch $mode __v $($body)*);
            __v
        }
    };

    (@munch $mode:ident $v:ident) => {};

    (@munch $mode:ident $v:ident $f:ident : { $($inner:tt)* } $(, $($rest:tt)*)?) => {
        $v.$f = $crate::__i18n_build!($mode $v.$f; $($inner)*);
        $crate::__i18n_build!(@munch $mode $v $($($rest)*)?);
    };

    (@munch $mode:ident $v:ident $f:ident : | $($arg:ident),* $(,)? | $lit:literal $(, $($rest:tt)*)?) => {
        $v.$f = $crate::__i18n_wrap_expr!(
            $mode
            $crate::I18NDynamicFormatter::<($( $crate::__i18n_string_type!($arg), )*)>::new(
                |($($arg,)*)| ::std::format!($lit),
            )
        );
        $crate::__i18n_build!(@munch $mode $v $($($rest)*)?);
    };

    (@munch $mode:ident $v:ident $f:ident : $lit:literal $(, $($rest:tt)*)?) => {
        $v.$f = $crate::__i18n_wrap_expr!($mode $crate::I18NDynamicResourceValue::from($lit));
        $crate::__i18n_build!(@munch $mode $v $($($rest)*)?);
    };
}

/// Builds a value by applying DSL updates over a default base value.
///
/// Supported override forms:
/// - `field: { ... }` for nested objects
/// - `field: "text"` for static resources
/// - `field: |args...| "format string"` for generated formatter resources
///
/// Leaves become concrete resources and nested blocks recurse into the already-typed
/// field value so `deep-struct-update` can infer the right nested type.
#[doc(hidden)]
#[macro_export]
macro_rules! __i18n_build_value {
    ($base:expr; $($body:tt)*) => {
        $crate::__i18n_build!(value $base; $($body)*)
    };
}

/// Builds a sparse override value from the DSL.
///
/// Leaves become `Some(resource)` while omitted fields stay `None`.
#[doc(hidden)]
#[macro_export]
macro_rules! __i18n_build_override {
    ($base:expr; $($body:tt)*) => {
        $crate::__i18n_build!(option $base; $($body)*)
    };
}

/// Defines the canonical fallback i18n schema and values from a single DSL source.
///
/// DSL forms:
/// - `field: { ... }`
/// - `field: "text"`
/// - `field: |a, b, c| "This is {a}, {b}, {c}"`
///
/// The macro requires a locale key:
/// - `define_i18n_fallback! { I18NUsage => en ... }`
///
/// Generated items:
/// - `pub mod <root_type_snake> { pub struct <RootType> { ... } }`
/// - `pub fn <locale_key_lower>() -> <root_type_snake>::<RootType>`
#[macro_export]
macro_rules! define_i18n_fallback {
    (
        $root_type:ident =>
        $locale_key:ident
        $($body:tt)*
    ) => {
        $crate::__i18n_define_types!($root_type { $($body)* });

        ::paste::paste! {
            impl $crate::I18NFallback for [<$root_type:snake>]::$root_type {
                fn fallback() -> Self {
                    $crate::__i18n_build_value!(
                        <[<$root_type:snake>]::$root_type as ::core::default::Default>::default();
                        $($body)*
                    )
                }
            }

            pub fn [<$locale_key:lower>]() -> [<$root_type:snake>]::$root_type {
                <[<$root_type:snake>]::$root_type as $crate::I18NFallback>::fallback()
            }
        }
    };
}

/// Defines a sparse locale constructor by applying DSL overrides over defaults.
///
/// Fields not explicitly overridden stay absent in the override value and are
/// resolved against the fallback locale at access time.
///
/// Override forms:
/// - `field: { ... }`
/// - `field: "text"`
/// - `field: |a, b, c| "This is {a}, {b}, {c}"`
///
/// The macro requires a locale key:
/// - `define_i18n! { I18NUsage => pt ... }`
///
/// It generates:
/// - `pub fn <locale_key_lower>() -> I18NUsage::Override`
#[macro_export]
macro_rules! define_i18n {
    (
        $base_i18n:path =>
        $locale_key:ident
        $($body:tt)*
    ) => {
        ::paste::paste! {
            pub fn [<$locale_key:lower>]() -> $base_i18n::Override {
                $crate::__i18n_build_override!(
                    <$base_i18n::Override as ::core::default::Default>::default();
                    $($body)*
                )
            }
        }
    };
}

/// Generates the locale key enum and the i18n wrapper constructors from locale modules.
///
/// The first listed locale is treated as the default fallback locale.
///
/// Locale modules are listed using `|` separators:
/// - `en|pt|ru|anylang`
///
/// Each locale module is expected to expose a constructor function with the same
/// name as the module, as generated by `define_i18n_fallback!` / `define_i18n!`.
///
/// Example:
/// `define_i18n_locales! { I18NUsage => en|pt }`
///
/// It generates a namespace module:
/// - `pub mod I18NUsage { ... }`
/// - `pub type I18NUsage::Value = <default_locale_type>`
/// - `pub type I18NUsage::Override = <default_locale_override_type>`
/// - `pub enum I18NUsage::Key { en, pt, ... }`
/// - `pub fn I18NUsage::locales() -> I18NStore<I18NUsage::Key, I18NUsage::Value, I18NUsage::Override>`
/// - `pub fn I18NUsage::locales_dynamic() -> I18NDynamicWrapper<I18NUsage::Key, I18NUsage::Value, I18NUsage::Override>`
#[macro_export]
macro_rules! define_i18n_locales {
    (
        $i18n_usage:ident =>
        $default_locale_mod:ident
        $(| $locale_mod:ident )* $(|)?
    ) => {
        ::paste::paste! {
            #[allow(non_snake_case)]
            pub mod $i18n_usage {
                pub type Value = super::$default_locale_mod::[<$i18n_usage:snake>]::$i18n_usage;
                pub type Override = super::$default_locale_mod::[<$i18n_usage:snake>]::[<$i18n_usage Overrides>];

                #[allow(non_camel_case_types)]
                #[derive(Debug, Default, Clone, Copy, Hash, Eq, PartialEq)]
                pub enum Key {
                    #[default]
                    $default_locale_mod,
                    $(
                        $locale_mod,
                    )*
                }

                pub fn locales() -> $crate::I18NStore<Key, Value, Override> {
                    $crate::I18NStore::new(
                        super::$default_locale_mod::[<$default_locale_mod:snake>](),
                        vec![
                        $(
                            (Key::$locale_mod, super::$locale_mod::[<$locale_mod:snake>]()),
                        )*
                        ],
                    )
                }

                pub fn locales_dynamic() -> $crate::I18NDynamicWrapper<Key, Value, Override> {
                    $crate::I18NDynamicWrapper::new(
                        super::$default_locale_mod::[<$default_locale_mod:snake>](),
                        vec![
                        $(
                            (Key::$locale_mod, super::$locale_mod::[<$locale_mod:snake>]),
                        )*
                        ],
                    )
                }
            }
        }
    };
}

/// Defines a local accessor macro bound to a resolved locale view.
///
/// This is a convenience wrapper around `resolve` for a shared prefix path.
#[macro_export]
macro_rules! t_prefix {
    ($dollar:tt$name:ident, $prefix_var:ident $(. $prefix_access:tt)*) => {
        macro_rules! $name {
            ($dollar($access:tt).*) => (
                $prefix_var$(. $prefix_access)*.resolve(
                    |v| &v $dollar(. $access)*,
                    |o| o $dollar(. $access)*.as_ref(),
                )
            )
        }
    };

    ($dollar:tt$prefix_var:ident $(.$prefix_access:tt)*) => {
        $crate::t_prefix!($dollar t, $prefix_var $(. $prefix_access)*)
    };
}

/// Reads a translation value from a resolved locale view.
///
/// This resolves a value against the fallback locale and any active override.
#[macro_export]
macro_rules! t {
    ($var:ident.$($access:tt).*) => {
        $var.resolve(
            |v| &v.$($access).*,
            |o| o.$($access).*.as_ref(),
        )
    };
}
