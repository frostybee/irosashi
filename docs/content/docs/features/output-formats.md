---
title: Output formats
description: "Produce HTML, ANSI, SVG, JSON, plain text, or raw tokens from the same tokenization."
sidebar:
  order: 1
---

Irosashi tokenizes once and renders to six formats. Each format has its own `code_to_*` method
on `Highlighter` and its own options struct. Every options struct has a `::new(lang, theme)`
constructor that fills in defaults.

All examples on this page assume a highlighter created with `Highlighter::new()`.

## HTML

`code_to_html` produces a `<pre><code>` block with inline `style` attributes. Two dialects are
available.

**Irosashi dialect** (default): `.iro` class on `<pre>`, `--iro-*` CSS variables for
multi-theme output.

```rust
use irosashi::{CodeToHtmlOptions, Highlighter};

let hl = Highlighter::new()?;
let html = hl.code_to_html("fn main() {}", &CodeToHtmlOptions::new("rust", "github-dark"))?;
```

**Shiki dialect**: `.shiki` class on `<pre>`, `--shiki-*` variables, hast escaping rules.
Byte-identical to Shiki 4.4.3 output:

```rust
let html = hl.code_to_html(
    "fn main() {}",
    &CodeToHtmlOptions::new("rust", "github-dark").shiki(),
)?;
```

### Multi-theme HTML

Tokenize once, emit inline colours from the first key and CSS variables for the rest:

```rust
use std::collections::BTreeMap;
use irosashi::CodeToHtmlOptions;

let themes = BTreeMap::from([
    ("dark".to_owned(), "github-dark".to_owned()),
    ("light".to_owned(), "github-light".to_owned()),
]);
let html = hl.code_to_html("fn main() {}", &CodeToHtmlOptions::multi("rust", themes))?;
```

For a single `light-dark()` CSS declaration per token (no variables, no media queries):

```rust
use irosashi::{CodeToHtmlOptions, DefaultColor, HtmlOptions};

let html = hl.code_to_html("fn main() {}", &CodeToHtmlOptions {
    themes: BTreeMap::from([
        ("dark".to_owned(), "github-dark".to_owned()),
        ("light".to_owned(), "github-light".to_owned()),
    ]),
    html: HtmlOptions { default_color: DefaultColor::LightDark, ..Default::default() },
    ..Default::default()
})?;
```

### Style-to-class HTML

Replace inline styles with deterministic hashed class names. The `StyleClassMap` accumulates
mappings across multiple calls, and `css()` returns one stylesheet for the page:

```rust
use irosashi::{CodeToHtmlOptions, HtmlRenderer, StyleClassMap};

let mut classes = StyleClassMap::new();
let html = hl.code_to_html_with(
    "fn main() {}",
    &CodeToHtmlOptions::new("rust", "github-dark"),
    &mut HtmlRenderer::with_class_map(&mut classes),
)?;
let css = classes.css();
```

## ANSI

`code_to_ansi` wraps each token in SGR escape sequences for terminal output. Four colour
depths are supported: `Truecolor` (default), `Colors256`, `Colors16`, and `Colors8`.

```rust
use irosashi::{AnsiOptions, CodeToAnsiOptions, ColorDepth};

let out = hl.code_to_ansi("fn main() {}", &CodeToAnsiOptions {
    tokens: irosashi::CodeToTokensOptions::new("rust", "github-dark"),
    ansi: AnsiOptions { color_depth: ColorDepth::Colors256 },
})?;
print!("{out}");
```

Lines are joined by `\n`. Backgrounds are not emitted.

## SVG

`code_to_svg` renders a self-contained `<svg>` element with `<text>` and `<tspan>` elements.
The output is suitable for README images, email, and static export.

```rust
use irosashi::{CodeToSvgOptions, SvgOptions};

let svg = hl.code_to_svg("fn main() {}", &CodeToSvgOptions {
    tokens: irosashi::CodeToTokensOptions::new("rust", "github-dark"),
    svg: SvgOptions { font_size: 16.0, corner_radius: 12.0, ..Default::default() },
})?;
```

`SvgOptions` controls font family, font size, line height, padding, tab width, corner radius,
and whether to draw the background rectangle.

## JSON

`code_to_json` serializes the token grid as JSON. Each token includes its text, colour, font
style, and optionally its scope names. Set `indent: true` for readable output:

```rust
use irosashi::CodeToJsonOptions;

let json = hl.code_to_json("fn main() {}", &CodeToJsonOptions {
    tokens: irosashi::CodeToTokensOptions::new("rust", "github-dark"),
    indent: true,
    ..Default::default()
})?;
```

Multi-theme JSON includes colours from all themes in each token. Pass the `themes` map the same
way as `CodeToHtmlOptions::multi`.

## Plain text

`code_to_plaintext` runs the tokenizer (to resolve language detection and line splitting) and
returns the source text with no styling:

```rust
use irosashi::CodeToTokensOptions;

let text = hl.code_to_plaintext("fn main() {}", &CodeToTokensOptions::new("rust", "github-dark"))?;
```

## Raw tokens

`code_to_tokens` returns the full `TokensResult` struct: the source, per-line `ThemedLine`s
with `ThemedToken`s, theme slots, colour table, and diagnostics. Use this when you need
programmatic access to the tokenization:

```rust
use irosashi::CodeToTokensOptions;

let result = hl.code_to_tokens("fn main() {}", &CodeToTokensOptions::new("rust", "github-dark"))?;
for line in &result.lines {
    let text = result.line_text(line);
    for token in &line.tokens {
        let color = token.style.color.map(|c| result.color(c));
        println!("{:?} {:?}", token.text(text), color);
    }
}
```

Each `ThemedToken` has byte offsets (`start`, `end`) into the line, a `TokenStyle` with colour
and font style indices, and when `include_scopes` is set on the options, the scope list for
theme matching.
