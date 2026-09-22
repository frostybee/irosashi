use crate::error::Error;
use crate::types::ThemeInfo;

bitflags::bitflags! {
    /// Font style of a token; the same bit assignment as VS Code themes.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct FontStyle: u8 {
        const ITALIC = 1;
        const BOLD = 2;
        const UNDERLINE = 4;
        const STRIKETHROUGH = 8;
    }
}

/// The colours of a token under one theme. Colours are CSS hex strings
/// (`#rrggbb` or `#rrggbbaa`); `None` means the theme default.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Style {
    pub color: Option<String>,
    pub bg: Option<String>,
    pub font_style: FontStyle,
}

/// A run of one line's text with its style under the light theme and, when a
/// dark theme was requested, under the dark theme. `start` and `end` are byte
/// offsets into [`Line::text`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub start: usize,
    pub end: usize,
    pub light: Style,
    pub dark: Option<Style>,
}

impl Token {
    pub fn text<'l>(&self, line: &'l str) -> &'l str {
        &line[self.start..self.end]
    }
}

/// One line of highlighted code without its line ending.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Line {
    pub text: String,
    pub tokens: Vec<Token>,
}

/// Default foreground and background of a theme.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ThemeDefaults {
    pub fg: String,
    pub bg: String,
}

/// The result of tokenizing one block.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Highlighted {
    pub lines: Vec<Line>,
    pub light: ThemeDefaults,
    pub dark: Option<ThemeDefaults>,
}

/// A syntax highlighting backend.
///
/// Kazari asks a backend for styled tokens and theme colours and does everything
/// else itself (fence meta, markers, line numbers, frames, HTML and Typst output).
/// The `irosashi` and `syntect` Cargo features each ship an implementation.
///
/// Contract for implementations:
///
/// - The tokens of a line are contiguous, in order, and cover [`Line::text`]
///   exactly. Empty tokens are allowed.
/// - `Highlighted::dark` is `Some` when a dark theme was requested and `None`
///   otherwise; in the first case every token's [`Token::dark`] is `Some` too.
/// - A theme that does not exist is [`Error::ThemeNotFound`].
/// - A language that does not exist is not an error: the text comes back as
///   unstyled tokens.
/// - `ansi` is a pseudo-language whose SGR escape sequences a backend may turn
///   into styles (and remove from the text) or leave as plain text.
pub trait Highlighter: Send + Sync {
    /// Tokenizes `code` as `lang` under the `light` theme and optionally the
    /// `dark` theme.
    fn tokenize(
        &self,
        code: &str,
        lang: &str,
        light: &str,
        dark: Option<&str>,
    ) -> Result<Highlighted, Error>;

    /// The editor colours of a theme, used for the block's CSS variables.
    fn theme_info(&self, theme: &str) -> Result<ThemeInfo, Error>;
}

impl<T: Highlighter + ?Sized> Highlighter for Box<T> {
    fn tokenize(
        &self,
        code: &str,
        lang: &str,
        light: &str,
        dark: Option<&str>,
    ) -> Result<Highlighted, Error> {
        (**self).tokenize(code, lang, light, dark)
    }

    fn theme_info(&self, theme: &str) -> Result<ThemeInfo, Error> {
        (**self).theme_info(theme)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_style_bits_match_vscode() {
        assert_eq!(FontStyle::ITALIC.bits(), 1);
        assert_eq!(FontStyle::BOLD.bits(), 2);
        assert_eq!(FontStyle::UNDERLINE.bits(), 4);
        assert_eq!(FontStyle::STRIKETHROUGH.bits(), 8);
    }

    #[test]
    fn token_text_slices_the_line() {
        let token = Token {
            start: 4,
            end: 7,
            light: Style::default(),
            dark: None,
        };
        assert_eq!(token.text("let x = 1;"), "x =");
    }
}
