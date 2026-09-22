//! [`syntect`] as a Kazari backend: Sublime Text grammars and `.tmTheme` themes,
//! pure Rust. Faster to build and to start than Irosashi, less faithful to VS Code.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};

use syntect::highlighting::{
    Color, HighlightIterator, HighlightState, Highlighter as ThemeHighlighter, Theme, ThemeSet,
};
use syntect::parsing::{ParseState, ScopeStack, SyntaxReference, SyntaxSet};

use crate::error::Error;
use crate::highlighter::{FontStyle, Highlighted, Highlighter, Line, Style, ThemeDefaults, Token};
use crate::types::ThemeInfo;

/// VS Code theme names mapped to the closest of syntect's bundled themes. Names
/// syntect knows resolve directly; anything else falls back on `light` in the name.
pub const DEFAULT_THEME_MAP: &[(&str, &str)] = &[
    ("github-light", "InspiredGitHub"),
    ("github-light-default", "InspiredGitHub"),
    ("github-light-high-contrast", "InspiredGitHub"),
    ("light-plus", "InspiredGitHub"),
    ("one-light", "InspiredGitHub"),
    ("min-light", "InspiredGitHub"),
    ("solarized-light", "Solarized (light)"),
    ("solarized-dark", "Solarized (dark)"),
    ("github-dark", "base16-ocean.dark"),
    ("github-dark-default", "base16-ocean.dark"),
    ("github-dark-dimmed", "base16-ocean.dark"),
    ("github-dark-high-contrast", "base16-ocean.dark"),
    ("dark-plus", "base16-ocean.dark"),
    ("one-dark-pro", "base16-ocean.dark"),
    ("dracula", "base16-ocean.dark"),
    ("nord", "base16-ocean.dark"),
    ("monokai", "base16-mocha.dark"),
    ("material-theme", "base16-mocha.dark"),
    ("everforest-dark", "base16-eighties.dark"),
    ("gruvbox-dark-medium", "base16-eighties.dark"),
];

const FALLBACK_LIGHT: &str = "InspiredGitHub";
const FALLBACK_DARK: &str = "base16-ocean.dark";

pub struct SyntectHighlighter {
    syntaxes: SyntaxSet,
    themes: HashMap<String, Arc<Theme>>,
    theme_map: HashMap<String, String>,
    loaded: RwLock<HashMap<String, Arc<Theme>>>,
}

impl Default for SyntectHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl SyntectHighlighter {
    /// syntect's bundled syntaxes and themes with [`DEFAULT_THEME_MAP`].
    pub fn new() -> Self {
        Self::with_theme_map(
            DEFAULT_THEME_MAP
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect(),
        )
    }

    /// Like [`new`](Self::new) with a custom map from the theme names Kazari is
    /// configured with to syntect theme names.
    pub fn with_theme_map(theme_map: HashMap<String, String>) -> Self {
        let themes = ThemeSet::load_defaults()
            .themes
            .into_iter()
            .map(|(name, theme)| (name, Arc::new(theme)))
            .collect();
        Self {
            syntaxes: SyntaxSet::load_defaults_newlines(),
            themes,
            theme_map,
            loaded: RwLock::new(HashMap::new()),
        }
    }

    /// Registers a `.tmTheme` file under `name`.
    pub fn add_theme_file(&mut self, name: &str, path: &Path) -> Result<(), Error> {
        let theme = ThemeSet::get_theme(path).map_err(|e| Error::Highlight(e.to_string()))?;
        self.themes.insert(name.to_owned(), Arc::new(theme));
        Ok(())
    }

    /// The registered theme names, sorted.
    pub fn themes(&self) -> Vec<String> {
        let mut names: Vec<String> = self.themes.keys().cloned().collect();
        names.sort();
        names
    }

    /// The names of the bundled syntaxes, sorted.
    pub fn languages(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .syntaxes
            .syntaxes()
            .iter()
            .map(|s| s.name.clone())
            .collect();
        names.sort();
        names
    }

    /// The language token for a file name, from its extension, when a bundled syntax
    /// claims it.
    pub fn detect_language(&self, file_name: &str) -> Option<String> {
        let ext = Path::new(file_name).extension()?.to_str()?;
        self.syntaxes
            .find_syntax_by_extension(ext)
            .map(|_| ext.to_ascii_lowercase())
    }

