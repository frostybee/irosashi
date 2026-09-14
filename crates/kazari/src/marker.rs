use std::collections::{HashMap, HashSet};

use regex::Regex;

use crate::types::{InlineMarker, LineMarker, LineRange, MarkerType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedLine {
    pub marker_type: MarkerType,
    pub label: String,
    pub has_mark: bool,
}

pub fn resolve_line_markers(markers: &[LineMarker]) -> Option<HashMap<usize, ResolvedLine>> {
    if markers.is_empty() {
        return None;
    }

    let mut result = HashMap::new();

    for m in markers {
        let priority = m.marker_type as u8;

        for lr in &m.lines {
            let mut is_first_line = true;
            for line in lr.start..=lr.end {
                let entry = result.get(&line);
                let existing_priority = entry.map(|e: &ResolvedLine| e.marker_type as u8);

                if existing_priority.is_none_or(|p| priority >= p) {
                    let mut new_label = String::new();
                    if !m.label.is_empty() && is_first_line {
                        new_label = m.label.clone();
                    }
                    if let Some(existing) = entry
                        && existing_priority == Some(priority)
                        && new_label.is_empty()
                        && !existing.label.is_empty()
                    {
                        new_label = existing.label.clone();
                    }
                    result.insert(
                        line,
                        ResolvedLine {
                            marker_type: m.marker_type,
                            label: new_label,
                            has_mark: true,
                        },
                    );
                }
                is_first_line = false;
            }
        }
    }

    Some(result)
}

pub fn resolve_focus_set(focus_lines: &[LineRange]) -> Option<HashSet<usize>> {
    if focus_lines.is_empty() {
        return None;
    }
    let mut set = HashSet::new();
    for lr in focus_lines {
        for line in lr.start..=lr.end {
            set.insert(line);
        }
    }
    Some(set)
}

// ---------------------------------------------------------------------------
// Inline marker processing
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineAnnotation {
    pub marker_type: MarkerType,
    pub open_start: bool,
    pub open_end: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    pub start: usize,
    pub end: usize,
    pub marker: Option<InlineAnnotation>,
}

#[derive(Debug, Clone)]
pub struct AnnotatedToken {
    pub token_idx: usize,
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone)]
struct InlineMatch {
    start: usize,
    end: usize,
    marker_type: MarkerType,
    priority: u8,
}

pub fn process_inline_markers(
    plain_text: &str,
    token_ranges: &[(usize, usize)],
    markers: &[InlineMarker],
) -> Option<Vec<AnnotatedToken>> {
    if markers.is_empty() || plain_text.is_empty() {
        return None;
    }

    let matches = find_all_matches(plain_text, markers);
    if matches.is_empty() {
        return None;
    }

    let matches = resolve_inline_overlaps(matches);
    Some(split_tokens(token_ranges, &matches))
}

fn find_all_matches(text: &str, markers: &[InlineMarker]) -> Vec<InlineMatch> {
    let mut matches = Vec::new();
    for m in markers {
        if m.text.is_empty() {
            continue;
        }
        if m.is_regex {
            if let Ok(re) = Regex::new(&m.text) {
                let has_captures = re.captures_len() > 1;
                if has_captures {
                    for caps in re.captures_iter(text) {
                        let (start, end) = if let Some(g1) = caps.get(1) {
                            (g1.start(), g1.end())
                        } else {
                            let g0 = caps.get(0).unwrap();
                            (g0.start(), g0.end())
                        };
                        matches.push(InlineMatch {
                            start,
                            end,
                            marker_type: m.marker_type,
                            priority: m.marker_type as u8,
                        });
                    }
                } else {
                    for mat in re.find_iter(text) {
                        matches.push(InlineMatch {
                            start: mat.start(),
                            end: mat.end(),
                            marker_type: m.marker_type,
                            priority: m.marker_type as u8,
                        });
                    }
                }
            }
        } else {
            let needle = &m.text;
            let mut offset = 0;
            while let Some(idx) = text[offset..].find(needle) {
                let start = offset + idx;
                let end = start + needle.len();
                matches.push(InlineMatch {
                    start,
                    end,
                    marker_type: m.marker_type,
                    priority: m.marker_type as u8,
                });
                offset = end;
            }
        }
    }
    matches
}

