use crate::fixtures::I18NUsage;

mod fixtures {
    rusty18n::define_i18n_locales! { I18NUsage => en|pt }

    pub mod en {
        rusty18n::define_i18n_fallback! {
            I18NUsage => en
            messages: {
                inferred: "This is {a}, {b}, {c}",
                literal: "Fallback literal",
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
fn loads_and_unloads_locales_on_demand() {
    let mut locales = I18NUsage::locales_dynamic();

    assert!(locales.is_registered(I18NUsage::Key::pt));
    assert!(!locales.is_loaded(I18NUsage::Key::pt));

    let pt = locales.get(I18NUsage::Key::pt);
    assert_eq!(
        rusty18n::t!(pt.messages.inferred).with(("C", "A", "B")),
        "This is C, A, B"
    );
    assert_eq!(rusty18n::t!(pt.messages.literal), "Fallback literal");

    assert!(locales.load(I18NUsage::Key::pt));
    assert!(locales.is_loaded(I18NUsage::Key::pt));

    let pt = locales.get(I18NUsage::Key::pt);
    assert_eq!(
        rusty18n::t!(pt.messages.inferred).with(("C", "A", "B")),
        "C depois A depois B"
    );
    assert_eq!(rusty18n::t!(pt.messages.literal), "Fallback literal");

    let unloaded = locales.unload(I18NUsage::Key::pt);
    assert!(unloaded.is_some());
    assert!(!locales.is_loaded(I18NUsage::Key::pt));

    let pt = locales.get(I18NUsage::Key::pt);
    assert_eq!(
        rusty18n::t!(pt.messages.inferred).with(("C", "A", "B")),
        "This is C, A, B"
    );
}
