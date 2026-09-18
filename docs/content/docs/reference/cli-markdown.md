---
title: "`kazari markdown` reference"
description: "Flags, Markdown extensions, code group support, and output modes for the markdown command."
sidebar:
  order: 10
---

`kazari markdown` renders an entire Markdown document to HTML. Every fenced code block is highlighted and decorated by Kazari; prose is rendered by pulldown-cmark. This page covers every flag, the enabled Markdown extensions, and code group support.

For installation and a quick walkthrough, see the [command line getting started](/docs/getting-started/cli).

## Flags

```
kazari markdown <input> [flags]
```

`input` is a file path or `-` for stdin.

| Flag | Default | Effect |
|---|---|---|
| `--page` | off | Wrap the output in a standalone HTML page with the stylesheet and script inlined. The page title is the input file stem (`README.md` becomes `README`), or `kazari` for stdin. |
| `--config <PATH>` | auto-discover | Path to a config file. Without it, the tool probes `kazari.config.yaml`, `.yml`, and `.json` in the working directory. |
| `--theme-light <NAME>` | `github-light` | Light syntax theme. Overrides the config file. |
| `--theme-dark <NAME>` | `github-dark` | Dark syntax theme. Overrides the config file. |

## Markdown extensions

The parser enables the following GFM and pulldown-cmark extensions unconditionally:

| Extension | Effect |
|---|---|
| Tables | Pipe tables with alignment (`\|---:\|`) |
| Footnotes | `[^label]` references and definitions |
| Strikethrough | `~~deleted~~` |
| Task lists | `- [x] done`, `- [ ] pending` |
| Heading attributes | `## Title {#custom-id .class}` |

These extensions cannot be disabled from the command line.

## Code blocks

Every fenced code block is passed to `Kazari::render_with_meta`. The info string (the text after the opening fence) is used as the full meta string, so all [meta string options](/docs/reference/meta-string-syntax) work:

````markdown
```rust title="main.rs" showLineNumbers {2-4}
fn main() {
    let x = 1;
    let y = 2;
    println!("{}", x + y);
}
```
````

Indented code blocks (four-space indent, no fence) pass through as plain HTML without Kazari decoration.

Unknown languages are not errors. Irosashi renders them as plain text and attaches a diagnostic.

## Code groups

Code groups are enabled automatically. Use the `:::code-group` container syntax to create tabbed panels:

````markdown
:::code-group

```rust title="Rust"
println!("hello");
```

```python title="Python"
print("hello")
```

:::
````

The output is a tabbed interface with `role="tablist"` and `role="tabpanel"` attributes. Tab labels come from the `title=` meta key, then the extracted file name, then the capitalized language name. Add `sync="key"` to synchronize tab selection across groups on the same page:

````markdown
:::code-group sync="lang"
````

Non-fence content inside a code group is dropped. Nested code groups are not supported.

## Mermaid pass-through

Blocks with `mermaid` as the language are rendered as `<pre class="mermaid">` with the source HTML-escaped, matching the convention that Mermaid's client-side JavaScript expects. This behavior is always on.

## Output modes

Without `--page`, the output is an HTML fragment: the rendered prose and code blocks without `<html>` or `<head>`. Inject the stylesheet and script on the hosting page:

```bash
kazari css > kazari.css
kazari js  > kazari.js
kazari markdown doc.md >> page.html
```

With `--page`, the output is a complete HTML document with the stylesheet and script inlined:

```bash
kazari markdown README.md --page > readme.html
```

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Success. |
| 2 | Usage error, invalid config, unknown theme, or a rendering error (for example, the input file does not exist). |

Errors from the highlighting engine (a grammar that fails to compile a regex) abort the conversion. Unknown languages are not errors.
