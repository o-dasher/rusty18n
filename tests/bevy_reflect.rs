#![cfg(feature = "bevy_reflect")]

use bevy_reflect::Reflect;
use rusty18n::{I18NFallback, I18NReflected, I18NWrapper};

#[derive(Debug, Default, Reflect)]
struct Greetings {
    waves: Option<String>,
    nested: Option<String>,
}

#[derive(Debug, Default, Reflect)]
struct Messages {
    literal: Option<String>,
    translated: Option<String>,
}

#[derive(Debug, Default, Reflect)]
struct FixtureI18n {
    greetings: Greetings,
    messages: Messages,
}

impl I18NFallback for FixtureI18n {
    fn fallback() -> Self {
        Self {
            greetings: Greetings {
                waves: Some("Waves".to_string()),
                nested: Some("Fallback nested".to_string()),
            },
            messages: Messages {
                literal: Some("Fallback literal".to_string()),
                translated: Some("Fallback translated".to_string()),
            },
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
enum Locale {
    #[default]
    En,
    Pt,
}

fn pt() -> FixtureI18n {
    FixtureI18n {
        greetings: Greetings {
            nested: Some("Portuguese nested".to_string()),
            ..Default::default()
        },
        messages: Messages {
            translated: Some("Portuguese translated".to_string()),
            ..Default::default()
        },
    }
}

#[test]
fn reflects_sparse_locale_values_by_path() {
    let pt = pt();

    assert_eq!(
        pt.by_path::<String>("greetings.nested").map(String::as_str),
        Some("Portuguese nested")
    );
    assert_eq!(
        pt.by_path::<String>("messages.translated")
            .map(String::as_str),
        Some("Portuguese translated")
    );
    assert!(pt.by_path::<String>("greetings.waves").is_none());
    assert!(pt.by_path::<String>("messages.literal").is_none());
}

#[test]
fn reflects_access_values_with_fallback() {
    let locales = I18NWrapper::<Locale, FixtureI18n>::new(vec![(Locale::Pt, pt)]);
    let pt = locales.get(Locale::Pt);

    assert_eq!(
        pt.by_path::<String>("greetings.waves").map(String::as_str),
        Some("Waves")
    );
    assert_eq!(
        pt.by_path::<String>("greetings.nested").map(String::as_str),
        Some("Portuguese nested")
    );
    assert_eq!(
        pt.by_path::<String>("messages.literal").map(String::as_str),
        Some("Fallback literal")
    );
    assert_eq!(
        pt.by_path::<String>("messages.translated")
            .map(String::as_str),
        Some("Portuguese translated")
    );
    assert!(pt.by_path::<String>("messages.missing").is_none());
}
