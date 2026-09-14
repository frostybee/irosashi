use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use iro::{CodeToHtmlOptions, DefaultColor, Highlighter, HighlighterBuilder};
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
    theme: Option<String>,
    themes: Option<BTreeMap<String, String>>,
    #[serde(default)]
    default_color: Option<serde_json::Value>,
    merge_whitespace: Option<bool>,
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn highlighter() -> Highlighter {
    HighlighterBuilder::from_dir(root().join("assets"))
        .build()
        .unwrap()
}

fn options(case: &Case) -> CodeToHtmlOptions {
    let mut options = match (&case.theme, &case.themes) {
        (Some(theme), None) => CodeToHtmlOptions::new(&case.lang, theme),
        (None, Some(themes)) => CodeToHtmlOptions::multi(&case.lang, themes.clone()),
        _ => panic!("{}: exactly one of theme or themes", case.name),
    }
    .shiki();
    match &case.default_color {
        None => {}
        Some(serde_json::Value::Bool(false)) => options.html.default_color = DefaultColor::Off,
        Some(serde_json::Value::String(key)) => {
            options.html.default_color = DefaultColor::Key(key.clone());
        }
        Some(other) => panic!("{}: unsupported default_color {other}", case.name),
    }
    if let Some(merge) = case.merge_whitespace {
        options.html.merge_whitespace = merge;
    }
    options
}

fn first_difference(expected: &str, actual: &str) -> String {
    let at = expected
        .bytes()
        .zip(actual.bytes())
        .position(|(e, a)| e != a)
        .unwrap_or(expected.len().min(actual.len()));
    let window = |s: &str| {
        let start = s.floor_char_boundary(at.saturating_sub(40));
        let end = s.ceil_char_boundary((at + 40).min(s.len()));
        s[start..end].to_owned()
    };
    format!(
        "first difference at byte {at}\nexpected: {:?}\nactual:   {:?}",
        window(expected),
        window(actual)
    )
}

#[test]
fn shiki_goldens_are_byte_identical() {
    let dir = root().join("testdata/html");
    let manifest: Manifest =
        serde_json::from_slice(&fs::read(dir.join("manifest.json")).unwrap()).unwrap();
    assert!(!manifest.cases.is_empty());
    let h = highlighter();
    let actual_dir = root().join("../../target/html-goldens");
    let mut failures = Vec::new();
    for case in &manifest.cases {
        let expected = fs::read_to_string(dir.join(format!("{}.html", case.name))).unwrap();
        let actual = h.code_to_html(&case.source, &options(case)).unwrap();
        if actual != expected {
            fs::create_dir_all(&actual_dir).unwrap();
            fs::write(
                actual_dir.join(format!("{}.actual.html", case.name)),
                &actual,
            )
            .unwrap();
            failures.push(format!(
                "{}: {}",
                case.name,
                first_difference(&expected, &actual)
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} goldens differ:\n{}",
        failures.len(),
        manifest.cases.len(),
        failures.join("\n\n")
    );
}
