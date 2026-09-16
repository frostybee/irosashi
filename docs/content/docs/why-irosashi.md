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

- No Node.js or WASM runtime to install, start, or keep alive
- No subprocess IPC or serialization between processes
- Warm tokenization speed matches Shiki (see [Performance](#performance) below)
- Cold start measured in single-digit milliseconds, not hundreds
- Per-line incremental API with explicit `StateStack` handles, for editors and live previews that re-tokenize from a dirty line
- Typst output in the same process through kazari-rs, not a post-processing step on an HTML blob
- Single static binary. No `node_modules`, no sidecar

## Performance

Measured on an Intel Core i9-10850K, Windows 10, rustc 1.93.0, theme `github-dark`. Irosashi numbers are [Criterion](https://github.com/bheisler/criterion.rs) medians. Shiki numbers are from `tools/shiki-bench` running Shiki 4.4.3 on the same inputs. The three-size comparison is in [`docs/perf/2026-09-16-shiki-sizes.md`](https://github.com/frostybee/irosashi/blob/main/docs/perf/2026-09-16-shiki-sizes.md); the allocation audit and the cold-start breakdown are in [`docs/perf/2026-09-14-bench.md`](https://github.com/frostybee/irosashi/blob/main/docs/perf/2026-09-14-bench.md).

### Warm speed is comparable to Shiki

Both Irosashi and Shiki run Oniguruma over identical TextMate grammars, so the warm cost is the regex engine itself. On short snippets Irosashi is faster on 7 of 10 tested languages; JavaScript and TypeScript (the two largest grammar rule sets) are the exceptions, and Markdown is close to even.

| Language | Bytes | Lines | Irosashi (ms) | Shiki (ms) |
|---|---:|---:|---:|---:|
| Go | 117 | 11 | 0.26 | 0.39 |
| JavaScript | 309 | 13 | 2.47 | 1.18 |
| HTML | 304 | 16 | 0.30 | 0.58 |
| TypeScript | 203 | 11 | 0.82 | 0.56 |
| Markdown | 135 | 12 | 0.26 | 0.21 |
| Python | 415 | 16 | 0.68 | 0.81 |
| Bash | 339 | 16 | 0.48 | 0.55 |
| PHP | 385 | 20 | 0.91 | 1.04 |
| CSS | 424 | 25 | 0.62 | 0.73 |
| Rust | 607 | 25 | 1.13 | 1.22 |

Real files tell the same story, and the gap does not shrink with size. On 50 KiB inputs (the fixture sources repeated) Irosashi is 10 to 30 percent faster on six languages, even on Rust, and slower on JavaScript, TypeScript and Markdown:

| Language | Bytes | Lines | Irosashi (ms) | Shiki (ms) |
|---|---:|---:|---:|---:|
| Go | 51324 | 2653 | 79.4 | 88.8 |
| JavaScript | 51298 | 1963 | 198.0 | 105.9 |
| HTML | 52668 | 1716 | 24.6 | 35.6 |
| TypeScript | 52920 | 1848 | 190.2 | 96.3 |
| Markdown | 52150 | 2380 | 34.7 | 20.0 |
| Python | 51345 | 1794 | 46.3 | 54.1 |
| Bash | 51558 | 2184 | 46.6 | 52.3 |
| PHP | 51350 | 2291 | 61.4 | 69.4 |
| CSS | 51442 | 4094 | 64.8 | 71.0 |
| Rust | 51561 | 1989 | 47.4 | 46.8 |

The three slower languages are the ones with the most patterns to try per scan step: JavaScript and TypeScript have the largest rule contexts, Markdown injects the most grammars per line. Shiki's `vscode-oniguruma` caches each pattern's last match per line and re-searches only the patterns whose cached match is behind the cursor; Irosashi searches the whole pattern set once per step through Oniguruma's regset API. That cache is the one known optimisation Irosashi does not have. It is about 2x on JavaScript and TypeScript and 1.7x on Markdown, at every input size, and it is unchanged from snippet to 50 KiB.

Choosing Irosashi does not cost tokenization speed on most languages. The win is removing the runtime, not being faster at regex.

### Cold start is the real difference

Shiki's published cold numbers (20 to 90 ms per language) exclude Node.js startup and WASM instantiation, which add 50 to 150 ms per process. A Rust program calling Shiki via subprocess pays that cost on every invocation. Irosashi's `Highlighter::new()` runs in 1.5 ms inside the host process.

| Language | Irosashi cold (ms) | Shiki cold (ms) |
|---|---:|---:|
| Go | 8.8 | 99.8 |
| JavaScript | 57.9 | 80.4 |
| HTML | 29.5 | 48.4 |
| TypeScript | 67.3 | 81.2 |
| Markdown | 24.4 | 9.5 |
| Python | | 17.7 |
| Bash | | 8.5 |
| PHP | | 111.2 |
| CSS | | 59.8 |
| Rust | | 6.1 |

Irosashi cold time is Oniguruma compiling one pattern set per rule context on first use (40 sets for JavaScript and TypeScript). Shiki cold time includes WASM compilation of the grammar but not the Node process that runs it.

### No runtime overhead

The performance tables show that tokenization speed is comparable. The difference is everything around it: no Node.js process, no WASM instantiation, no IPC serialization, no `node_modules`, no second runtime to deploy. A Rust program ships a single static binary.

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
