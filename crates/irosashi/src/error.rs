#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("regex compilation failed for pattern {index}: {message}")]
    RegexCompilation { index: usize, message: String },

    #[error("grammar parse failed: {0}")]
    GrammarParse(String),

    #[error("language not found: {0}")]
    LanguageNotFound(String),

    #[error("theme not found: {0}")]
    ThemeNotFound(String),

    #[error("theme parse failed: {0}")]
    ThemeParse(String),

    #[error("io error: {0}")]
    Io(String),

    #[error("grammar include cycle detected")]
    GrammarCycle,

    #[error("grammar include depth exceeded")]
    GrammarDepth,

    #[error("grammar rule count exceeds the arena limit")]
    GrammarTooLarge,
}
