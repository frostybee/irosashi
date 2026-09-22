---
title: Decorated code blocks with Kazari
description: "Render framed, themed code blocks with line numbers, markers, and a toolbar."
sidebar:
  order: 3
---

Irosashi tokenizes code. Kazari turns those tokens into the finished code blocks a documentation
site or PDF needs: editor and terminal frames, line numbers, highlight and diff markers, copy
buttons, collapsible sections, and dual-theme support. This page walks through rendering a
single block, injecting the page-wide assets, and the two alternative output paths (Typst and
Markdown).

## Prerequisites

- A working `Highlighter` from the [quick start](/docs/getting-started/quick-start).
- `kazari-rs` added to your dependencies:

```toml title="Cargo.toml"
[dependencies]
irosashi = "0.2"
kazari-rs = "0.2"
```

## Build a Kazari engine

`Kazari::builder` takes a highlighter and returns a builder with sensible defaults. Set a
light and dark theme so the block supports both modes:

```rust
use kazari_rs::Kazari;

let hl = irosashi::Highlighter::new()?;
let kz = Kazari::builder(hl)
    .themes("github-light", Some("github-dark"))
    .notation_comments(true)
    .build()?;
```

The highlighter is any type implementing `kazari_rs::Highlighter`. Irosashi is the default
backend. To build without Oniguruma, or to keep an existing syntect setup, use the `syntect`
feature and pass a `SyntectHighlighter` instead; every Kazari feature works the same on either
backend:

```toml title="Cargo.toml"
[dependencies]
kazari-rs = { version = "0.2", default-features = false, features = ["syntect"] }
```

```rust
use kazari_rs::backends::syntect::SyntectHighlighter;

let kz = Kazari::builder(SyntectHighlighter::new())
    .themes("github-light", Some("github-dark"))
    .build()?;
```

`SyntectHighlighter` maps common VS Code theme names to syntect's bundled themes and loads a
name ending in `.tmTheme` from that path. The [`backends` example](/docs/features/examples)
renders the same document through both backends.

`themes` sets the light theme (required) and an optional dark theme. When a dark theme is
present, the rendered HTML contains CSS custom properties for both themes, and theme switching
is pure CSS with no page reload.

`.notation_comments(true)` enables Shiki-style notation comments (`// [!code highlight]`,
`// [!code ++]`, `// [!code focus]`). These are opt-in because they modify the source before
rendering.

## Render a code block

Pass the source code and a meta string. The meta string uses the same syntax as a Markdown
fence info line:

```rust
let code = r#"fn main() {
    println!("Hello, world!");
}"#;

let html = kz.render_with_meta(code, r#"rust title="main.rs" showLineNumbers {2}"#)?;
```

This produces a framed code block titled "main.rs" with line numbers enabled and line 2
highlighted. The full [meta string syntax](/docs/reference/meta-string-syntax) is documented in
the reference section.

For programmatic control without parsing a meta string, use `render` with an `Options` struct:

```rust
use kazari_rs::Options;

let html = kz.render(code, &Options {
    lang: "rust".into(),
    title: "main.rs".into(),
    line_numbers: Some(true),
    ..Default::default()
})?;
```

## Inject CSS and JS

Each rendered block is self-contained HTML, but the page needs one shared stylesheet and one
shared script. Inject them once:

```rust
let page = format!(
    r#"<!doctype html>
<html>
<head><style>{}</style></head>
<body>
{}
<script>{}</script>
</body>
</html>"#,
    kz.css(),
    html,
    kz.js(),
);
```

`css()` returns the full Kazari stylesheet, including theme variables, marker colours, toolbar
styles, and the cascade layer. `js()` returns the scripts for the copy button, word wrap
toggle, fullscreen, and the theme toggle. Both are minified by default.

For long-lived caching with content-hashed filenames, use `assets()` instead:

```rust
let assets = kz.assets();
// assets.css.filename: "kazari-a1b2c3.css"
// assets.css.content:  the stylesheet
// assets.js.filename:  "kazari-d4e5f6.js"
// assets.js.content:   the script
```

## Typst output

Kazari can render the same block as a Typst `#code-block(...)` call for PDF export. The light
theme colours are used:

```rust
let typst_block = kz.render_with_meta_typst(code, r#"rust title="main.rs" showLineNumbers"#)?;
let preamble = kazari_rs::typst_preamble();

let document = format!("{preamble}\n{typst_block}");
std::fs::write("output.typ", &document)?;
// then: typst compile output.typ
```

`typst_preamble()` returns the `#code-block` template definition. Include it once at the top of
the document. Each `render_with_meta_typst` call emits one `#code-block(...)` invocation that
the template draws.

To change the font, text size, or marker colours of Typst blocks, use the builder methods
`.typst_font("...")?`, `.typst_size("10pt")?`, and `.typst_marker_color("ins", "#c8f7d0")?`, or
the [`typst` section of the config file](/docs/reference/configuration#typst-options).

## Markdown integration

Enable the `markdown` feature to render entire Markdown documents through Kazari. Every fenced
code block is highlighted and decorated; the prose is rendered by pulldown-cmark:

```toml title="Cargo.toml"
[dependencies]
kazari-rs = { version = "0.2", features = ["markdown"] }
pulldown-cmark = "0.13"
```

```rust
use kazari_rs::markdown::render_markdown;
use pulldown_cmark::Options;

let html = render_markdown(&kz, markdown_source, Options::empty())?;
```

For `:::code-group` tabbed containers, enable code groups on the builder:

```rust
let kz = Kazari::builder(hl)
    .themes("github-light", Some("github-dark"))
    .code_groups(true)
    .build()?;
```

If you already process pulldown-cmark events in a pipeline, use `highlight_events` to transform
the event stream without taking over the final HTML rendering:

```rust
use kazari_rs::markdown::highlight_events;
use pulldown_cmark::Parser;

let events = highlight_events(&kz, Parser::new(markdown_source))?;
```

## Next steps

- [Command line](/docs/getting-started/cli) to use Kazari from the terminal without writing
  Rust, or to upgrade code blocks in a built static site.
- [Meta string syntax](/docs/reference/meta-string-syntax) for the full set of per-block
  options.
- [Configuration file](/docs/reference/configuration) for engine-wide defaults via
  `kazari.config.yaml`.
- [CSS variables](/docs/reference/css-variables) for visual customization.
- [Themes and dark mode](/docs/styling/themes-and-dark-mode) for dual-theme setup and
  switching strategies.
