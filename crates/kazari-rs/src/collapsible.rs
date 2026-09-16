use std::collections::HashSet;

use crate::config::{
    CollapseRange, CollapseSpec, CollapseStyle, CollapsibleConfig, PreviewSegment,
};
use crate::locale::{self, UIStrings};
use crate::types::{LineMarker, LineRange};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CollapseResult {
    pub threshold: bool,
    pub preview_segments: Vec<PreviewSegment>,
    pub beyond_cap_count: usize,
    pub ranges: Vec<CollapseRange>,
}

/// Resolves both collapse modes for one block. Threshold collapse needs the engine
/// config; range collapse works without it.
pub fn resolve_collapse(
    line_count: usize,
    spec: Option<&CollapseSpec>,
    cfg: Option<&CollapsibleConfig>,
    code: &str,
    markers: &[LineMarker],
    focus_lines: &[LineRange],
) -> CollapseResult {
    let mut result = CollapseResult::default();

    // Labeled ranges guide readers through specific sections, so the automatic
    // threshold stays out of their way unless the block asks for it.
    if should_threshold_collapse(line_count, spec, cfg) {
        let forced = spec.is_some_and(|s| s.enabled);
        if forced || !has_labeled_markers(markers) {
            result.threshold = true;
            let preview = match cfg.map(|c| c.preview_lines) {
                Some(n) if n > 0 => n,
                _ => 8,
            };
            let (segments, beyond) =
                compute_preview_segments(preview, line_count, markers, focus_lines);
            result.preview_segments = segments;
            result.beyond_cap_count = beyond;
        }
    }

    if let Some(spec) = spec
        && !spec.ranges.is_empty()
    {
        let mut base_style = CollapseStyle::Github;
        if let Some(c) = cfg
            && c.style != CollapseStyle::Github
        {
            base_style = c.style;
        }
        if let Some(s) = spec.style {
            base_style = s;
        }
        for r in validate_ranges(&spec.ranges, line_count) {
            result.ranges.push(CollapseRange {
                start: r.start,
                end: r.end,
                line_count: r.end - r.start + 1,
                min_indent: compute_min_indent(code, r.start, r.end),
                style: resolve_collapse_style(base_style, r.end, line_count),
            });
        }
    }

    result
}

fn should_threshold_collapse(
    line_count: usize,
    spec: Option<&CollapseSpec>,
    cfg: Option<&CollapsibleConfig>,
) -> bool {
    let Some(cfg) = cfg else {
        return false;
    };
    if spec.is_some_and(|s| s.disabled) {
        return false;
    }
    if spec.is_some_and(|s| s.enabled) {
        return true;
    }
    let mut threshold = cfg.line_threshold;
    if let Some(t) = spec.and_then(|s| s.threshold)
        && t > 0
    {
        threshold = t;
    }
    if threshold == 0 {
        threshold = 15;
    }
    line_count > threshold
}

/// Drops reversed, out-of-range and overlapping ranges (first wins), clamps the
/// end to the block, and sorts by start.
fn validate_ranges(ranges: &[LineRange], line_count: usize) -> Vec<LineRange> {
    let mut valid: Vec<LineRange> = ranges
        .iter()
        .filter(|r| r.start <= r.end && r.start >= 1 && r.end >= 1 && r.start <= line_count)
        .map(|r| LineRange::new(r.start, r.end.min(line_count)))
        .collect();
    valid.sort_by_key(|r| r.start);

    let mut result: Vec<LineRange> = Vec::new();
    for r in valid {
        if result.last().is_some_and(|last| r.start <= last.end) {
            continue;
        }
        result.push(r);
    }
    result
}

/// The visible lines of a threshold preview: the base preview plus every marked
/// or focused line within twice the preview (with one line of context, merged when
/// adjacent). Returns the segments and the count of marked lines beyond the cap.
fn compute_preview_segments(
    preview_lines: usize,
    line_count: usize,
    markers: &[LineMarker],
    focus_lines: &[LineRange],
) -> (Vec<PreviewSegment>, usize) {
    let base = preview_lines;
    if base >= line_count {
        return (
            vec![PreviewSegment {
                start: 1,
                end: line_count,
            }],
            0,
        );
    }

    let max_cap = (base * 2).min(line_count);
    let marked = build_marked_set(markers, focus_lines);
    let base_segment = PreviewSegment {
        start: 1,
        end: base,
    };
    if marked.is_empty() {
        return (vec![base_segment], 0);
    }

    let mut within_cap: Vec<usize> = Vec::new();
    let mut beyond_cap = 0;
    for &line in &marked {
        if line <= base {
            continue;
        }
        if line <= max_cap {
            within_cap.push(line);
        } else {
            beyond_cap += 1;
        }
    }
    if within_cap.is_empty() {
        return (vec![base_segment], beyond_cap);
    }
    within_cap.sort_unstable();

    let mut segments = vec![base_segment];
    for line in within_cap {
        let seg = PreviewSegment {
            start: line.saturating_sub(1).max(1),
            end: (line + 1).min(line_count),
        };
        let last = segments.last_mut().unwrap();
        if seg.start <= last.end + 1 {
            if seg.end > last.end {
                last.end = seg.end;
            }
        } else {
            segments.push(seg);
        }
    }
    (segments, beyond_cap)
}

