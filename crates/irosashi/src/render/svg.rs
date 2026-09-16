use std::fmt::Write;

use crate::render::Renderer;
use crate::token::TokensResult;

const DEFAULT_FONT_FAMILY: &str =
    "Consolas, Monaco, Lucida Console, Liberation Mono, DejaVu Sans Mono, monospace";
const CHAR_WIDTH_RATIO: f64 = 0.6;
const NBSP: &str = "&#160;";

/// Layout of the SVG. Zero or empty fields fall back to the defaults, as in Nuri.
#[derive(Debug, Clone, PartialEq)]
pub struct SvgOptions {
    pub font_family: String,
    pub font_size: f64,
    /// Multiplier of `font_size`.
    pub line_height: f64,
    pub pad_x: f64,
    pub pad_y: f64,
    pub tab_width: usize,
    pub corner_radius: f64,
    /// `None` and `Some(true)` draw the background rectangle.
    pub show_background: Option<bool>,
}

impl Default for SvgOptions {
    fn default() -> Self {
        Self {
            font_family: DEFAULT_FONT_FAMILY.to_owned(),
            font_size: 14.0,
            line_height: 1.2,
            pad_x: 16.0,
            pad_y: 16.0,
            tab_width: 4,
            corner_radius: 8.0,
            show_background: None,
        }
    }
}

impl SvgOptions {
    fn normalized(&self) -> Self {
        let defaults = Self::default();
        let or = |value: f64, default: f64| if value == 0.0 { default } else { value };
        Self {
            font_family: if self.font_family.is_empty() {
                defaults.font_family
            } else {
                self.font_family.clone()
            },
            font_size: or(self.font_size, defaults.font_size),
            line_height: or(self.line_height, defaults.line_height),
            pad_x: or(self.pad_x, defaults.pad_x),
            pad_y: or(self.pad_y, defaults.pad_y),
            tab_width: if self.tab_width == 0 {
                defaults.tab_width
            } else {
                self.tab_width
            },
            corner_radius: or(self.corner_radius, defaults.corner_radius),
            show_background: self.show_background,
        }
    }
}

fn escape_text(out: &mut String, text: &str, tab_width: usize) {
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            ' ' => out.push_str(NBSP),
            '\t' => {
                for _ in 0..tab_width {
                    out.push_str(NBSP);
                }
            }
            c => out.push(c),
        }
    }
}

fn escape_attr(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            c => out.push(c),
        }
    }
    out
}

/// A standalone SVG: a rounded background rectangle, one `text` element per line and a
/// `tspan` per styled token, sized from a 0.6 em monospace character width.
#[derive(Debug, Default)]
pub struct SvgRenderer;

impl Renderer for SvgRenderer {
    type Options = SvgOptions;
    type Output = String;

