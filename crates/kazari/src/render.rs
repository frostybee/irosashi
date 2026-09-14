use std::collections::{HashMap, HashSet};
use std::fmt::Write;

use iro::FontStyle;

use crate::config::{Config, ResolvedBlock};
use crate::escape::{escape_attr, escape_text};
use crate::marker::{self, ResolvedLine, Segment};
use crate::tokenize::Tokens;
use crate::types::{Frame, MarkerType, TerminalDotStyle};

const COPY_SVG: &str = r#"<svg class="kz-copy-icon" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/></svg>"#;

const WRAP_SVG: &str = r#"<svg class="kz-wrap-icon" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 6h18M3 12h15a3 3 0 110 6h-4m0 0l2-2m-2 2l2 2"/></svg>"#;

const WRAP_OFF_SVG: &str = r#"<svg class="kz-wrap-off-icon" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 6h18M3 12h18M3 18h18"/></svg>"#;

struct LineCtx<'a> {
    resolved: &'a ResolvedBlock,
    is_dual: bool,
    resolved_markers: Option<HashMap<usize, ResolvedLine>>,
    focus_set: Option<HashSet<usize>>,
    has_focus: bool,
}

pub fn render_block(tokens: &Tokens, resolved: &ResolvedBlock, cfg: &Config) -> String {
    let mut sb = String::with_capacity(4096);

    let mut wrapper_class = String::from("kazari-block");
    wrapper_class.push_str(" not-content");
    write!(sb, "<div class=\"{}\"", wrapper_class).unwrap();
    if cfg.data_line_count {
        write!(sb, " data-lines=\"{}\"", tokens.line_count()).unwrap();
    }
    sb.push_str(">\n");

    match resolved.frame {
        Frame::None => render_no_frame(&mut sb, tokens, resolved, cfg),
        Frame::Terminal => render_terminal_frame(&mut sb, tokens, resolved, cfg),
        _ => render_framed_block(&mut sb, tokens, resolved, cfg),
    }

    sb.push_str("</div>");
    sb
}

fn render_framed_block(sb: &mut String, tokens: &Tokens, resolved: &ResolvedBlock, cfg: &Config) {
    let mut classes = String::from("frame");
    if !resolved.title.is_empty() {
        classes.push_str(" has-title");
    }
    write!(
        sb,
        "<figure class=\"{}\" data-lang=\"{}\">",
        classes,
        escape_attr(&resolved.lang)
    )
    .unwrap();

    render_toolbar(sb, resolved, cfg);
    render_pre_code(sb, tokens, resolved, cfg);

    sb.push_str("</figure>\n");
}

fn render_terminal_frame(sb: &mut String, tokens: &Tokens, resolved: &ResolvedBlock, cfg: &Config) {
    let mut classes = String::from("frame is-terminal");
    if !resolved.title.is_empty() {
        classes.push_str(" has-title");
    }
    write!(
        sb,
        "<figure class=\"{}\" data-lang=\"{}\">",
        classes,
        escape_attr(&resolved.lang)
    )
    .unwrap();

    if cfg.terminal_dot_style == TerminalDotStyle::Minimal {
        sb.push_str("<div class=\"kz-terminal-header kz-dots-minimal\">");
    } else {
        sb.push_str("<div class=\"kz-terminal-header\">");
        sb.push_str("<span class=\"kz-terminal-dots\" aria-hidden=\"true\"><span></span><span></span><span></span></span>");
    }
    if !resolved.title.is_empty() {
        write!(
            sb,
            "<span class=\"kz-title\">{}</span>",
            escape_text(&resolved.title)
        )
        .unwrap();
    } else {
        sb.push_str("<span class=\"sr-only\">Terminal</span>");
    }
    if cfg.copy_button || cfg.wrap_button {
        sb.push_str("<div class=\"kz-terminal-actions\">");
        render_action_buttons(sb, resolved, cfg);
        sb.push_str("</div>");
    }
    sb.push_str("</div>");

    render_pre_code(sb, tokens, resolved, cfg);

    sb.push_str("</figure>\n");
}

