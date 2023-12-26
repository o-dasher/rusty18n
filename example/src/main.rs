use rusty18n::{t, I18NAccessible, I18NWrapper};

use crate::i18n::ptbr::i18n_ptbr;

mod i18n;

#[derive(Default, Clone, Copy, Hash, Eq, PartialEq)]
pub enum I18NKey {
    #[default]
    US,
    PTBR,
}

fn main() {
    let locales = I18NWrapper::new(vec![(I18NKey::PTBR, i18n_ptbr)]);
    let br_locale = locales.get(I18NKey::PTBR);

    let a = 1;
    let b = 2;
    let result = a + b;

    let response_static = t!(br_locale.greetings.waves);
    let response_dynamic =
        t!(br_locale.calculus.answers).access((a.to_string(), b.to_string(), result.to_string()));

    println!("{}", response_static);
    println!("{}", response_dynamic);
}
