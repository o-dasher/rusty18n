use crate::fixtures::I18NUsage;

mod fixtures {
    rusty18n::define_i18n_locales! { I18NUsage => en|pt }

    pub mod en {
        rusty18n::define_i18n_fallback! {
            I18NUsage => en
            greetings {
                waves: "Waves",
                oi {
                    a: "English nested",
                },
            },
        }
    }

    pub mod pt {
        rusty18n::define_i18n! {
            super::I18NUsage => pt
            greetings {
                oi {
                    a: "Portuguese nested",
                },
            }
        }
    }
}

#[test]
fn supports_nested_blocks() {
    let locales = I18NUsage::locales();

    let en = locales.get(I18NUsage::Key::en);
    assert_eq!(rusty18n::t!(en.greetings.waves), "Waves");
    assert_eq!(rusty18n::t!(en.greetings.oi.a), "English nested");

    let pt = locales.get(I18NUsage::Key::pt);
    assert_eq!(rusty18n::t!(pt.greetings.waves), "Waves");
    assert_eq!(rusty18n::t!(pt.greetings.oi.a), "Portuguese nested");
}