fn render_no_frame(sb: &mut String, tokens: &Tokens, resolved: &ResolvedBlock, cfg: &Config) {
    render_pre_code(sb, tokens, resolved, cfg);
    if cfg.copy_button {
        render_copy_button(sb, &resolved.raw_code, cfg);
    }
}

fn render_toolbar(sb: &mut String, resolved: &ResolvedBlock, cfg: &Config) {
    sb.push_str("<div class=\"kz-toolbar\">");

    sb.push_str("<div class=\"kz-toolbar-left\">");
    if cfg.language_badge && !resolved.lang.is_empty() {
        render_lang_badge(sb, &resolved.lang);
    }
    if !resolved.title.is_empty() {
        write!(
            sb,
            "<span class=\"kz-title\">{}</span>",
            escape_text(&resolved.title)
        )
        .unwrap();
    }
    sb.push_str("</div>");

    sb.push_str("<div class=\"kz-toolbar-right\">");
    render_action_buttons(sb, resolved, cfg);
    sb.push_str("</div>");

    sb.push_str("</div>");
}

fn render_action_buttons(sb: &mut String, resolved: &ResolvedBlock, cfg: &Config) {
    if cfg.copy_button {
        render_copy_button(sb, &resolved.raw_code, cfg);
    }
    if cfg.wrap_button {
        render_wrap_button(sb, resolved);
    }
}

fn render_copy_button(sb: &mut String, raw_code: &str, _cfg: &Config) {
    let encoded = encode_for_data_code(raw_code);
    write!(
        sb,
        "<button class=\"kz-copy-btn\" aria-label=\"Copy\" data-tooltip=\"Copy\" data-copied=\"Copied!\" data-code=\"{}\">",
        escape_attr(&encoded)
    )
    .unwrap();
    sb.push_str(COPY_SVG);
    sb.push_str("</button>");
    sb.push_str("<span class=\"kz-sr-announce\" aria-live=\"polite\"></span>");
}

fn render_wrap_button(sb: &mut String, resolved: &ResolvedBlock) {
    let (pressed, title) = if resolved.wrap {
        ("true", "Disable word wrap")
    } else {
        ("false", "Enable word wrap")
    };
    write!(
        sb,
        "<button class=\"kz-wrap-btn\" aria-pressed=\"{}\" aria-label=\"{}\" data-tooltip=\"{}\" data-enable=\"Enable word wrap\" data-disable=\"Disable word wrap\">",
        pressed,
        escape_attr(title),
        escape_attr(title),
    )
    .unwrap();
    sb.push_str(WRAP_SVG);
    sb.push_str(WRAP_OFF_SVG);
    sb.push_str("</button>");
}

fn render_lang_badge(sb: &mut String, lang: &str) {
    write!(
        sb,
        "<span class=\"kz-lang\">{}</span>",
        escape_text(&display_lang(lang))
    )
    .unwrap();
}

fn display_lang(lang: &str) -> String {
    match lang.to_lowercase().as_str() {
        "javascript" => "JavaScript".to_owned(),
        "typescript" => "TypeScript".to_owned(),
        "css" => "CSS".to_owned(),
        "html" => "HTML".to_owned(),
        "json" => "JSON".to_owned(),
        "yaml" => "YAML".to_owned(),
        "sql" => "SQL".to_owned(),
        "php" => "PHP".to_owned(),
        "xml" => "XML".to_owned(),
        "svg" => "SVG".to_owned(),
        "jsx" => "JSX".to_owned(),
        "tsx" => "TSX".to_owned(),
        "graphql" => "GraphQL".to_owned(),
        _ => {
            let mut chars = lang.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        }
    }
}