fn resolve_inline_overlaps(mut matches: Vec<InlineMatch>) -> Vec<InlineMatch> {
    if matches.len() <= 1 {
        return matches;
    }

    matches.sort_by(|a, b| b.priority.cmp(&a.priority).then(a.start.cmp(&b.start)));

    let mut resolved: Vec<InlineMatch> = Vec::new();
    for m in matches {
        let mut fragments = vec![m];
        for r in &resolved {
            fragments = subtract_claimed_range(&fragments, r.start, r.end);
            if fragments.is_empty() {
                break;
            }
        }
        resolved.extend(fragments);
    }

    resolved.sort_by(|a, b| a.start.cmp(&b.start).then(a.priority.cmp(&b.priority)));
    resolved
}

fn subtract_claimed_range(
    fragments: &[InlineMatch],
    claim_start: usize,
    claim_end: usize,
) -> Vec<InlineMatch> {
    let mut out = Vec::new();
    for f in fragments {
        if f.end <= claim_start || f.start >= claim_end {
            out.push(f.clone());
            continue;
        }
        if f.start < claim_start {
            out.push(InlineMatch {
                start: f.start,
                end: claim_start,
                marker_type: f.marker_type,
                priority: f.priority,
            });
        }
        if f.end > claim_end {
            out.push(InlineMatch {
                start: claim_end,
                end: f.end,
                marker_type: f.marker_type,
                priority: f.priority,
            });
        }
    }
    out
}

