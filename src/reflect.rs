#[cfg(feature = "bevy_reflect")]
use crate::core::I18NFallback;

#[cfg(feature = "bevy_reflect")]
use bevy_reflect::{FromReflect, GetPath, Reflect, TypePath};

#[cfg(feature = "bevy_reflect")]
pub trait I18NReflected {
    /// Acquires a given resource through a string path. e.g "this.thing.here".
    fn by_path<R: Reflect + TypePath + FromReflect>(&self, path: &str) -> Option<&R>;
}

#[cfg(feature = "bevy_reflect")]
impl<V: I18NFallback + Reflect> I18NReflected for V {
    fn by_path<R: Reflect + TypePath + FromReflect>(&self, path: &str) -> Option<&R> {
        self.path(path).ok()
    }
}
