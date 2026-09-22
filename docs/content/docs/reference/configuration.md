---
title: "Configuration"
description: "Builder methods, YAML config file, and per-block meta string overrides."
sidebar:
  order: 5
---

Kazari has three configuration layers. Each overrides the previous:

1. **Builder methods** set defaults at construction via `Kazari::builder(hl)`.
2. **Config file** (`kazari.config.yaml`) loaded with `.config_file(yaml)`.
3. **Meta string** on each fenced code block (highest priority).

## Builder

```rust
use kazari_rs::{CollapsibleConfig, DarkMode, Kazari};

let hl = irosashi::Highlighter::new()?;
let kz = Kazari::builder(hl)
    .themes("github-light", Some("github-dark"))
    .dark_mode(DarkMode::Selector(".dark".into()))
    .copy_button(true)
    .line_numbers(true)
    .collapsible(CollapsibleConfig {
        line_threshold: 15,
        preview_lines: 5,
        ..Default::default()
    })
    .min_contrast(5.5)
    .minify(true)
    .locale("en-US")
    .build()?;
```

See [KazariBuilder on docs.rs](https://docs.rs/kazari-rs/latest/kazari_rs/struct.KazariBuilder.html) for the full method list.

## Config file

Create `kazari.config.yaml` in the project directory. Keys use camelCase for compatibility with content written for Go Kazari. A config written for the Go library works unchanged.

```rust
let yaml = std::fs::read_to_string("kazari.config.yaml").expect("config file");
let kz = Kazari::builder(hl).config_file(&yaml)?.build()?;
```

Complete example with all recognized keys:

```yaml title="kazari.config.yaml"
themes:
  light: github-light
  dark: github-dark-default
darkMode:
  kind: selector
  selector: '[data-theme="dark"]'
copyButton: true
fullscreenButton: true
wrapButton: true
lineNumbers: true
frameDetection: true
fileNameExtraction: true
languageBadge: true
langIconMode: iconAndText
fileIcons: true
mermaidPassThrough: true
inlineLinks: true
themedScrollbars: true
themedSelection: true
terminalCommentStripping: true
terminalDotStyle: colored
dataLineCount: true
tabWidth: 2
minContrast: 5.5
minify: true
styleReset: true
cascadeLayer: kazari
themeCssRoot: ":root"
locale: en-US
outputPanel: true
outputDefaultCollapsed: false
outputSeparator: "---output---"
notationComments: true
visibleWhitespace: false
defaults:
  wrap: true
  preserveIndent: true
  hangingIndent: 0
  lineNumbers: true
  frame: auto
languageDefaults:
  "bash,sh,zsh,shell":
    frame: terminal
    lineNumbers: false
  "console,powershell":
    frame: terminal
    lineNumbers: false
languageAliases:
  ts: typescript
  js: javascript
  rs: rust
styleOverrides:
  --kz-radius: "0.75rem"
  --kz-font-size: "0.85rem"
collapsible:
  lineThreshold: 15
  previewLines: 5
  defaultCollapsed: true
  style: github
uiStrings:
  copy.label: Copy
  copy.success: Copied!
  collapse.expand: Show more
  collapse.collapse: Show less
typst:
  font: "JetBrains Mono"
  size: 10pt
  markerColors:
    mark: "#fff8c5"
    ins: "#dafbe1"
    del: "#ffebe9"
```

## Key reference

Each row maps a YAML key to its builder method and the default value from `Config::default()`.

| YAML key | Builder method | Default |
|----------|---------------|---------|
| `themes.light` | `.themes("...", ...)` | `"github-light"` |
| `themes.dark` | `.themes(..., Some("..."))` | `Some("github-dark")` |
| `darkMode` | `.dark_mode(DarkMode::...)` | `Selector(".dark")` |
| `copyButton` | `.copy_button(bool)` | `true` |
| `wrapButton` | `.wrap_button(bool)` | `true` |
| `fullscreenButton` | `.fullscreen_button(bool)` | `true` |
| `themeToggle` | `.theme_toggle(bool)` | `false` |
| `outputPanel` | `.output_panel(bool)` | `false` |
| `outputDefaultCollapsed` | `.output_collapsed(bool)` | `false` |
| `outputSeparator` | `.output_separator("...")` | `""` |
| `inlineLinks` | `.inline_links(bool)` | `false` |
| `codeGroups` | `.code_groups(bool)` | `false` (requires `markdown` feature). `kazari markdown` defaults to `true`. |
| `mermaidPassThrough` | `.mermaid_pass_through(bool)` | `true` |
| `lineNumbers` | `.line_numbers(bool)` | depends on `defaults.lineNumbers` |
| `frameDetection` | `.frame_detection(bool)` | `true` |
| `fileNameExtraction` | `.file_name_extraction(bool)` | `true` |
| `languageBadge` | `.language_badge(bool)` | `true` |
| `fileIcons` | `.file_icons(bool)` | `true` |
| `langIconMode` | `.lang_icon_mode(LangIconMode::...)` | `None` |
| `minContrast` | `.min_contrast(f64)` | `0.0` (disabled) |
| `minify` | `.minify(bool)` | `true` |
| `tabWidth` | `.tab_width(usize)` | `2` |
| `terminalDotStyle` | `.terminal_dot_style(TerminalDotStyle::...)` | `Colored` |
| `notationComments` | `.notation_comments(bool)` | `false` |
| `visibleWhitespace` | `.visible_whitespace(bool)` | `false` |
| `locale` | `.locale("...")` | `"en-US"` |
| `cascadeLayer` | `.cascade_layer("...")` | `"kazari"` |
| `themeCssRoot` | `.theme_css_root("...")` | `":root"` |
| `styleOverrides` | `.style_overrides(BTreeMap)` | empty |
| `styleReset` | config only | `true` |
| `themedScrollbars` | config only | `true` |
| `themedSelection` | config only | `false` |
| `terminalCommentStripping` | config only | `true` |
| `dataLineCount` | config only | `true` |
| `collapsible` | `.collapsible(CollapsibleConfig { ... })` | `None` (disabled) |
| `uiStrings` | `.ui_strings(HashMap)` | built-in en-US strings |
| `typst.font` | `.typst_font("...")?` | template default, `DejaVu Sans Mono` |
| `typst.size` | `.typst_size("...")?` | template default, `9pt` |
| `typst.markerColors` | `.typst_marker_color("...", "...")?` | template palette |
| `languageDefaults` | YAML only | empty |
| `languageAliases` | YAML only | empty |
| `defaults` | YAML only | `wrap: false`, `frame: auto` |

## Typst options

The `typst` section applies to Typst output only. HTML output ignores it.

| Key | Accepted values |
|-----|-----------------|
| `font` | A font family name installed where the document is compiled. |
| `size` | A Typst length: a number followed by `pt`, `em`, `mm`, `cm`, or `in`. |
| `markerColors` | A map from `mark`, `ins`, `del`, `error`, or `warning` to a hex colour (`#rgb`, `#rrggbb`, or `#rrggbbaa`). Kinds left out keep the template colour. |

The builder methods return `Result`, and an invalid value is a config error in both the builder and the config file. The overrides are written into each `#code-block(...)` call, so the preamble stays the same for every configuration.

## Meta string

Per-block overrides go after the language in the opening fence. They take the highest priority.

````
```rust title="main.rs" showLineNumbers {2, 4-6}
````

See [Meta string syntax](/docs/reference/meta-string-syntax/) for the full token reference.
