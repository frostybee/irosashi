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

### HTML

```rust
use std::collections::BTreeMap;
use iro::{CodeToHtmlOptions, Highlighter, HtmlRenderer, StyleClassMap};

let highlighter = Highlighter::new()?;

// pre.iro.github-dark > code > span.line > span[style]
let html = highlighter.code_to_html("fn main() {}", &CodeToHtmlOptions::new("rust", "github-dark"))?;

// Light and dark in one document: inline colors from the first key, the rest as
// --iro-{key} and --iro-{key}-bg variables.
let themes = BTreeMap::from([
    ("dark".to_owned(), "github-dark".to_owned()),
    ("light".to_owned(), "github-light".to_owned()),
]);
let html = highlighter.code_to_html("fn main() {}", &CodeToHtmlOptions::multi("rust", themes))?;

// Hashed classes instead of inline styles, one stylesheet for the whole page.
let mut classes = StyleClassMap::new();
let html = highlighter.code_to_html_with(
    "fn main() {}",
    &CodeToHtmlOptions::new("rust", "github-dark"),
    &mut HtmlRenderer::with_class_map(&mut classes),
)?;
let css = classes.css();

// Byte-identical to Shiki's codeToHtml: shiki classes, --shiki-* variables, hast escaping.
let html = highlighter.code_to_html("fn main() {}", &CodeToHtmlOptions::new("rust", "github-dark").shiki())?;
```

## Status

Early development. The engine is complete through the public token API and the minimal
HTML renderer: native Oniguruma via `onig-regset`, grammar compiler, tokenizer, theme
resolution, embedded assets, the fidelity gate, and HTML output in two dialects. See
`FIDELITY.md`: all 32 core grammars are byte-identical to `vscode-textmate` on
`github-dark` and `github-light`; the full 234-grammar matrix is at 227 identical. The
Shiki HTML preset is held byte-identical to Shiki 4.4.3 on generated goldens. The
`kazari` presentation layer is next.
