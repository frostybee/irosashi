use crate::theme::{Theme, TokenColor, TokenSettings};

/// A theme selector split into its parent parts, compiled once at theme load.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CompiledSelector {
    parts: Vec<String>,
    scope_depth: usize,
}

impl CompiledSelector {
    pub(crate) fn new(selector: &str) -> Self {
        let parts: Vec<String> = selector.split_whitespace().map(str::to_owned).collect();
        let scope_depth = parts
            .last()
            .map(|last| last.matches('.').count() + 1)
            .unwrap_or(0);
        Self { parts, scope_depth }
    }

    pub(crate) fn score(&self, scopes: &[&str]) -> Option<MatchScore> {
        if self.parts.is_empty() {
            return None;
        }
        let last = self.parts.len() - 1;
        let mut part_idx = last as isize;
        let mut depth = 0;
        for (stack_idx, scope) in scopes.iter().enumerate().rev() {
            if part_idx < 0 {
                break;
            }
            if scope_prefix_match(&self.parts[part_idx as usize], scope) {
                if part_idx as usize == last {
                    depth = stack_idx;
                }
                part_idx -= 1;
            }
        }
        if part_idx >= 0 {
            return None;
        }
        Some(MatchScore {
            depth,
            scope_depth: self.scope_depth,
            parents: self.parts.len(),
        })
    }
}

/// Specificity of a selector match. Compared lexicographically: deeper stack index of
/// the target scope, then more dot segments in the target, then more parent parts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MatchScore {
    pub depth: usize,
    pub scope_depth: usize,
    pub parents: usize,
}

impl MatchScore {
    pub fn greater_than(self, other: MatchScore) -> bool {
        if self.depth != other.depth {
            return self.depth > other.depth;
        }
        if self.scope_depth != other.scope_depth {
            return self.scope_depth > other.scope_depth;
        }
        self.parents > other.parents
    }
}

/// `selector` equals `scope` or is a prefix of it ending on a dot boundary.
pub fn scope_prefix_match(selector: &str, scope: &str) -> bool {
    scope == selector
        || (scope.len() > selector.len()
            && scope.starts_with(selector)
            && scope.as_bytes()[selector.len()] == b'.')
}

fn best_selector_score(rule: &TokenColor, scopes: &[&str]) -> Option<MatchScore> {
    let mut best: Option<MatchScore> = None;
    for selector in &rule.selectors {
        if let Some(score) = selector.score(scopes) {
            match best {
                Some(current) if !score.greater_than(current) => {}
                _ => best = Some(score),
            }
        }
    }
    best
}

