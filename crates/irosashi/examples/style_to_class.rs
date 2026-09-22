//! Several blocks share one `StyleClassMap`: tokens get hashed class names instead of
//! `style` attributes, and one stylesheet serves the whole page.
//! `cargo run -p irosashi --example style_to_class > classes.html`

use irosashi::{CodeToHtmlOptions, Highlighter, HtmlRenderer, StyleClassMap};

const BLOCKS: [(&str, &str); 3] = [
    ("rust", "fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n"),
    (
        "python",
        "def add(a, b):\n    return a + b  # ints or floats\n",
    ),
    ("go", "func add(a, b int) int {\n\treturn a + b\n}\n"),
];

fn main() -> Result<(), irosashi::Error> {
    let highlighter = Highlighter::new()?;
    let mut map = StyleClassMap::new();
    let mut blocks = String::new();
    for (lang, code) in BLOCKS {
        let html = highlighter.code_to_html_with(
            code,
            &CodeToHtmlOptions::new(lang, "github-dark"),
            &mut HtmlRenderer::with_class_map(&mut map),
        )?;
        blocks.push_str(&html);
        blocks.push('\n');
    }
    eprintln!("{} blocks share {} classes", BLOCKS.len(), map.len());

    // Class names are a stable hash of the style, so the same colour gets the same class
    // in every block and across runs. The map's CSS is emitted once.
    println!(
        "<!DOCTYPE html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><title>Style to class</title>\n<style>\npre {{ padding: 1rem; font: 14px/1.5 ui-monospace, monospace; }}\n{}\n</style></head>\n<body>\n{blocks}</body></html>",
        map.css()
    );
    Ok(())
}