    fn syntax(&self, lang: &str) -> &SyntaxReference {
        self.syntaxes
            .find_syntax_by_token(lang)
            .or_else(|| self.syntaxes.find_syntax_by_extension(lang))
            .unwrap_or_else(|| self.syntaxes.find_syntax_plain_text())
    }

    fn theme(&self, name: &str) -> Result<Arc<Theme>, Error> {
        if let Some(theme) = self.themes.get(name) {
            return Ok(theme.clone());
        }
        if let Some(mapped) = self.theme_map.get(name) {
            return self
                .themes
                .get(mapped)
                .cloned()
                .ok_or_else(|| Error::ThemeNotFound(name.to_owned()));
        }
        if name.ends_with(".tmTheme") {
            if let Some(theme) = self.loaded.read().unwrap().get(name) {
                return Ok(theme.clone());
            }
            let theme = ThemeSet::get_theme(name)
                .map(Arc::new)
                .map_err(|_| Error::ThemeNotFound(name.to_owned()))?;
            self.loaded
                .write()
                .unwrap()
                .insert(name.to_owned(), theme.clone());
            return Ok(theme);
        }
        let fallback = if name.to_ascii_lowercase().contains("light") {
            FALLBACK_LIGHT
        } else {
            FALLBACK_DARK
        };
        self.themes
            .get(fallback)
            .cloned()
            .ok_or_else(|| Error::ThemeNotFound(name.to_owned()))
    }
}

impl Highlighter for SyntectHighlighter {
    fn tokenize(
        &self,
        code: &str,
        lang: &str,
        light: &str,
        dark: Option<&str>,
    ) -> Result<Highlighted, Error> {
        let light_theme = self.theme(light)?;
        let dark_theme = dark.map(|name| self.theme(name)).transpose()?;

        let syntax = self.syntax(lang);
        let mut parse_state = ParseState::new(syntax);
        let light_hl = ThemeHighlighter::new(&light_theme);
        let mut light_state = HighlightState::new(&light_hl, ScopeStack::new());
        let dark_hl = dark_theme.as_ref().map(|t| ThemeHighlighter::new(t));
        let mut dark_state = dark_hl
            .as_ref()
            .map(|hl| HighlightState::new(hl, ScopeStack::new()));

        let mut lines = Vec::new();
        for raw in split_lines_with_endings(code) {
            let ops = parse_state
                .parse_line(raw, &self.syntaxes)
                .map_err(|e| Error::Highlight(e.to_string()))?;
            let text = raw.trim_end_matches('\n').trim_end_matches('\r');
            let light_styles = styles_of(&mut light_state, &ops, raw, &light_hl, &light_theme);
            let dark_styles = match (&mut dark_state, &dark_hl, &dark_theme) {
                (Some(state), Some(hl), Some(theme)) => {
                    Some(styles_of(state, &ops, raw, hl, theme))
                }
                _ => None,
            };
            debug_assert!(
                dark_styles
                    .as_ref()
                    .is_none_or(|d| d.len() == light_styles.len())
            );

            let mut tokens = Vec::with_capacity(light_styles.len());
            let mut start = 0;
            for (i, (len, light_style)) in light_styles.into_iter().enumerate() {
                let end = (start + len).min(text.len());
                if end > start {
                    tokens.push(Token {
                        start,
                        end,
                        light: light_style,
                        dark: dark_styles
                            .as_ref()
                            .and_then(|d| d.get(i))
                            .map(|(_, s)| s.clone()),
                    });
                }
                start = end;
            }
            if tokens.is_empty() {
                tokens.push(Token {
                    start: 0,
                    end: text.len(),
                    light: Style::default(),
                    dark: dark.map(|_| Style::default()),
                });
            }
            lines.push(Line {
                text: text.to_owned(),
                tokens,
            });
        }

        Ok(Highlighted {
            lines,
            light: defaults_of(&light_theme),
            dark: dark_theme.as_ref().map(|t| defaults_of(t)),
        })
    }

