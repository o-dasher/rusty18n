/// Defines the full struct type tree for the fallback DSL.
///
/// Leaf forms:
/// - `"text"` => fallback `R`, override `Option<R>`
/// - `"text with {placeholders}"` => fallback `R`, override `Option<R>`
///
/// The DSL supports:
/// - `field: "text"` leaves
/// - `field: { ... }` nested blocks
#[doc(hidden)]
#[macro_export]
macro_rules! __i18n_resource_type {
    ($lit:literal) => {
        $crate::R
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __i18n_build_resource {
    ($lit:literal) => {{
        static DISPLAY_TEXT: ::std::sync::OnceLock<::std::boxed::Box<str>> =
            ::std::sync::OnceLock::new();
        const _: usize = $crate::__template_arity($lit);

        $crate::I18NDynamicResourceValue::new_static(
            if $crate::__template_has_escapes($lit) {
                DISPLAY_TEXT
                    .get_or_init(|| $crate::__normalize_template($lit).into_boxed_str())
                    .as_ref()
            } else {
                $lit
            },
            $lit,
        )
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! i18n_define_types {
    ($type_name:ident { $($body:tt)* }) => {
        $crate::i18n_define_type_fields!(@parse [finish $type_name] [] [] $($body)*);
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "bevy_reflect")]
macro_rules! i18n_define_struct {
    ($type_name:ident { $($fields:tt)* }) => {
        $crate::__structstruck_strike! {
            #[structstruck::each[derive(Debug, Default, $crate::Reflect)]]
            #[structstruck::each[structstruck::long_names]]
            pub struct $type_name {
                $($fields)*
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "bevy_reflect"))]
macro_rules! i18n_define_struct {
    ($type_name:ident { $($fields:tt)* }) => {
        $crate::__structstruck_strike! {
            #[structstruck::each[derive(Debug, Default)]]
            #[structstruck::each[structstruck::long_names]]
            pub struct $type_name {
                $($fields)*
            }
        }
    };
}

/// Walks the nested i18n DSL and rewrites it into `structstruck::strike!` fields.
///
/// This uses a small accumulator so `structstruck` receives a fully-built
/// field tree instead of nested macro calls in its input.
///
/// Leaf semantics:
/// - fallback leaf: `R`
/// - override leaf: `Option<R>`
#[doc(hidden)]
#[macro_export]
macro_rules! i18n_define_type_fields {
    (@parse [$callback:ident $($ctx:tt)*] [$($value_out:tt)*] [$($override_out:tt)*]) => {
        $crate::i18n_define_type_fields!(@$callback $($ctx)* [$($value_out)*] [$($override_out)*]);
    };

    (@parse [$callback:ident $($ctx:tt)*] [$($value_out:tt)*] [$($override_out:tt)*] $field:ident : { $($nested:tt)* } $(, $($rest:tt)*)?) => {
        $crate::i18n_define_type_fields!(
            @parse
            [finish_nested [$callback $($ctx)*] [$($value_out)*] [$($override_out)*] $field [$($($rest)*)?]]
            []
            []
            $($nested)*
        );
    };

    (@parse [$callback:ident $($ctx:tt)*] [$($value_out:tt)*] [$($override_out:tt)*] $field:ident : $lit:literal $(, $($rest:tt)*)?) => {
        $crate::i18n_define_type_fields!(
            @parse
            [$callback $($ctx)*]
            [
                $($value_out)*
                pub $field: $crate::__i18n_resource_type!($lit),
            ]
            [
                $($override_out)*
                pub $field: Option<$crate::__i18n_resource_type!($lit)>,
            ]
            $($($rest)*)?
        );
    };

    (@finish_nested [$callback:ident $($ctx:tt)*] [$($outer_value_out:tt)*] [$($outer_override_out:tt)*] $field:ident [$($rest:tt)*] [$($nested_value_out:tt)*] [$($nested_override_out:tt)*]) => {
        $crate::i18n_define_type_fields!(
            @parse
            [$callback $($ctx)*]
            [
                $($outer_value_out)*
                pub $field: struct {
                    $($nested_value_out)*
                },
            ]
            [
                $($outer_override_out)*
                pub $field: struct {
                    $($nested_override_out)*
                },
            ]
            $($rest)*
        );
    };

    (@finish $type_name:ident [$($value_fields:tt)*] [$($override_fields:tt)*]) => {
        ::paste::paste! {
            pub mod [<$type_name:snake>] {
                $crate::i18n_define_struct!($type_name { $($value_fields)* });
                $crate::i18n_define_struct!([<$type_name Overrides>] { $($override_fields)* });
            }
        }
    };
}

/// Builds a value by applying DSL updates over a default base value.
///
/// Supported override forms:
/// - `field: { ... }` for nested objects
/// - `field: "text"` for static resources
/// - `field: "text with {placeholders}"` for inferred dynamic resources
///
/// Leaves become concrete resources and nested blocks recurse into the already-typed
/// field value so `deep-struct-update` can infer the right nested type.
#[doc(hidden)]
#[macro_export]
macro_rules! i18n_build_value {
    ($base:expr; $($body:tt)*) => {
        $crate::i18n_build_value!(@collect [$base] [] $($body)*)
    };

    (@collect [$base:expr] [$($fields:tt)*]) => {
        $crate::__deep_update! {
            $($fields)*
            ..$base
        }
    };

    (@collect [$base:expr] [$($fields:tt)*] $field:ident : { $($nested:tt)* } $(, $($rest:tt)*)?) => {
        $crate::i18n_build_value!(
            @collect
            [$base]
            [
                $($fields)*
                $field: $crate::i18n_build_value!(($base).$field; $($nested)*),
            ]
            $($($rest)*)?
        )
    };

    (@collect [$base:expr] [$($fields:tt)*] $field:ident : $lit:literal $(, $($rest:tt)*)?) => {
        $crate::i18n_build_value!(
            @collect [$base]
            [
                $($fields)*
                $field: $crate::__i18n_build_resource!($lit),
            ]
            $($($rest)*)?
        )
    };
}

/// Builds a sparse override value from the DSL.
///
/// Leaves become `Some(resource)` while omitted fields stay `None`.
#[doc(hidden)]
#[macro_export]
macro_rules! i18n_build_override {
    ($base:expr; $($body:tt)*) => {
        $crate::i18n_build_override!(@collect [$base] [] $($body)*)
    };

    (@collect [$base:expr] [$($fields:tt)*]) => {
        $crate::__deep_update! {
            $($fields)*
            ..$base
        }
    };

    (@collect [$base:expr] [$($fields:tt)*] $field:ident : { $($nested:tt)* } $(, $($rest:tt)*)?) => {
        $crate::i18n_build_override!(
            @collect
            [$base]
            [
                $($fields)*
                $field: $crate::i18n_build_override!(($base).$field; $($nested)*),
            ]
            $($($rest)*)?
        )
    };

    (@collect [$base:expr] [$($fields:tt)*] $field:ident : $lit:literal $(, $($rest:tt)*)?) => {
        $crate::i18n_build_override!(
            @collect [$base]
            [
                $($fields)*
                $field: Some($crate::__i18n_build_resource!($lit)),
            ]
            $($($rest)*)?
        )
    };
}

/// Defines the canonical fallback i18n schema and values from a single DSL source.
///
/// DSL forms:
/// - `field: { ... }`
/// - `field: "text"`
/// - `field: "This is {a}, {b}, {c}"`
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
        ::paste::paste! {
            $crate::i18n_define_types!($root_type { $($body)* });

            impl $crate::I18NFallback for [<$root_type:snake>]::$root_type {
                fn fallback() -> Self {
                    $crate::i18n_build_value!(
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
/// - `field: "This is {a}, {b}, {c}"`
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
                $crate::i18n_build_override!(
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

    ($dollar:tt $name:ident, $prefix_var:ident $(. $prefix_access:tt)*) => {
        $crate::t_prefix!($dollar$name, $prefix_var $(.$prefix_access)*)
    };

    ($dollar:tt$prefix_var:ident $(.$prefix_access:tt)*) => {
        $crate::t_prefix!($dollar t, $prefix_var $(. $prefix_access)*)
    };
}

/// Reads a translation value from a resolved locale view.
#[macro_export]
macro_rules! t {
    ($var:ident.$($access:tt).*) => {
        $var.resolve(
            |v| &v.$($access).*,
            |o| o.$($access).*.as_ref(),
        )
    };
}
