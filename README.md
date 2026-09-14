# iro

Shiki's output, in-process Rust, MIT licensed: VS Code grammars and themes, byte-identical
tokens and HTML, incremental per-line tokenization, and Typst output.

Iro is a Rust port of [Shiki](https://shiki.style): TextMate grammar-based syntax highlighting
with VS Code themes, verified against `vscode-textmate` fixtures. The workspace has three crates:

- `iro`: tokenizer, grammar compiler, theme resolution, token APIs, minimal HTML renderer.
- `kazari`: presentation layer on top of `iro` (fence meta, line numbers, markers, decorated
  HTML, Typst output, `kazari.config.yaml`).
- `iro-fidelity`: fixture comparison and scoring against `vscode-textmate` output.

## Quick start

```rust
use iro::{CodeToTokensOptions, Highlighter};

let highlighter = Highlighter::new()?;
let result = highlighter.code_to_tokens(
    "fn main() {}",
    &CodeToTokensOptions::new("rust", "github-dark"),
)?;
for line in &result.lines {
    let text = result.line_text(line);
    for token in &line.tokens {
        let color = token.style.color.map(|c| result.color(c));
        println!("{:?} {:?}", token.text(text), color);
    }
}
```

`Highlighter::new()` uses the 257 grammars and 65 themes embedded in the crate (feature
`embedded-assets`, on by default). `HighlighterBuilder` loads assets from a directory,
registers extra grammars, themes, aliases and file extensions, and sets a maximum line
length. `code_to_tokens_multi` tokenizes once and styles for several themes;
`session(lang)` gives a per-line tokenizer with an explicit state handle for editors.

## Status

Early development. The engine is complete through the public token API: native
Oniguruma via `onig-regset`, grammar compiler, tokenizer, theme resolution, embedded
assets, and the fidelity gate. See `FIDELITY.md`: all 32 core grammars are byte-identical
to `vscode-textmate` on `github-dark` and `github-light`; the full 234-grammar matrix is
at 227 identical. The HTML renderer and the `kazari` presentation layer are next.
