---
title: "Meta string syntax"
description: "Complete reference for every token in the markdown fence info string."
sidebar:
  order: 4
---

The meta string is the text after the opening triple backticks in a fenced code block. Kazari parses it into per-block options, markers, focus ranges, and collapse directives. Tokens are separated by whitespace.

````
```rust title="main.rs" showLineNumbers {2, 4-6}
````

This produces a Rust code block with a `main.rs` title tab, lines 2 and 4 through 6 highlighted, and line numbers enabled.

## Block options

The language must be the first token. It must not contain `=` or start with `{`, `"`, or `'`.

Values after `=` accept double quotes, single quotes, or bare words: `title="My Title"`, `title='My Title'`, and `title=MyTitle` are all valid.

| Token | Description | Example |
|-------|-------------|---------|
| *language* | First bare word sets the language | `rust`, `typescript`, `bash` |
| `title="..."` | Title shown in the frame title bar | `title="main.rs"` |
| `frame=` | Frame type: `code`, `terminal`, `none`, `auto` | `frame=terminal` |
| `theme="..."` | Override the theme for this block | `theme="dracula"` |
| `showLineNumbers` | Enable line numbers | `showLineNumbers` |
| `showLineNumbers=false` | Disable line numbers (overrides the engine default) | `showLineNumbers=false` |
| `startLineNumber=N` | First displayed line number | `startLineNumber=10` |
| `wrap` | Enable word wrap | `wrap` |
| `preserveIndent` | Preserve indentation on wrapped lines | `preserveIndent` |
| `preserveIndent=false` | Disable indent preservation | `preserveIndent=false` |
| `hangingIndent=N` | Extra indent columns for wrapped continuation lines | `hangingIndent=2` |
| `lang="..."` | Language for diff+syntax hybrid rendering | `lang="rust"` |

## Line markers

Line markers highlight, insert, or delete entire lines. When markers overlap on the same line, higher-priority types win: mark (lowest) < del < ins (highest).

| Token | Type | Description | Example |
|-------|------|-------------|---------|
| `{N-M,...}` | mark | Highlight lines | `{3-5}`, `{2,4-6}` |
| `{"Label":N-M}` | mark | Highlight with a labeled badge | `{"Added":3-5}` |
| `ins={N-M,...}` | ins | Mark lines as inserted (green) | `ins={10-12}` |
| `ins={"Label":N-M}` | ins | Inserted with a label | `ins={"New":1-3}` |
| `del={N-M,...}` | del | Mark lines as deleted (red) | `del={7}` |
| `del={"Label":N-M}` | del | Deleted with a label | `del={"Removed":4-6}` |
| `add={...}` | ins | Alias for `ins=` | `add={1-3}` |
| `rem={...}` | del | Alias for `del=` | `rem={7}` |

Multiple line marker tokens in the same meta string each produce a separate marker entry.

## Inline markers

Inline markers highlight text spans within a line. Bare quoted strings at any position are parsed as inline markers, not as the language.

| Token | Type | Description | Example |
|-------|------|-------------|---------|
| `"text"` | mark | Highlight all occurrences of text | `"useState"` |
| `'text'` | mark | Same, with single quotes | `'myFunction'` |
| `/regex/` | mark | Highlight all regex matches | `/fn\s+\w+/` |
| `ins="text"` | ins | Mark text as inserted | `ins="added"` |
| `ins=/regex/` | ins | Mark regex matches as inserted | `ins=/new\w+/` |
| `del="text"` | del | Mark text as deleted | `del="removed"` |
| `del=/regex/` | del | Mark regex matches as deleted | `del=/old\w+/` |
| `add="text"` | ins | Alias for `ins="text"` | `add="added"` |
| `rem="text"` | del | Alias for `del="text"` | `rem="removed"` |

Use `\/` to escape literal slashes inside regex patterns: `/\/path\//` matches `/path/`.

## Focus

Focus dims every line outside the specified ranges. The focused lines keep full opacity.

| Token | Description | Example |
|-------|-------------|---------|
| `focus={N-M,...}` | Dim all lines except the specified ranges | `focus={1-3,7}` |

## Collapsible sections

Collapse directives control whether and how a code block is collapsible.

| Token | Description | Example |
|-------|-------------|---------|
| `collapse` | Force collapse on this block | `collapse` |
| `nocollapse` | Disable collapse on this block | `nocollapse` |
| `collapse={N-M}` | Collapse specific line ranges | `collapse={5-15}` |
| `collapse={N-M,P-Q}` | Collapse multiple ranges | `collapse={3-8,20-24}` |
| `collapseThreshold=N` | Per-block line threshold | `collapseThreshold=20` |
| `collapseStyle="..."` | Collapse style | `collapseStyle="collapsible-auto"` |

Collapse styles: `github` (default), `collapsible-start`, `collapsible-end`, `collapsible-auto`.

## Output panel

The output panel splits a single code block into a syntax-highlighted command section and a plain-text output section below it.

| Token | Description | Example |
|-------|-------------|---------|
| `withOutput` | Split on `---output---` into code and output | `withOutput` |
| `outputCollapsed` | Start the output panel collapsed | `outputCollapsed` |
| `outputCollapsed=false` | Start the output panel expanded | `outputCollapsed=false` |
| `outputLabel="..."` | Label for the output section | `outputLabel="Result"` |

The separator line `---output---` must appear literally in the code. Everything above it is highlighted as the declared language. Everything below it renders as plain text in the output panel.

## Diff+syntax hybrid

Use `diff` as the fence language with `lang=` to strip diff prefixes and apply ins/del markers while highlighting in the original language.

````
```diff lang="rust"
 use std::collections::HashMap;
-use std::fmt;
+use std::fmt::Display;
````

Lines starting with `+` receive insertion markers, lines starting with `-` receive deletion markers, and lines starting with a space are unmarked. The `lang=` token sets the language for syntax highlighting after the prefixes are stripped.

The copy button copies the code after the change: prefixes are stripped and lines starting with `-` are left out. With `notationComments` enabled, lines annotated with `[!code --]` are left out of the copied text in the same way. A plain `diff` fence without `lang=` is copied verbatim, so it stays a valid patch.
