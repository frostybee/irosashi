use std::collections::HashMap;
use std::ops::Range;

use crate::render::Node;
use crate::render::transformer::{SpanContext, Transformer};
use crate::token::{ThemedLine, ThemedToken, TokensResult};

const OPEN: &str = "[!code";

/// Finds the first `[!code X]` in `text`: the byte range of the whole annotation and
/// `X`, which is made of ASCII word characters, `:`, `+` and `-`.
pub fn parse_annotation(text: &str) -> Option<(Range<usize>, &str)> {
    let mut from = 0;
    while let Some(pos) = text[from..].find(OPEN) {
        let start = from + pos;
        let rest = &text[start + OPEN.len()..];
        let ws = rest.len() - rest.trim_start_matches(is_space).len();
        if ws > 0 {
            let body = &rest[ws..];
            let name_len = body
                .bytes()
                .take_while(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b':' | b'+' | b'-'))
                .count();
            if name_len > 0 && body.as_bytes().get(name_len) == Some(&b']') {
                let name_start = start + OPEN.len() + ws;
                let end = name_start + name_len + 1;
                return Some((start..end, &text[name_start..name_start + name_len]));
            }
        }
        from = start + 1;
    }
    None
}

fn is_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0c')
}

/// Whether what is left of a comment token, once its annotation is cut out, is only
/// a comment opener or closer.
pub fn is_comment_prefix(s: &str) -> bool {
    matches!(s.trim(), "//" | "#" | "/*" | "*/" | "/* */")
}

/// The classes an annotation adds to its line; `focus` and `word:X` are recorded on
/// the transformer.
fn annotation_classes(annot: &str, transformer: &mut Notation) -> &'static [&'static str] {
    match annot {
        "++" => &["diff", "add"],
        "--" => &["diff", "remove"],
        "highlight" => &["highlighted"],
        "focus" => {
            transformer.has_focus = true;
            &["focused"]
        }
        "error" => &["highlighted", "error"],
        "warning" => &["highlighted", "warning"],
        _ => {
            if let Some(word) = annot.strip_prefix("word:")
                && !word.is_empty()
            {
                transformer.words.push(word.to_owned());
            }
            &[]
        }
    }
}

/// Strips the annotation at `rel` (relative to token `idx`) out of the line.
fn strip_annotation(line: ThemedLine, text: &str, idx: usize, rel: Range<usize>) -> ThemedLine {
    let tok = line.tokens[idx];
    let cut_start = tok.start + rel.start;
    let cut_end = tok.start + rel.end;
    let before = &text[tok.start..cut_start];
    let before_len = before.trim_end_matches(' ').len();
    let after = &text[cut_end..tok.end];
    let remaining = format!("{}{after}", &before[..before_len]);
    let remaining = remaining.trim();

    if remaining.is_empty() || is_comment_prefix(remaining) {
        // Grammars usually split `// [!code x] */` into opener, body and closer
        // tokens, so a bare opener or closer next to the annotation goes with it.
        let is_blank = |t: &ThemedToken| t.text(text).trim().is_empty();
        let is_opener = |t: &ThemedToken| is_comment_prefix(t.text(text));
        let mut keep_before = idx;
        if keep_before > 0 && is_blank(&line.tokens[keep_before - 1]) {
            keep_before -= 1;
        }
        if keep_before > 0 && is_opener(&line.tokens[keep_before - 1]) {
            keep_before -= 1;
            if keep_before > 0 && is_blank(&line.tokens[keep_before - 1]) {
                keep_before -= 1;
            }
        }
        let mut skip_after = idx + 1;
        let after_tokens = &line.tokens[skip_after..];
        let closer_at = usize::from(after_tokens.first().is_some_and(is_blank));
        if after_tokens.get(closer_at).is_some_and(is_opener) {
            skip_after += closer_at + 1;
        }
        let mut tokens: Vec<ThemedToken> = line.tokens[..keep_before]
            .iter()
            .chain(&line.tokens[skip_after..])
            .copied()
            .collect();
        if let Some(last) = tokens.last_mut() {
            while last.end > last.start && text.as_bytes()[last.end - 1] == b' ' {
                last.end -= 1;
            }
            if last.end == last.start {
                tokens.pop();
            }
        }
        return ThemedLine::new(line.range, tokens);
    }

    // Offsets cannot express a gap inside one token, so the token becomes the part
    // before the annotation and the part after it.
    let mut tokens = line.tokens;
    let head = ThemedToken {
        end: tok.start + before_len,
        ..tok
    };
    let tail = ThemedToken {
        start: cut_end,
        ..tok
    };
    tokens.remove(idx);
    let mut at = idx;
    if head.end > head.start {
        tokens.insert(at, head);
        at += 1;
    }
    if tail.end > tail.start {
        tokens.insert(at, tail);
    }
    ThemedLine::new(line.range, tokens)
}

