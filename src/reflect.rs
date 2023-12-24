use bevy_reflect::{FromReflect, GetPath, Reflect, TypePath};
use crate::core::{I18NAccess, I18NFallback, I18NTrait};

pub trait I18NReflected {
    /// Acquires a given resource through a string path. e.g "this.thing.here".
    fn rs<Resource: Reflect + TypePath + FromReflect>(&self, accessing: &str) -> Option<&Resource>;
}

impl<V: I18NFallback + Reflect> I18NReflected for V {
    fn rs<Resource: Reflect + TypePath + FromReflect>(&self, accessing: &str) -> Option<&Resource> {
        self.path::<Option<Resource>>(accessing)
            .ok()
            .and_then(|x| x.as_ref())
    }
}

impl<L: I18NTrait> I18NReflected for I18NAccess<L>
    where
        L::Value: Reflect,
{
    fn rs<Resource: Reflect + TypePath + FromReflect>(&self, acessing: &str) -> Option<&Resource> {
        self.to.rs(acessing).or_else(|| self.fallback.rs(acessing))
    }
}