fn render_pre_code(sb: &mut String, tokens: &Tokens, resolved: &ResolvedBlock, _cfg: &Config) {
    let lctx = LineCtx {
        resolved,
        is_dual: tokens.is_dual(),
        resolved_markers: marker::resolve_line_markers(&resolved.line_markers),
        focus_set: marker::resolve_focus_set(&resolved.focus_lines),
        has_focus: !resolved.focus_lines.is_empty(),
    };

    if resolved.wrap {
        sb.push_str("<pre class=\"wrap\"");
    } else {
        sb.push_str("<pre");
    }
    write!(sb, " data-language=\"{}\">", escape_attr(&resolved.lang)).unwrap();

    let ln_width = if resolved.line_numbers {
        let end_num = resolved.start_line_number + tokens.line_count().saturating_sub(1);
        let max_digits = digit_count(resolved.start_line_number).max(digit_count(end_num));
        if max_digits > 2 { max_digits } else { 0 }
    } else {
        0
    };

    if lctx.has_focus {
        if ln_width > 0 {
            write!(
                sb,
                "<code class=\"has-focus\" style=\"--kz-ln-width:{}ch\">",
                ln_width
            )
            .unwrap();
        } else {
            sb.push_str("<code class=\"has-focus\">");
        }
    } else if ln_width > 0 {
        write!(sb, "<code style=\"--kz-ln-width:{}ch\">", ln_width).unwrap();
    } else {
        sb.push_str("<code>");
    }

    for i in 0..tokens.line_count() {
        let line_num = resolved.start_line_number + i;
        render_line(sb, tokens, i, line_num, &lctx);
    }

    sb.push_str("</code></pre>");
}

fn render_line(
    sb: &mut String,
    tokens: &Tokens,
    line_idx: usize,
    line_num: usize,
    lctx: &LineCtx<'_>,
) {
    let mut classes = String::from("kz-line");
    let mut label_attr = String::new();
    let mut _marker_type: Option<MarkerType> = None;

    if let Some(entry) = lctx
        .resolved_markers
        .as_ref()
        .and_then(|m| m.get(&line_num))
        && entry.has_mark
    {
        classes.push_str(" highlight");
        _marker_type = Some(entry.marker_type);
        match entry.marker_type {
            MarkerType::Mark => classes.push_str(" mark"),
            MarkerType::Del => classes.push_str(" del"),
            MarkerType::Ins => classes.push_str(" ins"),
        }
        if !entry.label.is_empty() {
            classes.push_str(" tm-label");
            label_attr = format!(" data-label=\"{}\"", escape_attr(&entry.label));
        }
    }

    if lctx.has_focus
        && lctx
            .focus_set
            .as_ref()
            .is_some_and(|f| f.contains(&line_num))
    {
        classes.push_str(" focused");
    }

    let line_text = tokens.line_text(line_idx);
    let line_tokens = tokens.tokens(line_idx);

    let mut indent_attr = String::new();
    let mut indent_ws = String::new();
    let mut skip_ws_tokens = 0;

    if lctx.resolved.wrap {
        let (ws, first_non_ws_idx) = split_leading_whitespace(line_text, line_tokens);
        let mut indent = lctx.resolved.hanging_indent;
        if lctx.resolved.preserve_indent {
            indent += ws.len();
        }
        if indent > 0 {
            indent_attr = format!(" style=\"--kz-indent:{}ch\"", indent);
            indent_ws = ws.to_owned();
            skip_ws_tokens = first_non_ws_idx;
        }
    }

    write!(sb, "<div class=\"{}\">", classes).unwrap();

    if lctx.resolved.line_numbers {
        write!(
            sb,
            "<div class=\"kz-gutter\"><div class=\"kz-ln\" aria-hidden=\"true\">{}</div></div>",
            line_num
        )
        .unwrap();
    }

    write!(sb, "<div class=\"kz-code\"{}{}>", label_attr, indent_attr).unwrap();

    if !indent_ws.is_empty() {
        write!(sb, "<span class=\"indent\">{}</span>", indent_ws).unwrap();
    }

    if !lctx.resolved.inline_markers.is_empty() {
        let mut plain_text = String::new();
        let mut token_ranges: Vec<(usize, usize)> = Vec::new();
        let mut renderable_indices: Vec<usize> = Vec::new();

        for (tok_idx, token) in line_tokens.iter().enumerate() {
            if tok_idx < skip_ws_tokens {
                continue;
            }
            let text = if tok_idx == skip_ws_tokens && skip_ws_tokens > 0 {
                let full = token.text(line_text);
                let trimmed = full.trim_start_matches([' ', '\t']);
                if trimmed.is_empty() {
                    continue;
                }
                trimmed
            } else {
                token.text(line_text)
            };
            if text.is_empty() {
                continue;
            }
            let start = plain_text.len();
            plain_text.push_str(text);
            token_ranges.push((start, plain_text.len()));
            renderable_indices.push(tok_idx);
        }

        if let Some(annotated) = marker::process_inline_markers(
            &plain_text,
            &token_ranges,
            &lctx.resolved.inline_markers,
        ) {
            for at in &annotated {
                let real_idx = renderable_indices[at.token_idx];
                let token = &line_tokens[real_idx];
                render_annotated_token(sb, &plain_text, token, &at.segments, tokens, lctx);
            }
        } else {
            render_plain_tokens(sb, line_tokens, line_text, skip_ws_tokens, tokens, lctx);
        }
    } else {
        render_plain_tokens(sb, line_tokens, line_text, skip_ws_tokens, tokens, lctx);
    }

    sb.push_str("</div></div>");
}

