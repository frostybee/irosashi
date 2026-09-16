# Changelog

All notable changes to the Irosashi workspace are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2026-09-16

### Fixed

- **crates.io README images.** Image sources and in-repo links are now absolute GitHub
  URLs so they render correctly on crates.io. The workspace README previously used
  relative paths, which crates.io resolved against the crate subdirectory.

### Changed

- `kazari-rs` now has its own README, brand assets (logo and favicon from Go Kazari),
  and crate description on crates.io, instead of inheriting the workspace README.
- Showcase examples ported from Go to Rust; the engine is named Irosashi throughout.
- CI: `actions/checkout` bumped from v4 to v5 (silences the Node.js 20 deprecation
  warning on GitHub runners).
- CI: `.gitattributes` forces LF checkout for all text files, fixing a
  Windows-only test failure where `include_str!` CSS assets gained CRLF line endings.

## [0.1.0] - 2026-09-16

Initial release of `irosashi` and `kazari-rs` on crates.io.

### irosashi

- **TextMate grammar engine.** A Rust port of vscode-textmate's tokenizer covering
  begin/end rules, begin/while rules, captures, end-pattern backreferences, the `\G`
  anchor, injections, embedded grammars, and `$self`/`$base`/`#repository` includes.
  Pattern quirks are mirrored: `\z` rewrite, empty `match` no-op, `captures` fallback
  for `beginCaptures`/`endCaptures`, empty rules removed to a fixpoint.
- **Native Oniguruma.** The real regex engine via `onig-regset`, using its regset API to
  search a whole pattern set per call. No WASM, no subprocess.
- **234/234 fidelity.** Every tested grammar produces output byte-identical to
  `vscode-textmate` across both `github-dark` and `github-light` (468 of 468
  grammar/theme pairs). The Shiki HTML preset is byte-identical to Shiki 4.4.3 on
  generated goldens.
- **Six output formats.** HTML (two dialects: Irosashi and Shiki), ANSI terminal
  (8/16/256/truecolor), SVG, JSON, plain text, and raw tokens.
- **Multi-theme mode.** Tokenize once and resolve colours from several themes. Supports
  inline styles from the first key with CSS variables for the rest, or a single
  `light-dark()` declaration per token.
- **Transformer pipeline.** Built-in `Notation` (`[!code ++]`, `[!code highlight]`,
  `[!code focus]`), `Meta` (`{1,3-5}`), and `Whitespace` (visible tabs and spaces),
  plus the `Transformer` trait for custom hooks.
- **Per-line session API.** `Session::tokenize_line` with an explicit `StateStack`
  handle for incremental editor and preview use.
- **Style-to-class mode.** Deterministic hashed class names via `StyleClassMap`, with
  one shared stylesheet across code blocks.
- **Embedded assets.** 257 grammars and 65 VS Code themes compiled into the binary
  (feature `embedded-assets`, on by default), or loaded from a directory with
  `HighlighterBuilder::from_dir`.
- **Language detection.** By file extension, exact filename, or first-line shebang,
  plus runtime alias and extension registration.
- **WCAG contrast correction.** Optional foreground adjustment against the theme
  background (`HighlighterBuilder::min_contrast`), off by default.
- **Safety.** Max-line-length guard, per-line panic recovery reported as `Diagnostic`,
  fuzz targets for `parse_grammar` and `tokenize`. No panics on any input.
- **Deterministic output.** Attributes and styles in sorted key order; class hashes
  stable across runs.

### kazari-rs

- **Frames and chrome.** Editor and terminal frames with macOS-style dots (coloured or
  minimal), automatic frame detection, title bars with file icons and language badges,
  file name extraction from code comments.
- **Toolbar.** Copy to clipboard, word wrap toggle, fullscreen with font size controls,
  per-block theme toggle.
- **Line markers.** Highlight, insertion, and deletion markers with coloured backgrounds
  and gutter accents, range syntax (`{3, 5-8}`), labeled ranges (`{"API":3-7}`),
  overlap resolution.
- **Inline markers.** Literal text, regex with capture groups, inline `ins=`/`del=`
  spanning multiple tokens.
- **Focus lines.** `focus={N-M}` dims every line outside the focused range.
- **Notation comments.** `// [!code ++]`, `// [!code highlight]`, `// [!code focus]`,
  `// [!code error]`, `// [!code warning]`.
- **Line numbers.** Engine-wide, per-language, and per-block control with
  `startLineNumber=N` and auto-width gutter.
- **Word wrap.** `wrap`, `preserveIndent`, `hangingIndent=N`.
- **Collapsible sections.** Threshold-based auto-collapse, range-based
  `collapse={N-M}`, four styles (github, collapsible-start, collapsible-end,
  collapsible-auto).
- **Code groups.** `:::code-group` tabbed containers with tab sync (feature
  `markdown`).
- **Themes.** Dual-theme rendering via CSS custom properties, per-block override
  (`theme="dracula"`), dark mode via class selector, media query, or both, OKLCH-based
  adjustments and a theme customizer callback, theme-only stylesheet.
- **Output panel.** `withOutput` and `---output---` separator split code and output
  into one block.
- **Mermaid pass-through.** Fenced `mermaid` blocks emit raw code for Mermaid.js.
- **Typst renderer.** `render_with_meta_typst` emits `#code-block(...)` calls with the
  same colours as the HTML; `typst_preamble()` ships the template.
- **pulldown-cmark adapter.** `render_markdown` and `highlight_events` (feature
  `markdown`).
- **YAML config.** `kazari.config.yaml` with the same camelCase keys as Go Kazari.
- **Post-render callbacks.** `KazariBuilder::post_render` for wrapping or injecting
  elements after rendering.
- **CSS/JS minification** and content-hashed asset file names.
- **Showcase generator.** 59-example multi-page site (`cargo run -p kazari-rs
  --features markdown --example showcase`).
- **i18n.** en-US, fr-FR, ja-JP with per-string overrides.
- **Accessibility.** WCAG contrast per token (including against marker backgrounds),
  ARIA labels, live regions, `all: revert` style reset, cascade layer.

## Releasing

The changelog is maintained by hand. To cut a release:

1. Add entries under `[Unreleased]` as work lands on `main`.
2. Rename `[Unreleased]` to the new version with today's date, add a fresh empty
   `[Unreleased]` above it, and update the link references at the bottom of this file.
3. Tag the release: `git tag -a vX.Y.Z -m "vX.Y.Z"` and push it.
4. Publish the GitHub release: `gh release create vX.Y.Z --notes-file <section>`.

[Unreleased]: https://github.com/frostybee/irosashi/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/frostybee/irosashi/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/frostybee/irosashi/releases/tag/v0.1.0
