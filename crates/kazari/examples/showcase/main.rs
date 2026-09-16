//! Generates a single-page, searchable showcase of every Kazari feature.
//! `cargo run -p kazari --features markdown --example showcase -- --out target/showcase`,
//! then serve the directory and open `showcase.html`.

mod catalog;
mod page;

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
        title: "Kazari Showcase with Iro",
        subtitle: "Code blocks highlighted by Iro, a Rust port of Shiki.",
        categories: &catalog.categories,
    };
    let html = page::render(&page);

    std::fs::create_dir_all(&out)?;
    std::fs::write(out.join("showcase.html"), html)?;
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
        "wrote {} examples in {} categories to {}",
        count,
        catalog.categories.len(),
        out.display()
    );
    Ok(())
}
