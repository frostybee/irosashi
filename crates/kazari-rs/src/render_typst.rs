use std::collections::{HashMap, HashSet};
use std::fmt::Write;

use irosashi::{FontStyle, TokenStyle};

use crate::config::ResolvedBlock;
use crate::escape::escape_typst_string;
use crate::marker::{self, ResolvedLine, Segment};
use crate::render::digit_count;
use crate::tokenize::Tokens;
use crate::types::{LinkAnnotation, MarkerType};

const CODE_BLOCK_TYP: &str = include_str!("../assets/typst/code-block.typ");

/// Advance width of one monospace character, used for gutter and indent widths.
const CHAR_WIDTH: &str = "0.6em";
const DIGIT_WIDTH: &str = "0.65em";
const DIM: &str = ".transparentize(60%)";

pub fn preamble() -> &'static str {
    CODE_BLOCK_TYP
}

struct LineCtx<'a> {
    resolved: &'a ResolvedBlock,
    fg: &'a str,
    markers: Option<HashMap<usize, ResolvedLine>>,
    focus_set: Option<HashSet<usize>>,
    has_focus: bool,
}

pub fn render_block(tokens: &Tokens, resolved: &ResolvedBlock) -> String {
    let mut sb = String::with_capacity(4096);
    let line_count = tokens.line_count();

    sb.push_str("#code-block(");
    if !resolved.lang.is_empty() {
        write!(sb, "lang: \"{}\", ", escape_typst_string(&resolved.lang)).unwrap();
    }
    if !resolved.title.is_empty() {
        write!(sb, "title: \"{}\", ", escape_typst_string(&resolved.title)).unwrap();
    }
    write!(
        sb,
        "fg: rgb(\"{}\"), bg: rgb(\"{}\")",
        tokens.light_fg(),
        tokens.light_bg()
    )
    .unwrap();
    if resolved.line_numbers {
        let end_num = resolved.start_line_number + line_count.saturating_sub(1);
        let digits = digit_count(resolved.start_line_number).max(digit_count(end_num));
        write!(
            sb,
            ", numbers: true, gutter-width: {} * {}",
            digits, DIGIT_WIDTH
        )
        .unwrap();
    }
    writeln!(sb, ", lines: {})[", line_count).unwrap();

    let lctx = LineCtx {
        resolved,
        fg: tokens.light_fg(),
        markers: marker::resolve_line_markers(&resolved.line_markers),
        focus_set: marker::resolve_focus_set(&resolved.focus_lines),
        has_focus: !resolved.focus_lines.is_empty(),
    };

    for i in 0..line_count {
        render_line(&mut sb, tokens, i, resolved.start_line_number + i, &lctx);
    }

    sb.push(']');
    sb
}

fn render_line(
    sb: &mut String,
    tokens: &Tokens,
    line_idx: usize,
    line_num: usize,
    lctx: &LineCtx<'_>,
) {
    let line_text = tokens.line_text(line_idx);
    let line_tokens = tokens.tokens(line_idx);

    let mut args: Vec<String> = Vec::new();
    if lctx.resolved.line_numbers {
        args.push(format!("num: {line_num}"));
    }
    if let Some(entry) = lctx.markers.as_ref().and_then(|m| m.get(&line_num))
        && entry.has_mark
    {
        args.push(format!("mark: \"{}\"", marker_name(entry.marker_type)));
        if !entry.label.is_empty() {
            args.push(format!("label: \"{}\"", escape_typst_string(&entry.label)));
        }
    }
    let blank = line_text.trim().is_empty();
    let indent = if blank {
        0
    } else {
        indent_chars(line_text, lctx.resolved)
    };
    if indent > 0 {
        args.push(format!("indent: {indent} * {CHAR_WIDTH}"));
    }

    sb.push_str("#code-line");
    if !args.is_empty() {
        write!(sb, "({})", args.join(", ")).unwrap();
    }
    sb.push('[');

    let dimmed = lctx.has_focus
        && !lctx
            .focus_set
            .as_ref()
            .is_some_and(|f| f.contains(&line_num));

    let line_links = lctx
        .resolved
        .links
        .get(line_idx)
        .map(Vec::as_slice)
        .unwrap_or(&[]);

    if blank {
        sb.push_str("#text(\" \")");
    } else if lctx.resolved.inline_markers.is_empty() && line_links.is_empty() {
        for token in line_tokens {
            let text = token.text(line_text);
            if !text.is_empty() {
                write_token(sb, text, tokens.light_style(token), tokens, lctx.fg, dimmed);
            }
        }
    } else {
        render_with_inline_markers(sb, tokens, line_text, line_tokens, line_links, lctx, dimmed);
    }

    sb.push_str("]\n");
}

