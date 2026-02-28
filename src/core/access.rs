use super::{I18NTrait, Result};

/// A struct representing access to i18n resources, with fallback support.
///
/// This struct holds references to both the fallback and target i18n resources.
/// It allows accessing resources by applying a provided accessor function.
pub struct I18NAccess<'a, L: I18NTrait> {
    pub fallback: &'a L::V,
    pub to: &'a L::V,
}

impl<L: I18NTrait> I18NAccess<'_, L> {
    /// Acquires a resource by applying the provided accessor function.
    ///
    /// This method attempts to access the target resource first and falls back to
    /// the fallback resource if the target resource is not available.
    ///
    /// # Arguments
    /// * `accessing` - A function that takes a reference to an i18n value and returns
    ///   an optional reference to the desired resource.
    ///
    /// # Returns
    /// A reference to the acquired resource.
    ///
    /// # Errors
    /// Returns [`crate::Error::MissingResource`] when neither the target nor the fallback
    /// contains the requested resource.
    pub fn acquire<R>(&self, accessing: fn(&L::V) -> Option<&R>) -> Result<&R> {
        accessing(self.to)
            .or_else(|| accessing(self.fallback))
            .ok_or(crate::Error::MissingResource)
    }
}
