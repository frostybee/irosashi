//! Generates the demo site: a searchable showcase of every Kazari feature, side by side
//! comparisons with Shiki and with the syntect backend, and a contrast correction page.
//! `cargo run -p kazari-rs --features markdown,syntect --example showcase -- --out target/showcase`,
//! then serve the directory and open `showcase.html`.

mod catalog;
mod compare;
mod page;
mod snippets;

use std::path::PathBuf;

static PAGE_CSS: &str = include_str!("assets/showcase.css");
static PAGE_JS: &str = include_str!("assets/showcase.js");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut out = PathBuf::from("target/showcase");
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" => out = PathBuf::from(args.next().ok_or("--out needs a directory")?),
            other => return Err(format!("unknown argument {other:?}").into()),
        }
    }

    let catalog = catalog::build()?;
    let page = page::Page {
        title: "Kazari Showcase with Irosashi",
        subtitle: "Code blocks highlighted by Irosashi, a Rust port of Shiki.",
        categories: &catalog.categories,
    };
    let html = page::render(&page);

    std::fs::create_dir_all(&out)?;
    std::fs::write(out.join("showcase.html"), html)?;
    std::fs::write(
        out.join("irosashi-vs-shiki.html"),
        compare::irosashi_vs_shiki()?,
    )?;
    std::fs::write(
        out.join("irosashi-vs-syntect.html"),
        compare::irosashi_vs_syntect()?,
    )?;
    std::fs::write(out.join("color-contrast.html"), compare::color_contrast()?)?;
    std::fs::write(
        out.join("showcase.css"),
        format!("{}\n{}", catalog.css, PAGE_CSS),
    )?;
    std::fs::write(
        out.join("showcase.js"),
        format!("{}\n{}", catalog.js, PAGE_JS),
    )?;

    let count: usize = catalog.categories.iter().map(|c| c.examples.len()).sum();
    println!(
        "wrote {} examples in {} categories and 3 comparison pages to {}",
        count,
        catalog.categories.len(),
        out.display()
    );
    Ok(())
}
