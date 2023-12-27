use crate::i18n::i_18_n_usage::I18NUsage;
use rusty18n::{define_i18n, r, I18NFallback};
use update::deep_struct_update;

pub fn i18n_ptbr() -> I18NUsage {
    // Using the define_i18n macro we don't need to change all the i18n implementations once
    // something in the base fallback implementation structure changes.
    define_i18n!(
        I18NUsage,
        {
            greetings: {
                waves: r!("Oi!"),
            }
        }
    )
}
