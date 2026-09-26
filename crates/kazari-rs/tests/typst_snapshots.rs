#![cfg(feature = "irosashi")]

use kazari_rs::{InlineMarker, Kazari, MarkerType, Options};

fn engine() -> Kazari {
    let hl = irosashi::Highlighter::new().unwrap();
    Kazari::builder(hl)
        .themes("github-light", Some("github-dark"))
        .notation_comments(true)
        .build()
        .unwrap()
}

fn render(code: &str, meta: &str) -> String {
    engine().render_with_meta_typst(code, meta).unwrap()
}

#[test]
fn single_line() {
    insta::assert_snapshot!(render("let x = 1;", "javascript"));
}

#[test]
fn line_numbers_across_digit_boundary() {
    insta::assert_snapshot!(render(
        "package main\n\nfunc main() {\n}",
        "go showLineNumbers startLineNumber=98"
    ));
}

#[test]
fn line_markers_with_label() {
    insta::assert_snapshot!(render(
        "a\nb\nc\nd\ne\nf",
        r#"text {1} ins={2} del={3} {"API":4} error={5} warning={6}"#
    ));
}

#[test]
fn notation_error_and_warning_lines() {
    insta::assert_snapshot!(render(
        "x = 1 // [!code error]\ny = 2 // [!code warning]\nz = 3",
        "javascript"
    ));
}

#[test]
fn focus_lines_dim_the_rest() {
    insta::assert_snapshot!(render(
        "const a = 1;\nconst b = 2;\nconst c = 3;",
        "typescript focus={2}"
    ));
}

#[test]
fn title_from_meta() {
    insta::assert_snapshot!(render("fn main() {}", r#"rust title="app.rs""#));
}

#[test]
fn title_from_file_name_comment() {
    insta::assert_snapshot!(render("// main.go\npackage main", "go"));
}

#[test]
fn inline_markers() {
    let out = engine()
        .render_typst(
            "old_fn calls new_fn twice: new_fn()",
            &Options {
                lang: "text".into(),
                inline_markers: vec![
                    InlineMarker {
                        marker_type: MarkerType::Del,
                        text: "old_fn".into(),
                        is_regex: false,
                    },
                    InlineMarker {
                        marker_type: MarkerType::Ins,
                        text: "new_fn".into(),
                        is_regex: false,
                    },
                ],
                ..Default::default()
            },
        )
        .unwrap();
    insta::assert_snapshot!(out);
}

#[test]
fn bold_and_italic_tokens() {
    insta::assert_snapshot!(render("# Title\n\n**bold** and *italic*", "markdown"));
}

#[test]
fn string_escaping() {
    insta::assert_snapshot!(render(
        "s = \"a\\b\" # comment\n$x = 1 -- @me ~ 'q'\nu = \"http://x.y/z\"",
        "text"
    ));
}

#[test]
fn blank_lines_keep_height() {
    insta::assert_snapshot!(render("a\n\n   \nb", "text showLineNumbers"));
}

#[test]
fn diff_hybrid() {
    insta::assert_snapshot!(render(
        " func main() {\n-    old()\n+    new()\n }",
        r#"diff lang="go""#
    ));
}

#[test]
fn font_size_and_marker_palette_overrides() {
    let hl = irosashi::Highlighter::new().unwrap();
    let kz = Kazari::builder(hl)
        .themes("github-light", None)
        .typst_font("JetBrains Mono")
        .unwrap()
        .typst_size("10pt")
        .unwrap()
        .typst_marker_color("ins", "#c8f7d0")
        .unwrap()
        .typst_marker_color("mark", "#ffe58f")
        .unwrap()
        .build()
        .unwrap();
    insta::assert_snapshot!(
        kz.render_with_meta_typst("a\nb\nc word", r#"text ins={1} del={2} "word""#)
            .unwrap()
    );
}

#[test]
fn hanging_indent_and_preserved_indent() {
    insta::assert_snapshot!(render(
        "    indented line\nflush line",
        "text hangingIndent=2"
    ));
}

#[test]
fn string_tokens_option() {
    let hl = irosashi::Highlighter::new().unwrap();
    let kz = Kazari::builder(hl)
        .themes("github-light", None)
        .typst_tokens("string")
        .unwrap()
        .build()
        .unwrap();
    insta::assert_snapshot!(
        kz.render_with_meta_typst("let s = `a ${b}`;\nlet x = 1;", "javascript {2}")
            .unwrap()
    );
}

/// Inverse of Kazari's Typst string escaping, for checking string-form tokens.
fn unescape_typst_string(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('u') => {
                let hex: String = chars.by_ref().skip(1).take_while(|c| *c != '}').collect();
                out.push(char::from_u32(u32::from_str_radix(&hex, 16).unwrap()).unwrap());
            }
            Some(other) => out.push(other),
            None => {}
        }
    }
    out
}

#[test]
fn token_table_points_at_the_highlighted_code() {
    let cases = [
        (
            "<?php\n$app->get('/u/{id}', fn($r) => $r); // [!code highlight]\n\t$x = 1;",
            "php {3}",
        ),
        ("let s = `a ${b}`;\nconst é = \"ü\";", "javascript"),
        (
            "def greet(name):\n    print(f\"Hi {name}\")",
            r#"python "print" del=/greet/"#,
        ),
        (
            " func main() {\n-    old()\n+    new()\n }",
            r#"diff lang="go""#,
        ),
    ];
    for tokens in ["raw", "string"] {
        let hl = irosashi::Highlighter::new().unwrap();
        let kz = Kazari::builder(hl)
            .themes("github-light", None)
            .notation_comments(true)
            .typst_tokens(tokens)
            .unwrap()
            .build()
            .unwrap();
        for (code, meta) in cases {
            let block = kz.render_with_meta_typst_block(code, meta).unwrap();
            let lines: Vec<&str> = block.code.split('\n').collect();
            assert!(!block.tokens.is_empty(), "{meta}");
            for t in &block.tokens {
                let expected = &lines[t.line][t.column.clone()];
                let body = &block.typst[t.output.clone()];
                let got = if t.exact {
                    body.to_owned()
                } else {
                    unescape_typst_string(body)
                };
                assert_eq!(got, expected, "{tokens} {meta}: {t:?}");
            }
            let covered: usize = block.tokens.iter().map(|t| t.column.len()).sum();
            let non_blank: usize = lines
                .iter()
                .filter(|l| !l.trim().is_empty())
                .map(|l| l.len())
                .sum();
            assert_eq!(
                covered, non_blank,
                "{tokens} {meta}: every byte has one entry"
            );
            assert_eq!(
                block.tokens.iter().all(|t| t.exact),
                tokens == "raw" && !code.contains('`'),
                "{tokens} {meta}"
            );
        }
    }
}
