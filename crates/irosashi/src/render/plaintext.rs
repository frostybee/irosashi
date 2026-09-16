use crate::render::Renderer;
use crate::token::TokensResult;

/// Every token's text in order, lines joined by `\n`, no escaping.
#[derive(Debug, Default)]
pub struct PlainTextRenderer;

impl Renderer for PlainTextRenderer {
    type Options = ();
    type Output = String;

    fn render(&mut self, result: &TokensResult, _options: &()) -> String {
        let mut out = String::with_capacity(result.source.len());
        for (index, line) in result.lines.iter().enumerate() {
            if index > 0 {
                out.push('\n');
            }
            let text = result.line_text(line);
            for token in &line.tokens {
                out.push_str(token.text(text));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::{CodeToTokensOptions, Highlighter, HighlighterBuilder};

    fn highlighter() -> Highlighter {
        HighlighterBuilder::from_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("assets"))
            .build()
            .unwrap()
    }

    #[test]
    fn round_trips_the_source() {
        let h = highlighter();
        for (lang, code) in [
            ("rust", "fn main() {\n    println!(\"<hi> & bye\");\n}"),
            ("text", "a b\n\nc"),
            ("nope", "x\ny"),
            ("text", ""),
        ] {
            let out = h
                .code_to_plaintext(code, &CodeToTokensOptions::new(lang, "github-dark"))
                .unwrap();
            assert_eq!(out, code, "{lang}");
        }
        // A trailing newline does not open a line, as in Shiki and Nuri.
        let out = h
            .code_to_plaintext("a\n", &CodeToTokensOptions::new("text", "github-dark"))
            .unwrap();
        assert_eq!(out, "a");
    }

    #[test]
    fn too_long_lines_keep_their_text() {
        let h = HighlighterBuilder::from_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("assets"))
            .max_line_length(Some(3))
            .build()
            .unwrap();
        let out = h
            .code_to_plaintext(
                "[1, 2, 3]\n[]",
                &CodeToTokensOptions::new("json", "github-dark"),
            )
            .unwrap();
        assert_eq!(out, "[1, 2, 3]\n[]");
    }
}
