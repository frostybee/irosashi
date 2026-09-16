---
title: "Why Irosashi"
description: "When Irosashi is the right choice, how it compares to Shiki, syntect, and giallo, and when to use Shiki instead."
sidebar:
  order: 1
  icon: sparkles
---

## The problem Irosashi solves

A Rust program needs syntax highlighting with VS Code themes and grammars. The reference highlighter is [Shiki](https://shiki.style), which runs on Node.js. Calling Shiki from Rust means one of two things:

- **Subprocess:** spawn Node, load Shiki, send code over IPC, read HTML back. Node startup adds 50 to 150 ms per process. Every call crosses a serialization boundary. The deployed artifact ships a Node runtime and `node_modules` alongside the Rust binary.
- **WASM bundle:** embed Shiki's JavaScript and its WASM Oniguruma in the Rust process. Instantiation costs memory and startup time. The FFI boundary complicates error handling and cancellation.

Both paths add a runtime dependency that the Rust program would not otherwise need. Irosashi removes that dependency entirely.

## What Irosashi provides instead

Irosashi runs the real Oniguruma regex engine natively through `onig-regset`, in-process, with no subprocess and no WASM. The concrete differences:

- `Highlighter::new()` builds the embedded registry (257 grammars, 65 themes) in 1.5 ms
- First tokenization of Go completes in 9 ms. Shiki's published 20 to 90 ms cold numbers exclude Node startup and WASM instantiation, which add another 50 to 150 ms
- Warm tokenization speed matches Shiki, since both execute Oniguruma over the same grammars. The warm cost is the regset search itself
- No Node.js or WASM runtime to install, start, or keep alive
- No IPC serialization between processes
- Per-line incremental API with explicit `StateStack` handles, for editors and live previews that re-tokenize from a dirty line
- Typst output in the same process through kazari-rs, not a post-processing step on an HTML blob
- Single static binary. No `node_modules`, no sidecar

## Performance

Measured on an Intel Core i9-10850K, Windows 10, rustc 1.93.0, `onig-regset` 6.7.0, theme `github-dark`. Irosashi numbers are Criterion medians. Nuri numbers are from Nuri's `tools/compare` on the same five snippets (no-interrupt engine). Shiki numbers are from the same comparison table. Full data, the allocation audit and the cold-start breakdown are in [`docs/perf/`](https://github.com/frostybee/irosashi/blob/main/docs/perf/2026-09-14-bench.md).

### Warm, small snippets

| Input | Irosashi (ms) | Nuri (ms) | Shiki (ms) |
|---|---:|---:|---:|
| Go (117 B) | 0.25 | 1.26 | 0.98 |
| HTML (304 B) | 0.30 | 4.00 | 1.23 |
| JavaScript (309 B) | 2.09 | 7.03 | 1.86 |
| Markdown (135 B) | 0.26 | 1.57 | 0.51 |
| TypeScript (203 B) | 0.82 | 4.00 | 0.96 |

Irosashi and Shiki are within 2x on every snippet because both run Oniguruma over the same grammars. The warm cost is the regex engine, not the wrapper. Nuri runs Oniguruma compiled to WASM inside a Go WASM runtime, which adds overhead per regex call.

On 50 KiB inputs, Irosashi tokenizes Go at 74 ms and JavaScript at 188 ms, against Nuri's 2.4 s and 3.4 s.

### Cold start

| Bench | Irosashi (ms) | Nuri (ms) | Shiki (ms) |
|---|---:|---:|---:|
| `Highlighter::new()` | 1.4 | | |
| First tokens, Go | 8.8 | 77 | 40 |
| First tokens, HTML | 29.5 | 354 | 48 |
| First tokens, JavaScript | 57.9 | 557 | 74 |
| First tokens, Markdown | 24.4 | 145 | 21 |
| First tokens, TypeScript | 67.3 | 677 | 88 |

Cold time is dominated by Oniguruma compiling one pattern set per rule context on first use (40 sets for JavaScript and TypeScript). Parsing a grammar costs 1.2 to 2.5 ms. Shiki's cold numbers in the table above exclude Node startup and WASM instantiation.

## Fidelity

234 of 234 tested grammars produce output byte-identical to `vscode-textmate` across both `github-dark` and `github-light` (468 of 468 grammar/theme pairs). The Shiki HTML preset is byte-identical to Shiki 4.4.3 on generated goldens. See [FIDELITY.md](https://github.com/frostybee/irosashi/blob/main/FIDELITY.md) for the full matrix.

A grammar ships in the fidelity gate only at 100% on the shipping theme set. Anything below that is listed in `held.toml` and excluded from the core gate.

**syntect** uses Sublime Text syntaxes and `.tmTheme` colour files. These are a different format from VS Code's TextMate JSON grammars and JSON themes, so syntect cannot reproduce VS Code scoping or theme colours byte for byte.

**giallo** also runs native Oniguruma and parses VS Code grammars, but does not publish fidelity scores against `vscode-textmate`.

## The presentation layer

[kazari-rs](https://crates.io/crates/kazari-rs) adds the decoration a documentation site or PDF pipeline needs on top of Irosashi's token output: editor and terminal frames, line numbers, highlight/insert/delete/focus markers, titles, collapsible sections, toolbar buttons, dual-theme switching, output panels, Typst rendering, and a `pulldown-cmark` adapter with code groups and Mermaid pass-through.

It reads the same fence meta syntax and `kazari.config.yaml` as [Go Kazari](https://github.com/frostybee/kazari), so content written for the Go library works unchanged. A Rust program that renders documentation can tokenize, decorate, and export to HTML and Typst in one process.

## Comparison

|                      | Irosashi     | syntect        | Shiki (JS)   | giallo        |
|----------------------|--------------|----------------|--------------|---------------|
| Languages            | 257          | ~50 maintained | 257          | 220+          |
| Themes               | 65 VS Code   | .tmTheme       | 65 VS Code   | 60+           |
| Fidelity vs VS Code  | 234/234      | N/A            | reference    | not published |
| Per-line API         | Yes          | Yes            | No           | No            |
| Typst output         | Yes (kazari) | No             | No           | No            |
| Licence              | MIT          | MIT            | MIT          | EUPL          |
| Runtime              | Native + C   | Pure Rust      | Node + WASM  | Native + C    |

syntect's advantage is a pure-Rust build with no C dependency. It is the right choice when VS Code theme fidelity does not matter and the Sublime syntax set covers the needed languages.

## When to use Shiki instead

If the project already runs on Node.js, use Shiki. A Next.js site, an Astro build, a Vite plugin: Shiki is the reference implementation with the largest ecosystem, and calling it from JavaScript costs nothing when the runtime is already there.

Adding Irosashi to a Node project would introduce a native C dependency (`onig-sys` requires a C compiler at build time) for no performance or fidelity gain. Irosashi removes a runtime, not a feature.
