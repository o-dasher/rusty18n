#[macro_export]
macro_rules! deep_struct_update {
    ($e:expr, $($t:tt)*) => {{
        let mut temp = $e;
        deep_struct_update! {
            @main temp, $($t)*
        }
        temp
    }};
    (@main $e:expr, { $($t:tt)* }) => {
        deep_struct_update! {
            @helper {$e} {} {} $($t)*
        }
    };
    (@main $e:expr, $value:expr $(,)?) => {
        $e = $value;
    };
    (@helper {$e:expr} {$($parsed_name:ident {$($parsed_inner:tt)*})*} {$($current:tt)*} $name:ident:) => {
        deep_struct_update! {
            @emit {$e} {$($parsed_name {$($parsed_inner)*})* $name {$($current)*}}
        }
    };
    (@helper {$e:expr} {$($parsed_name:ident {$($parsed_inner:tt)*})*} {$($current:tt)*} $name:ident: , $other:ident $($rest:tt)*) => {
        deep_struct_update! {
            @helper {$e} {$($parsed_name {$($parsed_inner)*})* $name {$($current)*}} {} $other $($rest)*
        }
    };
    (@helper {$e:expr} {$($parsed_name:ident {$($parsed_inner:tt)*})*} {$($current:tt)*} $name:ident: $t:tt $($rest:tt)*) => {
        deep_struct_update! {
            @helper {$e} {$($parsed_name {$($parsed_inner)*})* } {$($current)* $t} $name: $($rest)*
        }
    };
    (@emit {$e:expr} {$($parsed_name:ident {$($parsed_inner:tt)*})*}) => {
        $(
            deep_struct_update! { @main $e.$parsed_name, $($parsed_inner)* }
        )*
    };
}
