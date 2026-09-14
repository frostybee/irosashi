use std::collections::{BTreeMap, HashMap};

use serde::Deserialize;

use crate::Error;
use crate::theme::matcher::CompiledSelector;
use crate::theme::normalize::default_color;
use crate::theme::{FontStyle, Theme, TokenColor, TokenSettings};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawTheme {
    #[serde(default)]
    name: String,
    #[serde(default)]
    display_name: String,
    #[serde(default, rename = "type")]
    kind: String,
    #[serde(default)]
    colors: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    token_colors: Vec<RawTokenColor>,
}

#[derive(Deserialize)]
struct RawTokenColor {
    #[serde(default)]
    scope: Option<RawScope>,
    #[serde(default)]
    settings: RawSettings,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RawScope {
    One(String),
    Many(Vec<String>),
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct RawSettings {
    #[serde(default)]
    foreground: Option<String>,
    #[serde(default)]
    background: Option<String>,
    #[serde(default)]
    font_style: Option<String>,
}

impl Theme {
    pub fn parse(json: &[u8]) -> Result<Theme, Error> {
        let raw: RawTheme =
            serde_json::from_slice(json).map_err(|err| Error::ThemeParse(err.to_string()))?;
        let colors: BTreeMap<String, String> = raw
            .colors
            .into_iter()
            .filter_map(|(key, value)| match value {
                serde_json::Value::String(color) => Some((key, color)),
                _ => None,
            })
            .collect();

        let mut theme = Theme {
            id: Theme::next_id(),
            name: raw.name,
            display_name: raw.display_name,
            kind: raw.kind,
            colors,
            token_colors: Vec::with_capacity(raw.token_colors.len()),
            color_table: Vec::new(),
            color_ids: HashMap::new(),
            default_foreground: super::ColorId(0),
            default_background: super::ColorId(0),
        };

        for entry in raw.token_colors {
            let scopes = parse_scopes(entry.scope);
            if scopes.is_empty() {
                continue;
            }
            let settings = TokenSettings {
                foreground: non_empty(entry.settings.foreground.as_deref())
                    .map(|c| theme.intern_color(c)),
                background: non_empty(entry.settings.background.as_deref())
                    .map(|c| theme.intern_color(c)),
                font_style: parse_font_style(entry.settings.font_style.as_deref()),
            };
            let selectors = scopes.iter().map(|s| CompiledSelector::new(s)).collect();
            theme.token_colors.push(TokenColor {
                scopes,
                settings,
                selectors,
            });
        }

        let fg = default_color(theme.colors.get("editor.foreground").map(String::as_str));
        let bg = default_color(theme.colors.get("editor.background").map(String::as_str));
        theme.default_foreground = theme.intern_color(&fg);
        theme.default_background = theme.intern_color(&bg);
        Ok(theme)
    }
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.filter(|v| !v.is_empty())
}

fn parse_scopes(scope: Option<RawScope>) -> Vec<String> {
    match scope {
        None => Vec::new(),
        Some(RawScope::One(s)) => s
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect(),
        Some(RawScope::Many(list)) => list
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect(),
    }
}

/// `None` when the key is absent; `Some(empty())` for `""` or only unknown words.
pub fn parse_font_style(value: Option<&str>) -> Option<FontStyle> {
    let value = value?;
    let mut style = FontStyle::empty();
    for word in value.split_whitespace() {
        match word.to_ascii_lowercase().as_str() {
            "italic" => style |= FontStyle::ITALIC,
            "bold" => style |= FontStyle::BOLD,
            "underline" => style |= FontStyle::UNDERLINE,
            "strikethrough" => style |= FontStyle::STRIKETHROUGH,
            _ => {}
        }
    }
    Some(style)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn theme(json: &str) -> Theme {
        Theme::parse(json.as_bytes()).expect("theme parses")
    }

    #[test]
    fn font_style_words() {
        assert_eq!(parse_font_style(None), None);
        assert_eq!(parse_font_style(Some("")), Some(FontStyle::empty()));
        assert_eq!(parse_font_style(Some("normal")), Some(FontStyle::empty()));
        assert_eq!(
            parse_font_style(Some("Bold  italic")),
            Some(FontStyle::BOLD | FontStyle::ITALIC)
        );
        assert_eq!(
            parse_font_style(Some("underline strikethrough")),
            Some(FontStyle::UNDERLINE | FontStyle::STRIKETHROUGH)
        );
    }

    #[test]
    fn comma_string_splits_but_array_elements_do_not() {
        let t = theme(
            r##"{"tokenColors": [
                {"scope": "keyword, storage.type ", "settings": {"foreground": "#f00"}},
                {"scope": ["comment, string", " entity"], "settings": {"foreground": "#0f0"}}
            ]}"##,
        );
        assert_eq!(t.token_colors[0].scopes, ["keyword", "storage.type"]);
        assert_eq!(t.token_colors[1].scopes, ["comment, string", "entity"]);
    }

    #[test]
    fn scopeless_entry_is_skipped_and_defaults_come_from_colors() {
        let t = theme(
            r##"{"colors": {"editor.foreground": "#abcdef", "editor.background": 12},
                "tokenColors": [{"settings": {"foreground": "#111111"}}, {"scope": "", "settings": {}}]}"##,
        );
        assert!(t.token_colors.is_empty());
        assert_eq!(t.default_foreground(), "#abcdef");
        assert_eq!(t.default_background(), "#000000");
    }

    #[test]
    fn invalid_default_color_falls_back_to_black() {
        let t = theme(r##"{"colors": {"editor.foreground": "red"}}"##);
        assert_eq!(t.default_foreground(), "#000000");
    }

    #[test]
    fn colors_are_interned_once() {
        let t = theme(
            r##"{"tokenColors": [
                {"scope": "a", "settings": {"foreground": "#111111"}},
                {"scope": "b", "settings": {"foreground": "#111111", "background": "#222222"}}
            ]}"##,
        );
        assert_eq!(
            t.token_colors[0].settings.foreground,
            t.token_colors[1].settings.foreground
        );
        assert_eq!(
            t.color(t.token_colors[1].settings.background.unwrap()),
            "#222222"
        );
    }
}
