use crate::i18n::I18NUsage;
use rusty18n::t_prefix;

mod i18n;

fn main() -> rusty18n::Result<()> {
    let locales = I18NUsage::locales()?;
    let i18n = locales.get(I18NUsage::Key::pt)?;

    let a = 3;
    let b = 2;
    let result = a + b;

    t_prefix!($wah, i18n);

    let response_static = wah!(greetings.waves)?;
    let response_dynamic = wah!(calculus.answers)?.with((a, b, result))?;

    println!("{response_static}");
    println!("{response_dynamic}");

    Ok(())
}
