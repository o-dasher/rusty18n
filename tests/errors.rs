use rusty18n::{Error, R};

use crate::fixtures::ErrorUsage;

mod fixtures {
    rusty18n::define_i18n_locales! { ErrorUsage => en|pt }

    pub mod en {
        rusty18n::define_i18n_fallback! {
            ErrorUsage => en
            value: "Hello {name}",
        }
    }

    pub mod pt {
        rusty18n::define_i18n! {
            super::ErrorUsage => pt
        }
    }
}

#[test]
fn returns_errors_for_invalid_templates_and_argument_mismatches() {
    assert_eq!(
        R::new("Hello {"),
        Err(Error::InvalidTemplate {
            template: "Hello {".to_string(),
        })
    );

    let resource = fixtures::en::en().value.expect("resource should exist");
    assert_eq!(
        resource.with(()),
        Err(Error::InvalidArgumentCount {
            template: "Hello {name}".to_string(),
            expected: 1,
            got: 0,
        })
    );
}

#[test]
fn returns_an_error_when_target_and_fallback_are_missing_the_resource() {
    let locales = ErrorUsage::locales();
    let access = locales
        .get(ErrorUsage::Key::pt)
        .expect("locale access should succeed");

    assert_eq!(
        access.acquire(|_| None::<&rusty18n::R>),
        Err(Error::MissingResource)
    );
}
