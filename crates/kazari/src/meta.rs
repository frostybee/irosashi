use crate::types::{Frame, InlineMarker, LineMarker, LineRange, MarkerType};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockOptions {
    pub lang: String,
    pub title: String,
    pub theme: String,
    pub frame: Option<Frame>,
    pub line_numbers: Option<bool>,
    pub start_line_number: Option<usize>,
    pub wrap: Option<bool>,
    pub preserve_indent: Option<bool>,
    pub hanging_indent: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParseResult {
    pub block_options: BlockOptions,
    pub line_markers: Vec<LineMarker>,
    pub inline_markers: Vec<InlineMarker>,
    pub focus_lines: Vec<LineRange>,
    pub diff_lang: String,
}

pub fn parse(meta: &str) -> ParseResult {
    let mut result = ParseResult::default();
    let tokens = tokenize(meta);

    for (i, tok) in tokens.iter().enumerate() {
        if i == 0 && is_bare_lang(tok) {
            result.block_options.lang = tok.clone();
        } else if tok == "showLineNumbers" {
            result.block_options.line_numbers = Some(true);
        } else if tok == "showLineNumbers=false" {
            result.block_options.line_numbers = Some(false);
        } else if tok == "wrap" {
            result.block_options.wrap = Some(true);
        } else if tok == "preserveIndent" {
            result.block_options.preserve_indent = Some(true);
        } else if tok == "preserveIndent=false" {
            result.block_options.preserve_indent = Some(false);
        } else if let Some(val) = tok.strip_prefix("hangingIndent=") {
            if let Ok(n) = val.parse::<usize>() {
                result.block_options.hanging_indent = Some(n);
            }
        } else if let Some(val) = tok.strip_prefix("title=") {
            result.block_options.title = unquote(val);
        } else if let Some(val) = tok.strip_prefix("theme=") {
            result.block_options.theme = unquote(val);
        } else if let Some(val) = tok.strip_prefix("lang=") {
            result.diff_lang = unquote(val);
        } else if let Some(val) = tok.strip_prefix("frame=") {
            let frame = match unquote(val).as_str() {
                "code" => Frame::Code,
                "terminal" => Frame::Terminal,
                "none" => Frame::None,
                _ => Frame::Auto,
            };
            result.block_options.frame = Some(frame);
        } else if let Some(val) = tok.strip_prefix("startLineNumber=") {
            if let Ok(n) = val.parse::<usize>() {
                result.block_options.start_line_number = Some(n);
            }
        } else if let Some(val) = tok.strip_prefix("focus=") {
            let range_str = extract_braces(val);
            result.focus_lines = parse_ranges(&range_str);
        } else if let Some(remainder) = tok.strip_prefix("ins=") {
            parse_marker_token(remainder, MarkerType::Ins, &mut result);
        } else if let Some(remainder) = tok.strip_prefix("del=") {
            parse_marker_token(remainder, MarkerType::Del, &mut result);
        } else if let Some(remainder) = tok.strip_prefix("add=") {
            parse_marker_token(remainder, MarkerType::Ins, &mut result);
        } else if let Some(remainder) = tok.strip_prefix("rem=") {
            parse_marker_token(remainder, MarkerType::Del, &mut result);
        } else if tok.starts_with('{') && tok.ends_with('}') {
            let inner = &tok[1..tok.len() - 1];
            if let Some((label, ranges)) = parse_labeled_range(inner) {
                result.line_markers.push(LineMarker {
                    marker_type: MarkerType::Mark,
                    lines: ranges,
                    label,
                });
            } else {
                result.line_markers.push(LineMarker {
                    marker_type: MarkerType::Mark,
                    lines: parse_ranges(inner),
                    label: String::new(),
                });
            }
        } else if is_quoted_string(tok) {
            result.inline_markers.push(InlineMarker {
                marker_type: MarkerType::Mark,
                text: unquote(tok),
                is_regex: false,
            });
        } else if is_regex_pattern(tok) {
            result.inline_markers.push(InlineMarker {
                marker_type: MarkerType::Mark,
                text: extract_regex(tok),
                is_regex: true,
            });
        }
        // Unknown tokens (collapse, withOutput, etc.) are silently ignored in K1.
    }

    result
}

fn parse_marker_token(remainder: &str, mtype: MarkerType, result: &mut ParseResult) {
    if is_regex_pattern(remainder) {
        result.inline_markers.push(InlineMarker {
            marker_type: mtype,
            text: extract_regex(remainder),
            is_regex: true,
        });
    } else if is_quoted_string(remainder) {
        result.inline_markers.push(InlineMarker {
            marker_type: mtype,
            text: unquote(remainder),
            is_regex: false,
        });
    } else if remainder.starts_with('{') {
        let inner = extract_braces(remainder);
        if let Some((label, ranges)) = parse_labeled_range(&inner) {
            result.line_markers.push(LineMarker {
                marker_type: mtype,
                lines: ranges,
                label,
            });
        } else {
            result.line_markers.push(LineMarker {
                marker_type: mtype,
                lines: parse_ranges(&inner),
                label: String::new(),
            });
        }
    }
}

fn parse_labeled_range(s: &str) -> Option<(String, Vec<LineRange>)> {
    if s.is_empty() {
        return None;
    }
    let quote_char = s.as_bytes()[0];
    if quote_char != b'"' && quote_char != b'\'' {
        return None;
    }
    let end = s[1..].find(quote_char as char)?;
    let label = s[1..end + 1].to_owned();
    let rest = &s[end + 2..];
    if !rest.starts_with(':') {
        return None;
    }
    let ranges = parse_ranges(&rest[1..]);
    Some((label, ranges))
}

fn parse_ranges(s: &str) -> Vec<LineRange> {
    let mut ranges = Vec::new();
    for part in s.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some(dash) = part.find('-') {
            if dash > 0
                && let (Ok(start), Ok(end)) = (
                    part[..dash].trim().parse::<usize>(),
                    part[dash + 1..].trim().parse::<usize>(),
                )
            {
                ranges.push(LineRange::new(start, end));
            }
        } else if let Ok(n) = part.parse::<usize>() {
            ranges.push(LineRange::single(n));
        }
    }
    ranges
}

