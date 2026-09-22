---
title: Runnable examples
description: "Small programs in the repository that show one feature each, from terminal output to custom grammars."
sidebar:
  order: 3
---

The repository ships runnable examples under `crates/irosashi/examples/` and
`crates/kazari-rs/examples/`. Each one is a complete program that shows one feature and
prints its result to standard output. They are compiled in CI, so they stay in step with
the API.

## Run an example

From a checkout of the repository:

```bash
cargo run -p irosashi --example ansi
cargo run -p irosashi --example dual_theme > dual.html
```

Examples that produce HTML write a complete page, so redirect the output to a file and
open it in a browser. Arguments for the example go after `--`:

```bash
cargo run -p irosashi --example ansi -- src/main.rs --depth 256
```

## Irosashi

| Example | Shows | Output |
|---|---|---|
| [`ansi`](https://github.com/frostybee/irosashi/blob/main/crates/irosashi/examples/ansi.rs) | Terminal output for a file, with `--depth 256`, `16` or `8` and `--backgrounds` | Terminal |
| [`dual_theme`](https://github.com/frostybee/irosashi/blob/main/crates/irosashi/examples/dual_theme.rs) | One tokenization rendered for a light and a dark theme, as CSS variables and as `light-dark()`, with a toggle | HTML page |
| [`incremental`](https://github.com/frostybee/irosashi/blob/main/crates/irosashi/examples/incremental.rs) | The `Session` API: tokenize line by line, edit a line, re-tokenize only until the state settles | Text |
| [`custom_grammar`](https://github.com/frostybee/irosashi/blob/main/crates/irosashi/examples/custom_grammar.rs) | A grammar and a theme that are not bundled, registered on the builder and at runtime | HTML and tokens |
| [`detect`](https://github.com/frostybee/irosashi/blob/main/crates/irosashi/examples/detect.rs) | Language detection by file name and by first line, and registering an extension | Text |
| [`style_to_class`](https://github.com/frostybee/irosashi/blob/main/crates/irosashi/examples/style_to_class.rs) | Several blocks sharing one `StyleClassMap` and one stylesheet | HTML page |
| [`transformers`](https://github.com/frostybee/irosashi/blob/main/crates/irosashi/examples/transformers.rs) | The `Notation`, `Meta` and `Whitespace` transformers plus a custom one that numbers lines | HTML page |
| [`tokens_json`](https://github.com/frostybee/irosashi/blob/main/crates/irosashi/examples/tokens_json.rs) | Every token with its scopes and style, or the JSON form with `--json` | Text or JSON |
| [`html`](https://github.com/frostybee/irosashi/blob/main/crates/irosashi/examples/html.rs) | The Irosashi and Shiki dialects and a dual-theme block on one page | HTML page |
| [`svg`](https://github.com/frostybee/irosashi/blob/main/crates/irosashi/examples/svg.rs) | A snippet as a standalone SVG image | SVG |

## Kazari

| Example | Shows | Output |
|---|---|---|
| [`demo`](https://github.com/frostybee/irosashi/blob/main/crates/kazari-rs/examples/demo.rs) | Decorated blocks: toolbar, collapse bar, output panel, theme toggle | HTML page |
| [`config_file`](https://github.com/frostybee/irosashi/blob/main/crates/kazari-rs/examples/config_file.rs) | An engine built from a `kazari.config.yaml` document | HTML page |
| [`demo_typst`](https://github.com/frostybee/irosashi/blob/main/crates/kazari-rs/examples/demo_typst.rs) | Typst output, ready for `typst compile` | Typst |
| [`demo_markdown`](https://github.com/frostybee/irosashi/blob/main/crates/kazari-rs/examples/demo_markdown.rs) | A Markdown document rendered through Kazari (needs `--features markdown`) | HTML page |
| [`backends`](https://github.com/frostybee/irosashi/blob/main/crates/kazari-rs/examples/backends.rs) | The same document through the Irosashi and syntect backends, chosen by argument (needs `--features markdown,syntect`, run with `-- all`) | HTML fragments |
| [`showcase`](https://github.com/frostybee/irosashi/blob/main/crates/kazari-rs/examples/showcase) | The demo site: every feature, Irosashi next to Shiki, Irosashi next to syntect, and contrast correction (needs `--features markdown,syntect`) | Directory of pages |
