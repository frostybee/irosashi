use std::collections::BTreeMap;
use std::ops::Range;

use iro::transformers::notation::{is_comment_prefix, parse_annotation};

use crate::types::{InlineMarker, LineMarker, LineRange, MarkerType};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct NotationResult {
    pub code: String,
    pub line_markers: Vec<LineMarker>,
    pub focus_lines: Vec<LineRange>,
    pub inline_markers: Vec<InlineMarker>,
    pub warnings: Vec<String>,
}

/// Turns `[!code ...]` comment annotations into markers. The annotation is cut out
/// of its line together with a comment opener or closer left bare around it; a line
/// left blank is removed, so later line numbers shift.
pub fn process_notation(code: &str) -> NotationResult {
    let mut result = NotationResult::default();
    let mut out: Vec<String> = Vec::new();
    let mut by_type: BTreeMap<MarkerType, Vec<LineRange>> = BTreeMap::new();

    for line in code.split('\n') {
        let Some((range, annot)) = parse_annotation(line) else {
            out.push(line.to_owned());
            continue;
        };
        let cleaned = strip_line(line, range);
        let line_num = out.len() + 1;
        let marker = match annot {
            "++" => Some(MarkerType::Ins),
            "--" => Some(MarkerType::Del),
            "highlight" => Some(MarkerType::Mark),
            "error" => Some(MarkerType::Error),
            "warning" => Some(MarkerType::Warning),
            "focus" => {
                if cleaned.is_some() {
                    result.focus_lines.push(LineRange::single(line_num));
                }
                None
            }
            _ => {
                match annot.strip_prefix("word:") {
                    Some(word) if !word.is_empty() => result.inline_markers.push(InlineMarker {
                        marker_type: MarkerType::Mark,
                        text: word.to_owned(),
                        is_regex: false,
                    }),
                    Some(_) => {}
                    None => result
                        .warnings
                        .push(format!("unknown code notation [!code {annot}]")),
                }
                None
            }
        };
        let Some(cleaned) = cleaned else {
            continue;
        };
        if let Some(marker) = marker {
            by_type
                .entry(marker)
                .or_default()
                .push(LineRange::single(line_num));
        }
        out.push(cleaned);
    }

    result.line_markers = by_type
        .into_iter()
        .map(|(marker_type, lines)| LineMarker {
            marker_type,
            lines,
            label: String::new(),
        })
        .collect();
    result.code = out.join("\n");
    result
}

/// The line without its annotation, or `None` when nothing but whitespace is left.
fn strip_line(line: &str, range: Range<usize>) -> Option<String> {
    let head = line[..range.start].trim_end();
    let after = &line[range.end..];
    let tail = after.trim_start();
    let (head, tail) = match tail {
        "" => (strip_opener(head), ""),
        "*/" | "-->" => {
            let opener = if tail == "*/" { "/*" } else { "<!--" };
            match head.strip_suffix(opener) {
                Some(h) => (h.trim_end(), ""),
                None => (head, after),
            }
        }
        _ => (head, after),
    };
    let cleaned = format!("{head}{tail}");
    if cleaned.trim().is_empty() {
        None
    } else {
        Some(cleaned)
    }
}

fn strip_opener(head: &str) -> &str {
    let last_word = head.rsplit(char::is_whitespace).next().unwrap_or("");
    if !last_word.is_empty() && (is_comment_prefix(last_word) || last_word == "<!--") {
        head[..head.len() - last_word.len()].trim_end()
    } else {
        head
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ranges(r: &NotationResult, t: MarkerType) -> Vec<LineRange> {
        r.line_markers
            .iter()
            .filter(|m| m.marker_type == t)
            .flat_map(|m| m.lines.clone())
            .collect()
    }

    #[test]
    fn diff_highlight_error_and_warning_become_line_markers() {
        let r = process_notation(
            "a // [!code ++]\nb // [!code --]\nc # [!code highlight]\nd -- [!code error]\ne // [!code warning]\nf",
        );
        assert_eq!(r.code, "a\nb\nc\nd --\ne\nf");
        assert_eq!(ranges(&r, MarkerType::Ins), [LineRange::single(1)]);
        assert_eq!(ranges(&r, MarkerType::Del), [LineRange::single(2)]);
        assert_eq!(ranges(&r, MarkerType::Mark), [LineRange::single(3)]);
        assert_eq!(ranges(&r, MarkerType::Error), [LineRange::single(4)]);
        assert_eq!(ranges(&r, MarkerType::Warning), [LineRange::single(5)]);
        assert!(r.warnings.is_empty());
    }

    #[test]
    fn blank_lines_are_removed_and_numbers_shift() {
        let r = process_notation("x\n// [!code ++]\n/* [!code focus] */\ny // [!code highlight]");
        assert_eq!(r.code, "x\ny");
        assert_eq!(ranges(&r, MarkerType::Ins), []);
        assert!(r.focus_lines.is_empty());
        assert_eq!(ranges(&r, MarkerType::Mark), [LineRange::single(2)]);
    }

    #[test]
    fn block_comment_wrappers_and_trailing_text() {
        let r =
            process_notation("a /* [!code ++] */\nb <!-- [!code focus] -->\nc // [!code --] tail");
        assert_eq!(r.code, "a\nb\nc // tail");
        assert_eq!(r.focus_lines, [LineRange::single(2)]);
        assert_eq!(ranges(&r, MarkerType::Del), [LineRange::single(3)]);
    }

    #[test]
    fn word_becomes_an_inline_marker_and_unknown_annotations_warn() {
        let r =
            process_notation("foo // [!code word:foo]\nbar // [!code word:]\nbaz // [!code nope]");
        assert_eq!(r.code, "foo\nbar\nbaz");
        assert_eq!(
            r.inline_markers,
            [InlineMarker {
                marker_type: MarkerType::Mark,
                text: "foo".to_owned(),
                is_regex: false,
            }]
        );
        assert_eq!(r.warnings, ["unknown code notation [!code nope]"]);
    }

    #[test]
    fn same_type_lines_share_one_marker() {
        let r = process_notation("a // [!code ++]\nb\nc // [!code ++]");
        assert_eq!(r.line_markers.len(), 1);
        assert_eq!(
            r.line_markers[0].lines,
            [LineRange::single(1), LineRange::single(3)]
        );
    }

    #[test]
    fn code_without_annotations_is_unchanged() {
        let r = process_notation("a\n\nb");
        assert_eq!(r.code, "a\n\nb");
        assert!(r.line_markers.is_empty());
    }
}
