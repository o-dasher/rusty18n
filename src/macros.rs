// This is some **MAGICAL** code that i wrote that i will probably never be able
// to understand again. But these are rust macros i guess...

#[macro_export]
// Macro to easily define an i18n resource.
macro_rules! r {
    (|($($args:pat),*)| $lit:literal) => {
        Some(I18NDynamicResource:new(|($($args),*)| format!($lit)))
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
macro_rules! t_prefix {
    ($dollar:tt, $name:ident, $prefix_var:ident $(. $prefix_access:tt)*) => (
        macro_rules! $name {
            ($dollar($access:tt).*) => (
                $prefix_var.r(|v| &v$(. $prefix_access)* $dollar(. $access)*)
            )
        }
    );

    ($dollar:tt, $prefix_var:ident $(.$prefix_access:tt)*) => (
        t_prefix!($dollar, t, $prefix_var $(. $prefix_access)*)
    );
}

#[macro_export]
macro_rules! t {
    ($var:ident.$($access:tt).*) => {
        $var.r(|v| &v.$($access).*)
    };
}