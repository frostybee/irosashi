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
pub(crate) mod notation;
pub(crate) mod render;
pub(crate) mod render_typst;
pub(crate) mod theme_css;
pub(crate) mod tokenize;
pub(crate) mod types;

pub use config::{Config, FileConfig};
pub use engine::{Kazari, KazariBuilder, Options};
pub use error::Error;
pub use render_typst::preamble as typst_preamble;
pub use types::{
    DarkMode, Frame, InlineMarker, LineMarker, LineRange, MarkerType, TerminalDotStyle, ThemeInfo,
    Themes,
};
