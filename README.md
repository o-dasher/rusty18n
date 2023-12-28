# Rusty18n
A pretty simple yet robust library to deal with i18n on Rust.

# Why Rusty18n?
Wouldn't it be pretty useful and handy if you could write all your translations directly in your
source code? This is what this crate is solving. I was in need of a simple and easy to use i18n
solution for my discord bot, and I had the idea to create a simple in-memory i18n handler. It
provides all the basis for what you would expect for an in-memory i18n in Rust, covering simple
static translations to dynamic ones.

# So how do i start?
You just need to do add some dependencies to your project:
```
cargo add rusty18n
cargo add deep_struct_update
```

`deep_struct_update` is used so you can write partial i18n implementations for languages that are
still being working on, like in the example:
```rs
// We need to import deep_struct_update before we can use the define_i18n macro
use deep_struct_update;

pub fn i18n_ptbr() -> I18NUsage {
    // Using the define_i18n macro we don't need to change all the i18n implementations once
    // something in the base fallback implementation structure changes.
    define_i18n! {
        I18NUsage,
        greetings: {
            waves: r!("Oi!"),
        }
    }
}
```
You can see an example usage [here](https://github.com/o-dasher/rusty18n/tree/master/example)
