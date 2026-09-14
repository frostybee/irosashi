#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("highlighting failed: {0}")]
    Highlight(String),

    #[error("theme not found: {0}")]
    ThemeNotFound(String),

    #[error("config error: {0}")]
    Config(String),
}

impl From<iro::Error> for Error {
    fn from(e: iro::Error) -> Self {
        match e {
            iro::Error::ThemeNotFound(name) => Error::ThemeNotFound(name),
            other => Error::Highlight(other.to_string()),
        }
    }
}
