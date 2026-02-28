mod access;
mod dynamic;
mod error;
mod resource;
mod store;
mod traits;

pub use self::{
    access::I18NAccess,
    dynamic::{I18NDynamicWrapper, I18NLocaleLoader},
    error::{Error, Result},
    resource::{I18NDynamicResourceValue, IntoDynamicResourceArgs},
    store::I18NStore,
    traits::{I18NFallback, I18NTrait},
};