    fn theme_info(&self, theme: &str) -> Result<ThemeInfo, Error> {
        let theme = self.theme(theme)?;
        let s = &theme.settings;
        let hex = |c: Option<Color>| c.map(color_hex).unwrap_or_default();
        Ok(ThemeInfo {
            fg: hex(s.foreground),
            bg: hex(s.background),
            selection_bg: hex(s.selection),
            line_number_fg: hex(s.gutter_foreground),
            fold_bg: hex(s.line_highlight),
        })
    }
}

/// The lines of `code` with their endings kept, so the grammar sees the
/// newline; the same lines Irosashi's splitter yields (no final empty line after
/// a trailing newline, none at all for empty input).
fn split_lines_with_endings(code: &str) -> impl Iterator<Item = &str> {
    let mut rest = code;
    std::iter::from_fn(move || {
        if rest.is_empty() {
            return None;
        }
        let end = rest.find('\n').map(|i| i + 1).unwrap_or(rest.len());
        let (line, tail) = rest.split_at(end);
        rest = tail;
        Some(line)
    })
}

/// Byte length and style of each piece of `raw` under one theme.
fn styles_of(
    state: &mut HighlightState,
    ops: &[(usize, syntect::parsing::ScopeStackOp)],
    raw: &str,
    hl: &ThemeHighlighter<'_>,
    theme: &Theme,
) -> Vec<(usize, Style)> {
    let theme_bg = theme.settings.background;
    HighlightIterator::new(state, ops, raw, hl)
        .map(|(style, piece)| (piece.len(), convert_style(style, theme_bg)))
        .collect()
}

fn convert_style(style: syntect::highlighting::Style, theme_bg: Option<Color>) -> Style {
    use syntect::highlighting::FontStyle as Sf;
    let mut fs = FontStyle::empty();
    if style.font_style.contains(Sf::ITALIC) {
        fs |= FontStyle::ITALIC;
    }
    if style.font_style.contains(Sf::BOLD) {
        fs |= FontStyle::BOLD;
    }
    if style.font_style.contains(Sf::UNDERLINE) {
        fs |= FontStyle::UNDERLINE;
    }
    let bg = if theme_bg.is_some_and(|bg| bg == style.background) {
        None
    } else {
        Some(color_hex(style.background))
    };
    Style {
        color: Some(color_hex(style.foreground)),
        bg,
        font_style: fs,
    }
}

fn defaults_of(theme: &Theme) -> ThemeDefaults {
    ThemeDefaults {
        fg: theme.settings.foreground.map(color_hex).unwrap_or_default(),
        bg: theme.settings.background.map(color_hex).unwrap_or_default(),
    }
}

