/// Errors produced by `rusty18n`.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// The default locale entry is missing from the locale store.
    #[error("missing fallback locale in store")]
    MissingFallbackLocale,
    /// The requested resource is missing from both the target and fallback locale values.
    #[error("resource is missing from both target and fallback locales")]
    MissingResource,
}

/// A crate-local `Result` alias.
pub type Result<T> = std::result::Result<T, Error>;
