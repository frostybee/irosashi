pub(crate) mod error;
#[allow(dead_code)]
pub(crate) mod tokenize;
pub(crate) mod types;

pub use error::Error;
pub use types::{
    DarkMode, Frame, InlineMarker, LineMarker, LineRange, MarkerType, TerminalDotStyle, ThemeInfo,
    Themes,
};
