pub mod grammar;
pub mod regex;
pub mod registry;
pub mod render;
pub mod scope;
pub mod theme;
pub mod token;
pub mod tokenizer;

mod error;

pub use error::Error;
pub use registry::Registry;
pub use scope::{ScopeId, ScopeListId};
pub use theme::{FontStyle, Theme};
pub use token::{
    Diagnostic, DiagnosticKind, ThemedLine, ThemedToken, Token, TokenStyle, TokensResult,
};
pub use tokenizer::{Session, StateStack, TokenizeOptions};
