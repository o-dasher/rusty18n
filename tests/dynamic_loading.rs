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

fn inferred_text<L>(access: &rusty18n::I18NAccess<'_, L>) -> String
where
    L: rusty18n::I18NTrait<V = I18NUsage::Value>,
{
    let value = access
        .acquire(|v| v.messages.inferred.as_ref())
        .expect("inferred resource should resolve");

    value
        .with(("C", "A", "B"))
        .expect("inferred resource should resolve")
}

#[test]
fn loads_and_unloads_locales_on_demand() {
    let mut locales = I18NUsage::locales_dynamic();

    assert!(locales.is_loaded(I18NUsage::Key::en));
    assert!(locales.is_registered(I18NUsage::Key::pt));
    assert!(!locales.is_loaded(I18NUsage::Key::pt));

    let pt = locales
        .get(I18NUsage::Key::pt)
        .expect("pt locale access should be available");
    assert_eq!(
        rusty18n::t!(pt.messages.inferred).and_then(|value| value.with(("C", "A", "B"))),
        Ok("This is C, A, B".to_string())
    );
    assert_eq!(
        rusty18n::t!(pt.messages.literal).map(std::convert::AsRef::as_ref),
        Ok("Fallback literal")
    );

    assert!(locales.load(I18NUsage::Key::pt));
    assert!(locales.is_loaded(I18NUsage::Key::pt));

    let pt = locales
        .get(I18NUsage::Key::pt)
        .expect("pt locale access should be available");
    assert_eq!(
        rusty18n::t!(pt.messages.inferred).and_then(|value| value.with(("C", "A", "B"))),
        Ok("C depois A depois B".to_string())
    );
    assert_eq!(
        rusty18n::t!(pt.messages.literal).map(std::convert::AsRef::as_ref),
        Ok("Fallback literal")
    );

    let unloaded = locales.unload(I18NUsage::Key::pt);
    assert!(unloaded.is_some());
    assert!(!locales.is_loaded(I18NUsage::Key::pt));

    let pt = locales
        .get(I18NUsage::Key::pt)
        .expect("pt locale access should be available");
    assert_eq!(
        rusty18n::t!(pt.messages.inferred).and_then(|value| value.with(("C", "A", "B"))),
        Ok("This is C, A, B".to_string())
    );

    assert!(locales.unload(I18NUsage::Key::en).is_none());
    assert!(locales.is_loaded(I18NUsage::Key::en));
}

#[test]
fn shares_access_behavior_between_wrappers() {
    let eager = I18NUsage::locales();
    let eager_pt = eager
        .get(I18NUsage::Key::pt)
        .expect("locale access should resolve");
    assert_eq!(inferred_text(&eager_pt), "C depois A depois B");

    let mut dynamic = I18NUsage::locales_dynamic();
    let dynamic_pt = dynamic
        .get(I18NUsage::Key::pt)
        .expect("locale access should resolve");
    assert_eq!(inferred_text(&dynamic_pt), "This is C, A, B");

    assert!(dynamic.load(I18NUsage::Key::pt));
    let dynamic_pt = dynamic
        .get(I18NUsage::Key::pt)
        .expect("locale access should resolve");
    assert_eq!(inferred_text(&dynamic_pt), "C depois A depois B");
}
