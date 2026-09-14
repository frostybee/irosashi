use std::fmt::Write;

use crate::score::FidelityReport;

/// Renders the `FIDELITY.md` table. The format matches Nuri's report byte for byte
/// so the two files are directly comparable.
pub fn render_markdown(report: &FidelityReport, themes: &[String]) -> String {
    let mut out = String::new();
    out.push_str("# Fidelity Report\n\n");
    let _ = writeln!(
        out,
        "**Overall**: {} / {} pass ({:.1}%)\n",
        report.global.pass,
        report.global.total,
        report.global.rate() * 100.0
    );

    out.push_str("| Grammar |");
    for theme in themes {
        let _ = write!(out, " {theme} |");
    }
    out.push_str(" Status |\n");

    out.push_str("|---------|");
    for _ in themes {
        out.push_str(":---:|");
    }
    out.push_str("----------|\n");

    for grammar in report.by_grammar.keys() {
        let _ = write!(out, "| {grammar} |");
        let mut all_pass = true;
        for theme in themes {
            if report.triple_pass(grammar, theme) {
                out.push_str(" ✅ |");
            } else {
                out.push_str(" ❌ |");
                all_pass = false;
            }
        }
        out.push_str(if all_pass {
            " shipping |\n"
        } else {
            " held |\n"
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compare::{DiffKind, TokenDiff};
    use crate::score::{TripleResult, compute_report};

    #[test]
    fn matches_nuri_format() {
        let fail = vec![TokenDiff {
            line: 0,
            kind: DiffKind::ScopeMismatch,
            want: None,
            got: None,
            detail: String::new(),
        }];
        let report = compute_report(vec![
            TripleResult {
                grammar: "go".into(),
                theme: "github-dark".into(),
                diffs: Vec::new(),
            },
            TripleResult {
                grammar: "go".into(),
                theme: "github-light".into(),
                diffs: Vec::new(),
            },
            TripleResult {
                grammar: "bat".into(),
                theme: "github-dark".into(),
                diffs: fail,
            },
            TripleResult {
                grammar: "bat".into(),
                theme: "github-light".into(),
                diffs: Vec::new(),
            },
        ]);
        let themes = vec!["github-dark".to_owned(), "github-light".to_owned()];
        let expected = "# Fidelity Report\n\n\
**Overall**: 3 / 4 pass (75.0%)\n\n\
| Grammar | github-dark | github-light | Status |\n\
|---------|:---:|:---:|----------|\n\
| bat | ❌ | ✅ | held |\n\
| go | ✅ | ✅ | shipping |\n";
        assert_eq!(render_markdown(&report, &themes), expected);
    }
}
