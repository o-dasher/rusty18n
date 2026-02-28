mod dynamic;
mod resolved;
mod resource;
mod store;
mod traits;

pub use self::{
    dynamic::{I18NDynamicWrapper, I18NLocaleLoader},
    resolved::I18NResolved,
    resource::{
        __normalize_template, __template_arity, __template_has_escapes, I18NDynamicResourceValue,
        IntoDynamicResourceArgs,
    },
    store::I18NStore,
    traits::{I18NFallback, I18NTrait},
};
