mod en;

use nestruct::nest;

nest!(
    #[derive(Default)]
    I18NUsage {
        greetings: {
            waves: rusty18n::R?
        },
        calculus: {
            answers: rusty18n::DR<(String, String, String)>?
        }
    }
);
