use rusty18n::define_i18n_fallback;

define_i18n_fallback! {
    I18NUsage => en
    greetings: {
        waves: "Waves",
        cool: "Hey that is cool",
    },
    calculus: {
        answers: "{a}+{b}={c} yeah!",
    },
}
