---
title: "`kazari process` reference"
description: "Flags, recognized HTML shapes, asset handling, exit codes, and per-generator notes for the process command."
sidebar:
  order: 8
---

`kazari process` upgrades code blocks in a directory of built HTML. This page covers every flag, the HTML shapes the processor recognizes, how assets and per-block options work, and generator-specific setup.

For installation and a quick walkthrough, see the [command line getting started](/docs/getting-started/cli).

## Flags

```
kazari process [dir] [flags]
```

`dir` defaults to `.` (the current directory).

| Flag | Default | Effect |
|---|---|---|
| `--check` | off | Report would-be changes without writing. Exit 1 if anything would change. |
| `--config <PATH>` | auto-discover | Path to a config file. Without it, the tool probes `kazari.config.yaml`, `.yml`, and `.json` in `dir`, then in the working directory. |
| `--engine <NAME>` | `engine` from the config, else `irosashi` | Highlighting backend: `irosashi` (VS Code grammars and themes, Shiki-exact) or `syntect` (Sublime grammars, faster start). With `syntect`, theme names are mapped to its bundled themes and not validated. See [backends](/docs/getting-started/cli#backends). |
| `--theme-light <NAME>` | `github-light` | Light syntax theme. Overrides the config file. |
| `--theme-dark <NAME>` | `github-dark` | Dark syntax theme. Overrides the config file. |
| `--min-contrast <RATIO>` | `minContrast` from the config, else off | Minimum WCAG contrast ratio of token colours against the block background, from 0 to 21. Colours below the ratio are moved toward black or white. `0` turns the correction off. Overrides the config file. See [`minContrast`](/docs/reference/configuration#key-reference). |
| `--assets-base <URL>` | relative | Fixed asset URL prefix instead of per-file relative paths. |
| `--hashed-assets` | off | Content-hashed filenames (`kazari-a1b2c3d4.css`) instead of `kazari.css`. |
| `--skip-unlabeled` | off | Leave blocks without a detectable language untouched instead of rendering them as plain text. |
| `--concurrency <N>` | CPU count | Number of files processed concurrently. |
| `--verbose` | off | Log per-file progress to stderr. |

A typo in a theme name produces a suggestion: `unknown theme "github-drak", did you mean "github-dark"?`

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Success. All blocks processed, no errors. |
| 1 | `--check` mode and at least one file or asset would change. |
| 2 | Usage error, invalid config, unknown theme, or a per-file error. |

## Recognized HTML shapes

The processor identifies code blocks from seven static site generator families plus the generic `pre > code` pattern. Each row shows the root element structure the unwrapper looks for.

| Generator | Shape |
|---|---|
| Hugo (Chroma classes mode) | `div.highlight > pre.chroma > code[data-lang]` |
| Hugo (Chroma inline styles) | `div.highlight > pre > code[data-lang]` (no `.chroma` class) |
| Hugo (Chroma line number table) | `div.highlight > table.lntable` with two `td.lntd` cells |
| Jekyll (Rouge, kramdown) | `div.language-x.highlighter-rouge > div.highlight > pre.highlight > code` |
| Jekyll (Rouge, Liquid tag) | `figure.highlight > pre > code[data-lang]` |
| Jekyll (Rouge, line number table) | `table.rouge-table` with `td.code` |
| Sphinx, Pelican (Pygments) | `div.highlight-x > div.highlight > pre` (no `code` element) |
| Sphinx (Pygments, line numbers) | `div.highlight-x > table.highlighttable > td.code > div.highlight > pre` |
| Eleventy (Prism) | `pre.language-x > code.language-x` (language class on both elements) |
| Zola | `pre[style*=background-color] > code[data-lang]` |
| Astro (Shiki) | `pre.astro-code[data-language] > code` |
| mdBook, markdown-it, hand-written | `pre > code.language-x` or `pre > code` (with or without a language) |

Line number gutters (`ln`, `lnt`, `lntd`, `gutter`, `gl`, `linenos`, `linenodiv`, `line-numbers`, `line-numbers-rows`) are excluded from the recovered source. Chroma's `hl` class on line wrapper spans produces `{n-m}` highlight ranges in the meta string.

Expressive Code output (Starlight) uses `div.ec-line` inside the code element and is left alone. Mermaid blocks (`pre.mermaid`, `code.language-mermaid`, or `code[data-lang=mermaid]`) are also skipped.

## Asset handling

`kazari.css` and `kazari.js` are written to the root of `dir` before any page is processed. Each page that contains at least one rewritten block gets a `<link>` tag before `</head>` and a `<script>` tag before `</body>`, both marked with `data-kazari="assets"`.

Asset URLs default to relative paths computed from each page's depth below the output root (`../../kazari.css?v=<hash>`). This keeps subpath deployments such as GitHub Pages project sites working without configuration. `--assets-base https://cdn.example/static` makes every page use that prefix instead.

`--hashed-assets` embeds the content hash in the filename (`kazari-a1b2c3d4.css`). Old hashed files from previous runs are not removed.

On re-runs, existing `data-kazari="assets"` tags are updated to the current hash even on pages that have no code blocks. This means a configuration change followed by another run refreshes the entire site.

## Per-block options with `data-kz-meta`

A `data-kz-meta` attribute on a `<pre>` or `<code>` element carries a full fence meta string through the build:

```html
<pre><code class="language-go" data-kz-meta='go title="main.go" {3} ins={6-7}'>
package main
// ...
</code></pre>
```

The attribute value uses the same syntax as the [meta string reference](/docs/reference/meta-string-syntax). Hugo users add it from a `render-codeblock.html` template. The attribute value arrives entity-decoded from the HTML parser, so `title="main.go"` works as written; use `&#34;` only if the outer attribute uses double quotes.

To leave a block untouched, add `data-kazari="ignore"` to its root element:

```html
<pre data-kazari="ignore"><code class="language-go">// left as-is</code></pre>
```

## Skipped and suppressed blocks

Blocks are skipped (left untouched) when:

- No unwrapper recognized the HTML shape (`unrecognized-shape`).
- The root element carries a `kz-` or `kazari-` prefixed class, indicating the block is already Kazari output (`kz-class`). These are counted as **suppressed** in the summary.
- The root has `data-kazari="ignore"` (`data-kazari-ignore`).
- The block contains a Mermaid diagram (`mermaid`).
- The block has no detectable language and `--skip-unlabeled` is set (`unlabeled`).

The summary line distinguishes skipped (the processor decided to leave it) from suppressed (already processed by Kazari).

## Config file `process` section

The `process` key in `kazari.config.yaml` sets defaults for `kazari process`. Flags override these values.

```yaml title="kazari.config.yaml"
process:
  skipUnlabeled: false
  assetsBase: ""
  hashedAssets: false
  concurrency: 4
  maxFileBytes: 33554432
```

| Key | Type | Default | Effect |
|---|---|---|---|
| `skipUnlabeled` | bool | `false` | Same as `--skip-unlabeled`. |
| `assetsBase` | string | `""` | Same as `--assets-base`. |
| `hashedAssets` | bool | `false` | Same as `--hashed-assets`. |
| `concurrency` | int | CPU count | Same as `--concurrency`. Must be at least 1. |
| `maxFileBytes` | int | 33554432 | Skip files larger than this byte count. Must be at least 1. |

## Per-generator setup

### Hugo

Build the site, then process the output:

```bash
hugo
kazari process ./public
```

Both `noClasses = true` (Chroma inline styles) and `noClasses = false` (Chroma classes) are recognized, including the `lntable` table layout for line numbers and `codeFences = false` for non-fenced blocks.

To pass per-block options through the build, create a `layouts/_default/_markup/render-codeblock.html` template that adds the `data-kz-meta` attribute:

```go-html-template title="layouts/_default/_markup/render-codeblock.html"
<pre><code class="language-{{ .Type }}" data-kz-meta="{{ .Type }} {{ .Attributes | jsonify }}">
{{- .Inner -}}
</code></pre>
```

### Jekyll

```bash
jekyll build
kazari process ./_site
```

Kramdown fences and the Liquid `{% highlight %}` tag are both recognized, with or without `linenos`.

### mdBook

```bash
mdbook build
kazari process ./book
```

mdBook emits plain `pre > code.language-x` blocks. Its built-in highlight.js styling is replaced by Kazari's.

### Sphinx

```bash
make html
kazari process ./_build/html
```

The `:linenos:` directive produces a `highlighttable` that the processor handles. Pygments class-based output and the leading empty span it emits both work.

### Eleventy

With the syntax highlight plugin:

```bash
npx @11ty/eleventy
kazari process ./_site
```

The Prism shape requires the `language-` class on both the `pre` and `code` elements, which is what the plugin produces.

### Zola

```bash
zola build
kazari process ./public
```

Set `highlight_code = true` in `config.toml`. Zola's recognizable shape is a `pre` with an inline `background-color` style wrapping a `code` element with `data-lang`.

### Astro

Astro's built-in Shiki integration produces `pre.astro-code[data-language]` blocks, which are recognized. Starlight's Expressive Code output uses `div.ec-line` internally and is left as-is.