fn render_plain_tokens(
    sb: &mut String,
    line_tokens: &[iro::ThemedToken],
    line_text: &str,
    skip_ws_tokens: usize,
    tokens: &Tokens,
    lctx: &LineCtx<'_>,
) {
    for (tok_idx, token) in line_tokens.iter().enumerate() {
        if tok_idx < skip_ws_tokens {
            continue;
        }
        let text = if tok_idx == skip_ws_tokens && skip_ws_tokens > 0 {
            let full = token.text(line_text);
            let trimmed = full.trim_start_matches([' ', '\t']);
            if trimmed.is_empty() {
                continue;
            }
            trimmed
        } else {
            token.text(line_text)
        };
        if text.is_empty() {
            continue;
        }
        let style = build_token_style(tokens, token, lctx);
        if style.is_empty() {
            write!(sb, "<span>{}</span>", escape_text(text)).unwrap();
        } else {
            write!(sb, "<span style=\"{}\">{}</span>", style, escape_text(text)).unwrap();
        }
    }
}

fn marker_element(mt: MarkerType) -> &'static str {
    match mt {
        MarkerType::Ins => "ins",
        MarkerType::Del => "del",
        MarkerType::Mark => "mark",
    }
}

fn render_annotated_token(
    sb: &mut String,
    plain_text: &str,
    token: &iro::ThemedToken,
    segments: &[Segment],
    tokens: &Tokens,
    lctx: &LineCtx<'_>,
) {
    let has_inline_marker = segments.iter().any(|s| s.marker.is_some());

    if !has_inline_marker {
        let text: String = segments
            .iter()
            .map(|s| &plain_text[s.start..s.end])
            .collect();
        let style = build_token_style(tokens, token, lctx);
        if style.is_empty() {
            write!(sb, "<span>{}</span>", escape_text(&text)).unwrap();
        } else {
            write!(
                sb,
                "<span style=\"{}\">{}</span>",
                style,
                escape_text(&text)
            )
            .unwrap();
        }
        return;
    }

    let single_spanning = segments.len() == 1
        && segments[0]
            .marker
            .as_ref()
            .is_some_and(|m| m.open_start || m.open_end);

    if single_spanning {
        let seg = &segments[0];
        let ann = seg.marker.as_ref().unwrap();
        let elem = marker_element(ann.marker_type);
        let mut classes = String::new();
        if ann.open_start {
            classes.push_str("open-start");
        }
        if ann.open_end {
            if !classes.is_empty() {
                classes.push(' ');
            }
            classes.push_str("open-end");
        }
        write!(sb, "<{} class=\"{}\">", elem, classes).unwrap();
        let style = build_token_style(tokens, token, lctx);
        let text = &plain_text[seg.start..seg.end];
        if style.is_empty() {
            write!(sb, "<span>{}</span>", escape_text(text)).unwrap();
        } else {
            write!(sb, "<span style=\"{}\">{}</span>", style, escape_text(text)).unwrap();
        }
        write!(sb, "</{}>", elem).unwrap();
        return;
    }

    let style = build_token_style(tokens, token, lctx);
    if style.is_empty() {
        sb.push_str("<span>");
    } else {
        write!(sb, "<span style=\"{}\">", style).unwrap();
    }
    for seg in segments {
        let text = &plain_text[seg.start..seg.end];
        if let Some(ann) = &seg.marker {
            let elem = marker_element(ann.marker_type);
            write!(sb, "<{}>", elem).unwrap();
            sb.push_str(&escape_text(text));
            write!(sb, "</{}>", elem).unwrap();
        } else {
            sb.push_str(&escape_text(text));
        }
    }
    sb.push_str("</span>");
}

