//! Builds a Kazari engine from a `kazari.config.yaml` document instead of builder calls,
//! the way `kazari process` and site integrations do.
//! `cargo run -p kazari-rs --example config_file > config.html`

use kazari_rs::Kazari;

/// The keys are the camelCase names of the config file reference. Unknown keys are an
/// error, so a typo fails at build time rather than being ignored.
const CONFIG: &str = r#"
themes:
  light: github-light
  dark: github-dark
darkMode:
  kind: selector
  selector: ".dark"
inlineLinks: true
notationComments: true
themeToggle: true
collapsible:
  lineThreshold: 6
  previewLines: 3
  style: collapsibleStart
languageDefaults:
  "bash,sh":
    frame: terminal
    lineNumbers: false
  rust:
    lineNumbers: true
styleOverrides:
  --kz-radius: "10px"
typst:
  font: JetBrains Mono
"#;

const CODE: &str = r#"use std::fs;

/// Reads the config, see @[the docs](https://example.com/config).
fn main() -> std::io::Result<()> {
    let text = fs::read_to_string("kazari.config.yaml")?;
    println!("{}", text.len()); // [!code highlight]
    println!("done"); // [!code ++]
    Ok(())
}
"#;

fn main() -> Result<(), kazari_rs::Error> {
    let hl =
        irosashi::Highlighter::new().map_err(|e| kazari_rs::Error::Highlight(e.to_string()))?;
    let kz = Kazari::builder(hl).config_file(CONFIG)?.build()?;

    // `rust` gets line numbers from `languageDefaults`, the block collapses because it is
    // longer than `lineThreshold`, and the fence meta still overrides the file.
    let rust = kz.render_with_meta(CODE, r#"rust title="main.rs""#)?;
    let shell = kz.render_with_meta("cargo run --example config_file > config.html\n", "bash")?;

    println!(
        "<!DOCTYPE html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><title>Config file</title>\n<style>{}</style></head>\n<body style=\"max-width:60rem;margin:2rem auto;font-family:system-ui\">\n{rust}\n{shell}\n<script>{}</script>\n</body></html>",
        kz.css(),
        kz.js()
    );
    Ok(())
}
