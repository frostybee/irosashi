//! Tokenizes once and renders a light and a dark theme in the two dual-theme forms.
//! `cargo run -p irosashi --example dual_theme > dual.html`, then open the file and use
//! the checkbox to switch.

use std::collections::BTreeMap;

use irosashi::{CodeToHtmlOptions, DefaultColor, Highlighter};

const CODE: &str = r#"import { readFile } from "node:fs/promises";

export async function count(path) {
  const text = await readFile(path, "utf8");
  return text.split(/\s+/).filter(Boolean).length; // words
}
"#;

fn main() -> Result<(), irosashi::Error> {
    let highlighter = Highlighter::new()?;
    // The keys are part of the output: `light` and `dark` name the CSS variables, and
    // `DefaultColor::LightDark` requires exactly those two.
    let themes = BTreeMap::from([
        ("light".to_owned(), "github-light".to_owned()),
        ("dark".to_owned(), "github-dark".to_owned()),
    ]);

    // Form 1: the light theme inline, the dark one in `--iro-dark` variables that a
    // `.dark` rule switches to. Works in every browser.
    let mut variables = CodeToHtmlOptions::multi("javascript", themes.clone());
    variables.html.default_color = DefaultColor::Key("light".to_owned());
    let variables = highlighter.code_to_html(CODE, &variables)?;

    // Form 2: one `light-dark()` value per colour. No variables and no rule of your own,
    // but it follows the page's `color-scheme`.
    let mut light_dark = CodeToHtmlOptions::multi("javascript", themes);
    light_dark.html.default_color = DefaultColor::LightDark;
    let light_dark = highlighter.code_to_html(CODE, &light_dark)?;

    println!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Dual theme</title>
<style>
  body {{ font-family: system-ui, sans-serif; margin: 2rem; }}
  pre {{ padding: 1rem; border-radius: 6px; font: 14px/1.5 ui-monospace, monospace; }}
  .dark .iro-themes, .dark .iro-themes span {{ color: var(--iro-dark); background-color: var(--iro-dark-bg); }}
  html.dark {{ color-scheme: dark; background: #0d1117; color: #e1e4e8; }}
</style>
</head>
<body>
<label><input type="checkbox" onchange="document.documentElement.classList.toggle('dark', this.checked)"> Dark</label>
<h2>CSS variables</h2>
{variables}
<h2>light-dark()</h2>
{light_dark}
</body>
</html>"#
    );
    Ok(())
}