fn render_with_inline_markers(
    sb: &mut String,
    tokens: &Tokens,
    line_text: &str,
    line_tokens: &[irosashi::ThemedToken],
    line_links: &[LinkAnnotation],
    lctx: &LineCtx<'_>,
    dimmed: bool,
) {
    let mut plain_text = String::new();
    let mut token_ranges: Vec<(usize, usize)> = Vec::new();
    let mut renderable: Vec<&irosashi::ThemedToken> = Vec::new();
    for token in line_tokens {
        let text = token.text(line_text);
        if text.is_empty() {
            continue;
        }
        let start = plain_text.len();
        plain_text.push_str(text);
        token_ranges.push((start, plain_text.len()));
        renderable.push(token);
    }

    let Some(annotated) = marker::process_inline_markers_and_links(
        &plain_text,
        &token_ranges,
        &lctx.resolved.inline_markers,
        line_links,
    ) else {
        for token in renderable {
            write_token(
                sb,
                token.text(line_text),
                tokens.light_style(token),
                tokens,
                lctx.fg,
                dimmed,
            );
        }
        return;
    };

    for at in &annotated {
        let token = renderable[at.token_idx];
        let style = tokens.light_style(token);
        for seg in &at.segments {
            write_segment(sb, &plain_text, seg, style, tokens, lctx.fg, dimmed);
        }
    }
}

fn write_segment(
    sb: &mut String,
    plain_text: &str,
    seg: &Segment,
    style: TokenStyle,
    tokens: &Tokens,
    fg: &str,
    dimmed: bool,
) {
    let text = &plain_text[seg.start..seg.end];
    if text.is_empty() {
        return;
    }
    match &seg.marker {
        Some(ann) => {
            if let Some(url) = &ann.link {
                write!(sb, "#link(\"{}\")[", escape_typst_string(url)).unwrap();
            }
            match ann.kind {
                Some(kind) => {
                    write!(
                        sb,
                        "#highlight(fill: kz-marker-colors.{})[",
                        marker_name(kind)
                    )
                    .unwrap();
                    write_token(sb, text, style, tokens, fg, dimmed);
                    sb.push(']');
                }
                None => write_token(sb, text, style, tokens, fg, dimmed),
            }
            if ann.link.is_some() {
                sb.push(']');
            }
        }
        None => write_token(sb, text, style, tokens, fg, dimmed),
    }
}

fn write_token(
    sb: &mut String,
    text: &str,
    style: TokenStyle,
    tokens: &Tokens,
    fg: &str,
    dimmed: bool,
) {
    let mut args: Vec<String> = Vec::new();
    let color = style
        .color
        .map(|id| tokens.light_color(id))
        .filter(|c| !c.eq_ignore_ascii_case(fg));
    match (color, dimmed) {
        (Some(c), true) => args.push(format!("fill: rgb(\"{c}\"){DIM}")),
        (Some(c), false) => args.push(format!("fill: rgb(\"{c}\")")),
        (None, true) => args.push(format!("fill: rgb(\"{fg}\"){DIM}")),
        (None, false) => {}
    }
    let fs = style.font_style;
    if fs.contains(FontStyle::BOLD) {
        args.push("weight: \"bold\"".to_owned());
    }
    if fs.contains(FontStyle::ITALIC) {
        args.push("style: \"italic\"".to_owned());
    }

    let mut wrappers = 0;
    if let Some(bg) = style.bg {
        let bg = tokens.light_color(bg);
        if !bg.is_empty() {
            write!(sb, "#highlight(fill: rgb(\"{bg}\"))[").unwrap();
            wrappers += 1;
        }
    }
    if fs.contains(FontStyle::UNDERLINE) {
        sb.push_str("#underline[");
        wrappers += 1;
    }
    if fs.contains(FontStyle::STRIKETHROUGH) {
        sb.push_str("#strike[");
        wrappers += 1;
    }

    sb.push_str("#text(");
    for arg in &args {
        sb.push_str(arg);
        sb.push_str(", ");
    }
    write!(sb, "\"{}\")", escape_typst_string(text)).unwrap();

    for _ in 0..wrappers {
        sb.push(']');
    }
}

fn indent_chars(line_text: &str, resolved: &ResolvedBlock) -> usize {
    let mut indent = resolved.hanging_indent;
    if resolved.preserve_indent {
        indent += line_text.len() - line_text.trim_start_matches([' ', '\t']).len();
    }
    indent
}

