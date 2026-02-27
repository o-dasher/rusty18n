pub mod en;
pub mod pt;
pub use en::I18NUsage;

rusty18n::define_i18n_locales! {
    I18NUsage => en|pt
}
