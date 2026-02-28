/// A resolved locale view over a fallback value and an optional sparse override.
#[derive(Debug, Clone, Copy)]
pub struct I18NResolved<'a, V, O> {
    pub(crate) fallback: &'a V,
    pub(crate) override_locale: Option<&'a O>,
}

impl<V, O> I18NResolved<'_, V, O> {
    /// Resolves a resource by preferring the override value when present.
    #[must_use]
    pub fn resolve<R>(
        &self,
        fallback: for<'v> fn(&'v V) -> &'v R,
        override_locale: for<'o> fn(&'o O) -> Option<&'o R>,
    ) -> &R {
        self.override_locale
            .and_then(override_locale)
            .unwrap_or_else(|| fallback(self.fallback))
    }
}
