---
title: Irosashi
description: "Rust port of Shiki with native Oniguruma, 234/234 grammar fidelity, and six output formats."
sidebar:
  order: 0
  label: "Introduction"
  icon: info
---

Irosashi is a Rust port of [Shiki](https://shiki.style), the TextMate grammar-based syntax highlighter used by VS Code. It runs the [Oniguruma](https://github.com/kkos/oniguruma) regex engine natively through the [`onig-regset`](https://crates.io/crates/onig-regset) crate and produces output byte-identical to `vscode-textmate` on 234 of 234 tested grammars across both `github-dark` and `github-light` (468 of 468 grammar/theme pairs).

257 languages, 65 VS Code themes, six output formats (HTML, ANSI, SVG, JSON, plain text, raw tokens), and a per-line tokenizer for editors. No Node, no browser, no WASM.

## When to use Irosashi

Irosashi is for Rust programs that need Shiki-quality highlighting without a JavaScript runtime. See [Why Irosashi](/docs/why-irosashi/) for the full comparison, performance numbers, and when to use Shiki instead.

## What Irosashi provides

**Fidelity** 234 of 234 grammars produce output byte-identical to `vscode-textmate`. The Shiki HTML preset is byte-identical to Shiki 4.4.3 on generated goldens. A grammar ships in the fidelity gate only at 100% on the shipping theme set.

**Native Oniguruma** The regex engine runs in-process through `onig-regset`, searching a whole pattern set per call. No WASM binary, no subprocess, no pool of instances.

**Six output formats** `code_to_html` (two dialects), `code_to_ansi` (8/16/256/truecolor), `code_to_svg`, `code_to_json`, `code_to_plaintext`, and `code_to_tokens` for raw token access.

**Two HTML dialects** Irosashi's own output uses `.iro` classes and `--iro-*` CSS variables. The Shiki preset uses `shiki` classes, `--shiki-*` variables, and hast escaping. Switch with `CodeToHtmlOptions::new("rust", "github-dark").shiki()`.

**Multi-theme mode** Tokenize once and resolve colours from several themes. The first theme emits inline styles; the rest emit CSS variables. A `light-dark()` mode writes a single CSS declaration per token with no variables or media queries.

**Per-line session API** `Session::tokenize_line` takes one line and an explicit `StateStack` handle and returns tokens plus the state for the next line. Designed for editors and previews that re-tokenize from a dirty line.

**Style-to-class mode** `StyleClassMap` maps each unique style to a deterministic hashed class name. One stylesheet covers every code block on the page.

**Contrast correction** `HighlighterBuilder::min_contrast` adjusts token foregrounds toward black or white until they meet a WCAG contrast ratio against the theme background. Off by default so fidelity output stays byte-identical to the fixtures.

## The presentation layer

[`kazari-rs`](https://crates.io/crates/kazari-rs) adds the decoration a documentation site or PDF pipeline needs on top of Irosashi's token output: editor and terminal frames, line numbers, highlight/insert/delete/focus markers, titles, collapsible sections, toolbar buttons, dual-theme switching, output panels, Typst rendering, and a `pulldown-cmark` adapter with code groups and Mermaid pass-through. It reads the same fence meta and `kazari.config.yaml` as [Go Kazari](https://github.com/frostybee/kazari).

## Command line

The [`kazari` binary](/docs/getting-started/cli) wraps both crates into a single command for use without writing Rust. `kazari process ./public` upgrades the code blocks in a built static site (Hugo, Jekyll, mdBook, Sphinx, Eleventy, Zola, Astro). `kazari render`, `kazari markdown`, and `kazari typst` render individual files from the terminal or a script.

