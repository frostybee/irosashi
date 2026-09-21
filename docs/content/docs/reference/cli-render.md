---
title: "`kazari render` reference"
description: "Flags, language detection, stdin usage, and output modes for the render command."
sidebar:
  order: 9
---

`kazari render` highlights a single source file and prints a decorated HTML block. This page covers every flag and the language detection logic.

For installation and a quick walkthrough, see the [command line getting started](/docs/getting-started/cli).

## Flags

```
kazari render <input> [flags]
```

`input` is a file path or `-` for stdin.

| Flag | Default | Effect |
|---|---|---|
| `--lang <NAME>` | detected from the file name | Language name. Overridden by `--meta` when both are given. |
| `--meta <META>` | none | Full fence meta string. Overrides `--lang` and adds per-block options (title, line numbers, markers, focus). Uses the same syntax as the [meta string reference](/docs/reference/meta-string-syntax). |
| `--page` | off | Wrap the output in a standalone HTML page with the stylesheet and script inlined. |
| `--config <PATH>` | auto-discover | Path to a config file. Without it, the tool probes `kazari.config.yaml`, `.yml`, and `.json` in the working directory. |
| `--theme-light <NAME>` | `github-light` | Light syntax theme. Overrides the config file. |
| `--theme-dark <NAME>` | `github-dark` | Dark syntax theme. Overrides the config file. |
| `--min-contrast <RATIO>` | `minContrast` from the config, else off | Minimum WCAG contrast ratio of token colours against the block background, from 0 to 21. Colours below the ratio are moved toward black or white. `0` turns the correction off. Overrides the config file. See [`minContrast`](/docs/reference/configuration#key-reference). |

## Language detection

When neither `--lang` nor `--meta` is given, the language is detected from the input file name using Irosashi's built-in tables: exact file name first (`Makefile`, `Dockerfile`, `.gitignore`), then the lowercased extension (`.rs` to `rust`, `.py` to `python`, `.tsx` to `tsx`). Stdin (`-`) has no file name, so it falls back to plain text.

`--lang` overrides detection. `--meta` overrides both, since the first token of a meta string is the language:

```bash
# Detected from file name: rust
kazari render src/main.rs

# Explicit language
kazari render snippet.txt --lang python

# Full meta string (language is the first token, overrides --lang)
kazari render snippet.txt --meta 'rust title="main.rs" showLineNumbers {2-4}'
```

## Output modes

Without `--page`, the output is an HTML fragment: one `div.kazari-block` element. To display it on a page, inject the stylesheet and script separately:

```bash
kazari css > kazari.css
kazari js  > kazari.js
kazari render main.rs >> page.html
```

With `--page`, the output is a complete HTML document with `<html>`, `<head>`, and `<body>`. The stylesheet and script are inlined. The page title is the input file name, or `kazari` for stdin.

```bash
kazari render main.rs --page > main.html
```

## Reading from stdin

Pass `-` as the input to read from stdin. Combine with `--lang` since there is no file name to detect from:

```bash
echo 'print("hello")' | kazari render - --lang python
```

```bash
cat snippet.go | kazari render - --meta 'go title="handler.go" {3}'
```

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Success. |
| 2 | Usage error, invalid config, unknown theme, or a rendering error. |
