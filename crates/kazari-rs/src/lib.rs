pub mod backends;
pub(crate) mod collapsible;
#[allow(dead_code)]
pub(crate) mod color;
#[allow(dead_code)]
pub(crate) mod config;
pub(crate) mod css;
pub(crate) mod diff;
pub(crate) mod engine;
pub(crate) mod error;
pub(crate) mod escape;
pub(crate) mod frame;
pub(crate) mod hash;
pub(crate) mod highlighter;
pub(crate) mod js;
pub(crate) mod link;
pub(crate) mod locale;
#[cfg(feature = "markdown")]
pub mod markdown;
pub(crate) mod marker;
pub(crate) mod meta;
pub(crate) mod minify;
pub(crate) mod notation;
pub(crate) mod render;
pub(crate) mod render_typst;
pub(crate) mod theme_css;
pub(crate) mod tokenize;
pub(crate) mod types;

pub use config::{
    CollapseRange, CollapseSpec, CollapseStyle, CollapsibleConfig, Config, FileConfig,
    LangIconMode, PreviewSegment, ProcessFile, StyleValue, TypstConfig,
};
pub use engine::{Kazari, KazariBuilder, Options, PostRender, ThemeCustomizer};
pub use error::Error;
pub use highlighter::{FontStyle, Highlighted, Highlighter, Line, Style, ThemeDefaults, Token};
pub use locale::UIStrings;
pub use render_typst::preamble as typst_preamble;
pub use types::{
    AdjustTargets, AssetFile, Assets, BlockInfo, DarkMode, Frame, InlineMarker, LineMarker,
    LineRange, LinkAnnotation, MarkerType, TerminalDotStyle, ThemeAdjustments, ThemeInfo, Themes,
};
