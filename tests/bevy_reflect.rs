#![cfg(feature = "bevy_reflect")]

use crate::fixtures::I18NUsage;
use rusty18n::{I18NReflected, R};

mod fixtures {
    rusty18n::define_i18n_locales! { I18NUsage => en|pt }

    pub mod en {
        rusty18n::define_i18n_fallback! {
            I18NUsage => en
            greetings: {
                waves: "Waves",
                nested: "Fallback nested",
            },
            messages: {
                literal: "Fallback literal",
                translated: "Fallback translated",
            },
        }
    }

    pub mod pt {
        rusty18n::define_i18n! {
            super::I18NUsage => pt
            greetings: {
                nested: "Portuguese nested",
            },
            messages: {
                translated: "Portuguese translated",
            }
        }
    }
}

#[test]
fn reflects_sparse_locale_values_by_path() {
    let pt = fixtures::pt::pt();
    assert_eq!(
        pt.by_path::<R>("greetings.nested")
            .expect("nested resource should exist"),
        "Portuguese nested"
    );
    assert_eq!(
        pt.by_path::<R>("messages.translated")
            .expect("translated resource should exist"),
        "Portuguese translated"
    );
    assert!(pt.by_path::<R>("greetings.waves").is_none());
    assert!(pt.by_path::<R>("messages.literal").is_none());
}

#[test]
fn reflects_access_values_with_fallback() -> rusty18n::Result<()> {
    let locales = I18NUsage::locales();
    let pt = locales.get(I18NUsage::Key::pt)?;
    assert_eq!(
        pt.by_path::<R>("greetings.waves")
            .expect("fallback waves should exist"),
        "Waves"
    );
    assert_eq!(
        pt.by_path::<R>("greetings.nested")
            .expect("nested resource should exist"),
        "Portuguese nested"
    );
    assert_eq!(
        pt.by_path::<R>("messages.literal")
            .expect("fallback literal should exist"),
        "Fallback literal"
    );
    assert_eq!(
        pt.by_path::<R>("messages.translated")
            .expect("translated resource should exist"),
        "Portuguese translated"
    );
    assert!(pt.by_path::<R>("messages.missing").is_none());

    Ok(())
}
