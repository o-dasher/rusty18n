use rusty18n::t_prefix;

mod i18n;

fn main() {
    let locales = i18n::i18n_locales();
    let i18n = locales.get(i18n::I18NKey::Pt);

    let a = 3;
    let b = 2;
    let result = a + b;

    t_prefix!($wah, i18n);

    let response_static = wah!(greetings.waves);
    let response_dynamic = wah!(calculus.answers).with((a, b, result));

    println!("{}", response_static);
    println!("{}", response_dynamic);
}
