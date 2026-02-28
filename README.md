# Rusty18n

Rusty18n is a small in-memory i18n library for Rust with:

- A single DSL source of truth for fallback schema and values
- Sparse locale overrides
- Fallback resolution for omitted locale fields
- Static string resources and explicit dynamic formatters

## Install

```bash
cargo add rusty18n
```

## Current Macro Flow

1. Define the fallback schema with `define_i18n_fallback!`.
2. Define each locale override with `define_i18n!`.
3. Generate `I18NUsage::Key`, `I18NUsage::locales()`, and `I18NUsage::locales_dynamic()` with `define_i18n_locales!`.
4. Read values with `I18NStore::get` / `I18NDynamicWrapper::get` and `t_prefix!` / `t!`.

## DSL

`define_i18n_fallback!` and `define_i18n!` support:

- Nested field: `field: { ... }`
- Static resource: `field: "Text"`
- Dynamic resource: `field: |a, b, c| "{a} {b} {c}"`

Both macros now require a locale key:

- Fallback: `define_i18n_fallback! { I18NUsage => en ... }`
- Locale: `define_i18n! { I18NUsage => pt ... }`

`define_i18n_fallback!` and `define_i18n!` generate constructor functions named from the
locale key in lowercase (for example `pt` -> `pt`), so you do not need a manual wrapper `fn`.

Dynamic resources use explicit formatter parameters and are rendered with `.with((...))`.
Formatter arguments are passed as tuples, and each item only needs to implement `Display`.

Non-default locales are stored as sparse overrides. A lookup always returns a resolved view,
so omitted override fields automatically read from the fallback locale.

## Example

### Fallback schema (`example/src/i18n/en.rs`)

```rust
use rusty18n::define_i18n_fallback;

define_i18n_fallback! {
    I18NUsage => en
    greetings: {
        waves: "Waves",
        cool: "Hey that is cool",
    },
    calculus: {
        answers: |a, b, c| "{a}+{b}={c} yeah!",
    },
}
```

### Locale override (`example/src/i18n/pt.rs`)

```rust
use crate::i18n::I18NUsage;
use rusty18n::define_i18n;

define_i18n! {
    I18NUsage => pt
    greetings: {
        waves: "Oi!",
    }
}
```

### Locale registry (`example/src/i18n/mod.rs`)

```rust
pub mod en;
pub mod pt;

rusty18n::define_i18n_locales! {
    I18NUsage => en|pt
}
```

### Use in app (`example/src/main.rs`)

```rust
use rusty18n::t_prefix;

mod i18n;

fn main() {
    let locales = i18n::I18NUsage::locales();
    let i18n = locales.get(i18n::I18NUsage::Key::pt);

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

- `greetings.waves` is overridden in `pt` (`"Oi!"`)
- `calculus.answers` is missing in `pt`, so it falls back to the fallback locale
- `wah!(calculus.answers)` returns a dynamic formatter, so `.with((a, b, result))` renders it

## Macros

- `define_i18n_fallback!`: defines fallback type + values and generates a locale constructor.
- `define_i18n!`: defines sparse locale overrides and generates a locale constructor.
- `define_i18n_locales!`: generates the `I18NUsage` namespace module with `Value`, `Override`, `Key`, `locales()`, and `locales_dynamic()`.
- `t_prefix!`: creates a scoped accessor macro for a resolved locale value.
- `t!`: direct accessor macro for one-off lookups.
