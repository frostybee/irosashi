use std::fmt::Write;

use crate::render::Renderer;
use crate::render::ansi_palette::{ColorDepth, Palette};
use crate::theme::FontStyle;
use crate::token::TokensResult;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AnsiOptions {
    pub color_depth: ColorDepth,
}

const RESET: &str = "\x1b[0m";

/// Terminal output: every token is wrapped in its own SGR sequence and reset, lines
/// are joined by `\n`. Backgrounds are not emitted (as in Nuri).
#[derive(Debug, Default)]
pub struct AnsiRenderer;

fn build_escape(color: &str, font_style: FontStyle, palette: &mut Palette) -> String {
    let mut parts: Vec<String> = Vec::new();
    if font_style.is_bold() {
        parts.push("1".to_owned());
    }
    if font_style.is_italic() {
        parts.push("3".to_owned());
    }
    if font_style.is_underline() {
        parts.push("4".to_owned());
    }
    if font_style.is_strikethrough() {
        parts.push("9".to_owned());
    }
    let color = palette.resolve(color, false);
    if !color.is_empty() {
        parts.push(color);
    }
    parts.join(";")
}

fn write_token(out: &mut String, text: &str, escape: &str) {
    if escape.is_empty() {
        out.push_str(text);
        return;
    }
    let parts: Vec<&str> = text.split('\n').collect();
    for (index, part) in parts.iter().enumerate() {
        if index > 0 {
            out.push_str(RESET);
            out.push('\n');
        }
        if part.is_empty() && index + 1 < parts.len() {
            continue;
        }
        let _ = write!(out, "\x1b[{escape}m{part}{RESET}");
    }
}

impl Renderer for AnsiRenderer {
    type Options = AnsiOptions;
    type Output = String;

    fn render(&mut self, result: &TokensResult, options: &AnsiOptions) -> String {
        let mut palette = Palette::new(options.color_depth);
        let default_fg = result.fg();
        let mut out = String::with_capacity(result.source.len() * 2);
        for (index, line) in result.lines.iter().enumerate() {
            if index > 0 {
                out.push('\n');
            }
            let text = result.line_text(line);
            for token in &line.tokens {
                let color = token.style.color.map_or(default_fg, |id| result.color(id));
                let escape = build_escape(color, token.style.font_style, &mut palette);
                write_token(&mut out, token.text(text), &escape);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::{CodeToAnsiOptions, Highlighter, HighlighterBuilder};

    fn highlighter() -> Highlighter {
        HighlighterBuilder::from_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("assets"))
            .build()
            .unwrap()
    }

    #[test]
    fn wraps_every_token_and_joins_lines() {
        let h = highlighter();
        let mut options = CodeToAnsiOptions::new("text", "github-dark");
        assert_eq!(
            h.code_to_ansi("hi\n\nyo", &options).unwrap(),
            "\x1b[38;2;225;228;232mhi\x1b[0m\n\n\x1b[38;2;225;228;232myo\x1b[0m"
        );
        options.ansi.color_depth = ColorDepth::Colors16;
        assert_eq!(h.code_to_ansi("hi", &options).unwrap(), "\x1b[37mhi\x1b[0m");
        options.ansi.color_depth = ColorDepth::Colors8;
        assert_eq!(h.code_to_ansi("hi", &options).unwrap(), "\x1b[37mhi\x1b[0m");
        options.ansi.color_depth = ColorDepth::Colors256;
        assert_eq!(
            h.code_to_ansi("hi", &options).unwrap(),
            "\x1b[38;5;254mhi\x1b[0m"
        );
        assert_eq!(h.code_to_ansi("", &options).unwrap(), "");
    }

    #[test]
    fn font_styles_precede_the_color_in_fixed_order() {
        let theme = br##"{"name":"t","type":"dark","colors":{"editor.foreground":"#ffffff","editor.background":"#000000"},"tokenColors":[{"scope":"source","settings":{"foreground":"#ff0000","fontStyle":"bold italic underline strikethrough"}}]}"##;
        let h = HighlighterBuilder::from_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("assets"))
            .theme("t", &theme[..])
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(
            h.code_to_ansi("x", &CodeToAnsiOptions::new("rust", "t"))
                .unwrap(),
            "\x1b[1;3;4;9;38;2;255;0;0mx\x1b[0m"
        );
    }

    #[test]
    fn token_text_with_newlines_is_reset_per_part() {
        let mut out = String::new();
        write_token(&mut out, "a\n\nb", "31");
        assert_eq!(out, "\x1b[31ma\x1b[0m\x1b[0m\n\x1b[0m\n\x1b[31mb\x1b[0m");
        let mut plain = String::new();
        write_token(&mut plain, "a\nb", "");
        assert_eq!(plain, "a\nb");
    }
}
