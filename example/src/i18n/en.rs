use crate::i18n::i_18_n_usage::I18NUsage;
use rusty18n::{r, I18NDynamicResource, I18NFallback};

use super::i_18_n_usage::{calculus::Calculus, greetings::Greetings};

impl I18NFallback for I18NUsage {
    fn fallback() -> Self {
        I18NUsage {
            greetings: Greetings { waves: r!("Waves") },
            calculus: Calculus {
                answers: r!(|(a, b, c)| "{a}+{b}={c}"),
            },
        }
    }
}
