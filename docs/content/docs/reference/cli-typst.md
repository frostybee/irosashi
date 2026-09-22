---
title: "`kazari typst` reference"
description: "Flags, preamble handling, compilation workflow, and output details for the typst command."
sidebar:
  order: 11
---

`kazari typst` renders a source file as a Typst `#code-block(...)` call for PDF export. This page covers every flag, the preamble, the compilation workflow, and the single-theme behavior.

For installation and a quick walkthrough, see the [command line getting started](/docs/getting-started/cli).

## Flags

```
kazari typst <input> [flags]
```

`input` is a file path or `-` for stdin.

| Flag | Default | Effect |
|---|---|---|
| `--lang <NAME>` | detected from the file name | Language name. Overridden by `--meta` when both are given. |
| `--meta <META>` | none | Full fence meta string. Overrides `--lang` and adds per-block options (title, line numbers, markers, focus). Uses the same syntax as the [meta string reference](/docs/reference/meta-string-syntax). |
| `--font <NAME>` | `typst.font` from the config, else `DejaVu Sans Mono` | Font family of the code block. The font must be installed where the document is compiled. |
| `--font-size <LENGTH>` | `typst.size` from the config, else `9pt` | Text size as a Typst length, for example `10pt` or `0.9em`. |
| `--no-preamble` | off | Omit the `#code-block` template from the output. Use this when appending blocks to a document that already includes the template. |
| `--config <PATH>` | auto-discover | Path to a config file. Without it, the tool probes `kazari.config.yaml`, `.yml`, and `.json` in the working directory. |
| `--engine <NAME>` | `engine` from the config, else `irosashi` | Highlighting backend, `irosashi` or `syntect`. See [backends](/docs/getting-started/cli#backends). |
| `--theme-light <NAME>` | `github-light` | Syntax theme for the output. |
| `--theme-dark <NAME>` | `github-dark` | Ignored. Typst output uses only the light theme (see [single-theme output](#single-theme-output)). |
| `--min-contrast <RATIO>` | off | Ignored. Contrast correction applies to HTML output only. The flag is accepted for consistency with the other commands. |

Language detection follows the same rules as [`kazari render`](/docs/reference/cli-render#language-detection): file name first, then `--lang`, then `--meta`.

## The preamble

By default, the output starts with the `#code-block` Typst template (about 80 lines). This template defines the `code-block` and `code-line` functions that the generated calls depend on. The template defaults to `DejaVu Sans Mono` at `9pt`. `--font` and `--font-size` change both per block without changing the preamble. To change the marker colours, set [`typst.markerColors`](/docs/reference/configuration#typst-options) in the config file.

A self-contained `.typ` file:

```bash
kazari typst main.rs > main.typ
typst compile main.typ main.pdf
```

When building a multi-block document, include the preamble once at the top and append blocks with `--no-preamble`:

```bash
kazari typst first.rs > document.typ
kazari typst second.py --no-preamble >> document.typ
```

Or emit the preamble separately and include it from a master document:

```bash
kazari typst first.rs --no-preamble > blocks.typ
```

```typst title="document.typ"
#import "blocks.typ": *
// Or inline the preamble from kazari typst's first run
```

## Single-theme output

Typst produces a static PDF. There is no dark-mode toggle or CSS variable mechanism, so the Typst path uses only the light theme. The `--theme-dark` flag is accepted for consistency with other commands but has no effect on the output.

To produce a dark-themed PDF, pass the dark theme as `--theme-light`:

```bash
kazari typst main.rs --theme-light github-dark > dark.typ
```

## Output structure

Each block is a `#code-block(...)` call containing `#code-line(...)` calls, one per source line. Tokens are `#text(fill: rgb("..."), weight: "bold", style: "italic", "content")` calls. All content is emitted as Typst string literals in code mode, so markup characters (`//`, `--`, `~`, quotes, URLs) are safe.

The generated output supports:

- Line numbers with auto-width gutter
- Line markers (highlight, ins, del, error, warning) with coloured backgrounds and labels
- Focus lines (unfocused lines are transparentized)
- Inline markers (`#highlight` spans)
- Inline links (`#link`)
- Hanging indent with preserved leading whitespace
- Titles and language labels

Terminal frames, wrap toggle, copy buttons, and visible whitespace have no Typst counterpart and are ignored.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Success. |
| 2 | Usage error, invalid config, unknown theme, or a rendering error. |
