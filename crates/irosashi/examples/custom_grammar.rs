//! Registers a grammar and a theme that are not bundled, at build time and at runtime.
//! `cargo run -p irosashi --example custom_grammar`

use irosashi::{CodeToHtmlOptions, CodeToTokensOptions, Highlighter};

/// A TextMate grammar in the JSON form VS Code uses. `fileTypes` feeds language
/// detection.
const GRAMMAR: &[u8] = br#"{
  "scopeName": "source.kv",
  "fileTypes": ["kv"],
  "patterns": [
    {"match": "^\\s*#.*$", "name": "comment.line.kv"},
    {"match": "^(\\w+)\\s*(=)", "captures": {"1": {"name": "variable.kv"}, "2": {"name": "keyword.operator.kv"}}},
    {"match": "\"[^\"]*\"", "name": "string.quoted.kv"},
    {"match": "\\b\\d+\\b", "name": "constant.numeric.kv"}
  ]
}"#;

/// A VS Code theme. Only `editor.foreground`, `editor.background` and `tokenColors`
/// are needed.
const THEME: &[u8] = br##"{
  "name": "kv-night",
  "type": "dark",
  "colors": {"editor.foreground": "#d0d0d0", "editor.background": "#101418"},
  "tokenColors": [
    {"scope": "comment", "settings": {"foreground": "#6a737d", "fontStyle": "italic"}},
    {"scope": "variable", "settings": {"foreground": "#79b8ff"}},
    {"scope": "keyword.operator", "settings": {"foreground": "#f97583"}},
    {"scope": "string", "settings": {"foreground": "#9ecbff"}},
    {"scope": "constant.numeric", "settings": {"foreground": "#ffab70"}}
  ]
}"##;

const CODE: &str = "# server settings\nhost = \"localhost\"\nport = 8080\n";

fn main() -> Result<(), irosashi::Error> {
    // At build time: the highlighter is born with the extra grammar and theme.
    let built = Highlighter::builder()
        .grammar("kv", GRAMMAR)?
        .theme("kv-night", THEME)?
        .build()?;
    println!(
        "{}",
        built.code_to_html(CODE, &CodeToHtmlOptions::new("kv", "kv-night"))?
    );

    // At runtime: an existing highlighter learns them. Later calls see them; sessions
    // created before the call keep the registry they started with.
    let runtime = Highlighter::new()?;
    runtime.load_language("kv", GRAMMAR)?;
    runtime.load_theme("kv-night", THEME)?;
    println!(
        "detected {:?} for settings.kv",
        runtime.detect_language("settings.kv")
    );

    let mut options = CodeToTokensOptions::new("kv", "kv-night");
    options.include_scopes = true;
    let tokens = runtime.code_to_tokens(CODE, &options)?;
    for line in &tokens.lines {
        let text = tokens.line_text(line);
        for token in &line.tokens {
            let scopes = tokens.scopes_of(token).unwrap_or(&[]);
            println!("{:>14?}  {}", token.text(text), scopes.join(" "));
        }
    }
    Ok(())
}
