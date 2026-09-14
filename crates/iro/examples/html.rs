//! Renders a snippet as a standalone HTML page in both dialects and a dual-theme block.
//! `cargo run -p iro --example html > demo.html`, then open the file in a browser.

use std::collections::BTreeMap;

use iro::{CodeToHtmlOptions, DefaultColor, Highlighter};

const CODE: &str = r#"use std::collections::HashMap;

/// Counts words, case-insensitively.
fn word_counts(text: &str) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for word in text.split_whitespace() {
        *counts.entry(word.to_lowercase()).or_insert(0) += 1;
    }
    counts
}

fn main() {
    let counts = word_counts("the quick brown fox jumps over the lazy dog <the end>");
    println!("{:?}", counts.get("the")); // Some(3)
}
"#;

fn main() -> Result<(), iro::Error> {
    let highlighter = Highlighter::new()?;

    let single = highlighter.code_to_html(CODE, &CodeToHtmlOptions::new("rust", "github-dark"))?;

    let themes = BTreeMap::from([
        ("dark".to_owned(), "github-dark".to_owned()),
        ("light".to_owned(), "github-light".to_owned()),
    ]);
    let mut dual = CodeToHtmlOptions::multi("rust", themes);
    dual.html.default_color = DefaultColor::Key("light".to_owned());
    let dual = highlighter.code_to_html(CODE, &dual)?;

    let shiki = highlighter.code_to_html(
        CODE,
        &CodeToHtmlOptions::new("rust", "github-light").shiki(),
    )?;

    println!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>iro demo</title>
<style>
  body {{ font-family: system-ui, sans-serif; margin: 2rem; }}
  pre {{ padding: 1rem; border-radius: 6px; overflow-x: auto; font: 14px/1.5 ui-monospace, monospace; }}
  @media (prefers-color-scheme: dark) {{
    .iro-themes, .iro-themes span {{ color: var(--iro-dark) !important; background-color: var(--iro-dark-bg) !important; }}
  }}
</style>
</head>
<body>
<h2>Iro dialect, github-dark</h2>
{single}
<h2>Iro dialect, dual theme (light inline, dark via --iro-dark variables, follows the OS setting)</h2>
{dual}
<h2>Shiki preset, github-light (byte-identical to Shiki's codeToHtml)</h2>
{shiki}
</body>
</html>"#
    );
    Ok(())
}
