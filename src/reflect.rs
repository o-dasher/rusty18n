use crate::core::{I18NAccess, I18NFallback, I18NTrait};
use bevy_reflect::{FromReflect, GetPath, Reflect, TypePath};

pub trait I18NReflected {
    /// Acquires a given resource through a string path. e.g "this.thing.here".
    fn rs<Resource: Reflect + TypePath + FromReflect>(&self, path: &str) -> Option<&Resource>;
}

impl<V: I18NFallback + Reflect> I18NReflected for V {
    fn rs<Resource: Reflect + TypePath + FromReflect>(&self, path: &str) -> Option<&Resource> {
        self.path(path).ok().and_then(Option::as_ref)
    }
}

impl<L: I18NTrait> I18NReflected for I18NAccess<L>
where
    L::Value: Reflect,
{
    fn rs<Resource: Reflect + TypePath + FromReflect>(&self, path: &str) -> Option<&Resource> {
        self.to.rs(path).or_else(|| self.fallback.rs(path))
    }
}
