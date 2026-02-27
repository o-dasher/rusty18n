use crate::fixtures::I18NUsage;

mod fixtures {
    rusty18n::define_i18n_locales! { I18NUsage => en|pt }

    pub mod en {
        rusty18n::define_i18n_fallback! {
            I18NUsage => en
            messages: {
                inferred: "This is {a}, {b}, {c}",
                repeated: "{name} + {name}",
                escaped: "Curly {{brace}} and {value}",
                literal: "Just {{braces}}",
            },
        }
    }

    pub mod pt {
        rusty18n::define_i18n! {
            super::I18NUsage => pt
            messages: {
                inferred: "{c} depois {a} depois {b}",
            }
        }
    }
}

#[test]
fn infers_placeholders_and_handles_escaped_braces() {
    let locales = I18NUsage::locales();
    let en = locales
        .get(I18NUsage::Key::en)
        .expect("en locale should exist");

    assert_eq!(
        rusty18n::t!(en.messages.inferred).and_then(|value| value.with((1, 2, 3))),
        Ok("This is 1, 2, 3".to_string())
    );
    assert_eq!(
        rusty18n::t!(en.messages.repeated).and_then(|value| value.with(("echo",))),
        Ok("echo + echo".to_string())
    );
    assert_eq!(
        rusty18n::t!(en.messages.escaped).and_then(|value| value.with(("value",))),
        Ok("Curly {brace} and value".to_string())
    );
    assert_eq!(
        rusty18n::t!(en.messages.literal).map(std::convert::AsRef::as_ref),
        Ok("Just {braces}")
    );
}

#[test]
fn infers_placeholder_order_from_first_appearance() {
    let locales = I18NUsage::locales();

    let pt = locales
        .get(I18NUsage::Key::pt)
        .expect("pt locale access should be available");
    assert_eq!(
        rusty18n::t!(pt.messages.inferred).and_then(|value| value.with(("C", "A", "B"))),
        Ok("C depois A depois B".to_string())
    );
}
