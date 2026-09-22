//! Pins the decorated output of the syntect backend. The bundled syntaxes and themes
//! are part of the pinned syntect release, so these are deterministic.

#![cfg(feature = "syntect")]

use kazari_rs::Kazari;
use kazari_rs::backends::syntect::SyntectHighlighter;

fn engine() -> Kazari {
    Kazari::builder(SyntectHighlighter::new())
        .themes("github-light", Some("github-dark"))
        .minify(false)
        .notation_comments(true)
        .build()
        .unwrap()
}

const RUST: &str = "fn main() {\n    let x = 1; // [!code highlight]\n    println!(\"{x}\");\n}\n";

#[test]
fn html_dual_theme_with_markers() {
    insta::assert_snapshot!(
        engine()
            .render_with_meta(RUST, r#"rust title="main.rs" showLineNumbers ins={3}"#)
            .unwrap()
    );
}

#[test]
fn html_unknown_language_is_plain() {
    insta::assert_snapshot!(engine().render_with_meta("a b\nc", "no-such-lang").unwrap());
}

#[test]
fn typst_single_theme() {
    insta::assert_snapshot!(
        engine()
            .render_with_meta_typst(RUST, "rust showLineNumbers {2}")
            .unwrap()
    );
}
