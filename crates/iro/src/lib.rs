pub mod grammar;
pub mod highlighter;
pub mod regex;
pub mod registry;
pub mod render;
pub mod scope;
pub mod theme;
pub mod token;
pub mod tokenizer;

mod error;

pub use error::Error;
pub use highlighter::{CodeToHtmlOptions, CodeToTokensOptions, Highlighter, HighlighterBuilder};
pub use registry::{AssetSource, Registry, RegistryBuilder, ThemeColors};
pub use render::{
    DefaultColor, Dialect, Escape, HtmlOptions, HtmlRenderer, Node, Renderer, StyleClassMap,
};
pub use scope::{ScopeId, ScopeListId};
pub use theme::{FontStyle, Theme};
pub use token::{
    Diagnostic, DiagnosticKind, ScopeTable, ThemeSlot, ThemedLine, ThemedToken, Token, TokenStyle,
    TokensResult,
};
pub use tokenizer::{Session, SessionStats, StateStack, TokenizeOptions};
