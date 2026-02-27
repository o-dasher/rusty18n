/// Creates i18n resources in a compact form.
///
/// This macro always wraps values in `Some(...)` so it fits fields defined as optional resources.
///
/// Supported forms:
/// - `r!("Static text")` for static resources (`Option<String>`).
/// - `r!(|args| "Formatted {args}")` for dynamic resources (`Option<I18NDynamicResource<_>>`).
/// - `r!(|(a, b)| "Formatted {a} {b}")` for tuple arguments.
/// - Braced variants (`{ "..." }`) are accepted for style consistency.
#[macro_export]
macro_rules! r {
    (|($($args:pat),*)| $lit:literal) => {
        Some(I18NDynamicResource::new(|($($args),*)| format!($lit)))
    };

    (|$args:ident| $lit:literal) => {
        r!(|($args)| $lit);
    };

    (|($($args:pat),*)| { $lit:literal }) => {
       r!(|($($args),*)| $lit);
    };

    (|$args:ident| { $lit:literal }) => {
        r!(|($args)| $lit);
    };

    ($lit:literal) => {
        Some(format!($lit))
    };
}

/// Builds a nested i18n struct using a braced DSL.
///
/// `ergo_braced!` is the constructor backend used by `define_i18n_fallback!`.
/// It recursively maps nested fields into the generated nested types and allows
/// each field to be either:
/// - another nested block (`field { ... }`)
/// - a direct expression (`field: expr`)
#[macro_export]
macro_rules! ergo_braced {(
    $base:path, $T:ty {
        $(
            $field_name:ident
                // either
                $({ $($body:tt)* })?
                // or
                $(: $value:expr)?
        ),* $(,)?
    }
    $(,)?
) => (::paste::paste! {
    $T {
        $(
            $field_name:
                // either
                $(
                    ::rusty18n::ergo_braced!(
                        $base::$field_name,
                        $base::$field_name::[< $field_name:camel >] {
                        $($body)*
                    })
                )? /* or */ $(
                    $value
                )?
            ,
        )*
    }
})}

/// Implements `I18NFallback` for a root i18n type.
///
/// This macro lets you declare the full fallback translation tree once, using
/// the same nested DSL as `ergo_braced!`. The generated implementation is used
/// as the base locale for all partial locale overrides.
#[macro_export]
macro_rules! define_i18n_fallback {
    ($base_path:path, $($body:tt)*) => {
        ::paste::paste! {
            impl ::rusty18n::I18NFallback for super::$base_path::[< $base_path:camel >]{
                fn fallback() -> Self {
                        ::rusty18n::ergo_braced!(
                            super::$base_path,
                            Self { $($body)* }
                        )
                }
            }
        }
    };
}

/// Creates a locale value by applying partial overrides to fallback.
///
/// The macro:
/// 1. Builds a full value from `<Base as I18NFallback>::fallback()`.
/// 2. Recursively applies only the fields provided in the macro body.
/// 3. Returns the patched value.
///
/// This is intended for locales that are incomplete while development is in progress.
#[macro_export]
macro_rules! define_i18n {
    ($base_i18n:path, $($body:tt)*) => {{
        let mut base_i18n = <$base_i18n as ::rusty18n::I18NFallback>::fallback();
        ::rusty18n::ergo_braced_update!(&mut base_i18n, { $($body)* });
        base_i18n
    }};
}

/// Recursively applies nested field overrides to an existing struct value.
///
/// This is an internal helper used by `define_i18n!`.
#[macro_export]
macro_rules! ergo_braced_update {(
    $target:expr, { $($body:tt)* }
) => {
    ::rusty18n::ergo_braced_update_inner!($target, $($body)*);
}}

/// Token-muncher backend for `ergo_braced_update!`.
///
/// Exported and hidden so nested recursive expansion works reliably across crates.
#[doc(hidden)]
#[macro_export]
macro_rules! ergo_braced_update_inner {
    ($target:expr) => {};
    ($target:expr,) => {};

    ($target:expr, $field_name:ident: { $($body:tt)* } $(, $($rest:tt)*)?) => {
        ::rusty18n::ergo_braced_update!(&mut ($target).$field_name, { $($body)* });
        ::rusty18n::ergo_braced_update_inner!($target $(, $($rest)*)?);
    };

    ($target:expr, $field_name:ident: $value:expr $(, $($rest:tt)*)?) => {
        ($target).$field_name = $value;
        ::rusty18n::ergo_braced_update_inner!($target $(, $($rest)*)?);
    };
}

/// Defines a local accessor macro bound to a specific `I18NAccess` value.
///
/// This avoids repeating the same prefix expression for every translation lookup.
///
/// Example:
/// - `t_prefix!($t, i18n);`
/// - `t!(greetings.waves)` then expands to lookup through `i18n`.
#[macro_export]
macro_rules! t_prefix {
    ($dollar:tt$name:ident, $prefix_var:ident $(. $prefix_access:tt)*) => {
        macro_rules! $name {
            ($dollar($access:tt).*) => (
                $prefix_var.acquire(|v| v$(. $prefix_access)* $dollar(. $access)*.as_ref())
            )
        }
    };

    ($dollar:tt $name:ident, $prefix_var:ident $(. $prefix_access:tt)*) => (
        rusty18n::t_prefix!($dollar$name, $prefix_var $(.$prefix_access)*)
    );

    ($dollar:tt$prefix_var:ident $(.$prefix_access:tt)*) => (
        rusty18n::t_prefix!($dollar t, $prefix_var $(. $prefix_access)*)
    );
}

/// Reads a translation value from an `I18NAccess` expression.
///
/// This is the direct form when you do not want to create a prefixed macro via `t_prefix!`.
#[macro_export]
macro_rules! t {
    ($var:ident.$($access:tt).*) => {
        $var.acquire(|v| v.$($access).*.as_ref())
    };
}
