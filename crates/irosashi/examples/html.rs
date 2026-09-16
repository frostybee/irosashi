//! Renders a snippet as a standalone HTML page in both dialects and a dual-theme block.
//! `cargo run -p irosashi --example html > demo.html`, then open the file in a browser.

use std::collections::BTreeMap;

use irosashi::{CodeToHtmlOptions, DefaultColor, Highlighter, HtmlOptions};

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

fn main() -> Result<(), irosashi::Error> {
    let highlighter = Highlighter::new()?;

    let single = highlighter.code_to_html(CODE, &CodeToHtmlOptions::new("rust", "github-dark"))?;

    let themes = BTreeMap::from([
        ("dark".to_owned(), "github-dark".to_owned()),
        ("light".to_owned(), "github-light".to_owned()),
    ]);
    let mut dual = CodeToHtmlOptions::multi("rust", themes);
    dual.html.default_color = DefaultColor::Key("light".to_owned());
    let dual = highlighter.code_to_html(CODE, &dual)?;
    let dual = dual.replace(
        r#"class="iro iro-themes"#,
        r#"class="iro iro-themes dual-vars"#,
    );

    let shiki = highlighter.code_to_html(
        CODE,
        &CodeToHtmlOptions::new("rust", "github-light").shiki(),
    )?;

    let themes_ld = BTreeMap::from([
        ("dark".to_owned(), "github-dark".to_owned()),
        ("light".to_owned(), "github-light".to_owned()),
    ]);
    let light_dark = highlighter.code_to_html(
        CODE,
        &CodeToHtmlOptions {
            tokens: irosashi::CodeToTokensOptions::new("rust", ""),
            themes: themes_ld,
            html: HtmlOptions {
                default_color: DefaultColor::LightDark,
                ..HtmlOptions::default()
            },
        },
    )?;

    println!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>irosashi demo</title>
<style>
  body {{ font-family: system-ui, sans-serif; margin: 2rem; color: light-dark(#24292e, #e1e4e8); background: light-dark(#fff, #161b22); }}
  pre {{ padding: 1rem; border-radius: 6px; overflow-x: auto; font: 14px/1.5 ui-monospace, monospace; }}
  .toggle {{ padding: 0.4rem 1rem; border-radius: 4px; border: 1px solid light-dark(#ccc, #555); background: light-dark(#f6f8fa, #21262d); color: inherit; cursor: pointer; font: inherit; }}
  @media (prefers-color-scheme: dark) {{
    .iro-themes.dual-vars, .iro-themes.dual-vars span {{ color: var(--iro-dark) !important; background-color: var(--iro-dark-bg) !important; }}
  }}
</style>
</head>
<body>
<p>
  <button class="toggle" onclick="toggle()">Toggle light / dark</button>
  <span id="scheme"></span>
</p>
<script>
  function toggle() {{
    const html = document.documentElement;
    const current = html.style.colorScheme;
    html.style.colorScheme = current === 'dark' ? 'light' : 'dark';
    update();
  }}
  function update() {{
    const forced = document.documentElement.style.colorScheme;
    document.getElementById('scheme').textContent = forced ? forced + ' (forced)' : 'OS default';
  }}
  update();
</script>

<h2>1. Irosashi dialect, github-dark (single theme, always dark)</h2>
{single}

<h2>2. Dual theme with CSS variables (light inline, dark via --iro-dark-*)</h2>
<p>Uses a <code>@media (prefers-color-scheme)</code> rule. Toggle the button above to switch.</p>
{dual}

<h2>3. light-dark() (native OS color scheme, no variables, no media query)</h2>
<p>Uses the CSS <code>light-dark()</code> function. Toggle the button above to switch.</p>
{light_dark}

<h2>4. Shiki preset, github-light (byte-identical to Shiki's codeToHtml)</h2>
{shiki}
</body>
</html>"#
    );
    Ok(())
}
