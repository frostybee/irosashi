use kazari::{InlineMarker, Kazari, MarkerType, Options};

fn engine() -> Kazari {
    let hl = iro::Highlighter::new().unwrap();
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
fn hanging_indent_and_preserved_indent() {
    insta::assert_snapshot!(render(
        "    indented line\nflush line",
        "text hangingIndent=2"
    ));
}
