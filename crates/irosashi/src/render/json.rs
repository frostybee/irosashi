use std::collections::BTreeMap;

use serde::Serialize;

use crate::render::Renderer;
use crate::token::{ThemedToken, TokensResult};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct JsonOptions {
    /// Pretty-print with two-space indentation.
    pub indent: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonResult<'a> {
    tokens: Vec<Vec<JsonToken<'a>>>,
    fg: &'a str,
    bg: &'a str,
    theme_name: &'a str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    diagnostics: Vec<JsonDiagnostic>,
    #[serde(rename = "themeFG", skip_serializing_if = "BTreeMap::is_empty")]
    theme_fg: BTreeMap<&'a str, &'a str>,
    #[serde(rename = "themeBG", skip_serializing_if = "BTreeMap::is_empty")]
    theme_bg: BTreeMap<&'a str, &'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    theme_names: Vec<&'a str>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonToken<'a> {
    content: &'a str,
    #[serde(skip_serializing_if = "str::is_empty")]
    color: &'a str,
    #[serde(skip_serializing_if = "str::is_empty")]
    bg_color: &'a str,
    font_style: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    scopes: Option<Vec<&'a str>>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    theme_styles: BTreeMap<&'a str, JsonStyle<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JsonStyle<'a> {
    #[serde(skip_serializing_if = "str::is_empty")]
    color: &'a str,
    #[serde(skip_serializing_if = "str::is_empty")]
    bg_color: &'a str,
    font_style: u8,
}

#[derive(Serialize)]
struct JsonDiagnostic {
    line: usize,
    kind: &'static str,
}

fn style_view<'a>(result: &'a TokensResult, token: &ThemedToken, slot: usize) -> JsonStyle<'a> {
    let style = result.style_in(token, slot);
    JsonStyle {
        color: style.color.map_or("", |id| result.color_in(slot, id)),
        bg_color: style.bg.map_or("", |id| result.color_in(slot, id)),
        font_style: style.font_style.bits(),
    }
}

fn view(result: &TokensResult) -> JsonResult<'_> {
    let multi = result.is_multi();
    let tokens = result
        .lines
        .iter()
        .map(|line| {
            let text = result.line_text(line);
            line.tokens
                .iter()
                .map(|token| {
                    let style = style_view(result, token, 0);
                    JsonToken {
                        content: token.text(text),
                        color: style.color,
                        bg_color: style.bg_color,
                        font_style: style.font_style,
                        scopes: result
                            .scopes_of(token)
                            .map(|names| names.iter().map(|n| &**n).collect()),
                        theme_styles: if multi {
                            result
                                .themes
                                .iter()
                                .enumerate()
                                .map(|(slot, s)| (s.key.as_str(), style_view(result, token, slot)))
                                .collect()
                        } else {
                            BTreeMap::new()
                        },
                    }
                })
                .collect()
        })
        .collect();
    let mut theme_fg = BTreeMap::new();
    let mut theme_bg = BTreeMap::new();
    let mut theme_names = Vec::new();
    if multi {
        for (slot, s) in result.themes.iter().enumerate() {
            theme_fg.insert(s.key.as_str(), result.fg_of(slot));
            theme_bg.insert(s.key.as_str(), result.bg_of(slot));
            theme_names.push(s.key.as_str());
        }
    }
    JsonResult {
        tokens,
        fg: result.fg(),
        bg: result.bg(),
        theme_name: &result.theme().name,
        diagnostics: result
            .diagnostics
            .iter()
            .map(|d| JsonDiagnostic {
                line: d.line,
                kind: d.kind.as_str(),
            })
            .collect(),
        theme_fg,
        theme_bg,
        theme_names,
    }
}

/// The result as JSON with the field names Nuri's `CodeToJSON` uses: `tokens` (lines of
/// `content`, `color`, `bgColor`, `fontStyle`, optional `scopes` and `themeStyles`),
/// `fg`, `bg`, `themeName`, optional `diagnostics`, and `themeFG`, `themeBG`,
/// `themeNames` for multi-theme results.
#[derive(Debug, Default)]
pub struct JsonRenderer;

impl Renderer for JsonRenderer {
    type Options = JsonOptions;
    type Output = String;

    fn render(&mut self, result: &TokensResult, options: &JsonOptions) -> String {
        let view = view(result);
        let serialized = if options.indent {
            serde_json::to_string_pretty(&view)
        } else {
            serde_json::to_string(&view)
        };
        serialized.unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::Path;

    use serde_json::Value;

    use crate::{CodeToJsonOptions, Highlighter, HighlighterBuilder};

    fn highlighter() -> Highlighter {
        HighlighterBuilder::from_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("assets"))
            .build()
            .unwrap()
    }

    #[test]
    fn single_theme_shape() {
        let h = highlighter();
        let json = h
            .code_to_json("a\n", &CodeToJsonOptions::new("text", "github-dark"))
            .unwrap();
        let v: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["themeName"], "github-dark");
        assert_eq!(v["fg"], "#e1e4e8");
        assert!(v["bg"].as_str().unwrap().starts_with('#'));
        assert!(v.get("diagnostics").is_none());
        assert!(v.get("themeNames").is_none());
        let lines = v["tokens"].as_array().unwrap();
        assert_eq!(lines.len(), 1);
        let token = &lines[0][0];
        assert_eq!(token["content"], "a");
        assert_eq!(token["color"], "#e1e4e8");
        assert_eq!(token["fontStyle"], 0);
        assert!(token.get("bgColor").is_none());
        assert!(token.get("scopes").is_none());
        assert!(token.get("themeStyles").is_none());
        assert!(!json.contains('\n'));
    }

    #[test]
    fn indent_scopes_and_diagnostics() {
        let h = highlighter();
        let mut options = CodeToJsonOptions::new("nope", "github-dark");
        options.indent = true;
        options.tokens.include_scopes = true;
        let json = h.code_to_json("x", &options).unwrap();
        assert!(json.contains("\n  "));
        let v: Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["diagnostics"][0]["kind"], "unknown_lang");
        assert_eq!(v["diagnostics"][0]["line"], 0);
        assert_eq!(v["tokens"][0][0]["scopes"], Value::Array(Vec::new()));

        let mut rust = CodeToJsonOptions::new("rust", "github-dark");
        rust.tokens.include_scopes = true;
        let v: Value = serde_json::from_str(&h.code_to_json("fn x() {}", &rust).unwrap()).unwrap();
        assert_eq!(v["tokens"][0][0]["scopes"][0], "source.rust");
    }

    #[test]
    fn multi_theme_shape() {
        let h = highlighter();
        let mut options = CodeToJsonOptions::new("text", "");
        options.themes = BTreeMap::from([
            ("light".to_owned(), "github-light".to_owned()),
            ("dark".to_owned(), "github-dark".to_owned()),
        ]);
        let v: Value = serde_json::from_str(&h.code_to_json("a", &options).unwrap()).unwrap();
        assert_eq!(v["themeNames"], serde_json::json!(["dark", "light"]));
        assert_eq!(v["themeName"], "github-dark");
        assert_eq!(v["themeFG"]["dark"], "#e1e4e8");
        assert!(v["themeBG"]["light"].as_str().unwrap().starts_with('#'));
        let token = &v["tokens"][0][0];
        assert_eq!(token["themeStyles"]["dark"]["color"], "#e1e4e8");
        assert_eq!(token["themeStyles"]["light"]["fontStyle"], 0);
        assert_ne!(token["themeStyles"]["light"]["color"], "#e1e4e8");
    }
}