fn tokenize(meta: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = meta.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }

        let start = i;

        if is_quote_char(chars[i]) {
            let qc = chars[i];
            i += 1;
            while i < chars.len() && chars[i] != qc {
                if chars[i] == '\\' {
                    i += 1;
                }
                i += 1;
            }
            if i < chars.len() {
                i += 1;
            }
            tokens.push(chars[start..i].iter().collect());
            continue;
        }

        if chars[i] == '/' {
            i += 1;
            while i < chars.len() && chars[i] != '/' {
                if chars[i] == '\\' {
                    i += 1;
                }
                i += 1;
            }
            if i < chars.len() {
                i += 1;
            }
            tokens.push(chars[start..i].iter().collect());
            continue;
        }

        while i < chars.len() && !chars[i].is_whitespace() {
            if is_quote_char(chars[i]) {
                let qc = chars[i];
                i += 1;
                while i < chars.len() && chars[i] != qc {
                    if chars[i] == '\\' {
                        i += 1;
                    }
                    i += 1;
                }
                if i < chars.len() {
                    i += 1;
                }
            } else if chars[i] == '/' {
                i += 1;
                while i < chars.len() && chars[i] != '/' {
                    if chars[i] == '\\' {
                        i += 1;
                    }
                    i += 1;
                }
                if i < chars.len() {
                    i += 1;
                }
            } else if chars[i] == '{' {
                let mut depth = 0;
                while i < chars.len() {
                    if chars[i] == '{' {
                        depth += 1;
                    } else if chars[i] == '}' {
                        depth -= 1;
                        if depth == 0 {
                            i += 1;
                            break;
                        }
                    }
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        tokens.push(chars[start..i].iter().collect());
    }

    tokens
}

fn is_bare_lang(tok: &str) -> bool {
    !tok.is_empty()
        && !tok.contains('=')
        && !tok.starts_with('{')
        && !tok.starts_with('"')
        && !tok.starts_with('\'')
}

fn is_quote_char(c: char) -> bool {
    c == '"' || c == '\''
}

fn is_quoted_string(tok: &str) -> bool {
    if tok.len() < 2 {
        return false;
    }
    let bytes = tok.as_bytes();
    (bytes[0] == b'"' && bytes[tok.len() - 1] == b'"')
        || (bytes[0] == b'\'' && bytes[tok.len() - 1] == b'\'')
}

fn unquote(s: &str) -> String {
    if s.len() >= 2 {
        let bytes = s.as_bytes();
        if bytes[0] == b'"' && bytes[s.len() - 1] == b'"' {
            return s[1..s.len() - 1].replace("\\\"", "\"");
        }
        if bytes[0] == b'\'' && bytes[s.len() - 1] == b'\'' {
            return s[1..s.len() - 1].replace("\\'", "'");
        }
    }
    s.to_owned()
}

fn extract_braces(s: &str) -> String {
    if s.starts_with('{') && s.ends_with('}') {
        s[1..s.len() - 1].to_owned()
    } else {
        s.to_owned()
    }
}

fn is_regex_pattern(tok: &str) -> bool {
    tok.len() >= 2 && tok.starts_with('/') && tok.ends_with('/')
}

fn extract_regex(tok: &str) -> String {
    let inner = &tok[1..tok.len() - 1];
    inner.replace("\\/", "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_lang_only() {
        let r = parse("javascript");
        assert_eq!(r.block_options.lang, "javascript");
        assert!(r.line_markers.is_empty());
    }

    #[test]
    fn lang_with_line_markers() {
        let r = parse("rust {1,3-5}");
        assert_eq!(r.block_options.lang, "rust");
        assert_eq!(r.line_markers.len(), 1);
        assert_eq!(r.line_markers[0].marker_type, MarkerType::Mark);
        assert_eq!(
            r.line_markers[0].lines,
            [LineRange::single(1), LineRange::new(3, 5)]
        );
    }

    #[test]
    fn ins_del_line_markers() {
        let r = parse("go ins={2-4} del={6}");
        assert_eq!(r.block_options.lang, "go");
        assert_eq!(r.line_markers.len(), 2);
        assert_eq!(r.line_markers[0].marker_type, MarkerType::Ins);
        assert_eq!(r.line_markers[0].lines, [LineRange::new(2, 4)]);
        assert_eq!(r.line_markers[1].marker_type, MarkerType::Del);
        assert_eq!(r.line_markers[1].lines, [LineRange::single(6)]);
    }

    #[test]
    fn add_rem_are_aliases() {
        let r = parse("py add={1} rem={2}");
        assert_eq!(r.line_markers[0].marker_type, MarkerType::Ins);
        assert_eq!(r.line_markers[1].marker_type, MarkerType::Del);
    }

    #[test]
    fn labeled_range() {
        let r = parse("js {\"API\":6-10}");
        assert_eq!(r.line_markers.len(), 1);
        assert_eq!(r.line_markers[0].label, "API");
        assert_eq!(r.line_markers[0].lines, [LineRange::new(6, 10)]);
    }

    #[test]
    fn labeled_ins_range() {
        let r = parse("ts ins={\"New\":3-5}");
        assert_eq!(r.line_markers[0].marker_type, MarkerType::Ins);
        assert_eq!(r.line_markers[0].label, "New");
    }

    #[test]
    fn inline_text_marker() {
        let r = parse("js \"searchText\"");
        assert_eq!(r.inline_markers.len(), 1);
        assert_eq!(r.inline_markers[0].text, "searchText");
        assert!(!r.inline_markers[0].is_regex);
        assert_eq!(r.inline_markers[0].marker_type, MarkerType::Mark);
    }

    #[test]
    fn inline_regex_marker() {
        let r = parse("js /fn\\s+\\w+/");
        assert_eq!(r.inline_markers.len(), 1);
        assert_eq!(r.inline_markers[0].text, "fn\\s+\\w+");
        assert!(r.inline_markers[0].is_regex);
    }

    #[test]
    fn ins_del_inline_markers() {
        let r = parse("rs ins=\"added\" del=\"removed\"");
        assert_eq!(r.inline_markers.len(), 2);
        assert_eq!(r.inline_markers[0].marker_type, MarkerType::Ins);
        assert_eq!(r.inline_markers[0].text, "added");
        assert_eq!(r.inline_markers[1].marker_type, MarkerType::Del);
        assert_eq!(r.inline_markers[1].text, "removed");
    }

    #[test]
    fn ins_regex_marker() {
        let r = parse("go ins=/pattern/");
        assert_eq!(r.inline_markers.len(), 1);
        assert_eq!(r.inline_markers[0].marker_type, MarkerType::Ins);
        assert!(r.inline_markers[0].is_regex);
        assert_eq!(r.inline_markers[0].text, "pattern");
    }

    #[test]
    fn focus_lines() {
        let r = parse("py focus={1,3-5}");
        assert_eq!(r.focus_lines, [LineRange::single(1), LineRange::new(3, 5)]);
    }

    #[test]
    fn block_options() {
        let r = parse("js showLineNumbers wrap title=\"app.js\" frame=terminal startLineNumber=10");
        assert_eq!(r.block_options.lang, "js");
        assert_eq!(r.block_options.line_numbers, Some(true));
        assert_eq!(r.block_options.wrap, Some(true));
        assert_eq!(r.block_options.title, "app.js");
        assert_eq!(r.block_options.frame, Some(Frame::Terminal));
        assert_eq!(r.block_options.start_line_number, Some(10));
    }

    #[test]
    fn show_line_numbers_false() {
        let r = parse("js showLineNumbers=false");
        assert_eq!(r.block_options.line_numbers, Some(false));
    }

    #[test]
    fn preserve_indent_and_hanging() {
        let r = parse("js preserveIndent hangingIndent=4");
        assert_eq!(r.block_options.preserve_indent, Some(true));
        assert_eq!(r.block_options.hanging_indent, Some(4));
    }

    #[test]
    fn preserve_indent_false() {
        let r = parse("js preserveIndent=false");
        assert_eq!(r.block_options.preserve_indent, Some(false));
    }

    #[test]
    fn theme_override() {
        let r = parse("js theme=\"monokai\"");
        assert_eq!(r.block_options.theme, "monokai");
    }

    #[test]
    fn diff_lang() {
        let r = parse("diff lang=\"go\"");
        assert_eq!(r.block_options.lang, "diff");
        assert_eq!(r.diff_lang, "go");
    }

    #[test]
    fn frame_values() {
        assert_eq!(
            parse("js frame=code").block_options.frame,
            Some(Frame::Code)
        );
        assert_eq!(
            parse("js frame=terminal").block_options.frame,
            Some(Frame::Terminal)
        );
        assert_eq!(
            parse("js frame=none").block_options.frame,
            Some(Frame::None)
        );
        assert_eq!(
            parse("js frame=auto").block_options.frame,
            Some(Frame::Auto)
        );
    }

    #[test]
    fn empty_input() {
        let r = parse("");
        assert_eq!(r.block_options.lang, "");
        assert!(r.line_markers.is_empty());
        assert!(r.inline_markers.is_empty());
    }

    #[test]
    fn complex_meta_string() {
        let r = parse("js {1,3} title=\"app.js\" showLineNumbers ins={2} del=\"old\" focus={5-7}");
        assert_eq!(r.block_options.lang, "js");
        assert_eq!(r.block_options.title, "app.js");
        assert_eq!(r.block_options.line_numbers, Some(true));
        assert_eq!(r.line_markers.len(), 2);
        assert_eq!(r.line_markers[0].marker_type, MarkerType::Mark);
        assert_eq!(
            r.line_markers[0].lines,
            [LineRange::single(1), LineRange::single(3)]
        );
        assert_eq!(r.line_markers[1].marker_type, MarkerType::Ins);
        assert_eq!(r.line_markers[1].lines, [LineRange::single(2)]);
        assert_eq!(r.inline_markers.len(), 1);
        assert_eq!(r.inline_markers[0].marker_type, MarkerType::Del);
        assert_eq!(r.inline_markers[0].text, "old");
        assert_eq!(r.focus_lines, [LineRange::new(5, 7)]);
    }

    #[test]
    fn escaped_quote_in_title() {
        let r = parse(r#"js title="file \"name\".js""#);
        assert_eq!(r.block_options.title, "file \"name\".js");
    }

    #[test]
    fn escaped_slash_in_regex() {
        let r = parse(r"js /path\/to\/file/");
        assert_eq!(r.inline_markers[0].text, "path/to/file");
    }

    #[test]
    fn single_quoted_inline_marker() {
        let r = parse("js 'hello'");
        assert_eq!(r.inline_markers[0].text, "hello");
    }

    #[test]
    fn single_quoted_labeled_range() {
        let r = parse("js {'Label':3-5}");
        assert_eq!(r.line_markers[0].label, "Label");
        assert_eq!(r.line_markers[0].lines, [LineRange::new(3, 5)]);
    }

    #[test]
    fn unknown_tokens_are_ignored() {
        let r = parse("js collapse nocollapse withOutput");
        assert_eq!(r.block_options.lang, "js");
        assert!(r.line_markers.is_empty());
    }

    #[test]
    fn tokenizer_keeps_quoted_values_together() {
        let tokens = tokenize("title=\"hello world\" wrap");
        assert_eq!(tokens, ["title=\"hello world\"", "wrap"]);
    }

    #[test]
    fn tokenizer_keeps_braces_together() {
        let tokens = tokenize("js {1,3-5} ins={2}");
        assert_eq!(tokens, ["js", "{1,3-5}", "ins={2}"]);
    }

    #[test]
    fn tokenizer_regex_at_top_level() {
        let tokens = tokenize("js /foo bar/");
        assert_eq!(tokens, ["js", "/foo bar/"]);
    }

    #[test]
    fn parse_ranges_handles_spaces() {
        let ranges = parse_ranges("1, 3 - 5 , 8");
        assert_eq!(
            ranges,
            [
                LineRange::single(1),
                LineRange::new(3, 5),
                LineRange::single(8),
            ]
        );
    }

    #[test]
    fn parse_ranges_empty() {
        assert!(parse_ranges("").is_empty());
        assert!(parse_ranges(",,,").is_empty());
    }
}