impl Theme {
    /// Resolves foreground, background and font style independently, each from the
    /// highest-scoring rule that sets it. Ties keep the earlier rule.
    pub fn resolve(&self, scopes: &[&str]) -> TokenSettings {
        let mut result = TokenSettings::default();
        let mut fg_score = MatchScore::default();
        let mut bg_score = MatchScore::default();
        let mut fs_score = MatchScore::default();

        for rule in &self.token_colors {
            let Some(score) = best_selector_score(rule, scopes) else {
                continue;
            };
            let s = rule.settings;
            if s.foreground.is_some()
                && (result.foreground.is_none() || score.greater_than(fg_score))
            {
                result.foreground = s.foreground;
                fg_score = score;
            }
            if s.background.is_some()
                && (result.background.is_none() || score.greater_than(bg_score))
            {
                result.background = s.background;
                bg_score = score;
            }
            if s.font_style.is_some()
                && (result.font_style.is_none() || score.greater_than(fs_score))
            {
                result.font_style = s.font_style;
                fs_score = score;
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::FontStyle;

    fn theme(json: &str) -> Theme {
        Theme::parse(json.as_bytes()).expect("theme parses")
    }

    fn fg(theme: &Theme, scopes: &[&str]) -> Option<String> {
        theme
            .resolve(scopes)
            .foreground
            .map(|id| theme.color(id).to_owned())
    }

    #[test]
    fn prefix_match_requires_dot_boundary() {
        assert!(scope_prefix_match("keyword", "keyword"));
        assert!(scope_prefix_match("keyword", "keyword.control.go"));
        assert!(!scope_prefix_match("keyword", "keywordx"));
        assert!(!scope_prefix_match("keyword", "keywords.other"));
        assert!(!scope_prefix_match("keyword.control", "keyword"));
    }

    #[test]
    fn deeper_stack_index_beats_longer_selector() {
        let t = theme(
            r##"{"tokenColors": [
                {"scope": "meta.function.name", "settings": {"foreground": "#111111"}},
                {"scope": "entity", "settings": {"foreground": "#222222"}}
            ]}"##,
        );
        assert_eq!(
            fg(&t, &["source.go", "meta.function.name", "entity.name"]).as_deref(),
            Some("#222222")
        );
    }

    #[test]
    fn scope_depth_then_parents_break_ties() {
        let t = theme(
            r##"{"tokenColors": [
                {"scope": "entity", "settings": {"foreground": "#111111"}},
                {"scope": "entity.name", "settings": {"foreground": "#222222"}},
                {"scope": "source.go entity.name", "settings": {"foreground": "#333333"}},
                {"scope": "source.js entity.name", "settings": {"foreground": "#444444"}}
            ]}"##,
        );
        assert_eq!(
            fg(&t, &["source.go", "entity.name.function"]).as_deref(),
            Some("#333333")
        );
        assert_eq!(
            fg(&t, &["source.py", "entity.name.function"]).as_deref(),
            Some("#222222")
        );
    }

    #[test]
    fn parent_parts_match_as_subsequence_right_to_left() {
        let t = theme(
            r##"{"tokenColors": [
                {"scope": "source string", "settings": {"foreground": "#111111"}}
            ]}"##,
        );
        assert_eq!(
            fg(&t, &["source.go", "meta.block", "string.quoted"]).as_deref(),
            Some("#111111")
        );
        assert_eq!(fg(&t, &["string.quoted", "source.go"]), None);
    }

    #[test]
    fn identical_scores_keep_the_first_rule() {
        let t = theme(
            r##"{"tokenColors": [
                {"scope": "keyword", "settings": {"foreground": "#111111"}},
                {"scope": "keyword", "settings": {"foreground": "#222222"}}
            ]}"##,
        );
        assert_eq!(fg(&t, &["keyword.control"]).as_deref(), Some("#111111"));
    }

    #[test]
    fn properties_resolve_independently() {
        let t = theme(
            r##"{"tokenColors": [
                {"scope": "comment", "settings": {"fontStyle": "italic"}},
                {"scope": "comment.line", "settings": {"foreground": "#111111"}}
            ]}"##,
        );
        let settings = t.resolve(&["comment.line.double-slash"]);
        assert_eq!(t.color(settings.foreground.unwrap()), "#111111");
        assert_eq!(settings.font_style, Some(FontStyle::ITALIC));
        assert_eq!(settings.background, None);
    }

    #[test]
    fn empty_font_style_resets_and_absent_inherits() {
        let t = theme(
            r##"{"tokenColors": [
                {"scope": "markup", "settings": {"fontStyle": "bold"}},
                {"scope": "markup.plain", "settings": {"fontStyle": ""}},
                {"scope": "markup.heading", "settings": {"foreground": "#111111"}}
            ]}"##,
        );
        assert_eq!(
            t.resolve(&["markup.plain"]).font_style,
            Some(FontStyle::empty())
        );
        assert_eq!(
            t.resolve(&["markup.heading"]).font_style,
            Some(FontStyle::BOLD)
        );
        assert_eq!(t.resolve(&["other"]).font_style, None);
    }

    #[test]
    fn multi_scope_rule_uses_its_best_selector() {
        let t = theme(
            r##"{"tokenColors": [
                {"scope": ["string", "string.quoted.double"], "settings": {"foreground": "#111111"}},
                {"scope": "string.quoted", "settings": {"foreground": "#222222"}}
            ]}"##,
        );
        assert_eq!(
            fg(&t, &["string.quoted.double"]).as_deref(),
            Some("#111111")
        );
        assert_eq!(
            fg(&t, &["string.quoted.single"]).as_deref(),
            Some("#222222")
        );
    }
}
