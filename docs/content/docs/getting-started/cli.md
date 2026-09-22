---
title: Command line
description: "Install the kazari binary and use it to highlight code or upgrade a built static site without writing Rust."
sidebar:
  order: 4
---

The `kazari` binary wraps Irosashi and Kazari into a single command. Use it to upgrade the code blocks in a built static site, render individual source files, or produce Typst output for PDF pipelines. No Rust code, no Node, no runtime dependencies.

## Install

Prebuilt binaries for Windows (x86_64), Linux (x86_64 and aarch64) and macOS (Intel and Apple Silicon) are attached to each [GitHub release](https://github.com/frostybee/irosashi/releases). Download the archive for your platform, unpack it, and put `kazari` on your `PATH`.

With a Rust toolchain:

```bash
cargo install kazari-cli
```

Building from source needs a C compiler for Irosashi's vendored Oniguruma. On Windows, install Visual Studio with the "Desktop development with C++" workload. On Linux and macOS, the system `cc` is enough.

## Upgrade a built static site

`kazari process` walks a directory of built HTML, finds every code block, recovers its source, re-renders it with Kazari, and splices the result back while preserving every other byte of the page. It writes `kazari.css` and `kazari.js` to the output root and injects a link and a script tag into each page that has code.

Preview changes before writing anything:

```bash
kazari process ./public --check
```

`--check` reports which files would change and exits 1 if anything needs updating. When you are ready, drop the flag:

```bash
kazari process ./public
```

The output is a summary line:

```
12 files, 18 blocks upgraded, 2 skipped, 0 suppressed, 14 changed
```

Running the command again is a no-op. Blocks already rendered by Kazari are skipped, and asset tag hashes are reconciled even on pages with no code. A configuration change followed by another run updates the whole site in place.

Hugo, Jekyll, Eleventy, mdBook, Sphinx, Zola, Astro, and plain `pre > code` pages are all recognized. See the [process reference](/docs/reference/cli-process) for the full flag list, recognized HTML shapes, per-generator notes, and the `data-kz-meta` hook for per-block options.

## Render a single file

`kazari render` highlights one source file (or stdin) and prints a decorated HTML block:

```bash
kazari render main.rs
```

The language is detected from the file name. Override it with `--lang`, or pass a full fence meta string with `--meta`:

```bash
kazari render main.rs --meta 'rust title="main.rs" showLineNumbers {2}'
```

Add `--page` to wrap the output in a standalone HTML page with the stylesheet and script inlined:

```bash
kazari render main.rs --page > main.html
```

Without `--page`, the output is a fragment. Inject `kazari css` once in `<head>` and `kazari js` once before `</body>` on the page that hosts it.

Read from stdin with `-`:

```bash
cat snippet.py | kazari render - --lang python
```

## Render Markdown

`kazari markdown` renders an entire Markdown document. Every fenced code block is highlighted and decorated by Kazari; the prose is rendered by pulldown-cmark with GFM extensions (tables, footnotes, strikethrough, task lists). Code groups (`:::code-group`) are enabled.

```bash
kazari markdown README.md --page > readme.html
```

## Produce Typst output

`kazari typst` emits a `#code-block(...)` call for PDF export. The preamble (the `#code-block` template) is included by default:

```bash
kazari typst main.rs > main.typ
typst compile main.typ
```

To append blocks to a document that already has the template, pass `--no-preamble`:

```bash
kazari typst main.rs --no-preamble >> document.typ
```

## Print assets and lists

```bash
kazari css              # the page-wide stylesheet
kazari js               # the page-wide script
kazari themes           # one theme name per line, sorted
kazari languages        # one language name per line, sorted
kazari version          # kazari 0.2.0
```

## Backends

The binary ships two highlighting backends and every command accepts `--engine` to pick one:

| Engine | Grammars and themes | Use it for |
|---|---|---|
| `irosashi` (default) | VS Code TextMate grammars and the 65 bundled VS Code themes; output matches Shiki byte for byte | The production build |
| `syntect` | Sublime Text grammars and syntect's seven bundled `.tmTheme` themes; faster to start | A dev server or live reload |

```bash
kazari process ./public --engine syntect
kazari themes --engine syntect
```

Every Kazari feature (frames, line numbers, markers, diff, dual themes) works on both; only the tokens differ. With `syntect`, theme names are mapped to the closest bundled theme (`github-light` to `InspiredGitHub`, `github-dark` to `base16-ocean.dark`), a name ending in `.tmTheme` is loaded from that path, and an unknown name falls back to a light or dark bundled theme instead of failing. See [theme names per backend](/docs/styling/themes-and-dark-mode#theme-names-per-backend).

## Configuration

Every command reads `kazari.config.yaml` (or `.yml`, `.json`) from the target directory, then from the working directory. `--config` names a file explicitly. The engine keys are the same as those documented in the [configuration reference](/docs/reference/configuration), plus an `engine` key and a `process` section for `kazari process`:

```yaml title="kazari.config.yaml"
engine: irosashi   # or syntect
themes:
  light: github-light
  dark: github-dark
lineNumbers: true
process:
  skipUnlabeled: false
  hashedAssets: false
  concurrency: 4
  maxFileBytes: 33554432
```

Flags override the config file. `--engine`, `--theme-light` and `--theme-dark` are accepted by every subcommand.

## Next steps

- [`kazari process` reference](/docs/reference/cli-process) for the full flag list, recognized HTML shapes, asset handling, exit codes, and per-generator setup.
- [`kazari render` reference](/docs/reference/cli-render) for language detection, stdin usage, and output modes.
- [`kazari markdown` reference](/docs/reference/cli-markdown) for Markdown extensions, code groups, and mermaid pass-through.
- [`kazari typst` reference](/docs/reference/cli-typst) for the preamble, single-theme output, and the compilation workflow.
- [Meta string syntax](/docs/reference/meta-string-syntax) for the per-block options that `--meta` and `data-kz-meta` accept.
- [Configuration](/docs/reference/configuration) for the full set of `kazari.config.yaml` keys.
