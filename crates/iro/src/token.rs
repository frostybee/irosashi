use std::collections::HashMap;
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

/// Style resolved from one theme for one scope stack. Colors are ids into that
/// theme's color table: resolve a style from theme slot `k` with slot `k`'s theme
/// (`TokensResult::color_in`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TokenStyle {
    pub color: Option<ColorId>,
    pub bg: Option<ColorId>,
    pub font_style: FontStyle,
}

/// A token styled with the default theme (slot 0 of the result).
///
/// `scopes` is opaque outside the result it came from. Two tokens with equal
/// `scopes` have identical scope stacks and therefore identical styles in every
/// theme, which renderers may use to share style output.
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

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ThemedLine {
    pub range: Range<usize>,
    pub tokens: Vec<ThemedToken>,
}

impl ThemedLine {
    pub fn new(range: Range<usize>, tokens: Vec<ThemedToken>) -> Self {
        Self { range, tokens }
    }
}

/// A theme participating in a result, under the key the caller chose for it.
#[derive(Debug, Clone)]
pub struct ThemeSlot {
    pub key: String,
    pub theme: Arc<Theme>,
}

/// Scope names for the scope stacks used in one result, filled on request.
#[derive(Debug, Clone, Default)]
pub struct ScopeTable {
    lists: HashMap<ScopeListId, Box<[Arc<str>]>>,
}

impl ScopeTable {
    pub fn new(lists: HashMap<ScopeListId, Box<[Arc<str>]>>) -> Self {
        Self { lists }
    }

    pub fn get(&self, scopes: ScopeListId) -> Option<&[Arc<str>]> {
        self.lists.get(&scopes).map(|l| &**l)
    }

    pub fn len(&self) -> usize {
        self.lists.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lists.is_empty()
    }
}

/// Tokens for a whole buffer against one or more themes. The source is stored once;
/// each line is a byte range into it and each token a byte range into its line.
///
/// `themes` is sorted by key and never empty; slot 0 is the default theme whose style
/// each token carries inline. With more than one theme, `styles` holds every slot's
/// style per distinct scope stack.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct TokensResult {
    pub source: String,
    pub lines: Vec<ThemedLine>,
    pub themes: Vec<ThemeSlot>,
    pub styles: HashMap<ScopeListId, Box<[TokenStyle]>>,
    pub scopes: Option<ScopeTable>,
    pub diagnostics: Vec<Diagnostic>,
}

impl TokensResult {
    pub fn new(
        source: String,
        lines: Vec<ThemedLine>,
        themes: Vec<ThemeSlot>,
        styles: HashMap<ScopeListId, Box<[TokenStyle]>>,
        scopes: Option<ScopeTable>,
        diagnostics: Vec<Diagnostic>,
    ) -> Self {
        assert!(!themes.is_empty(), "a result has at least one theme");
        Self {
            source,
            lines,
            themes,
            styles,
            scopes,
            diagnostics,
        }
    }

    /// The default theme (slot 0).
    pub fn theme(&self) -> &Arc<Theme> {
        &self.themes[0].theme
    }

    pub fn is_multi(&self) -> bool {
        self.themes.len() > 1
    }

    pub fn line_text(&self, line: &ThemedLine) -> &str {
        &self.source[line.range.clone()]
    }

    /// The token's style in theme slot `slot`.
    pub fn style_in(&self, token: &ThemedToken, slot: usize) -> TokenStyle {
        if slot == 0 {
            return token.style;
        }
        self.styles
            .get(&token.scopes)
            .and_then(|s| s.get(slot).copied())
            .unwrap_or_default()
    }

    pub fn color_in(&self, slot: usize, id: ColorId) -> &str {
        self.themes[slot].theme.color(id)
    }

    pub fn fg_of(&self, slot: usize) -> &str {
        self.themes[slot].theme.default_foreground()
    }

    pub fn bg_of(&self, slot: usize) -> &str {
        self.themes[slot].theme.default_background()
    }

    pub fn fg(&self) -> &str {
        self.fg_of(0)
    }

    pub fn bg(&self) -> &str {
        self.bg_of(0)
    }

    /// Resolves a color id of the default theme.
    pub fn color(&self, id: ColorId) -> &str {
        self.color_in(0, id)
    }

    /// Scope names of a token, when the result was built with scopes included.
    pub fn scopes_of(&self, token: &ThemedToken) -> Option<&[Arc<str>]> {
        self.scopes.as_ref()?.get(token.scopes)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DiagnosticKind {
    /// The line exceeded the configured maximum length and was emitted unstyled.
    TooLong,
    /// Tokenizing the line panicked; it was emitted unstyled and the state kept.
    Panic,
    /// A pattern set for the line failed to compile; the line was emitted unstyled.
    Regex,
    /// The language is unknown; the whole buffer was emitted as plain text.
    UnknownLanguage,
}

impl DiagnosticKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TooLong => "too_long",
            Self::Panic => "panic",
            Self::Regex => "regex",
            Self::UnknownLanguage => "unknown_lang",
        }
    }
}

/// A non-fatal problem. `line` is zero-based; `UnknownLanguage` reports line 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Diagnostic {
    pub line: usize,
    pub kind: DiagnosticKind,
}

/// A one-based inclusive line range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub struct LineRange {
    pub start: usize,
    pub end: usize,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
pub fn in_ranges(ranges: &[LineRange], line: usize) -> bool {
    ranges.iter().any(|range| range.contains(line))
}
