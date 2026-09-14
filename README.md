# iro

Shiki's output without Shiki's runtime. MIT licensed Rust: VS Code grammars and themes,
byte-identical tokens and HTML, incremental per-line tokenization, and Typst output, with
no Node, no browser and no WASM in the process.

Iro is a Rust port of [Shiki](https://shiki.style): TextMate grammar-based syntax highlighting
with VS Code themes, verified against `vscode-textmate` fixtures. It exists for programs that
cannot call Shiki: Rust desktop apps, command-line tools, servers and PDF pipelines. Warm
highlighting speed is on par with Shiki, since both run Oniguruma over the same grammars;
what Iro removes is the JavaScript runtime, the sidecar process and the cold start that come
with it. If a project already runs on Node, Shiki is the right choice. The workspace has
three crates:

- `iro`: tokenizer, grammar compiler, theme resolution, token APIs, minimal HTML renderer.
- `kazari`: presentation layer on top of `iro` (fence meta, line numbers, markers, decorated
  HTML, Typst output, `kazari.config.yaml`).
- `iro-fidelity`: fixture comparison and scoring against `vscode-textmate` output.

## Why Iro

Where Iro actually earns its place:

- **No JavaScript runtime:** Shiki needs Node or a browser plus a WASM Oniguruma. A Rust
  desktop app, a CLI, a Typst pipeline, or a server written in Rust cannot call Shiki
  without a sidecar process or embedding V8. That was the original reason for the project:
  Sarde Maker is a Tauri app, and Nuri, the Go engine it ships today, is 10 to 30 times
  slower than Iro on the same inputs.
- **Cold start is a different comparison:** Shiki's 20 to 90 ms cold numbers exclude Node
  startup and WASM instantiation, which add roughly 50 to 150 ms per process. Iro builds a
  highlighter in 1.5 ms and first-tokenizes Go in 9 ms, inside the host process.
- **Fidelity is strictly better than the alternatives in Rust:** syntect uses Sublime
  syntaxes and cannot reproduce VS Code themes and grammars byte for byte. Iro is at 234 of
  234 grammars against `vscode-textmate` and byte-identical to Shiki's HTML, which no other
  Rust crate offers.
- **The per-line API** with an explicit state handle is designed for editors and previews
  that re-tokenize from a dirty line. Shiki's `codeToHtml` is whole-document.
- **Typst output** is coming through Kazari, for PDF export in the same process that renders
  the preview.

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

// Native light/dark switching with the CSS light-dark() function, no variables or
// media queries needed. Requires theme keys "light" and "dark".
let themes = BTreeMap::from([
    ("dark".to_owned(), "github-dark".to_owned()),
    ("light".to_owned(), "github-light".to_owned()),
]);
let html = highlighter.code_to_html("fn main() {}", &CodeToHtmlOptions {
    themes,
    html: iro::HtmlOptions { default_color: iro::DefaultColor::LightDark, ..Default::default() },
    ..Default::default()
})?;
// Emits: <span style="color:light-dark(#d73a49, #f97583)">fn</span>

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

## Build and test

Requires a C compiler for the vendored Oniguruma build (`onig-sys`). On Windows, Visual
Studio with the VC tools component works. On Linux and macOS, a system `cc` is enough.

```bash
cargo build --workspace
cargo test --workspace
```

### Fidelity

The core gate (32 grammars x github-dark + github-light) runs as part of `cargo test`.
The full 234-grammar matrix requires locally synced fixtures (see `tools/sync-assets`):

```bash
cargo test -p iro-fidelity                             # core gate: 64/64
cargo test -p iro-fidelity -- --ignored golden_all     # full matrix: 468/468
IRO_WRITE_REPORT=1 cargo test -p iro-fidelity          # regenerate FIDELITY.md
```

### HTML goldens and snapshots

```bash
cargo test -p iro --test html_goldens                  # 19 Shiki-preset goldens
cargo test -p iro --test html_snapshots                # 12 Iro-dialect insta snapshots
```

### HTML demo

Writes a standalone page with three rendered blocks (single theme, dual theme, Shiki preset):

```bash
cargo run -p iro --example html > demo.html
```

### Benchmarks

Criterion benches live in `crates/iro-fidelity/benches/highlight`. The inputs are the
same five snippets Nuri's `tools/compare` uses, plus the fidelity fixture sources as
medium inputs and 50 KiB repetitions as large inputs:

```bash
cargo bench -p iro-fidelity --bench highlight                             # full run
cargo bench -p iro-fidelity --bench highlight -- --save-baseline before   # save a baseline
cargo bench -p iro-fidelity --bench highlight -- --baseline before        # compare against it
cargo run -p bench-report                                                 # markdown table from criterion
```

### Allocation audit and cold-start breakdown

Both are `#[ignore]` tests that print markdown tables. Run in release for meaningful numbers:

```bash
cargo test -p iro-fidelity --release --test alloc_audit -- --ignored --nocapture
cargo test -p iro-fidelity --release --test cold_start -- --ignored --nocapture
```

### Fuzz

Requires a nightly toolchain and `cargo-fuzz`. Two targets: `parse_grammar` and `tokenize`.

```bash
cargo +nightly fuzz run parse_grammar -- -max_total_time=120
cargo +nightly fuzz run tokenize -- -max_total_time=120
```

### Lint

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
```

## Status

Early development. The engine is complete through the public token API and the minimal
HTML renderer: native Oniguruma via `onig-regset`, grammar compiler, tokenizer, theme
resolution, embedded assets, the fidelity gate, and HTML output in two dialects. See
`FIDELITY.md`: all 32 core grammars are byte-identical to `vscode-textmate` on
`github-dark` and `github-light`, and so are all 234 grammars of the full matrix
(468 of 468 grammar and theme pairs). The Shiki HTML preset is held byte-identical to
Shiki 4.4.3 on generated goldens. Benchmarks are in `my-docs/perf/`. The `kazari`
presentation layer is next.
