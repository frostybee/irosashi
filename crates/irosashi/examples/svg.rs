//! Renders a snippet as a standalone SVG with the github-dark theme.
//! `cargo run -p irosashi --example svg > example-output.svg`

use irosashi::{CodeToSvgOptions, Highlighter};

const CODE: &str = r#"use irosashi::{CodeToHtmlOptions, Highlighter};

fn main() -> Result<(), irosashi::Error> {
    let hl = Highlighter::new()?;
    let opts = CodeToHtmlOptions::new("rust", "github-dark");
    let html = hl.code_to_html("fn main() {}", &opts)?;
    println!("{html}");
    Ok(())
}
"#;

fn main() -> Result<(), irosashi::Error> {
    let hl = Highlighter::new()?;
    let svg = hl.code_to_svg(CODE, &CodeToSvgOptions::new("rust", "github-dark"))?;
    println!("{svg}");
    Ok(())
}