fn has_labeled_markers(markers: &[LineMarker]) -> bool {
    markers.iter().any(|m| !m.label.is_empty())
}

fn build_marked_set(markers: &[LineMarker], focus_lines: &[LineRange]) -> HashSet<usize> {
    let mut set = HashSet::new();
    for m in markers {
        for lr in &m.lines {
            set.extend(lr.start..=lr.end);
        }
    }
    for lr in focus_lines {
        set.extend(lr.start..=lr.end);
    }
    set
}

/// Minimum indentation (spaces and tabs) of the non-blank lines in the 1-based
/// inclusive range.
fn compute_min_indent(code: &str, start_line: usize, end_line: usize) -> usize {
    let mut min_indent: Option<usize> = None;
    for line in code
        .split('\n')
        .skip(start_line.saturating_sub(1))
        .take(end_line + 1 - start_line.max(1))
    {
        let trimmed = line.trim_start_matches([' ', '\t']);
        if trimmed.is_empty() {
            continue;
        }
        let indent = line.len() - trimmed.len();
        min_indent = Some(min_indent.map_or(indent, |m| m.min(indent)));
    }
    min_indent.unwrap_or(0)
}

fn resolve_collapse_style(
    style: CollapseStyle,
    range_end: usize,
    line_count: usize,
) -> CollapseStyle {
    if style != CollapseStyle::CollapsibleAuto {
        return style;
    }
    if range_end >= line_count {
        CollapseStyle::CollapsibleEnd
    } else {
        CollapseStyle::CollapsibleStart
    }
}