fn color_hex(c: Color) -> String {
    if c.a == 255 {
        format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
    } else {
        format!("#{:02x}{:02x}{:02x}{:02x}", c.r, c.g, c.b, c.a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hl() -> SyntectHighlighter {
        SyntectHighlighter::new()
    }

    #[test]
    fn font_style_bits_are_remapped() {
        use syntect::highlighting::FontStyle as Sf;
        let style = |fs: Sf| syntect::highlighting::Style {
            foreground: Color::BLACK,
            background: Color::WHITE,
            font_style: fs,
        };
        assert_eq!(
            convert_style(style(Sf::BOLD), None).font_style,
            FontStyle::BOLD
        );
        assert_eq!(
            convert_style(style(Sf::ITALIC), None).font_style,
            FontStyle::ITALIC
        );
        assert_eq!(
            convert_style(style(Sf::UNDERLINE), None).font_style,
            FontStyle::UNDERLINE
        );
        assert_eq!(
            convert_style(style(Sf::BOLD | Sf::ITALIC), None).font_style,
            FontStyle::BOLD | FontStyle::ITALIC
        );
    }

    #[test]
    fn hex_with_and_without_alpha() {
        assert_eq!(
            color_hex(Color {
                r: 0x12,
                g: 0xab,
                b: 0xff,
                a: 255
            }),
            "#12abff"
        );
        assert_eq!(
            color_hex(Color {
                r: 0,
                g: 0,
                b: 0,
                a: 0x80
            }),
            "#00000080"
        );
    }

    #[test]
    fn token_background_equal_to_theme_background_is_dropped() {
        let bg = Color::WHITE;
        let style = syntect::highlighting::Style {
            foreground: Color::BLACK,
            background: bg,
            font_style: Default::default(),
        };
        assert_eq!(convert_style(style, Some(bg)).bg, None);
        assert_eq!(convert_style(style, None).bg.as_deref(), Some("#ffffff"));
    }

    #[test]
    fn theme_names_resolve_directly_mapped_or_by_fallback() {
        let hl = hl();
        assert!(hl.theme("base16-ocean.dark").is_ok());
        assert_eq!(
            hl.theme_info("github-light").unwrap(),
            hl.theme_info("InspiredGitHub").unwrap()
        );
        assert_eq!(
            hl.theme_info("some-light-theme").unwrap(),
            hl.theme_info(FALLBACK_LIGHT).unwrap()
        );
        assert_eq!(
            hl.theme_info("catppuccin-mocha").unwrap(),
            hl.theme_info(FALLBACK_DARK).unwrap()
        );
        assert_eq!(
            hl.theme("missing.tmTheme").unwrap_err(),
            Error::ThemeNotFound("missing.tmTheme".into())
        );
    }

    #[test]
    fn detects_language_from_the_extension() {
        let hl = hl();
        assert_eq!(hl.detect_language("main.rs").as_deref(), Some("rs"));
        assert_eq!(hl.detect_language("src/app.PY").as_deref(), Some("py"));
        assert_eq!(hl.detect_language("x.unknownext"), None);
        assert_eq!(hl.detect_language("Makefile.d/noext"), None);
    }

    #[test]
    fn theme_info_has_editor_colours() {
        let info = hl().theme_info("base16-ocean.dark").unwrap();
        assert!(info.fg.starts_with('#'));
        assert!(info.bg.starts_with('#'));
        assert!(info.selection_bg.starts_with('#'));
        assert_eq!(
            info.line_number_fg, "",
            "base16 themes set no gutter colour"
        );
    }

    #[test]
    fn tokens_cover_each_line_without_the_newline() {
        let out = hl()
            .tokenize(
                "fn main() {\r\n    let x = 1;\n}\n",
                "rust",
                "github-light",
                None,
            )
            .unwrap();
        assert_eq!(out.lines.len(), 3);
        assert_eq!(out.lines[0].text, "fn main() {");
        assert_eq!(out.lines[1].text, "    let x = 1;");
        for line in &out.lines {
            let joined: String = line.tokens.iter().map(|t| t.text(&line.text)).collect();
            assert_eq!(joined, line.text);
            assert!(line.tokens.iter().all(|t| t.end > t.start));
        }
        assert!(
            out.lines[0].tokens.len() > 1,
            "keywords get their own tokens"
        );
        assert!(out.dark.is_none());
    }

    #[test]
    fn dual_themes_style_every_token() {
        let out = hl()
            .tokenize("let x = 1;", "rust", "github-light", Some("github-dark"))
            .unwrap();
        assert!(out.dark.is_some());
        assert_ne!(out.light.bg, out.dark.as_ref().unwrap().bg);
        let line = &out.lines[0];
        assert!(line.tokens.iter().all(|t| t.dark.is_some()));
        let kw = &line.tokens[0];
        assert_eq!(kw.text(&line.text), "let");
        assert_ne!(kw.light.color, kw.dark.as_ref().unwrap().color);
    }

    #[test]
    fn unknown_language_keeps_the_text() {
        let out = hl()
            .tokenize("a b\n\nc", "no-such-lang", "github-light", None)
            .unwrap();
        let texts: Vec<&str> = out.lines.iter().map(|l| l.text.as_str()).collect();
        assert_eq!(texts, ["a b", "", "c"]);
        assert_eq!(out.lines[1].tokens.len(), 1);
    }

    #[test]
    fn empty_input_and_trailing_newline() {
        let hl = hl();
        assert!(
            hl.tokenize("", "rust", "github-light", None)
                .unwrap()
                .lines
                .is_empty()
        );
        assert_eq!(
            hl.tokenize("x\n", "rust", "github-light", None)
                .unwrap()
                .lines
                .len(),
            1
        );
    }

    #[test]
    fn unknown_theme_is_an_error_when_nothing_matches() {
        let mut hl = SyntectHighlighter::with_theme_map(HashMap::new());
        hl.themes.clear();
        assert_eq!(
            hl.tokenize("x", "rust", "github-light", None).unwrap_err(),
            Error::ThemeNotFound("github-light".into())
        );
    }
}
