//! Prints the tokens of a snippet with their scopes and styles, for debugging a grammar
//! or building a renderer of your own; `--json` prints the JSON form instead.
//! `cargo run -p irosashi --example tokens_json [-- --json]`

use irosashi::{CodeToJsonOptions, CodeToTokensOptions, Highlighter};

const CODE: &str = "let total: u32 = 40 + 2; // the answer\n";

fn main() -> Result<(), irosashi::Error> {
    let highlighter = Highlighter::new()?;
    if std::env::args().any(|a| a == "--json") {
        let mut options = CodeToJsonOptions::new("rust", "github-dark");
        options.indent = true;
        println!("{}", highlighter.code_to_json(CODE, &options)?);
        return Ok(());
    }

    // Scopes cost a little extra and are off by default.
    let mut options = CodeToTokensOptions::new("rust", "github-dark");
    options.include_scopes = true;
    let result = highlighter.code_to_tokens(CODE, &options)?;

    println!("foreground {}  background {}", result.fg(), result.bg());
    for (number, line) in result.lines.iter().enumerate() {
        let text = result.line_text(line);
        println!("line {}: {text:?}", number + 1);
        for token in &line.tokens {
            let color = token.style.color.map_or(result.fg(), |id| result.color(id));
            let style = [
                (token.style.font_style.is_bold(), "bold"),
                (token.style.font_style.is_italic(), "italic"),
            ]
            .iter()
            .filter(|(on, _)| *on)
            .map(|(_, name)| *name)
            .collect::<Vec<_>>()
            .join(" ");
            let scopes = result.scopes_of(token).unwrap_or(&[]);
            println!(
                "  {:>2}..{:<2} {:<14} {color} {style:<6} {}",
                token.start,
                token.end,
                format!("{:?}", token.text(text)),
                scopes.last().map_or("", |s| s)
            );
        }
    }
    Ok(())
}
