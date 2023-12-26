mod en;
pub mod ptbr;

use nestruct::nest;

nest!(
    I18NUsage {
        greetings: {
            waves: rusty18n::R?
        },
        calculus: {
            answers: rusty18n::DR<(String, String, String)>?
        }
    }
);
