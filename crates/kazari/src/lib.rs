#[allow(dead_code)]
pub(crate) mod config;
pub(crate) mod css;
pub(crate) mod diff;
pub(crate) mod engine;
pub(crate) mod error;
pub(crate) mod escape;
pub(crate) mod frame;
pub(crate) mod js;
pub(crate) mod marker;
pub(crate) mod meta;
pub(crate) mod render;
pub(crate) mod theme_css;
pub(crate) mod tokenize;
pub(crate) mod types;

pub use config::Config;
pub use engine::{Kazari, KazariBuilder, Options};
pub use error::Error;
pub use types::{
    DarkMode, Frame, InlineMarker, LineMarker, LineRange, MarkerType, TerminalDotStyle, ThemeInfo,
    Themes,
};
