use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use irosashi::{CodeToHtmlOptions, Highlighter, HighlighterBuilder};
use serde::Deserialize;

#[derive(Deserialize)]
struct Manifest {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    name: String,
    lang: String,
    source: String,
}

fn highlighter() -> Highlighter {
    HighlighterBuilder::from_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("assets"))
        .build()
        .unwrap()
}

/// One source per golden case, taken from the Shiki golden manifest so both suites
/// see the same input. Yields (snapshot base name, language, source).
fn sources() -> Vec<(String, String, String)> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/html/manifest.json");
    let manifest: Manifest = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    manifest
        .cases
        .into_iter()
        .filter_map(|c| {
            let base = c.name.strip_suffix("__github-dark")?.to_owned();
            Some((base, c.lang, c.source))
        })
        .collect()
}

#[test]
fn iro_dialect_snapshots() {
    let h = highlighter();
    let dual = BTreeMap::from([
        ("light".to_owned(), "github-light".to_owned()),
        ("dark".to_owned(), "github-dark".to_owned()),
    ]);
    for (base, lang, source) in sources() {
        let single = h
            .code_to_html(&source, &CodeToHtmlOptions::new(&lang, "github-dark"))
            .unwrap();
        insta::assert_snapshot!(format!("{base}__github-dark"), single);
        let multi = h
            .code_to_html(&source, &CodeToHtmlOptions::multi(&lang, dual.clone()))
            .unwrap();
        insta::assert_snapshot!(format!("{base}__themes"), multi);
    }
}
