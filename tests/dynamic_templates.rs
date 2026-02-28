use crate::fixtures::I18NUsage;

mod fixtures {
    rusty18n::define_i18n_locales! { I18NUsage => en|pt }

    pub mod en {
        rusty18n::define_i18n_fallback! {
            I18NUsage => en
            messages: {
                inferred: |a, b, c| "This is {a}, {b}, {c}",
                repeated: |name| "{name} + {name}",
                escaped: |value| "Curly {{brace}} and {value}",
                literal: "Just {braces}",
            },
        }
    }

    pub mod pt {
        rusty18n::define_i18n! {
            super::I18NUsage => pt
            messages: {
                inferred: |a, b, c| "{c} depois {a} depois {b}",
            }
        }
    }
}

#[test]
fn formats_named_arguments_and_handles_escaped_braces() {
    let locales = I18NUsage::locales();
    let en = locales.get(I18NUsage::Key::en);

    assert_eq!(
        rusty18n::t!(en.messages.inferred).with(("1", "2", "3")),
        "This is 1, 2, 3"
    );
    assert_eq!(
        rusty18n::t!(en.messages.repeated).with(("echo",)),
        "echo + echo"
    );
    assert_eq!(
        rusty18n::t!(en.messages.escaped).with(("value",)),
        "Curly {brace} and value"
    );
    assert_eq!(rusty18n::t!(en.messages.literal), "Just {braces}");
}

#[test]
fn uses_explicit_argument_names() {
    let locales = I18NUsage::locales();
    let pt = locales.get(I18NUsage::Key::pt);

    assert_eq!(
        rusty18n::t!(pt.messages.inferred).with(("A", "B", "C")),
        "C depois A depois B"
    );
}
