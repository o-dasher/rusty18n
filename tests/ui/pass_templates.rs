rusty18n::define_i18n_fallback! {
    PassUsage => en
    plain: "Hello",
    named: "Hello {name}",
    repeated: "{name} + {name}",
    escaped: "Curly {{brace}} and {value}",
}

fn main() {}
