//! Language detection by file name, by first line, and after registering an extension.
//! `cargo run -p irosashi --example detect`

use irosashi::Highlighter;

fn main() -> Result<(), irosashi::Error> {
    let highlighter = Highlighter::new()?;

    println!("by file name");
    for name in [
        "main.rs",
        "app.tsx",
        "Makefile",
        "Dockerfile",
        "notes.txt2",
        "archive.xyz",
    ] {
        println!("  {name:<14} {}", show(highlighter.detect_language(name)));
    }

    println!("by first line");
    for line in [
        "#!/usr/bin/env crystal",
        "#!/usr/bin/env swift -O",
        "<!DOCTYPE html>",
        "%YAML 1.2",
        "plain text",
    ] {
        println!(
            "  {line:<26} {}",
            show(highlighter.detect_language_by_first_line(line))
        );
    }

    // Extensions are registered without the dot. Aliases and exact file names work the
    // same way through `register_alias` and `register_filename`.
    highlighter.register_extension("txt2", "markdown");
    highlighter.register_filename("Justfile", "just");
    println!("after registering");
    for name in ["notes.txt2", "Justfile"] {
        println!("  {name:<14} {}", show(highlighter.detect_language(name)));
    }
    Ok(())
}

fn show(lang: Option<String>) -> String {
    lang.unwrap_or_else(|| "(none)".to_owned())
}
