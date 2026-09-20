<p align="center">
  <img src="https://raw.githubusercontent.com/frostybee/irosashi/main/crates/kazari-rs/brand/kazari-logo.svg" alt="Kazari" width="160">
</p>

<h1 align="center">kazari-cli</h1>

<p align="center">
  <a href="https://crates.io/crates/kazari-cli"><img src="https://img.shields.io/crates/v/kazari-cli.svg" alt="crates.io"></a>
  <a href="https://github.com/frostybee/irosashi/blob/main/LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
</p>

`kazari` is the command line for [Kazari](https://crates.io/crates/kazari-rs), the code block
presentation layer built on [Irosashi](https://crates.io/crates/irosashi), a Rust port of Shiki.
It gives any static site framed, syntax highlighted code blocks with VS Code themes, without
touching the site's Markdown pipeline:

```bash
kazari process ./public --check   # report pending changes, exit 1 if any
kazari process ./public           # write the upgraded HTML in place
```

`process` walks a directory of built HTML, recognizes the code blocks Hugo, Jekyll, Eleventy,
mdBook, Sphinx, Zola, Astro and hand written pages emit, recovers the source, re-renders every
block with Kazari (frames, copy button, line markers, dual light and dark themes), splices the
result back while preserving every other byte, and writes `kazari.css` and `kazari.js` once with
a link and a script tag injected into each page that needs them. Running it again is a no-op.

The other subcommands render code, Markdown or Typst from a file or stdin, print the assets, and
list the bundled themes and languages. No Node, no WASM, no runtime dependencies: one binary.

## Install

Prebuilt binaries for Windows, Linux and macOS are attached to each
[GitHub release](https://github.com/frostybee/irosashi/releases). Download the archive for your
platform, unpack it, and put `kazari` on your `PATH`.

With a Rust toolchain:

```bash
cargo install kazari-cli
```

Building from source needs a C compiler for Irosashi's vendored Oniguruma. On Windows, Visual
Studio with the VC tools component works; on Linux and macOS a system `cc` is enough.

## Commands

```
kazari process [dir] [flags]   upgrade code blocks in built HTML under dir (default ".")
kazari render <file|->         render one source file (or stdin) as a decorated HTML block
kazari markdown <file|->       render a Markdown file (or stdin) to HTML
kazari typst <file|->          render one source file (or stdin) as Typst source
kazari css                     print the page-wide stylesheet
kazari js                      print the page-wide script
kazari themes                  list bundled syntax theme names
kazari languages               list bundled language names
kazari version                 print the kazari version
```

Run `kazari <command> --help` for the flags of a command.

### process

```
kazari process [dir] [flags]

  --check              report would-be changes without writing; exit 1 if any
  --config <PATH>      config file (default: kazari.config.yaml|.yml|.json in dir, then the working directory)
  --theme-light <NAME> light syntax theme (default github-light; overrides config)
  --theme-dark <NAME>  dark syntax theme (default github-dark; overrides config)
  --assets-base <URL>  fixed asset URL prefix instead of per-file relative paths
  --hashed-assets      content hashed asset filenames (kazari-<hash>.css) instead of kazari.css
  --skip-unlabeled     leave blocks without a detectable language untouched
  --concurrency <N>    files processed concurrently (default: number of CPUs)
  --verbose            log per-file progress to stderr
```

The summary line reads `N files, N blocks upgraded, N skipped, N suppressed, N changed`.
Skipped blocks are ones the processor decided to leave alone (no recognizable shape, a
`data-kazari="ignore"` attribute, a Mermaid diagram, or an unlabeled block under
`--skip-unlabeled`). Suppressed blocks sit inside Kazari's own output from an earlier run.

Exit status: `0` on success, `1` from `--check` when something would change, `2` on a usage,
config, theme, or per-file error.

**Assets:** `kazari.css` and `kazari.js` are written to the root of `dir`. Pages reference them
by a relative path computed from their depth (`../../kazari.css?v=<hash>`), so subpath
deployments such as GitHub Pages project sites work unchanged. `--assets-base https://cdn/x`
makes every page use that prefix instead. `--hashed-assets` embeds the content hash in the
file name for long-lived caching; old hashed files from previous runs are not removed.

**Re-runs and config changes:** blocks already rendered by Kazari are skipped, and the link and
script tags from a previous run are rewritten to the current asset hashes even on pages that
have no code blocks. A config change followed by `kazari process` therefore updates the whole
site in place.

**Per-block options in built HTML:** a `data-kz-meta` attribute on the `<pre>` or `<code>`
element carries a full fence meta string through the build:

```html
<pre><code class="language-go" data-kz-meta='go title="main.go" {3} ins={6-7}'>...</code></pre>
```

Hugo users add it from a `render-codeblock.html` hook; the meta syntax is the same as the
fence info string documented in the [kazari-rs README](https://github.com/frostybee/irosashi/blob/main/crates/kazari-rs/README.md#meta-string).
`data-kazari="ignore"` on the block's root element leaves it untouched.

**Recognized shapes:**

| Generator | Shape |
|---|---|
| Hugo (Chroma) | `div.highlight > pre.chroma > code`, inline styles mode, and the `lntable` line number table |
| Jekyll (Rouge) | `div.language-x.highlighter-rouge`, the `figure.highlight` Liquid tag, and `rouge-table` line numbers |
| Sphinx, Pelican (Pygments) | `div.highlight-x > div.highlight > pre` and the `highlighttable` line number table |
| Eleventy (Prism) | `pre.language-x > code.language-x` |
| Zola | `pre[style*=background-color] > code[data-lang]` |
| Astro (Shiki) | `pre.astro-code[data-language] > code` |
| mdBook, markdown-it, hand written | `pre > code[class=language-x]`, with or without a language |

Line number gutters are dropped from the recovered source; Chroma's `hl` line markers become
`{n-m}` highlight ranges. Expressive Code output (Starlight) is left alone.

### render, markdown, typst

```bash
kazari render main.rs                                # HTML fragment, language from the file name
kazari render main.rs --meta 'rust title="main.rs" showLineNumbers {2}' --page > main.html
cat snippet.py | kazari render - --lang python
kazari markdown README.md --page > readme.html       # code groups, GFM tables, footnotes, task lists
kazari typst main.rs > main.typ && typst compile main.typ
```

`--page` wraps the output in a standalone HTML page with the stylesheet and script inlined.
Without it, inject `kazari css` once in `<head>` and `kazari js` once before `</body>`.
`typst` prints the `#code-block` template followed by the block; `--no-preamble` prints only
the block, for appending to a document that already has the template. `--font` and
`--font-size` set the block's font family and text size (a Typst length such as `10pt`).

All of these accept `--config`, `--theme-light` and `--theme-dark` like `process`.

## Configuration

Every command reads `kazari.config.yaml` (or `.yml`, `.json`) from the target directory, then
the working directory; `--config` names one explicitly. The keys are Kazari's, documented in
the [kazari-rs README](https://github.com/frostybee/irosashi/blob/main/crates/kazari-rs/README.md#config-file),
plus a `process` section for this command:

```yaml
themes:
  light: github-light
  dark: github-dark
lineNumbers: true
process:
  skipUnlabeled: false
  assetsBase: ""
  hashedAssets: false
  concurrency: 4
  maxFileBytes: 33554432
```

Flags override the file. `maxFileBytes` (default 32 MiB) skips pages larger than the limit.

## Per-generator notes

- **Hugo:** run `hugo` then `kazari process ./public`. The default `markup.highlight` settings
  are fine; both `noClasses = true` (inline styles) and `noClasses = false` are recognized, as is
  `codeFences = false`. A render hook can add `data-kz-meta` for per-block options.
- **Jekyll:** `jekyll build` then `kazari process ./_site`. Kramdown fences and the Liquid
  `{% highlight %}` tag are both recognized, with or without `linenos`.
- **mdBook:** `mdbook build` then `kazari process ./book`. Blocks come out as plain
  `pre > code.language-x`; mdBook's own highlight.js styling is replaced.
- **Sphinx:** `make html` then `kazari process ./_build/html`. `:linenos:` tables are handled.
- **Eleventy:** with the syntax highlight plugin, `kazari process ./_site`.
- **Zola:** `zola build` then `kazari process ./public`. Requires `highlight_code = true` in
  `config.toml`, which is what produces the recognizable `data-lang` shape.
- **Astro:** built-in Shiki blocks are recognized; Starlight's Expressive Code blocks are left
  as they are.

## Development

Part of the [`irosashi`](https://github.com/frostybee/irosashi) workspace.

```bash
cargo test -p kazari-cli                          # unit, fixture, corpus and CLI tests
KAZARI_UPDATE_GOLDEN=1 cargo test -p kazari-cli   # regenerate testdata/corpus goldens after a deliberate rendering change
cargo run -p kazari-cli -- process ./site --check
```

`testdata/unwrap` holds one fixture per recognized shape with the expected recovered source;
`testdata/corpus` holds small sites processed end to end against committed golden trees.

## License

Copyright (c) 2026 FrostyBee. Licensed under the [MIT License](https://github.com/frostybee/irosashi/blob/main/LICENSE).
