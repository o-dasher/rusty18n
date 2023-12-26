use crate::i18n::i_18_n_usage::I18NUsage;
use rusty18n::{r, I18NDynamicResource, I18NFallback};

use super::i_18_n_usage::{calculus::Calculus, greetings::Greetings};

pub fn i18n_ptbr() -> I18NUsage {
    I18NUsage {
        greetings: Greetings {
            waves: r!("Iai")
        },
        calculus: Calculus {
            answers: r!(|(a, b, c)| "{a}+{b}={c} batatudo"),
        },
        // So, we don't have to update all the i18n locales as soon as we add a single locale.
        ..I18NUsage::fallback()
    }
}
