//! [`irosashi::Highlighter`] as a Kazari backend: TextMate grammars and VS Code
//! themes, byte-identical to `vscode-textmate`.

use std::collections::BTreeMap;

use irosashi::{CodeToTokensOptions, TokensResult};

use crate::error::Error;
use crate::highlighter::{FontStyle, Highlighted, Highlighter, Line, Style, ThemeDefaults, Token};
use crate::types::ThemeInfo;

impl From<irosashi::Error> for Error {
    fn from(e: irosashi::Error) -> Self {
        match e {
            irosashi::Error::ThemeNotFound(name) => Error::ThemeNotFound(name),
            other => Error::Highlight(other.to_string()),
        }
    }
}

impl ThemeInfo {
    pub fn from_irosashi(tc: &irosashi::ThemeColors) -> Self {
        Self {
            fg: tc.foreground.clone(),
            bg: tc.background.clone(),
            selection_bg: tc.selection_background.clone().unwrap_or_default(),
            line_number_fg: tc
                .colors
                .get("editorLineNumber.foreground")
                .cloned()
                .unwrap_or_default(),
            fold_bg: tc
                .colors
                .get("editor.foldBackground")
                .cloned()
                .unwrap_or_default(),
        }
    }
}

impl Highlighter for irosashi::Highlighter {
    fn tokenize(
        &self,
        code: &str,
        lang: &str,
        light: &str,
        dark: Option<&str>,
    ) -> Result<Highlighted, Error> {
        let opts = CodeToTokensOptions::new(lang, light);
        let (result, light_slot, dark_slot) = match dark {
            Some(dark) => {
                let mut keys = BTreeMap::new();
                keys.insert("light".to_owned(), light.to_owned());
                keys.insert("dark".to_owned(), dark.to_owned());
                let result = self.code_to_tokens_multi(code, &keys, &opts)?;
                let light_slot = find_slot(&result, "light");
                let dark_slot = find_slot(&result, "dark");
                (result, light_slot, Some(dark_slot))
            }
            None => (self.code_to_tokens(code, &opts)?, 0, None),
        };
        Ok(convert(&result, light_slot, dark_slot))
    }

    fn theme_info(&self, theme: &str) -> Result<ThemeInfo, Error> {
        Ok(ThemeInfo::from_irosashi(&self.theme_colors(theme)?))
    }
}

fn find_slot(result: &TokensResult, key: &str) -> usize {
    result.themes.iter().position(|s| s.key == key).unwrap_or(0)
}

fn convert(result: &TokensResult, light_slot: usize, dark_slot: Option<usize>) -> Highlighted {
    let style_in = |token: &irosashi::ThemedToken, slot: usize| {
        let s = result.style_in(token, slot);
        Style {
            color: s.color.map(|id| result.color_in(slot, id).to_owned()),
            bg: s.bg.map(|id| result.color_in(slot, id).to_owned()),
            font_style: FontStyle::from_bits_truncate(s.font_style.bits()),
        }
    };
    let lines = result
        .lines
        .iter()
        .map(|line| Line {
            text: result.line_text(line).to_owned(),
            tokens: line
                .tokens
                .iter()
                .map(|token| Token {
                    start: token.start,
                    end: token.end,
                    light: style_in(token, light_slot),
                    dark: dark_slot.map(|slot| style_in(token, slot)),
                })
                .collect(),
        })
        .collect();
    let defaults = |slot: usize| ThemeDefaults {
        fg: result.fg_of(slot).to_owned(),
        bg: result.bg_of(slot).to_owned(),
    };
    Highlighted {
        lines,
        light: defaults(light_slot),
        dark: dark_slot.map(defaults),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_info_from_irosashi() {
        let tc = irosashi::ThemeColors {
            kind: "dark".into(),
            foreground: "#d4d4d4".into(),
            background: "#1e1e1e".into(),
            selection_background: Some("#264f78".into()),
            line_highlight_background: None,
            colors: {
                let mut m = BTreeMap::new();
                m.insert("editorLineNumber.foreground".into(), "#858585".into());
                m
            },
        };
        let info = ThemeInfo::from_irosashi(&tc);
        assert_eq!(info.fg, "#d4d4d4");
        assert_eq!(info.bg, "#1e1e1e");
        assert_eq!(info.selection_bg, "#264f78");
        assert_eq!(info.line_number_fg, "#858585");
        assert_eq!(info.fold_bg, "");
    }

    #[test]
    fn dual_tokens_carry_both_styles() {
        let hl = irosashi::Highlighter::new().unwrap();
        let out = hl
            .tokenize("let x = 1;\n", "rust", "github-light", Some("github-dark"))
            .unwrap();
        assert!(out.dark.is_some());
        assert_ne!(out.light.bg, out.dark.as_ref().unwrap().bg);
        for line in &out.lines {
            let joined: String = line.tokens.iter().map(|t| t.text(&line.text)).collect();
            assert_eq!(joined, line.text);
            assert!(line.tokens.iter().all(|t| t.dark.is_some()));
        }
        let keyword = &out.lines[0].tokens[0];
        assert_eq!(keyword.text(&out.lines[0].text), "let");
        assert_ne!(keyword.light.color, keyword.dark.as_ref().unwrap().color);
    }

    #[test]
    fn single_theme_has_no_dark() {
        let hl = irosashi::Highlighter::new().unwrap();
        let out = hl.tokenize("x", "rust", "github-light", None).unwrap();
        assert!(out.dark.is_none());
        assert!(out.lines[0].tokens.iter().all(|t| t.dark.is_none()));
    }

    #[test]
    fn unknown_language_is_plain_text() {
        let hl = irosashi::Highlighter::new().unwrap();
        let out = hl
            .tokenize("a b\nc", "no-such-lang", "github-light", None)
            .unwrap();
        assert_eq!(out.lines.len(), 2);
        assert_eq!(out.lines[0].text, "a b");
        assert_eq!(out.lines[1].text, "c");
    }

    #[test]
    fn unknown_theme_is_an_error() {
        let hl = irosashi::Highlighter::new().unwrap();
        let err = hl.tokenize("x", "rust", "no-such-theme", None).unwrap_err();
        assert_eq!(err, Error::ThemeNotFound("no-such-theme".into()));
    }
}
