use std::ops::Range;
use std::sync::Arc;

use crate::scope::ScopeListId;
use crate::theme::{ColorId, FontStyle, Theme};

/// A raw token: a byte range within its line and an interned scope stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub start: usize,
    pub end: usize,
    pub scopes: ScopeListId,
}

impl Token {
    pub fn text<'l>(&self, line: &'l str) -> &'l str {
        &line[self.start..self.end]
    }
}

/// Style resolved from a theme for one scope stack. Colors are ids into the theme's
/// color table (`Theme::color`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TokenStyle {
    pub color: Option<ColorId>,
    pub bg: Option<ColorId>,
    pub font_style: FontStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemedToken {
    pub start: usize,
    pub end: usize,
    pub style: TokenStyle,
    pub scopes: ScopeListId,
}

impl ThemedToken {
    pub fn text<'l>(&self, line: &'l str) -> &'l str {
        &line[self.start..self.end]
    }
}

/// Tokens for a whole buffer against one theme. The source is stored once; each line
/// is a byte range into it and each token a byte range into its line.
#[derive(Debug, Clone)]
pub struct TokensResult {
    pub source: String,
    pub lines: Vec<ThemedLine>,
    pub theme: Arc<Theme>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemedLine {
    pub range: Range<usize>,
    pub tokens: Vec<ThemedToken>,
}

impl TokensResult {
    pub fn line_text(&self, line: &ThemedLine) -> &str {
        &self.source[line.range.clone()]
    }

    pub fn fg(&self) -> &str {
        self.theme.default_foreground()
    }

    pub fn bg(&self) -> &str {
        self.theme.default_background()
    }

    pub fn color(&self, id: ColorId) -> &str {
        self.theme.color(id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticKind {
    /// The line exceeded the configured maximum length and was emitted unstyled.
    TooLong,
    /// Tokenizing the line panicked; it was emitted unstyled and the state kept.
    Panic,
    /// A pattern set for the line failed to compile; the line was emitted unstyled.
    Regex,
}

impl DiagnosticKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TooLong => "too_long",
            Self::Panic => "panic",
            Self::Regex => "regex",
        }
    }
}

/// A non-fatal per-line problem. `line` is zero-based.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Diagnostic {
    pub line: usize,
    pub kind: DiagnosticKind,
}

/// A one-based inclusive line range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineRange {
    pub start: usize,
    pub end: usize,
}

impl LineRange {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn single(line: usize) -> Self {
        Self::new(line, line)
    }

    pub fn contains(&self, line: usize) -> bool {
        line >= self.start && line <= self.end
    }
}

pub fn in_ranges(ranges: &[LineRange], line: usize) -> bool {
    ranges.iter().any(|range| range.contains(line))
}
