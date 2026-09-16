use std::fs;

use irosashi_fidelity::{
    FULL_MATRIX_HEADING, FidelityReport, golden_all_dir, golden_dir, held_all_path, held_path,
    highlighter, load_held, render_full_matrix, render_markdown, report_path, run_suite,
};

fn core_report() -> (FidelityReport, Vec<String>) {
    let highlighter = highlighter();
    run_suite(&highlighter, &golden_dir()).expect("core fixtures load")
}

fn held() -> Vec<String> {
    load_held(&held_path()).expect("held.toml parses")
}

fn held_all() -> Vec<String> {
    load_held(&held_all_path()).expect("held-all.toml parses")
}

fn diffs_to_show() -> usize {
    std::env::var("IRO_GOLDEN_ALL_DIFFS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10)
}

fn describe_failures(report: &FidelityReport, grammars: &[&str], limit: usize) -> String {
    let mut out = String::new();
    for result in &report.results {
        if result.pass() || !grammars.contains(&result.grammar.as_str()) {
            continue;
        }
        let shown: Vec<String> = result
            .diffs
            .iter()
            .take(limit)
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

/// Non-held failures and holds that now pass, for one report against one held list.
fn gate<'r>(report: &'r FidelityReport, held: &[String]) -> (Vec<&'r str>, Vec<String>) {
    let failing = report.failing_grammars();
    let regressions: Vec<&str> = failing
        .iter()
        .copied()
        .filter(|g| !held.iter().any(|h| h == g))
        .collect();
    let stale_holds: Vec<String> = held
        .iter()
        .filter(|h| !failing.contains(&h.as_str()))
        .cloned()
        .collect();
    (regressions, stale_holds)
}

/// Every core grammar not in `held.toml` must be byte-identical on every theme, and
/// every held grammar must still fail somewhere (otherwise un-hold it).
#[test]
fn core_gate() {
    let (report, _) = core_report();
    let (regressions, stale_holds) = gate(&report, &held());
    assert!(
        regressions.is_empty(),
        "non-held grammars are not identical: {regressions:?}{}",
        describe_failures(&report, &regressions, 10)
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
/// `IRO_WRITE_REPORT=1` to regenerate it. The full-matrix section needs the synced
/// `golden-all` fixtures; without them only the core section is checked.
#[test]
fn fidelity_report() {
    let (report, themes) = core_report();
    let mut markdown = render_markdown(&report, &themes);
    let all_dir = golden_all_dir();
    let has_all = all_dir.is_dir();
    if has_all {
        let highlighter = highlighter();
        let (all, all_themes) =
            run_suite(&highlighter, &all_dir).expect("golden-all fixtures load");
        markdown.push_str(&render_full_matrix(&all, &all_themes, &held_all()));
    }
    let path = report_path();
    if std::env::var_os("IRO_WRITE_REPORT").is_some() {
        assert!(
            has_all,
            "regenerating FIDELITY.md needs the golden-all fixtures (run sync-assets)"
        );
        fs::write(&path, &markdown).expect("write FIDELITY.md");
        println!("wrote {}", path.display());
        return;
    }
    let existing = fs::read_to_string(&path).unwrap_or_default();
    let core_only = |text: &str| {
        text.split(&format!("\n{FULL_MATRIX_HEADING}"))
            .next()
            .unwrap_or_default()
            .to_owned()
    };
    let (want, got) = if has_all {
        (existing.clone(), markdown.clone())
    } else {
        (core_only(&existing), core_only(&markdown))
    };
    assert!(
        want == got,
        "FIDELITY.md is stale or missing; run `IRO_WRITE_REPORT=1 cargo test -p irosashi-fidelity --test golden fidelity_report`"
    );
}

/// The full 234-grammar matrix, gated by `held-all.toml`. Skips when the fixtures are
/// not synced. `IRO_GOLDEN_ALL_DIFFS=N` controls how many diffs are shown per failing
/// triple.
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
    println!("failing: {failing:?}");
    println!("{}", describe_failures(&report, &failing, diffs_to_show()));
    let (regressions, stale_holds) = gate(&report, &held_all());
    assert!(
        regressions.is_empty(),
        "non-held grammars are not identical: {regressions:?}"
    );
    assert!(
        stale_holds.is_empty(),
        "held grammars now pass; remove them from held-all.toml: {stale_holds:?}"
    );
}