fn split_tokens(token_ranges: &[(usize, usize)], matches: &[InlineMatch]) -> Vec<AnnotatedToken> {
    let mut result = Vec::with_capacity(token_ranges.len());
    let mut mi = 0;

    for (tok_i, &(tok_start, tok_end)) in token_ranges.iter().enumerate() {
        if tok_start == tok_end {
            result.push(AnnotatedToken {
                token_idx: tok_i,
                segments: vec![Segment {
                    start: tok_start,
                    end: tok_end,
                    marker: None,
                }],
            });
            continue;
        }

        let mut segments = Vec::new();
        let mut pos = tok_start;

        while mi < matches.len() && pos < tok_end {
            let m = &matches[mi];

            if m.start >= tok_end {
                break;
            }
            if m.end <= pos {
                mi += 1;
                continue;
            }

            let seg_start = m.start.max(pos);
            let seg_end = m.end.min(tok_end);

            if seg_start > pos {
                segments.push(Segment {
                    start: pos,
                    end: seg_start,
                    marker: None,
                });
            }

            segments.push(Segment {
                start: seg_start,
                end: seg_end,
                marker: Some(InlineAnnotation {
                    marker_type: m.marker_type,
                    open_start: m.start < tok_start,
                    open_end: m.end > tok_end,
                }),
            });

            pos = seg_end;
            if m.end <= tok_end {
                mi += 1;
            } else {
                break;
            }
        }

        if pos < tok_end {
            segments.push(Segment {
                start: pos,
                end: tok_end,
                marker: None,
            });
        }

        if segments.is_empty() {
            segments.push(Segment {
                start: tok_start,
                end: tok_end,
                marker: None,
            });
        }

        result.push(AnnotatedToken {
            token_idx: tok_i,
            segments,
        });
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_markers_returns_none() {
        assert!(resolve_line_markers(&[]).is_none());
    }

    #[test]
    fn single_marker_range() {
        let markers = vec![LineMarker {
            marker_type: MarkerType::Mark,
            lines: vec![LineRange::new(2, 4)],
            label: String::new(),
        }];
        let map = resolve_line_markers(&markers).unwrap();
        assert_eq!(map.len(), 3);
        assert_eq!(map[&2].marker_type, MarkerType::Mark);
        assert_eq!(map[&3].marker_type, MarkerType::Mark);
        assert_eq!(map[&4].marker_type, MarkerType::Mark);
        assert!(map[&2].has_mark);
    }

    #[test]
    fn higher_priority_wins_overlap() {
        let markers = vec![
            LineMarker {
                marker_type: MarkerType::Mark,
                lines: vec![LineRange::new(1, 5)],
                label: String::new(),
            },
            LineMarker {
                marker_type: MarkerType::Ins,
                lines: vec![LineRange::new(3, 4)],
                label: String::new(),
            },
        ];
        let map = resolve_line_markers(&markers).unwrap();
        assert_eq!(map[&1].marker_type, MarkerType::Mark);
        assert_eq!(map[&3].marker_type, MarkerType::Ins);
        assert_eq!(map[&4].marker_type, MarkerType::Ins);
        assert_eq!(map[&5].marker_type, MarkerType::Mark);
    }

    #[test]
    fn lower_priority_does_not_override() {
        let markers = vec![
            LineMarker {
                marker_type: MarkerType::Ins,
                lines: vec![LineRange::single(3)],
                label: String::new(),
            },
            LineMarker {
                marker_type: MarkerType::Mark,
                lines: vec![LineRange::single(3)],
                label: String::new(),
            },
        ];
        let map = resolve_line_markers(&markers).unwrap();
        assert_eq!(map[&3].marker_type, MarkerType::Ins);
    }

    #[test]
    fn label_on_first_line_of_range() {
        let markers = vec![LineMarker {
            marker_type: MarkerType::Mark,
            lines: vec![LineRange::new(2, 4)],
            label: "API".to_owned(),
        }];
        let map = resolve_line_markers(&markers).unwrap();
        assert_eq!(map[&2].label, "API");
        assert_eq!(map[&3].label, "");
        assert_eq!(map[&4].label, "");
    }

    #[test]
    fn same_priority_keeps_existing_label() {
        let markers = vec![
            LineMarker {
                marker_type: MarkerType::Mark,
                lines: vec![LineRange::single(3)],
                label: "First".to_owned(),
            },
            LineMarker {
                marker_type: MarkerType::Mark,
                lines: vec![LineRange::single(3)],
                label: String::new(),
            },
        ];
        let map = resolve_line_markers(&markers).unwrap();
        assert_eq!(map[&3].label, "First");
    }

    #[test]
    fn focus_set_empty_returns_none() {
        assert!(resolve_focus_set(&[]).is_none());
    }

    #[test]
    fn focus_set_expands_ranges() {
        let set = resolve_focus_set(&[LineRange::new(2, 4), LineRange::single(7)]).unwrap();
        assert!(set.contains(&2));
        assert!(set.contains(&3));
        assert!(set.contains(&4));
        assert!(!set.contains(&5));
        assert!(set.contains(&7));
    }

    // -- inline marker tests --

    fn mark(text: &str) -> InlineMarker {
        InlineMarker {
            marker_type: MarkerType::Mark,
            text: text.to_owned(),
            is_regex: false,
        }
    }

    fn ins_marker(text: &str) -> InlineMarker {
        InlineMarker {
            marker_type: MarkerType::Ins,
            text: text.to_owned(),
            is_regex: false,
        }
    }

    fn regex_mark(pattern: &str) -> InlineMarker {
        InlineMarker {
            marker_type: MarkerType::Mark,
            text: pattern.to_owned(),
            is_regex: true,
        }
    }

    #[test]
    fn inline_empty_markers_returns_none() {
        let result = process_inline_markers("hello", &[(0, 5)], &[]);
        assert!(result.is_none());
    }

    #[test]
    fn inline_empty_text_returns_none() {
        let result = process_inline_markers("", &[], &[mark("x")]);
        assert!(result.is_none());
    }

    #[test]
    fn inline_no_match_returns_none() {
        let result = process_inline_markers("hello world", &[(0, 11)], &[mark("xyz")]);
        assert!(result.is_none());
    }

    #[test]
    fn inline_literal_single_token() {
        // "let x = 1" as one token, marking "x"
        let text = "let x = 1";
        let ranges = [(0, 9)];
        let result = process_inline_markers(text, &ranges, &[mark("x")]).unwrap();
        assert_eq!(result.len(), 1);
        let segs = &result[0].segments;
        assert_eq!(segs.len(), 3);
        assert_eq!(&text[segs[0].start..segs[0].end], "let ");
        assert!(segs[0].marker.is_none());
        assert_eq!(&text[segs[1].start..segs[1].end], "x");
        assert!(segs[1].marker.is_some());
        assert_eq!(
            segs[1].marker.as_ref().unwrap().marker_type,
            MarkerType::Mark
        );
        assert!(!segs[1].marker.as_ref().unwrap().open_start);
        assert!(!segs[1].marker.as_ref().unwrap().open_end);
        assert_eq!(&text[segs[2].start..segs[2].end], " = 1");
        assert!(segs[2].marker.is_none());
    }

    #[test]
    fn inline_match_spans_tokens() {
        // Two tokens: "hel" and "lo", marking "ello"
        let text = "hello";
        let ranges = [(0, 3), (3, 5)];
        let result = process_inline_markers(text, &ranges, &[mark("ello")]).unwrap();
        assert_eq!(result.len(), 2);

        // First token: "h" plain, "el" marked with open_end
        let s0 = &result[0].segments;
        assert_eq!(&text[s0[0].start..s0[0].end], "h");
        assert!(s0[0].marker.is_none());
        assert_eq!(&text[s0[1].start..s0[1].end], "el");
        let ann = s0[1].marker.as_ref().unwrap();
        assert!(!ann.open_start);
        assert!(ann.open_end);

        // Second token: "lo" marked with open_start
        let s1 = &result[1].segments;
        assert_eq!(s1.len(), 1);
        assert_eq!(&text[s1[0].start..s1[0].end], "lo");
        let ann = s1[0].marker.as_ref().unwrap();
        assert!(ann.open_start);
        assert!(!ann.open_end);
    }

    #[test]
    fn inline_regex_match() {
        let text = "count = 42";
        let ranges = [(0, 10)];
        let result = process_inline_markers(text, &ranges, &[regex_mark(r"\d+")]).unwrap();
        let segs = &result[0].segments;
        assert_eq!(segs.len(), 2);
        assert_eq!(&text[segs[0].start..segs[0].end], "count = ");
        assert_eq!(&text[segs[1].start..segs[1].end], "42");
        assert!(segs[1].marker.is_some());
    }

    #[test]
    fn inline_regex_capture_group() {
        let text = "let count = 42";
        let ranges = [(0, 14)];
        let result = process_inline_markers(text, &ranges, &[regex_mark(r"let (\w+)")]).unwrap();
        let segs = &result[0].segments;
        // Should mark only the capture group "count", not the full match "let count"
        let marked: Vec<_> = segs.iter().filter(|s| s.marker.is_some()).collect();
        assert_eq!(marked.len(), 1);
        assert_eq!(&text[marked[0].start..marked[0].end], "count");
    }

    #[test]
    fn inline_invalid_regex_skipped() {
        let text = "hello";
        let ranges = [(0, 5)];
        let markers = vec![InlineMarker {
            marker_type: MarkerType::Mark,
            text: "[invalid".to_owned(),
            is_regex: true,
        }];
        let result = process_inline_markers(text, &ranges, &markers);
        assert!(result.is_none());
    }

    #[test]
    fn inline_overlap_higher_priority_wins() {
        // "abcdef" with Mark on "bcde" and Ins on "cd"
        let text = "abcdef";
        let ranges = [(0, 6)];
        let markers = vec![mark("bcde"), ins_marker("cd")];
        let result = process_inline_markers(text, &ranges, &markers).unwrap();
        let segs = &result[0].segments;
        // Should have: "a" plain, "b" Mark, "cd" Ins, "e" Mark, "f" plain
        let marked: Vec<_> = segs
            .iter()
            .filter(|s| s.marker.is_some())
            .map(|s| {
                (
                    &text[s.start..s.end],
                    s.marker.as_ref().unwrap().marker_type,
                )
            })
            .collect();
        assert_eq!(marked.len(), 3);
        assert_eq!(marked[0], ("b", MarkerType::Mark));
        assert_eq!(marked[1], ("cd", MarkerType::Ins));
        assert_eq!(marked[2], ("e", MarkerType::Mark));
    }

    #[test]
    fn inline_multiple_literal_matches() {
        let text = "a x b x c";
        let ranges = [(0, 10)];
        let result = process_inline_markers(text, &ranges, &[mark("x")]).unwrap();
        let marked: Vec<_> = result[0]
            .segments
            .iter()
            .filter(|s| s.marker.is_some())
            .map(|s| &text[s.start..s.end])
            .collect();
        assert_eq!(marked, vec!["x", "x"]);
    }
}