pub fn summary_text(line_count: usize, strings: &UIStrings) -> String {
    locale::format_collapsed_lines(strings, line_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MarkerType;

    fn cfg() -> CollapsibleConfig {
        CollapsibleConfig::default()
    }

    fn marker(kind: MarkerType, start: usize, end: usize, label: &str) -> LineMarker {
        LineMarker {
            marker_type: kind,
            lines: vec![LineRange::new(start, end)],
            label: label.to_owned(),
        }
    }

    #[test]
    fn threshold_needs_config_and_respects_directives() {
        assert!(!resolve_collapse(100, None, None, "", &[], &[]).threshold);
        assert!(resolve_collapse(16, None, Some(&cfg()), "", &[], &[]).threshold);
        assert!(!resolve_collapse(15, None, Some(&cfg()), "", &[], &[]).threshold);

        let disabled = CollapseSpec {
            disabled: true,
            ..Default::default()
        };
        assert!(!resolve_collapse(100, Some(&disabled), Some(&cfg()), "", &[], &[]).threshold);
        let forced = CollapseSpec {
            enabled: true,
            ..Default::default()
        };
        assert!(resolve_collapse(3, Some(&forced), Some(&cfg()), "", &[], &[]).threshold);
        let custom = CollapseSpec {
            threshold: Some(5),
            ..Default::default()
        };
        assert!(resolve_collapse(6, Some(&custom), Some(&cfg()), "", &[], &[]).threshold);
        assert!(!resolve_collapse(5, Some(&custom), Some(&cfg()), "", &[], &[]).threshold);

        let zero = CollapsibleConfig {
            line_threshold: 0,
            preview_lines: 0,
            ..Default::default()
        };
        let r = resolve_collapse(16, None, Some(&zero), "", &[], &[]);
        assert!(r.threshold);
        assert_eq!(r.preview_segments, [PreviewSegment { start: 1, end: 8 }]);
    }

    #[test]
    fn labeled_markers_skip_threshold_unless_forced() {
        let labeled = [marker(MarkerType::Mark, 20, 22, "API")];
        assert!(!resolve_collapse(30, None, Some(&cfg()), "", &labeled, &[]).threshold);
        let forced = CollapseSpec {
            enabled: true,
            ..Default::default()
        };
        assert!(resolve_collapse(30, Some(&forced), Some(&cfg()), "", &labeled, &[]).threshold);
        let unlabeled = [marker(MarkerType::Mark, 20, 22, "")];
        assert!(resolve_collapse(30, None, Some(&cfg()), "", &unlabeled, &[]).threshold);
    }

    #[test]
    fn preview_segments_and_cap() {
        assert_eq!(
            compute_preview_segments(8, 5, &[], &[]),
            (vec![PreviewSegment { start: 1, end: 5 }], 0)
        );
        assert_eq!(
            compute_preview_segments(8, 30, &[], &[]),
            (vec![PreviewSegment { start: 1, end: 8 }], 0)
        );
        let markers = [
            marker(MarkerType::Ins, 12, 12, ""),
            marker(MarkerType::Del, 20, 21, ""),
        ];
        let (segs, beyond) = compute_preview_segments(8, 30, &markers, &[LineRange::single(3)]);
        assert_eq!(
            segs,
            [
                PreviewSegment { start: 1, end: 8 },
                PreviewSegment { start: 11, end: 13 }
            ]
        );
        assert_eq!(beyond, 2);

        let adjacent = [
            marker(MarkerType::Mark, 9, 9, ""),
            marker(MarkerType::Mark, 11, 11, ""),
        ];
        let (segs, beyond) = compute_preview_segments(8, 30, &adjacent, &[]);
        assert_eq!(segs, [PreviewSegment { start: 1, end: 12 }]);
        assert_eq!(beyond, 0);

        let (segs, _) =
            compute_preview_segments(8, 16, &[marker(MarkerType::Mark, 16, 16, "")], &[]);
        assert_eq!(segs[1], PreviewSegment { start: 15, end: 16 });
    }

    #[test]
    fn range_validation() {
        let ranges = [
            LineRange::new(5, 3),
            LineRange::new(0, 2),
            LineRange::new(40, 50),
            LineRange::new(10, 20),
            LineRange::new(15, 18),
            LineRange::new(25, 99),
            LineRange::new(2, 4),
        ];
        assert_eq!(
            validate_ranges(&ranges, 30),
            [
                LineRange::new(2, 4),
                LineRange::new(10, 20),
                LineRange::new(25, 30)
            ]
        );
    }

    #[test]
    fn min_indent_ignores_blank_lines() {
        let code = "fn a() {\n    x\n\n      y\n  \n}";
        assert_eq!(compute_min_indent(code, 2, 5), 4);
        assert_eq!(compute_min_indent(code, 1, 6), 0);
        assert_eq!(compute_min_indent(code, 3, 3), 0);
        assert_eq!(compute_min_indent("\tz", 1, 1), 1);
    }

    #[test]
    fn style_resolution_and_precedence() {
        assert_eq!(
            resolve_collapse_style(CollapseStyle::CollapsibleAuto, 10, 10),
            CollapseStyle::CollapsibleEnd
        );
        assert_eq!(
            resolve_collapse_style(CollapseStyle::CollapsibleAuto, 9, 10),
            CollapseStyle::CollapsibleStart
        );
        assert_eq!(
            resolve_collapse_style(CollapseStyle::Github, 10, 10),
            CollapseStyle::Github
        );

        let code = "a\n  b\n  c\nd";
        let spec = CollapseSpec {
            ranges: vec![LineRange::new(2, 3)],
            ..Default::default()
        };
        let r = resolve_collapse(4, Some(&spec), None, code, &[], &[]);
        assert_eq!(
            r.ranges,
            [CollapseRange {
                start: 2,
                end: 3,
                line_count: 2,
                min_indent: 2,
                style: CollapseStyle::Github
            }]
        );

        let engine_style = CollapsibleConfig {
            style: CollapseStyle::CollapsibleAuto,
            ..Default::default()
        };
        let r = resolve_collapse(4, Some(&spec), Some(&engine_style), code, &[], &[]);
        assert_eq!(r.ranges[0].style, CollapseStyle::CollapsibleStart);
        let spec_style = CollapseSpec {
            ranges: vec![LineRange::new(2, 4)],
            style: Some(CollapseStyle::CollapsibleEnd),
            ..Default::default()
        };
        let r = resolve_collapse(4, Some(&spec_style), Some(&engine_style), code, &[], &[]);
        assert_eq!(r.ranges[0].style, CollapseStyle::CollapsibleEnd);
    }

    #[test]
    fn summary_text_uses_locale() {
        let s = UIStrings::default();
        assert_eq!(summary_text(1, &s), "1 collapsed line");
        assert_eq!(summary_text(7, &s), "7 collapsed lines");
    }
}
