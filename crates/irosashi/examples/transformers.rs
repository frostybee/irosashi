//! The built-in transformers (notation comments, meta ranges, visible whitespace) and a
//! custom one that numbers each line, attached to an `HtmlRenderer`.
//! `cargo run -p irosashi --example transformers > transformers.html`

use irosashi::transformers::{Meta, Notation, Whitespace};
use irosashi::{CodeToHtmlOptions, Highlighter, HtmlRenderer, Node, Transformer};

const CODE: &str = r#"fn greet(name: &str) {
    println!("Hello, {name}"); // [!code focus]
    println!("Goodbye"); // [!code --]
    println!("See you, {name}"); // [!code ++]
	let tabbed = 1;
}
"#;

/// Adds a `data-line` attribute to every line element.
struct LineNumbers;

impl Transformer for LineNumbers {
    fn name(&self) -> &str {
        "line-numbers"
    }

    fn line(&mut self, el: &mut Node, line: usize) {
        el.set_attr("data-line", &line.to_string());
    }
}

fn main() -> Result<(), irosashi::Error> {
    let highlighter = Highlighter::new()?;
    // Hooks run in list order. `Notation` removes the `[!code ...]` comments before
    // tokenizing; `Meta` reads a fence meta string; `Whitespace` replaces tabs and spaces
    // with visible symbols.
    let mut renderer = HtmlRenderer::new().with_transformers(vec![
        Box::new(Notation::new()),
        Box::new(Meta::new("{1,5}")),
        Box::new(Whitespace::new()),
    ]);
    renderer.push_transformer(LineNumbers);

    let html = highlighter.code_to_html_with(
        CODE,
        &CodeToHtmlOptions::new("rust", "github-light"),
        &mut renderer,
    )?;
    println!(
        r#"<!DOCTYPE html>
<html lang="en"><head><meta charset="utf-8"><title>Transformers</title>
<style>
  pre {{ padding: 1rem; font: 14px/1.5 ui-monospace, monospace; }}
  .line {{ display: inline-block; width: 100%; }}
  .has-focused .line:not(.focused) {{ opacity: .4; }}
  .highlighted {{ background: #fff8c5; }}
  .diff.add {{ background: #dafbe1; }}
  .diff.remove {{ background: #ffebe9; }}
  .line::before {{ content: attr(data-line); display: inline-block; width: 2em; color: #999; }}
</style></head>
<body>
{html}
</body></html>"#
    );
    Ok(())
}