    fn render(&mut self, result: &TokensResult, options: &SvgOptions) -> String {
        let o = options.normalized();
        let char_width = o.font_size * CHAR_WIDTH_RATIO;
        let line_height_px = o.font_size * o.line_height;
        let max_cols = result
            .lines
            .iter()
            .map(|line| {
                result
                    .line_text(line)
                    .chars()
                    .map(|c| if c == '\t' { o.tab_width } else { 1 })
                    .sum::<usize>()
            })
            .max()
            .unwrap_or(0);
        let width = o.pad_x * 2.0 + max_cols as f64 * char_width;
        let height = o.pad_y * 2.0 + result.lines.len() as f64 * line_height_px;
        let default_fg = result.fg();
        let fg = if default_fg.is_empty() {
            "#000000"
        } else {
            default_fg
        };

        let mut out = String::new();
        let _ = write!(
            out,
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width:.0}px" height="{height:.0}px" viewBox="0 0 {width:.0} {height:.0}" font-family="{}" font-size="{:.0}px">"#,
            escape_attr(&o.font_family),
            o.font_size
        );
        if o.show_background.unwrap_or(true) && !result.bg().is_empty() {
            let _ = write!(
                out,
                r#"<rect width="100%" height="100%" fill="{}" rx="{:.0}"/>"#,
                escape_attr(result.bg()),
                o.corner_radius
            );
        }
        let _ = write!(out, r#"<g fill="{}">"#, escape_attr(fg));
        for (index, line) in result.lines.iter().enumerate() {
            let y = o.pad_y + o.font_size + index as f64 * line_height_px;
            let _ = write!(
                out,
                r#"<text x="{:.0}" y="{y:.1}" xml:space="preserve">"#,
                o.pad_x
            );
            let text = result.line_text(line);
            for token in &line.tokens {
                let mut attrs: Vec<String> = Vec::new();
                if let Some(id) = token.style.color {
                    let color = result.color(id);
                    if !color.is_empty() && color != default_fg {
                        attrs.push(format!(r#"fill="{}""#, escape_attr(color)));
                    }
                }
                let fs = token.style.font_style;
                if fs.is_bold() {
                    attrs.push(r#"font-weight="bold""#.to_owned());
                }
                if fs.is_italic() {
                    attrs.push(r#"font-style="italic""#.to_owned());
                }
                let mut decorations = Vec::new();
                if fs.is_underline() {
                    decorations.push("underline");
                }
                if fs.is_strikethrough() {
                    decorations.push("line-through");
                }
                if !decorations.is_empty() {
                    attrs.push(format!(r#"text-decoration="{}""#, decorations.join(" ")));
                }
                if attrs.is_empty() {
                    escape_text(&mut out, token.text(text), o.tab_width);
                } else {
                    let _ = write!(out, "<tspan {}>", attrs.join(" "));
                    escape_text(&mut out, token.text(text), o.tab_width);
                    out.push_str("</tspan>");
                }
            }
            out.push_str("</text>");
        }
        out.push_str("</g></svg>");
        out
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::{CodeToSvgOptions, CodeToTokensOptions, Highlighter, HighlighterBuilder};

    fn highlighter() -> Highlighter {
        HighlighterBuilder::from_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("assets"))
            .build()
            .unwrap()
    }

    #[test]
    fn exact_document_for_plain_text() {
        let h = highlighter();
        let tokens = h
            .code_to_tokens("ab\ncd", &CodeToTokensOptions::new("text", "github-dark"))
            .unwrap();
        let (fg, bg) = (tokens.fg().to_owned(), tokens.bg().to_owned());
        let svg = h
            .code_to_svg("ab\ncd", &CodeToSvgOptions::new("text", "github-dark"))
            .unwrap();
        assert_eq!(
            svg,
            format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="49px" height="66px" viewBox="0 0 49 66" font-family="{DEFAULT_FONT_FAMILY}" font-size="14px"><rect width="100%" height="100%" fill="{bg}" rx="8"/><g fill="{fg}"><text x="16" y="30.0" xml:space="preserve">ab</text><text x="16" y="46.8" xml:space="preserve">cd</text></g></svg>"#
            )
        );
    }

    #[test]
    fn escaping_tabs_background_and_geometry_options() {
        let h = highlighter();
        let mut options = CodeToSvgOptions::new("text", "github-dark");
        options.svg.show_background = Some(false);
        options.svg.tab_width = 2;
        options.svg.pad_x = 0.0;
        options.svg.font_size = 10.0;
        options.svg.line_height = 2.0;
        options.svg.font_family = "A \"B\" & <C>".to_owned();
        let svg = h.code_to_svg("<a & \"b\">\tc", &options).unwrap();
        assert!(!svg.contains("<rect"));
        assert!(svg.contains(r#"font-family="A &quot;B&quot; &amp; &lt;C&gt;""#));
        assert!(svg.contains(
            r#"xml:space="preserve">&lt;a&#160;&amp;&#160;&quot;b&quot;&gt;&#160;&#160;c</text>"#
        ));
        // pad_x 0 falls back to 16; 12 columns at 6px plus padding; one line at 20px.
        assert!(svg.starts_with(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="104px" height="52px" viewBox="0 0 104 52""#
        ));
        assert!(svg.contains(r#"<text x="16" y="26.0""#));
    }

    #[test]
    fn styled_tokens_get_a_tspan() {
        let theme = br##"{"name":"t","type":"dark","colors":{"editor.foreground":"#ffffff","editor.background":"#000000"},"tokenColors":[{"scope":"source","settings":{"foreground":"#ff0000","fontStyle":"bold italic underline strikethrough"}}]}"##;
        let h = HighlighterBuilder::from_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("assets"))
            .theme("t", &theme[..])
            .unwrap()
            .build()
            .unwrap();
        let svg = h
            .code_to_svg("x", &CodeToSvgOptions::new("rust", "t"))
            .unwrap();
        assert!(svg.contains(
            r##"<tspan fill="#ff0000" font-weight="bold" font-style="italic" text-decoration="underline line-through">x</tspan>"##
        ));
        let plain = h
            .code_to_svg("x", &CodeToSvgOptions::new("text", "t"))
            .unwrap();
        assert!(!plain.contains("tspan"));
    }
}
