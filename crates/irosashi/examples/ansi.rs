//! Highlights a file for the terminal.
//! `cargo run -p irosashi --example ansi -- [path] [--depth 256|16|8] [--backgrounds]`
//! With no path, a built-in Rust sample is used and the language is `rust`; with a path,
//! the language is detected from the file name.

use irosashi::{AnsiOptions, CodeToAnsiOptions, ColorDepth, Highlighter};

const SAMPLE: &str = r#"/// A three-line sample.
fn main() {
    println!("Hello, {}!", "terminal"); // TODO: read a file instead
}
"#;

fn main() -> Result<(), irosashi::Error> {
    let mut path = None;
    let mut depth = ColorDepth::Truecolor;
    let mut backgrounds = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--backgrounds" => backgrounds = true,
            "--depth" => {
                depth = match args.next().as_deref() {
                    Some("256") => ColorDepth::Colors256,
                    Some("16") => ColorDepth::Colors16,
                    Some("8") => ColorDepth::Colors8,
                    _ => ColorDepth::Truecolor,
                }
            }
            other => path = Some(other.to_owned()),
        }
    }

    let highlighter = Highlighter::new()?;
    let (code, lang) = match &path {
        Some(path) => {
            let code =
                std::fs::read_to_string(path).map_err(|e| irosashi::Error::Io(e.to_string()))?;
            let lang = highlighter
                .detect_language(path)
                .unwrap_or_else(|| "text".to_owned());
            (code, lang)
        }
        None => (SAMPLE.to_owned(), "rust".to_owned()),
    };

    let mut options = CodeToAnsiOptions::new(&lang, "github-dark");
    options.ansi = AnsiOptions::new(depth).with_token_backgrounds(backgrounds);
    let out = highlighter.code_to_ansi(&code, &options)?;

    println!("{out}");
    eprintln!(
        "{lang}, {depth:?}, {} lines, {} escape sequences",
        code.lines().count(),
        out.matches("\x1b[").count()
    );
    Ok(())
}
