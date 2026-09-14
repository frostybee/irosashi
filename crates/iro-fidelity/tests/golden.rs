use std::sync::Arc;

use iro::Registry;
use iro_fidelity::{assets_dir, golden_dir, load_fixture, run_triple};

fn registry() -> Arc<Registry> {
    Arc::new(Registry::from_dir(&assets_dir()).expect("assets directory loads"))
}

fn assert_identical(grammar: &str) {
    let registry = registry();
    let fixture = load_fixture(&golden_dir().join(format!("{grammar}__{grammar}.json")))
        .expect("fixture loads");
    let mut failures = Vec::new();
    for theme in fixture.themes.keys() {
        let result = run_triple(&registry, &fixture, theme);
        if !result.pass() {
            let shown: Vec<String> = result
                .diffs
                .iter()
                .take(10)
                .map(|d| d.to_string())
                .collect();
            failures.push(format!(
                "{grammar} x {theme}: {} diffs\n{}",
                result.diffs.len(),
                shown.join("\n")
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn json_core_themes_identical() {
    assert_identical("json");
}

/// Informational: pass/fail per core grammar and theme, with diff counts by kind.
/// The gate over all 32 grammars arrives with the held list in the next phase.
#[test]
#[ignore]
fn core_matrix_report() {
    use std::collections::BTreeMap;

    let registry = registry();
    let fixtures = iro_fidelity::load_fixtures(&golden_dir()).expect("fixtures load");
    let mut passed = 0;
    let mut total = 0;
    for fixture in &fixtures {
        for theme in fixture.themes.keys() {
            let result = run_triple(&registry, fixture, theme);
            total += 1;
            if result.pass() {
                passed += 1;
                println!("PASS {} x {theme}", fixture.grammar);
            } else {
                let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
                for diff in &result.diffs {
                    *kinds.entry(diff.kind.to_string()).or_default() += 1;
                }
                let first = result
                    .diffs
                    .first()
                    .map(|d| d.to_string())
                    .unwrap_or_default();
                println!(
                    "FAIL {} x {theme}: {} diffs {kinds:?}\n  first: {first}",
                    fixture.grammar,
                    result.diffs.len()
                );
            }
        }
    }
    println!("core matrix: {passed}/{total} triples identical");
}
