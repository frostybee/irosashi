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

Irosashi runs Ferroni, a pure Rust port of the Oniguruma regex engine, in-process, with no subprocess, no WASM and no C compiler. The concrete differences:

- No Node.js or WASM runtime to install, start, or keep alive
- No subprocess IPC or serialization between processes
- Warm tokenization faster than Shiki on 9 of 10 tested languages (see [Performance](#performance) below)
- Cold start measured in single-digit milliseconds, not hundreds
- Per-line incremental API with explicit `StateStack` handles, for editors and live previews that re-tokenize from a dirty line
- Typst output in the same process through kazari-rs, not a post-processing step on an HTML blob
- A single static binary, with no `node_modules` and no sidecar

## Performance

Measured on 2026-11-08 on an Intel Core i9-10850K, Windows 10, rustc 1.94.0, Ferroni 2.7.0, theme `github-dark`. Irosashi numbers are [Criterion](https://github.com/bheisler/criterion.rs) medians. Shiki numbers are from `tools/shiki-bench` running Shiki 4.4.3 on the same inputs in the same session, since Shiki's own timings vary by tens of percent between sessions. The raw dumps are in [`docs/perf/2026-11-08-ferroni-2.7.md`](https://github.com/frostybee/irosashi/blob/main/docs/perf/2026-11-08-ferroni-2.7.md); the per-pattern scanner record and the C Oniguruma numbers it replaced are in [`docs/perf/2026-09-16-per-pattern.md`](https://github.com/frostybee/irosashi/blob/main/docs/perf/2026-09-16-per-pattern.md).

### Warm speed

Both Irosashi and Shiki run Oniguruma's regex semantics (Ferroni in Rust, vscode-oniguruma in WASM) over identical TextMate grammars, and both use the same scan strategy: one compiled pattern per rule, and a per-pattern cache of where it last matched on the current line, so a scan step re-searches only the patterns whose remembered match is behind the cursor. On short snippets Irosashi is faster on 9 of 10 tested languages, by 1.4x (JavaScript) to 4.3x (CSS), and level on Markdown.

| Language | Bytes | Lines | Irosashi (ms) | Shiki (ms) |
|---|---:|---:|---:|---:|
| Go | 117 | 11 | 0.17 | 0.39 |
| JavaScript | 309 | 13 | 0.76 | 1.03 |
| HTML | 304 | 16 | 0.17 | 0.49 |
| TypeScript | 203 | 11 | 0.36 | 0.56 |
| Markdown | 135 | 12 | 0.22 | 0.21 |
| Python | 415 | 16 | 0.34 | 0.84 |
| Bash | 339 | 16 | 0.28 | 0.57 |
| PHP | 385 | 20 | 0.49 | 1.11 |
| CSS | 424 | 25 | 0.17 | 0.73 |
| Rust | 607 | 25 | 0.45 | 1.21 |

On 50 KiB inputs (the fixture sources repeated) Irosashi is faster on the same nine languages, from 1.3x (TypeScript) to 3.8x (CSS), and 1.4x slower on Markdown:

| Language | Bytes | Lines | Irosashi (ms) | Shiki (ms) |
|---|---:|---:|---:|---:|
| Go | 51324 | 2653 | 37.8 | 89.6 |
| JavaScript | 51298 | 1963 | 82.2 | 104.6 |
| HTML | 52668 | 1716 | 16.8 | 35.1 |
| TypeScript | 52920 | 1848 | 74.9 | 94.2 |
| Markdown | 52150 | 2380 | 27.3 | 19.6 |
| Python | 51345 | 1794 | 25.9 | 55.5 |
| Bash | 51558 | 2184 | 26.0 | 52.8 |
| PHP | 51350 | 2291 | 44.6 | 70.8 |
| CSS | 51442 | 4094 | 19.0 | 71.7 |
| Rust | 51561 | 1989 | 26.0 | 49.1 |

Markdown is the one shape the per-pattern design handles worse than a regset: contexts with many patterns and only one or two scan steps per line, so there is little for the cache to reuse, and a stale pattern's search runs to the end of the line, rejecting candidate positions the regset never looked at past the leftmost match. Oniguruma's public search call cannot bound where a match may start without also bounding where it may end, which would change tokens, so Irosashi does not bound it. Ferroni has the bounded search internally (`search_in_range`); exposing it is an upstream change on the roadmap.

### Cold start

Shiki's cold numbers (6 to 110 ms per language in the table) exclude Node.js startup and WASM instantiation, which add 50 to 150 ms per process. A Rust program calling Shiki via subprocess pays that cost on every invocation. Irosashi's `Highlighter::new()` runs in 1.6 ms inside the host process.

| Language | Irosashi cold (ms) | Shiki cold (ms) |
|---|---:|---:|
| Go | 3.40 | 103.4 |
| JavaScript | 29.4 | 83.4 |
| HTML | 24.8 | 51.2 |
| TypeScript | 28.0 | 82.8 |
| Markdown | 71.5 | 10.2 |
| Python | 2.21 | 17.9 |
| Bash | 1.82 | 9.60 |
| PHP | 57.7 | 113.3 |
| CSS | 13.2 | 60.0 |
| Rust | 1.60 | 6.20 |

Irosashi cold time is the engine compiling each distinct pattern of the grammar once on first use, plus parsing any grammar the language embeds (Markdown pulls in HTML, and through it CSS and JavaScript, which is most of its 72 ms). Shiki cold time includes WASM compilation of the grammar but not the Node process that runs it.

### No runtime overhead

The difference beyond the tables is everything around tokenization: no Node.js process, no WASM instantiation, no IPC serialization, no `node_modules`, no second runtime to deploy. A Rust program ships a single static binary.

## Fidelity

234 of 234 tested grammars produce output byte-identical to `vscode-textmate` across both `github-dark` and `github-light` (468 of 468 grammar/theme pairs). The Shiki HTML preset is byte-identical to Shiki 4.4.3 on generated goldens. See [FIDELITY.md](https://github.com/frostybee/irosashi/blob/main/FIDELITY.md) for the full matrix.

A grammar ships in the fidelity gate only at 100% on the shipping theme set. Anything below that is listed in `held.toml` and excluded from the core gate.

## The presentation layer

[kazari-rs](https://crates.io/crates/kazari-rs) adds the decoration a documentation site or PDF pipeline needs on top of Irosashi's token output: editor and terminal frames, line numbers, highlight/insert/delete/focus markers, titles, collapsible sections, toolbar buttons, dual-theme switching, output panels, Typst rendering, and a `pulldown-cmark` adapter with code groups and Mermaid pass-through.

It reads the same fence meta syntax and `kazari.config.yaml` as [Go Kazari](https://github.com/frostybee/kazari), so content written for the Go library works unchanged. A Rust program that renders documentation can tokenize, decorate, and export to HTML and Typst in one process.

## Comparison

The `kazari` timings in the table come from the run described under [Performance](#performance).

|                      | Irosashi     | syntect        | Shiki (JS)   | giallo        |
|----------------------|--------------|----------------|--------------|---------------|
| Languages            | 257          | ~50 maintained | 257          | 220+          |
| Themes               | 65 VS Code   | .tmTheme       | 65 VS Code   | 60+           |
| Fidelity vs VS Code  | 234/234      | N/A            | reference    | not published |
| Per-line API         | Yes          | Yes            | No           | No            |
| Typst output         | Yes (kazari) | No             | No           | No            |
| Licence              | MIT          | MIT            | MIT          | EUPL          |
| Runtime              | Pure Rust    | Pure Rust      | Node + WASM  | Native + C    |
| `kazari render`, 45 KB Rust file | 42 ms | 215 ms | n/a | n/a |
| `kazari process`, 60 pages | 167 ms | 391 ms | n/a | n/a |

**syntect** uses Sublime Text syntaxes and `.tmTheme` colour files. These are a different format from VS Code's TextMate JSON grammars and JSON themes, so syntect cannot reproduce VS Code scoping or theme colours byte for byte. It is also slower on the same input: the `kazari` binary ships both backends behind one flag, and a 2.5 KB Rust file renders in 11 ms on Irosashi against 61 ms on syntect, a 45 KB file in 42 against 215 ms, and a 60-page site in 167 against 391 ms (medians of 7 runs with `tools/kazari-bench`; syntect built on `fancy-regex`). See [Backends](/docs/getting-started/cli#backends).

**giallo** runs C Oniguruma natively and parses VS Code grammars, but does not publish fidelity scores against `vscode-textmate`.

syntect is the right choice when a site already renders other code blocks with syntect and wants them to match. Both backends are pure Rust.

## When to use Shiki instead

If the project already runs on Node.js, use Shiki. A Next.js site, an Astro build, a Vite plugin: Shiki is the reference implementation with the largest ecosystem, and calling it from JavaScript costs nothing when the runtime is already there.

Adding Irosashi to a Node project would add a second toolchain for no performance or fidelity gain. Irosashi removes a runtime, not a feature.
