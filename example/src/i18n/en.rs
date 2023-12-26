use crate::i18n::i_18_n_usage::I18NUsage;
use rusty18n::{r, I18NDynamicResource, I18NFallback};

use super::i_18_n_usage::{calculus::Calculus, greetings::Greetings};

// Yeah, i understand this is not ideal... having to specify classes, import them, kind of painful
// dx to be honest, still good though, but could for sure be improved! Ideally we would have some
// kind of procedural macro working here that would have as an input the default fallback i18n,
// automagically placing the class names for you and generating a base implementation that would
// be "inherited" by all other i18n implementations, so this way if we add another i18n key we
// don't have to put i18n in all places asap.
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
