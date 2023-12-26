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
            ..I18NUsage::fallback().calculus // this is kind of sad, a macro would be better to
                                             // recurse here...
        },
        // So, we don't have to update all the i18n locales as soon as we add a single locale.
        // ideally we would specify this once and it would recurse all the way deep the i18n tree
        // to not cumber dx in any way.
        ..I18NUsage::fallback()
    }
}
