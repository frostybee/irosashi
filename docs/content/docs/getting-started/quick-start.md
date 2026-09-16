---
title: Quick start
description: "Tokenize code and render syntax-highlighted HTML with Irosashi."
sidebar:
  order: 2
---

This page walks through the core highlighting workflow: creating a highlighter, reading tokens, rendering single-theme and dual-theme HTML, and generating a class-based stylesheet.

## Create a highlighter

`Highlighter::new()` loads the 257 embedded grammars and 65 themes:

```rust
use irosashi::Highlighter;

let highlighter = Highlighter::new()?;
```

The highlighter is safe to share across threads. Grammars are parsed lazily on first use.

## Tokenize code

Raw tokens give you full control over rendering. Each token carries byte offsets, a resolved colour, and a font style. Use this when you need custom output or when you want to inspect how a grammar maps scopes to colours.

`code_to_tokens` tokenizes a string and resolves colours from one theme:

```rust
use irosashi::CodeToTokensOptions;

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

Each `ThemedToken` carries byte offsets into the line, a `TokenStyle` with colour and font style, and optional scope names.

## Render single-theme HTML

Most use cases need HTML, not raw tokens. `code_to_html` tokenizes and renders in one call, producing a self-contained `<pre>` element with inline `style` attributes that works without any stylesheet:

```rust
use irosashi::CodeToHtmlOptions;

let html = highlighter.code_to_html(
    "fn main() {}",
    &CodeToHtmlOptions::new("rust", "github-dark"),
)?;
```

The `<pre>` carries the class `iro github-dark`. Each token is a `<span>` with `style="color:#..."`.

## Render dual-theme HTML

Sites that support light and dark modes can tokenize once and bake both sets of colours into the same HTML. The lexicographically first theme key provides inline styles; the rest emit `--iro-{key}` CSS variables that you activate with a class or media query.

```rust
use std::collections::BTreeMap;
use irosashi::CodeToHtmlOptions;

let themes = BTreeMap::from([
    ("dark".to_owned(), "github-dark".to_owned()),
    ("light".to_owned(), "github-light".to_owned()),
]);
let html = highlighter.code_to_html(
    "fn main() {}",
    &CodeToHtmlOptions::multi("rust", themes),
)?;
```

Each token emits `style="color:#..."` for the `dark` theme and a `--iro-light` CSS variable for the `light` theme. Toggle themes with a `.light` class or a `prefers-color-scheme` media query that sets the variables.

## Use `light-dark()` mode

If your page already uses the CSS `light-dark()` function for colour scheme switching, Irosashi can emit a single `light-dark(#light, #dark)` value per token instead of variables. This produces the smallest output and needs no extra CSS. It requires exactly two theme keys named `light` and `dark`:

```rust
use std::collections::BTreeMap;
use irosashi::{CodeToHtmlOptions, DefaultColor, HtmlOptions};

let themes = BTreeMap::from([
    ("dark".to_owned(), "github-dark".to_owned()),
    ("light".to_owned(), "github-light".to_owned()),
]);
let html = highlighter.code_to_html("fn main() {}", &CodeToHtmlOptions {
    themes,
    html: HtmlOptions {
        default_color: DefaultColor::LightDark,
        ..Default::default()
    },
    ..Default::default()
})?;
// Each token: <span style="color:light-dark(#light, #dark)">...</span>
```

No CSS variables, no media queries. The browser resolves the colour from its own `color-scheme`.

## Use the Shiki-compatible preset

If you are migrating from Shiki and want to keep existing CSS selectors and variable names working, the `.shiki()` preset switches the output to Shiki's class names (`shiki`, not `iro`), its `--shiki-` variable prefix, and its hast escaping rules. The HTML is byte-identical to Shiki 4.4.3:

```rust
let html = highlighter.code_to_html(
    "fn main() {}",
    &CodeToHtmlOptions::new("rust", "github-dark").shiki(),
)?;
// <pre class="shiki github-dark" ...>
```

## Use style-to-class mode

Inline styles increase HTML size and prevent Content Security Policy rules that block `style` attributes. Style-to-class mode replaces them with short, deterministic class names. A shared `StyleClassMap` collects every unique style across all rendered blocks and produces one stylesheet for the page:

```rust
use irosashi::{CodeToHtmlOptions, HtmlRenderer, StyleClassMap};

let mut classes = StyleClassMap::new();

let html1 = highlighter.code_to_html_with(
    rust_code,
    &CodeToHtmlOptions::new("rust", "github-dark"),
    &mut HtmlRenderer::with_class_map(&mut classes),
)?;

let html2 = highlighter.code_to_html_with(
    js_code,
    &CodeToHtmlOptions::new("javascript", "github-dark"),
    &mut HtmlRenderer::with_class_map(&mut classes),
)?;

let css = classes.css();
// Inject `css` once in <head>; both blocks share the same class names.
```

Each `<span>` gets a class like `iro-abc123` instead of an inline `style`. Call `classes.css()` after rendering all blocks to get the complete stylesheet.

## Next steps

- [Kazari quick start](/docs/getting-started/kazari): render decorated code blocks with frames, line numbers, and markers.
- [Meta string syntax](/docs/reference/meta-string-syntax): the full reference for fence meta tokens.
- [Themes and dark mode](/docs/styling/themes-and-dark-mode): configure dual-theme rendering and dark mode switching.
