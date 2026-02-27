# Rusty18n

Rusty18n is a small in-memory i18n library for Rust with:

- A single DSL source of truth for the fallback schema
- Sparse locale overrides
- Runtime fallback for missing translations
- Static and dynamic resources

## Install

```bash
cargo add rusty18n
```

## How It Works

1. Define your canonical i18n schema + fallback values with `define_i18n_fallback!`.
2. Define each locale with `define_i18n!` by overriding only what is translated.
3. Read translations through `I18NWrapper` + `I18NAccess`.
4. If a locale field is `None`, lookup automatically falls back to the fallback locale.

## DSL

`define_i18n_fallback!` and `define_i18n!` support:

- Nested field: `field { ... }`
- Static resource: `field: "Text"`
- Dynamic resource: `field: |a, b, c| => "{a} {b} {c}"`

Dynamic resource arguments passed to `.with((...))` can be any tuple items implementing `Display`.
They are converted to `String` internally.

## Example

### Fallback schema (`example/src/i18n/en.rs`)

```rust
use rusty18n::define_i18n_fallback;

define_i18n_fallback! {
    I18NUsage =>
    greetings {
        waves: "Waves",
        cool: "Hey that is cool",
    },
    calculus {
        answers: |a, b, c| => "{a}+{b}={c} yeah!",
    },
}
```

### Partial locale override (`example/src/i18n/ptbr.rs`)

```rust
use crate::i18n::I18NUsage;
use rusty18n::define_i18n;

pub fn i18n_ptbr() -> I18NUsage {
    define_i18n! {
        I18NUsage,
        greetings: {
            waves: "Oi!",
        }
    }
}
```

### Use in app (`example/src/main.rs`)

```rust
use rusty18n::{t_prefix, I18NWrapper};
use crate::i18n::ptbr::i18n_ptbr;

mod i18n;

#[derive(Default, Clone, Copy, Hash, Eq, PartialEq)]
pub enum I18NKey {
    #[default]
    US,
    PTBR,
}

fn main() {
    let locales = I18NWrapper::new(vec![(I18NKey::PTBR, i18n_ptbr)]);
    let i18n = locales.get(I18NKey::PTBR);

    let a = 3;
    let b = 2;
    let result = a + b;

    t_prefix!($wah, i18n);

    let response_static = wah!(greetings.waves);
    let response_dynamic = wah!(calculus.answers).with((a, b, result));

    println!("{}", response_static);
    println!("{}", response_dynamic);
}
```

In this example:

- `greetings.waves` is overridden in `PTBR` (`"Oi!"`)
- `calculus.answers` is missing in `PTBR`, so it falls back to the fallback locale

## Macros

- `define_i18n_fallback!`: generate the root i18n type and fallback implementation from the DSL.
- `define_i18n!`: create sparse locale values by overriding only selected fields.
- `t_prefix!`: create a scoped accessor macro for an `I18NAccess` value.
- `t!`: direct accessor macro when you do not want a custom prefix macro.
