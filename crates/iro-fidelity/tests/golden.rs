use std::fs;

use iro_fidelity::{
    FidelityReport, golden_all_dir, golden_dir, held_path, highlighter, load_held, render_markdown,
    report_path, run_suite,
};

fn core_report() -> (FidelityReport, Vec<String>) {
    let highlighter = highlighter();
    run_suite(&highlighter, &golden_dir()).expect("core fixtures load")
}

fn held() -> Vec<String> {
    load_held(&held_path()).expect("held.toml parses")
}

fn describe_failures(report: &FidelityReport, grammars: &[&str]) -> String {
    let mut out = String::new();
    for result in &report.results {
        if result.pass() || !grammars.contains(&result.grammar.as_str()) {
            continue;
        }
        let shown: Vec<String> = result
            .diffs
            .iter()
            .take(10)
            .map(|d| d.to_string())
            .collect();
        out.push_str(&format!(
            "\n{} x {}: {} diffs\n{}\n",
            result.grammar,
            result.theme,
            result.diffs.len(),
            shown.join("\n")
        ));
    }
    out
}

/// Every core grammar not in `held.toml` must be byte-identical on every theme, and
/// every held grammar must still fail somewhere (otherwise un-hold it).
#[test]
fn core_gate() {
    let (report, _) = core_report();
    let held = held();
    let failing = report.failing_grammars();
    let regressions: Vec<&str> = failing
        .iter()
        .copied()
        .filter(|g| !held.iter().any(|h| h == g))
        .collect();
    let stale_holds: Vec<&String> = held
        .iter()
        .filter(|h| !failing.contains(&h.as_str()))
        .collect();
    assert!(
        regressions.is_empty(),
        "non-held grammars are not identical: {regressions:?}{}",
        describe_failures(&report, &regressions)
    );
    assert!(
        stale_holds.is_empty(),
        "held grammars now pass; remove them from held.toml: {stale_holds:?}"
    );
    println!(
        "core: {}/{} triples identical",
        report.global.pass, report.global.total
    );
}

/// The core set ships only with an empty held list.
#[test]
fn core_ships_green() {
    let held = held();
    let (report, _) = core_report();
    let core: Vec<&str> = report.by_grammar.keys().map(String::as_str).collect();
    let held_core: Vec<&String> = held.iter().filter(|h| core.contains(&h.as_str())).collect();
    assert!(
        held_core.is_empty(),
        "core grammars are held (fidelity < 100%): {held_core:?}"
    );
}

/// `FIDELITY.md` must match the current results byte for byte. Set
/// `IRO_WRITE_REPORT=1` to regenerate it.
#[test]
fn fidelity_report() {
    let (report, themes) = core_report();
    let markdown = render_markdown(&report, &themes);
    let path = report_path();
    if std::env::var_os("IRO_WRITE_REPORT").is_some() {
        fs::write(&path, &markdown).expect("write FIDELITY.md");
        println!("wrote {}", path.display());
        return;
    }
    let existing = fs::read_to_string(&path).unwrap_or_default();
    assert!(
        existing == markdown,
        "FIDELITY.md is stale or missing; run `IRO_WRITE_REPORT=1 cargo test -p iro-fidelity --test golden fidelity_report`"
    );
}

/// Informational: the full 234-grammar matrix. Skips when the fixtures are not
/// synced; prints the score and the grammars that would be held.
#[test]
#[ignore]
fn golden_all() {
    let dir = golden_all_dir();
    if !dir.is_dir() {
        println!("golden-all not synced at {}; skipping", dir.display());
        return;
    }
    let highlighter = highlighter();
    let (report, themes) = run_suite(&highlighter, &dir).expect("golden-all fixtures load");
    let failing = report.failing_grammars();
    println!(
        "golden-all: {}/{} triples identical, {}/{} grammars, themes {themes:?}",
        report.global.pass,
        report.global.total,
        report.by_grammar.len() - failing.len(),
        report.by_grammar.len()
    );
    println!("would be held: {failing:?}");
    for grammar in &failing {
        let first = report
            .results
            .iter()
            .find(|r| r.grammar == *grammar && !r.pass())
            .and_then(|r| r.diffs.first())
            .map(|d| d.to_string())
            .unwrap_or_default();
        println!("  {grammar}: {first}");
    }
}
