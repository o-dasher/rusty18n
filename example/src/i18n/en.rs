use rusty18n::r;
use rusty18n::{define_i18n_fallback, I18NDynamicResource};

define_i18n_fallback! {
    i_18_n_usage,
    greetings {
        waves: r!("Waves"),
        cool: r!("Hey that is cool"),
    },
    calculus {
        answers: r!(|(a, b, c)| "{a}+{b}={c} yeah!"),
    },
}
