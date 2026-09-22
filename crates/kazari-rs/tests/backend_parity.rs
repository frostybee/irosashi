//! Both backends must agree on the shape of a block: the line count and the
//! text of every line, so markers, line numbers and diffs land on the same lines.

#![cfg(all(feature = "irosashi", feature = "syntect"))]

use kazari_rs::Highlighter;
use kazari_rs::backends::syntect::SyntectHighlighter;

const SAMPLES: &[(&str, &str)] = &[
    ("rust", "fn main() {\n    println!(\"hi\");\n}\n"),
    ("python", "def f(x):\n\treturn x  # tab indented\n"),
    ("no-such-lang", "plain\n\ntext"),
    ("ansi", "\u{1b}[31mred\u{1b}[0m plain\nsecond"),
    ("javascript", "const a = 1;\r\nconst b = 2;\r\n"),
    ("rust", "no trailing newline"),
    ("rust", ""),
    ("rust", "\n\n"),
];

#[test]
fn line_counts_and_texts_agree() {
    let iro = irosashi::Highlighter::new().unwrap();
    let syn = SyntectHighlighter::new();
    for (lang, code) in SAMPLES {
        let a = iro.tokenize(code, lang, "github-light", None).unwrap();
        let b = syn.tokenize(code, lang, "github-light", None).unwrap();
        assert_eq!(a.lines.len(), b.lines.len(), "{lang}: {code:?}");
        for (i, (la, lb)) in a.lines.iter().zip(&b.lines).enumerate() {
            if *lang == "ansi" {
                continue;
            }
            assert_eq!(la.text, lb.text, "{lang} line {i}: {code:?}");
            let ja: String = la.tokens.iter().map(|t| t.text(&la.text)).collect();
            let jb: String = lb.tokens.iter().map(|t| t.text(&lb.text)).collect();
            assert_eq!(
                ja, la.text,
                "{lang} line {i}: irosashi tokens tile the line"
            );
            assert_eq!(jb, lb.text, "{lang} line {i}: syntect tokens tile the line");
        }
    }
}

#[test]
fn dual_themes_agree_on_shape() {
    let iro = irosashi::Highlighter::new().unwrap();
    let syn = SyntectHighlighter::new();
    let code = "let x = 1;\nlet y = 2;\n";
    let a = iro
        .tokenize(code, "rust", "github-light", Some("github-dark"))
        .unwrap();
    let b = syn
        .tokenize(code, "rust", "github-light", Some("github-dark"))
        .unwrap();
    assert!(a.dark.is_some() && b.dark.is_some());
    assert_eq!(a.lines.len(), b.lines.len());
    for out in [&a, &b] {
        for line in &out.lines {
            assert!(line.tokens.iter().all(|t| t.dark.is_some()));
        }
    }
}
