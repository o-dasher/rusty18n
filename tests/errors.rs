use rusty18n::{Error, I18NFallback, I18NWrapper, R};

#[derive(Default)]
struct EmptyLocale {
    value: Option<String>,
}

impl I18NFallback for EmptyLocale {
    fn fallback() -> rusty18n::Result<Self> {
        Ok(Self::default())
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Hash)]
enum Locale {
    #[default]
    En,
}

#[test]
fn returns_errors_for_invalid_templates_and_argument_mismatches() {
    assert_eq!(
        R::new("Hello {"),
        Err(Error::InvalidTemplate {
            template: "Hello {".to_string(),
        })
    );

    let resource = R::new("Hello {name}").expect("template should parse");
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
    let locales = I18NWrapper::<Locale, EmptyLocale>::new(vec![])
        .expect("locale construction should succeed");
    let access = locales
        .get(Locale::En)
        .expect("default locale should exist");

    assert_eq!(
        access.acquire(|value| value.value.as_ref()),
        Err(Error::MissingResource)
    );
}
