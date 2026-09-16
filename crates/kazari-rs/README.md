<p align="center">
  <img src="https://raw.githubusercontent.com/frostybee/irosashi/main/crates/kazari-rs/brand/kazari-logo.svg" alt="Kazari" width="160">
</p>

<h1 align="center">kazari-rs</h1>

<p align="center">
  <a href="https://crates.io/crates/kazari-rs"><img src="https://img.shields.io/crates/v/kazari-rs.svg" alt="crates.io"></a>
  <a href="https://docs.rs/kazari-rs"><img src="https://docs.rs/kazari-rs/badge.svg" alt="docs.rs"></a>
  <a href="https://github.com/frostybee/irosashi/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
  <img src="https://img.shields.io/badge/Rust-%E2%89%A51.93-f74c00" alt="Rust Version">
</p>

<p align="center">
  <a href="https://docs.rs/kazari-rs">API reference</a> ·
  <a href="https://crates.io/crates/irosashi">Irosashi engine</a> ·
  <a href="https://github.com/frostybee/kazari">Go Kazari</a> ·
  <a href="https://github.com/frostybee/irosashi/releases">Releases</a>
</p>

Kazari (飾り, "decoration") is a Rust library that renders framed, syntax-highlighted HTML code
blocks. It wraps [Irosashi](https://crates.io/crates/irosashi), a Rust port of Shiki, for
tokenization and adds the presentation layer on top: editor and terminal frames, line markers,
copy buttons, collapsible sections, dual-theme support, Typst output for PDF pipelines, and 90+
CSS variables for styling. Think [Expressive Code](https://expressive-code.com/), but for Rust.
It is the Rust port of [Go Kazari](https://github.com/frostybee/kazari) and reads the same fence
meta and `kazari.config.yaml`.

Irosashi gives you coloured tokens, but not a finished code block. Kazari bridges that gap. Pass
code and a meta string to `render_with_meta()`, get back self-contained HTML. `css()` and `js()`
return the page-wide stylesheet and scripts to inject once. Everything renders server-side with
no framework dependency, no Node, no WASM.

## Features

**Frames and chrome**
- Editor and terminal frames (macOS-style dots, coloured or minimal)
- Automatic frame detection: shell languages get terminal frames
- Title bars with file icons and language badges
- File name extraction from code comments (`// config.rs` becomes the title)
- Copy-to-clipboard, word wrap toggle, fullscreen with font size controls
- Per-block theme toggle (light/dark independent of the page theme)

**Markers and annotations**
- Line markers: highlight, insertion, deletion with coloured backgrounds and border accents
- Labeled line ranges (`{"API":3-7}`)
- Inline text and regex markers
- Focus lines (dim everything except the focused range)
- Diff+syntax hybrid: `diff lang="rust"` strips prefixes, applies ins/del markers, highlights as Rust
- Shiki notation comments (`// [!code ++]`, `// [!code highlight]`, `// [!code focus]`)
- Inline links: `@[text](url)` renders clickable links inside code

**Collapsible sections**
- Threshold-based: auto-collapse blocks exceeding N lines with a preview and expand button
- Range-based: collapse specific line ranges with `collapse={3-10}`
- Four collapse styles: github, collapsible-start, collapsible-end, collapsible-auto

**Themes**
- Dual-theme rendering: light and dark token colours baked into HTML via CSS custom properties
- Theme switching is pure CSS. No JS, no flash on toggle or page load
- Per-block theme override via meta string (`theme="catppuccin-mocha"`)
- Dark mode via CSS class selector, `prefers-color-scheme`, or both
- OKLCH-based theme colour adjustments (hue/chroma tinting) and a theme customizer callback
- Theme-only stylesheet for sites that ship their own layout CSS

**Accessibility**
- WCAG contrast enforcement: token colours adjusted per token to meet a configurable minimum ratio,
  including against marker backgrounds
- ARIA labels, live regions, screen reader announcements
- `all: revert` style reset isolates blocks from page CSS

**Integration**
- `pulldown-cmark` adapter (feature `markdown`) with `:::code-group` tabbed containers and tab sync
- Mermaid pass-through
- Output panels (`withOutput`, code and its result in one block)
- ANSI escape sequence rendering
- Typst renderer: `#code-block(...)` calls with the same colours as the HTML, plus the template
- i18n: en-US, fr-FR, ja-JP with per-string overrides
- Post-render callbacks, CSS and JS minification, hashed asset file names

## Install

```toml
[dependencies]
irosashi = "0.1"
kazari-rs = { version = "0.1", features = ["markdown"] }  # `markdown` is optional
```

Requires a C compiler for Irosashi's vendored Oniguruma build. On Windows, Visual Studio with the
VC tools component works; on Linux and macOS a system `cc` is enough.

## Quick start

### Render a code block

```rust
use kazari_rs::Kazari;

fn main() -> Result<(), kazari_rs::Error> {
    let hl = irosashi::Highlighter::new()?;
    let kz = Kazari::builder(hl)
        .themes("github-light", Some("github-dark"))
        .build()?;

    let code = "fn main() {\n    println!(\"hello\");\n}";
    let html = kz.render_with_meta(code, r#"rust title="main.rs" showLineNumbers {2}"#)?;

    println!("{}", kz.css()); // inject once in <head>
    println!("{html}");       // per code block
    println!("{}", kz.js());  // inject once before </body>
    Ok(())
}
```

`css()` and `js()` return page-wide assets. Inject each once. `render()` and
`render_with_meta()` return per-block HTML. `assets()` returns the same CSS and JS as files
with content-hashed names for long-lived caching.

### With pulldown-cmark

Enable the `markdown` feature. Every fenced block in the document is rendered by Kazari; the
prose is rendered by pulldown-cmark.

```rust
use kazari_rs::markdown::render_markdown;
use pulldown_cmark::Options;

let html = render_markdown(&kz, markdown, Options::empty())?;
```

For `:::code-group` tabbed containers, enable them on the engine:

```rust
let kz = Kazari::builder(hl).code_groups(true).build()?;
```

`kazari_rs::markdown::highlight_events` does the same on an existing pulldown-cmark event
stream, for pipelines that already post-process events.

### Typst output

For PDF export in the same process that renders the preview:

```rust
let typst = kz.render_with_meta_typst(code, r#"rust title="main.rs" showLineNumbers {2}"#)?;
let document = format!("{}\n{typst}", kazari_rs::typst_preamble());
// write `document` to a .typ file, then `typst compile`
```

The preamble ships the `#code-block(...)` template; every block renders with the same colours
as the HTML from the light theme.

## Configuration

Kazari has three configuration layers. Each overrides the previous:

1. **Builder options** set once at construction via `Kazari::builder(hl)`
2. **Config file** (`kazari.config.yaml`) loaded with `.config_file(yaml)`
3. **Meta string** for per-block overrides in the markdown fence info string (highest priority)

### Builder options

```rust
use kazari_rs::{CollapsibleConfig, DarkMode, Kazari};

let kz = Kazari::builder(hl)
    .themes("github-light", Some("github-dark"))
    .dark_mode(DarkMode::Selector(".dark".into()))
    .copy_button(true)
    .fullscreen_button(true)
    .wrap_button(true)
    .line_numbers(true)
    .collapsible(CollapsibleConfig {
        line_threshold: 15,
        preview_lines: 5,
        ..Default::default()
    })
    .min_contrast(5.5)
    .minify(true)
    .locale("en-US")
    .build()?;
```

See `KazariBuilder` on docs.rs for the full list; every `FileConfig` key below has a builder
method of the same name in `snake_case`.

### Config file

Create `kazari.config.yaml` in the project directory. Keys keep Go Kazari's camelCase spelling,
so a config written for the Go library works unchanged:

```yaml
themes:
  light: github-light
  dark: github-dark
darkMode:
  kind: selector
  selector: ".dark"
copyButton: true
fullscreenButton: true
lineNumbers: false
collapsible:
  lineThreshold: 15
  previewLines: 5
languageDefaults:
  bash,sh,zsh:
    frame: terminal
styleOverrides:
  --kz-radius: "0.75rem"
  --kz-font-size: "0.85rem"
```

Load it on the builder:

```rust
let yaml = std::fs::read_to_string("kazari.config.yaml").expect("config file");
let kz = Kazari::builder(hl).config_file(&yaml)?.build()?;
```

### Meta string

Per-block overrides go after the language in the opening fence:

````
```rust title="main.rs" showLineNumbers {2, 4-6}
````

| Key | Example | Effect |
|---|---|---|
| `title="..."` | `title="config.rs"` | Title bar text |
| `showLineNumbers` | | Enable line numbers |
| `startLineNumber=N` | `startLineNumber=10` | First line number |
| `frame=` | `frame=terminal` | `code`, `terminal`, `none`, `auto` |
| `theme="..."` | `theme="dracula"` | Per-block theme override |
| `wrap` | | Enable word wrap |
| `preserveIndent=false` | | Do not indent wrapped lines |
| `hangingIndent=N` | `hangingIndent=4` | Extra indent on wrapped lines |
| `{N}` or `{N-M}` | `{3, 5-8}` | Highlight lines |
| `{"Label":N-M}` | `{"API":3-7}` | Labeled highlight |
| `ins={N-M}` | `ins={3-5}` | Insertion markers |
| `del={N-M}` | `del={1}` | Deletion markers |
| `ins="text"` | `ins="new"` | Inline insertion marker |
| `del="text"` | `del="old"` | Inline deletion marker |
| `"text"` | `"highlight"` | Inline highlight marker |
| `/regex/` | `/fmt\.\w+/` | Regex inline marker |
| `focus={N-M}` | `focus={3-5}` | Focus lines (dim the rest) |
| `collapse` | | Force collapse |
| `nocollapse` | | Disable collapse |
| `collapse={N-M}` | `collapse={5-15}` | Collapse specific range |
| `collapseThreshold=N` | `collapseThreshold=20` | Per-block threshold |
| `collapseStyle="..."` | `collapseStyle="collapsible-auto"` | Collapse style |
| `withOutput` | | Split on `---output---` into code and output panel |
| `lang=` | `lang="rust"` (on `diff`) | Diff+syntax hybrid |

The same options are available programmatically through `Options` and `render()`:

```rust
use kazari_rs::{LineMarker, LineRange, MarkerType, Options};

let html = kz.render(code, &Options {
    lang: "rust".into(),
    title: "main.rs".into(),
    line_numbers: Some(true),
    line_markers: vec![LineMarker {
        marker_type: MarkerType::Ins,
        lines: vec![LineRange::new(3, 5)],
        label: String::new(),
    }],
    ..Default::default()
})?;
```

### CSS variables

All visual properties are controlled through `--kz-*` CSS custom properties (90+ variables).
Override them in a stylesheet:

```css
:root {
  --kz-radius: 0.75rem;
  --kz-font-size: 0.85rem;
  --kz-font-family: 'Fira Code', monospace;
  --kz-code-padding-block: 1.25rem;
  --kz-mark-bg: rgba(255, 200, 0, 0.15);
}
```

Or pass them on the builder:

```rust
use std::collections::BTreeMap;

let kz = Kazari::builder(hl)
    .style_overrides(BTreeMap::from([("--kz-radius".to_owned(), "0.75rem".to_owned())]))
    .themed_style_overrides(BTreeMap::from([(
        "--kz-radius".to_owned(),
        ("0.75rem".to_owned(), "0.5rem".to_owned()), // (light, dark)
    )]))
    .build()?;
```

Everything is emitted inside a `@layer kazari { ... }` cascade layer by default (rename or disable
it with `.cascade_layer(...)`), so page styles always win without `!important`.

## Relationship to Go Kazari

`kazari-rs` ports the Go library's presentation layer feature for feature. The fence meta
grammar, the `kazari.config.yaml` keys, the HTML structure, the `kz-*` class names and the
`--kz-*` variables are the same, so content and stylesheets written for Go Kazari work
unchanged. What differs:

- **Engine:** Irosashi only, in process, instead of a pluggable Nuri or Chroma highlighter.
  Tokens are byte-identical to `vscode-textmate`.
- **Markdown:** a `pulldown-cmark` adapter instead of a Goldmark extension.
- **Typst output** for PDF pipelines, which the Go library does not have.
- **No post-build CLI yet:** there is no `kazari process ./public` equivalent in this crate.

## Development

Part of the [`irosashi`](https://github.com/frostybee/irosashi) workspace.

```bash
cargo test -p kazari-rs --all-features
cargo insta test -p kazari-rs                                              # snapshot tests
cargo run -p kazari-rs --example demo > demo.html                          # feature tour
cargo run -p kazari-rs --example demo_typst > demo.typ && typst compile demo.typ
cargo run -p kazari-rs --features markdown --example demo_markdown > demo_markdown.html
cargo run -p kazari-rs --features markdown --example showcase              # multi-page showcase site
```

## Acknowledgments

Kazari is inspired by [Expressive Code](https://expressive-code.com/) and brings its feature set
to Rust. The HTML structure, CSS patterns, and class conventions are derived from Expressive Code
by Hippo ([MIT License](https://github.com/expressive-code/expressive-code/blob/main/LICENSE)).

## License

Copyright (c) 2026 FrostyBee.

Kazari is licensed under the [MIT License](https://github.com/frostybee/irosashi/blob/main/LICENSE). It embeds CSS and JS patterns derived
from Expressive Code (MIT).

---

API documentation: [docs.rs/kazari-rs](https://docs.rs/kazari-rs) · Engine:
[crates.io/crates/irosashi](https://crates.io/crates/irosashi)
