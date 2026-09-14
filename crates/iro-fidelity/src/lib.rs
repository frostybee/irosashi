//! Fixture loader and comparator for checking `iro` against vscode-textmate goldens.

pub mod compare;
pub mod fixture;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use iro::{Registry, Session, TokenizeOptions};

pub use compare::{DiffKind, TokenDiff, compare_theme_tokens, normalize_color};
pub use fixture::{Fixture, FixtureToken, ThemeFixture, load_fixture, load_fixtures};

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

/// Path of the workspace `assets/` directory.
pub fn assets_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets")
}

/// Path of the core golden fixtures.
pub fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/golden")
}

/// Tokenizes the fixture's source with `iro` and converts the result into fixture
/// tokens, so it can be compared with the expected matrix.
pub fn actual_tokens(
    registry: &Arc<Registry>,
    fixture: &Fixture,
    theme_name: &str,
) -> Result<Vec<Vec<FixtureToken>>, iro::Error> {
    let grammar = registry.grammar(&fixture.grammar)?;
    let theme = registry.theme(theme_name)?;
    let resolver: Arc<dyn iro::tokenizer::Resolver> =
        Arc::clone(registry) as Arc<dyn iro::tokenizer::Resolver>;
    let mut session = Session::new(grammar, resolver, TokenizeOptions::default());
    let result = session.themed(&fixture.source, &theme);
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
                    scopes: session.scopes_vec(token.scopes),
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

pub fn run_triple(registry: &Arc<Registry>, fixture: &Fixture, theme_name: &str) -> TripleResult {
    let expected = fixture
        .themes
        .get(theme_name)
        .map(|t| t.tokens.as_slice())
        .unwrap_or(&[]);
    let diffs = match actual_tokens(registry, fixture, theme_name) {
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