fn marker_name(mt: MarkerType) -> &'static str {
    match mt {
        MarkerType::Mark => "mark",
        MarkerType::Warning => "warning",
        MarkerType::Error => "error",
        MarkerType::Del => "del",
        MarkerType::Ins => "ins",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::tokenize;
    use crate::types::{InlineMarker, LineMarker, LineRange, Themes};

    fn highlighter() -> irosashi::Highlighter {
        irosashi::Highlighter::new().unwrap()
    }

    fn render(code: &str, lang: &str, resolved: ResolvedBlock) -> String {
        let hl = highlighter();
        let tokens = tokenize::tokenize(&hl, code, lang, &Themes::single("github-light")).unwrap();
        render_block(&tokens, &resolved)
    }

    fn resolved(lang: &str) -> ResolvedBlock {
        let mut r = Config::default().resolve(lang, None);
        r.lang = lang.to_owned();
        r
    }

    #[test]
    fn preamble_defines_functions() {
        assert!(preamble().contains("#let code-block("));
        assert!(preamble().contains("#let code-line("));
        assert!(preamble().contains("#let kz-marker-colors"));
    }

    #[test]
    fn empty_line_emits_placeholder() {
        let out = render("a\n\n   \nb", "text", resolved("text"));
        assert_eq!(out.matches("#code-line[#text(\" \")]").count(), 2);
    }

    #[test]
    fn no_whitespace_between_text_calls() {
        let out = render("let x = 1;", "javascript", resolved("javascript"));
        let line = out.lines().nth(1).unwrap();
        assert!(line.starts_with("#code-line["), "{line}");
        assert!(
            !line.contains(")#text(\" ") || !line.contains(") #text("),
            "{line}"
        );
        assert!(!line.contains("] #"), "{line}");
    }

    #[test]
    fn indentation_preserved_in_string() {
        let mut r = resolved("text");
        r.preserve_indent = false;
        let out = render("    four  spaces", "text", r);
        assert!(out.contains("#text(\"    four  spaces\")"), "{out}");
    }

    #[test]
    fn preserve_indent_and_hanging_indent_become_indent_arg() {
        let mut r = resolved("text");
        r.hanging_indent = 2;
        let out = render("    x\ny", "text", r);
        assert!(
            out.contains("#code-line(indent: 6 * 0.6em)[#text(\"    x\")]"),
            "{out}"
        );
        assert!(
            out.contains("#code-line(indent: 2 * 0.6em)[#text(\"y\")]"),
            "{out}"
        );
    }

    #[test]
    fn unfocused_line_uses_transparentize() {
        let mut r = resolved("javascript");
        r.focus_lines = vec![LineRange::single(1)];
        let out = render("let a = 1;\nlet b = 2;", "javascript", r);
        let lines: Vec<&str> = out.lines().collect();
        assert!(!lines[1].contains("transparentize"), "{}", lines[1]);
        assert!(
            lines[2].contains("rgb(\"#d73a49\").transparentize(60%)"),
            "{}",
            lines[2]
        );
        assert!(!lines[2].contains("#text(\" "), "{}", lines[2]);
    }

    #[test]
    fn font_styles_are_text_args() {
        let out = render("**bold** *it*", "markdown", resolved("markdown"));
        assert!(out.contains("weight: \"bold\", \"bold\""), "{out}");
        assert!(out.contains("style: \"italic\", \"it\""), "{out}");
    }

    #[test]
    fn gutter_width_from_digit_count() {
        let mut r = resolved("text");
        r.line_numbers = true;
        r.start_line_number = 98;
        let out = render("a\nb\nc", "text", r);
        assert!(
            out.contains("numbers: true, gutter-width: 3 * 0.65em, lines: 3)[\n"),
            "{out}"
        );
        assert!(out.contains("#code-line(num: 100)[#text(\"c\")]"), "{out}");
    }

    #[test]
    fn line_markers_and_labels() {
        let mut r = resolved("text");
        r.line_markers = vec![
            LineMarker {
                marker_type: MarkerType::Ins,
                lines: vec![LineRange::new(1, 2)],
                label: "new".into(),
            },
            LineMarker {
                marker_type: MarkerType::Error,
                lines: vec![LineRange::single(3)],
                label: String::new(),
            },
        ];
        let out = render("a\nb\nc", "text", r);
        assert!(
            out.contains("#code-line(mark: \"ins\", label: \"new\")[#text(\"a\")]"),
            "{out}"
        );
        assert!(
            out.contains("#code-line(mark: \"ins\")[#text(\"b\")]"),
            "{out}"
        );
        assert!(
            out.contains("#code-line(mark: \"error\")[#text(\"c\")]"),
            "{out}"
        );
    }

    #[test]
    fn inline_markers_wrap_segments() {
        let mut r = resolved("text");
        r.inline_markers = vec![InlineMarker {
            marker_type: MarkerType::Mark,
            text: "b c".into(),
            is_regex: false,
        }];
        let out = render("a b c d", "text", r);
        assert!(
            out.contains(
                "#text(\"a \")#highlight(fill: kz-marker-colors.mark)[#text(\"b c\")]#text(\" d\")"
            ),
            "{out}"
        );
    }

    #[test]
    fn header_escapes_title_and_lang() {
        let mut r = resolved("rust");
        r.title = "say \"hi\"\\now".into();
        let out = render("x", "rust", r);
        assert!(
            out.starts_with(
                "#code-block(lang: \"rust\", title: \"say \\\"hi\\\"\\\\now\", fg: rgb(\"#"
            ),
            "{out}"
        );
    }

    #[test]
    fn string_content_is_escaped() {
        let out = render("s = \"a\\b\"", "text", resolved("text"));
        assert!(out.contains("#text(\"s = \\\"a\\\\b\\\"\")"), "{out}");
    }
}
