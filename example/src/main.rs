use rusty18n::{t_prefix, I18NWrapper};

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
    let i18n = locales.get(I18NKey::PTBR);

    let a = 3;
    let b = 2;
    let result = a + b;

    t_prefix!($wah, i18n);

    let response_static = wah!(greetings.waves);
    let response_dynamic =
        wah!(calculus.answers).with((a.to_string(), b.to_string(), result.to_string()));

    println!("{}", response_static);
    println!("{}", response_dynamic);
}
