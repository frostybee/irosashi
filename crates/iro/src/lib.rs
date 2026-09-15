//! Syntax highlighting with TextMate grammars and VS Code themes.
//!
//! Iro is a Rust port of [vscode-textmate], the syntax highlighting engine behind
//! VS Code. It produces token streams and themed HTML that are byte-identical to the
//! original on all 234 bundled grammars.
//!
//! [vscode-textmate]: https://github.com/microsoft/vscode-textmate
//!
//! # Quick start
//!
//! ```no_run
//! use iro::{Highlighter, CodeToHtmlOptions};
//!
//! let hl = Highlighter::new()?;
//! let html = hl.code_to_html("fn main() {}", &CodeToHtmlOptions::new("rust", "github-dark"))?;
//! # Ok::<(), iro::Error>(())
//! ```
//!
//! # Architecture
//!
//! - [`Highlighter`] is the main entry point: it owns a registry of grammars and
//!   themes and provides [`Highlighter::code_to_html`] and
//!   [`Highlighter::code_to_tokens`] convenience methods.
//! - [`Session`] is the low-level tokenizer for one grammar, useful for incremental
//!   or line-by-line tokenization (e.g. in an editor).
//! - [`HtmlRenderer`] and the [`Renderer`] trait convert token results to output
//!   formats.

pub(crate) mod grammar;
pub(crate) mod highlighter;
pub(crate) mod regex;
pub(crate) mod registry;
pub(crate) mod render;
pub(crate) mod scope;
pub(crate) mod theme;
pub(crate) mod token;
pub(crate) mod tokenizer;

mod error;

pub use error::Error;
pub use highlighter::{CodeToHtmlOptions, CodeToTokensOptions, Highlighter, HighlighterBuilder};
pub use registry::{AssetSource, Registry, RegistryBuilder, ThemeColors};
pub use render::{
    DefaultColor, Dialect, Escape, HtmlOptions, HtmlRenderer, Node, Renderer, SpanContext,
    StyleClassMap, Transformer, transformers,
};
pub use scope::{ScopeId, ScopeListId};
pub use theme::{ColorId, FontStyle, Theme};
pub use token::{
    Diagnostic, DiagnosticKind, LineRange, ScopeTable, ThemeSlot, ThemedLine, ThemedToken, Token,
    TokenStyle, TokensResult, in_ranges,
};
pub use tokenizer::{Session, SessionStats, StateStack, TokenizeOptions, split_lines};
