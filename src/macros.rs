// This is some **MAGICAL** code that i wrote that i will probably never be able
// to understand again. But these are rust macros i guess...

#[macro_export]
// Macro to easily define an i18n resource.
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

#[macro_export]
macro_rules! define_i18n {
    ($base_i18n:ident, $($body:tt)*) => {
        deep_struct_update::update! { $($body)* ..$base_i18n::fallback() }
    };
}

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

#[macro_export]
macro_rules! t {
    ($var:ident.$($access:tt).*) => {
        $var.acquire(|v| v.$($access).*.as_ref())
    };
}
