#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("regex compilation failed for pattern {index}: {message}")]
    RegexCompilation { index: usize, message: String },

    #[error("language not found: {0}")]
    LanguageNotFound(String),

    #[error("theme not found: {0}")]
    ThemeNotFound(String),

    #[error("grammar include cycle detected")]
    GrammarCycle,

    #[error("grammar include depth exceeded")]
    GrammarDepth,
}
