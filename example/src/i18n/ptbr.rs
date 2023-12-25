use crate::i18n::i_18_n_usage::I18NUsage;
use rusty18n::{r, I18NDynamicResource};

use super::i_18_n_usage::{calculus::Calculus, greetings::Greetings};

pub fn i18n_ptbr() -> I18NUsage {
    I18NUsage {
        greetings: Greetings { waves: r!("Olas!") },
        calculus: Calculus {
            answers: r!(|(a, b, c)| "{a}+{b}={c} batatudo"),
        },
    }
}