fn split_leading_whitespace<'a>(
    line_text: &'a str,
    tokens: &[iro::ThemedToken],
) -> (&'a str, usize) {
    let mut ws_end = 0;
    let mut first_non_ws_idx = 0;

    for (i, token) in tokens.iter().enumerate() {
        let text = token.text(line_text);
        let trimmed = text.trim_start_matches([' ', '\t']);
        ws_end += text.len() - trimmed.len();
        if !trimmed.is_empty() {
            first_non_ws_idx = i;
            break;
        }
        first_non_ws_idx = i + 1;
    }

    (&line_text[..ws_end], first_non_ws_idx)
}

fn build_token_style(tokens: &Tokens, token: &iro::ThemedToken, lctx: &LineCtx<'_>) -> String {
    let light = tokens.light_style(token);
    let dark = tokens.dark_style(token);

    let mut parts = Vec::new();

    if let Some(color_id) = light.color {
        parts.push(format!("--sl:{}", tokens.light_color(color_id)));
    }
    if let Some(bg_id) = light.bg {
        let bg = tokens.light_color(bg_id);
        if !bg.is_empty() {
            parts.push(format!("--slbg:{}", bg));
        }
    }

    if lctx.is_dual
        && let Some(dark_style) = dark
    {
        if let Some(color_id) = dark_style.color {
            let color = tokens.dark_color(color_id).unwrap_or("");
            if !color.is_empty() {
                parts.push(format!("--sd:{}", color));
            }
        }
        if let Some(bg_id) = dark_style.bg {
            let bg = tokens.dark_color(bg_id).unwrap_or("");
            if !bg.is_empty() {
                parts.push(format!("--sdbg:{}", bg));
            }
        }
    }

    let fs = light.font_style;
    if fs.contains(FontStyle::ITALIC) {
        parts.push("--sfs:italic".to_owned());
    }
    if fs.contains(FontStyle::BOLD) {
        parts.push("--sfw:bold".to_owned());
    }
    let has_underline = fs.contains(FontStyle::UNDERLINE);
    let has_strike = fs.contains(FontStyle::STRIKETHROUGH);
    if has_underline || has_strike {
        let dec = match (has_underline, has_strike) {
            (true, true) => "underline line-through",
            (true, false) => "underline",
            (false, true) => "line-through",
            _ => unreachable!(),
        };
        parts.push(format!("--std:{}", dec));
    }

    if parts.is_empty() {
        return String::new();
    }
    escape_attr(&parts.join(";"))
}

fn encode_for_data_code(code: &str) -> String {
    code.replace('\n', "\x7f")
}

fn digit_count(n: usize) -> usize {
    if n == 0 {
        return 1;
    }
    let mut count = 0;
    let mut n = n;
    while n > 0 {
        count += 1;
        n /= 10;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_lang_known() {
        assert_eq!(display_lang("javascript"), "JavaScript");
        assert_eq!(display_lang("css"), "CSS");
        assert_eq!(display_lang("graphql"), "GraphQL");
    }

    #[test]
    fn display_lang_capitalizes_first() {
        assert_eq!(display_lang("rust"), "Rust");
        assert_eq!(display_lang("go"), "Go");
    }

    #[test]
    fn display_lang_empty() {
        assert_eq!(display_lang(""), "");
    }

    #[test]
    fn digit_count_values() {
        assert_eq!(digit_count(0), 1);
        assert_eq!(digit_count(1), 1);
        assert_eq!(digit_count(9), 1);
        assert_eq!(digit_count(10), 2);
        assert_eq!(digit_count(99), 2);
        assert_eq!(digit_count(100), 3);
        assert_eq!(digit_count(1000), 4);
    }

    #[test]
    fn encode_for_data_code_replaces_newlines() {
        assert_eq!(encode_for_data_code("a\nb\nc"), "a\x7fb\x7fc");
        assert_eq!(encode_for_data_code("no newlines"), "no newlines");
    }
}