fn is_empty_line(line: &ThemedLine, text: &str) -> bool {
    line.tokens.iter().all(|t| t.text(text).trim().is_empty())
}

fn wrap_word(el: &mut Node, text: &str, word: &str) {
    let mut children = Vec::new();
    let mut rest = text;
    while let Some(pos) = rest.find(word) {
        if pos > 0 {
            children.push(Node::text(&rest[..pos]));
        }
        children.push(
            Node::element("span")
                .attr("class", "highlighted-word")
                .child(Node::text(word)),
        );
        rest = &rest[pos + word.len()..];
    }
    if !rest.is_empty() {
        children.push(Node::text(rest));
    }
    if let Some(existing) = el.children_mut() {
        *existing = children;
    }
}

/// Applies `[!code ++]`, `[!code --]`, `[!code highlight]`, `[!code focus]`,
/// `[!code error]`, `[!code warning]` and `[!code word:X]` comment annotations:
/// the annotation is cut out of its token, a comment left empty is dropped, a line
/// left empty is removed, and the matching classes go on the line (`focus` also
/// dims every other line and marks `pre` with `has-focused`; `word:X` wraps every
/// occurrence of `X` in `span.highlighted-word`).
#[derive(Debug, Default)]
pub struct Notation {
    line_classes: HashMap<usize, &'static [&'static str]>,
    has_focus: bool,
    words: Vec<String>,
}

impl Notation {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Transformer for Notation {
    fn name(&self) -> &str {
        "notation"
    }

    fn tokens(&mut self, result: &mut TokensResult) {
        self.line_classes.clear();
        self.has_focus = false;
        self.words.clear();

        let lines = std::mem::take(&mut result.lines);
        let mut kept = Vec::with_capacity(lines.len());
        for line in lines {
            let text = &result.source[line.range.clone()];
            let found = line
                .tokens
                .iter()
                .enumerate()
                .rev()
                .find_map(|(i, t)| parse_annotation(t.text(text)).map(|(r, a)| (i, r, a)));
            let Some((idx, rel, annot)) = found else {
                kept.push(line);
                continue;
            };
            let line_num = kept.len() + 1;
            let classes = annotation_classes(annot, self);
            let stripped = strip_annotation(line, text, idx, rel);
            if is_empty_line(&stripped, text) {
                continue;
            }
            if !classes.is_empty() {
                self.line_classes.insert(line_num, classes);
            }
            kept.push(stripped);
        }
        result.lines = kept;
    }

    fn span(&mut self, el: &mut Node, _line_el: &mut Node, ctx: &SpanContext<'_>) {
        if let Some(word) = self.words.iter().find(|w| ctx.text.contains(w.as_str())) {
            wrap_word(el, ctx.text, word);
        }
    }

    fn line(&mut self, el: &mut Node, line: usize) {
        let classes = self.line_classes.get(&line).copied().unwrap_or(&[]);
        for class in classes {
            el.push_class(class);
        }
        if self.has_focus && !classes.contains(&"focused") {
            el.push_class("dimmed");
        }
    }

    fn pre(&mut self, el: &mut Node) {
        if self.has_focus {
            el.push_class("has-focused");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_annotations_like_the_regex() {
        assert_eq!(parse_annotation("// [!code ++]"), Some((3..13, "++")));
        assert_eq!(
            parse_annotation("[!code\t word:foo]"),
            Some((0..17, "word:foo"))
        );
        assert_eq!(
            parse_annotation("x [!code] [!code --] y"),
            Some((10..20, "--"))
        );
        assert_eq!(parse_annotation("[!code highlight"), None);
        assert_eq!(parse_annotation("[!codefocus]"), None);
        assert_eq!(parse_annotation("[!code a b]"), None);
        assert_eq!(parse_annotation("plain"), None);
    }

    #[test]
    fn comment_prefixes() {
        for s in ["//", " # ", "/*", "*/", "/* */"] {
            assert!(is_comment_prefix(s), "{s:?}");
        }
        for s in ["// x", "", "<!--", "--"] {
            assert!(!is_comment_prefix(s), "{s:?}");
        }
    }
}
