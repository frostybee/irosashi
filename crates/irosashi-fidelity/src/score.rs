use std::collections::BTreeMap;

use crate::compare::TokenDiff;

/// The outcome of one grammar x theme comparison.
#[derive(Debug, Clone)]
pub struct TripleResult {
    pub grammar: String,
    pub theme: String,
    pub diffs: Vec<TokenDiff>,
}

impl TripleResult {
    pub fn pass(&self) -> bool {
        self.diffs.is_empty()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Score {
    pub pass: usize,
    pub total: usize,
}

impl Score {
    pub fn rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.pass as f64 / self.total as f64
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct FidelityReport {
    pub results: Vec<TripleResult>,
    pub by_grammar: BTreeMap<String, Score>,
    pub by_theme: BTreeMap<String, Score>,
    pub global: Score,
}

impl FidelityReport {
    pub fn triple_pass(&self, grammar: &str, theme: &str) -> bool {
        self.results
            .iter()
            .find(|r| r.grammar == grammar && r.theme == theme)
            .is_some_and(TripleResult::pass)
    }

    /// Grammars with at least one failing theme, sorted.
    pub fn failing_grammars(&self) -> Vec<&str> {
        self.by_grammar
            .iter()
            .filter(|(_, score)| score.pass < score.total)
            .map(|(name, _)| name.as_str())
            .collect()
    }
}

pub fn compute_report(results: Vec<TripleResult>) -> FidelityReport {
    let mut report = FidelityReport::default();
    for result in &results {
        let pass = result.pass();
        report.global.total += 1;
        let by_grammar = report.by_grammar.entry(result.grammar.clone()).or_default();
        by_grammar.total += 1;
        let by_theme = report.by_theme.entry(result.theme.clone()).or_default();
        by_theme.total += 1;
        if pass {
            report.global.pass += 1;
            by_grammar.pass += 1;
            by_theme.pass += 1;
        }
    }
    report.results = results;
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compare::DiffKind;

    fn triple(grammar: &str, theme: &str, pass: bool) -> TripleResult {
        TripleResult {
            grammar: grammar.to_owned(),
            theme: theme.to_owned(),
            diffs: if pass {
                Vec::new()
            } else {
                vec![TokenDiff {
                    line: 0,
                    kind: DiffKind::ExtraToken,
                    want: None,
                    got: None,
                    detail: String::new(),
                }]
            },
        }
    }

    #[test]
    fn counts_per_grammar_theme_and_global() {
        let report = compute_report(vec![
            triple("go", "dark", true),
            triple("go", "light", false),
            triple("json", "dark", true),
            triple("json", "light", true),
        ]);
        assert_eq!(report.global, Score { pass: 3, total: 4 });
        assert_eq!(report.by_grammar["go"], Score { pass: 1, total: 2 });
        assert_eq!(report.by_theme["light"], Score { pass: 1, total: 2 });
        assert_eq!(report.failing_grammars(), ["go"]);
        assert!(report.triple_pass("json", "dark"));
        assert!(!report.triple_pass("go", "light"));
        assert!(!report.triple_pass("missing", "dark"));
        assert_eq!(Score::default().rate(), 0.0);
    }
}
