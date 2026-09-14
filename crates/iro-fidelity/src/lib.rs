//! Fixture loader, comparator, scoring and report for checking `iro` against
//! vscode-textmate goldens.

pub mod compare;
pub mod fixture;
pub mod held;
pub mod report;
pub mod score;

use std::path::{Path, PathBuf};

use iro::{CodeToTokensOptions, Highlighter, HighlighterBuilder};

pub use compare::{DiffKind, TokenDiff, compare_theme_tokens, normalize_color};
pub use fixture::{Fixture, FixtureToken, ThemeFixture, load_fixture, load_fixtures};
pub use held::{load_held, parse_held};
pub use report::render_markdown;
pub use score::{FidelityReport, Score, TripleResult, compute_report};

/// Path of the `iro` crate's `assets/` directory.
pub fn assets_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../iro/assets")
}

/// Path of the core golden fixtures.
pub fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/golden")
}

/// Path of the full 234-grammar fixture matrix (synced locally, not committed).
pub fn golden_all_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/golden-all")
}

pub fn held_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("held.toml")
}

/// Path of the report at the repository root.
pub fn report_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../FIDELITY.md")
}

/// A highlighter over the crate's asset directory, as the fixtures expect
/// (no contrast correction, no line guards).
pub fn highlighter() -> Highlighter {
    HighlighterBuilder::from_dir(assets_dir())
        .build()
        .expect("assets directory loads")
}

/// Tokenizes the fixture's source through the public API and converts the result
/// into fixture tokens, so it can be compared with the expected matrix.
pub fn actual_tokens(
    highlighter: &Highlighter,
    fixture: &Fixture,
    theme_name: &str,
) -> Result<Vec<Vec<FixtureToken>>, iro::Error> {
    let mut options = CodeToTokensOptions::new(&fixture.grammar, theme_name);
    options.include_scopes = true;
    let result = highlighter.code_to_tokens(&fixture.source, &options)?;
    let lines = result
        .lines
        .iter()
        .map(|line| {
            let text = result.line_text(line);
            line.tokens
                .iter()
                .map(|token| FixtureToken {
                    start: token.start,
                    end: token.end,
                    text: token.text(text).to_owned(),
                    scopes: result
                        .scopes_of(token)
                        .map(|s| s.iter().map(|n| n.to_string()).collect())
                        .unwrap_or_default(),
                    color: token
                        .style
                        .color
                        .map(|c| result.color(c).to_owned())
                        .unwrap_or_default(),
                    font_style: token.style.font_style.bits(),
                })
                .collect()
        })
        .collect();
    Ok(lines)
}

pub fn run_triple(highlighter: &Highlighter, fixture: &Fixture, theme_name: &str) -> TripleResult {
    let expected = fixture
        .themes
        .get(theme_name)
        .map(|t| t.tokens.as_slice())
        .unwrap_or(&[]);
    let diffs = match actual_tokens(highlighter, fixture, theme_name) {
        Ok(actual) => compare_theme_tokens(expected, &actual),
        Err(err) => vec![TokenDiff {
            line: 0,
            kind: DiffKind::MissingToken,
            want: None,
            got: None,
            detail: format!("tokenization failed: {err}"),
        }],
    };
    TripleResult {
        grammar: fixture.grammar.clone(),
        theme: theme_name.to_owned(),
        diffs,
    }
}

/// Runs every fixture in `dir` against every theme it carries. Returns the report and
/// the sorted theme names seen.
pub fn run_suite(
    highlighter: &Highlighter,
    dir: &Path,
) -> std::io::Result<(FidelityReport, Vec<String>)> {
    let fixtures = load_fixtures(dir)?;
    let mut results = Vec::new();
    let mut themes = std::collections::BTreeSet::new();
    for fixture in &fixtures {
        for theme in fixture.themes.keys() {
            themes.insert(theme.clone());
            results.push(run_triple(highlighter, fixture, theme));
        }
    }
    Ok((compute_report(results), themes.into_iter().collect()))
}
