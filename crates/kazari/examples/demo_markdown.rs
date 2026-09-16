//! Renders a markdown document through the pulldown-cmark adapter.
//! `cargo run -p kazari --features markdown --example demo_markdown > demo_markdown.html`

use kazari::Kazari;
use kazari::markdown::render_markdown;
use pulldown_cmark::Options;

const MARKDOWN: &str = r#"# Kazari markdown demo

Every fenced block below is rendered by Kazari; the prose is rendered by pulldown-cmark.

```rust title="hello.rs" showLineNumbers {2}
fn main() {
    println!("Hello, world!");
}
```

A terminal frame is detected from the language:

```bash
cargo build --workspace
```

## Code group

:::code-group sync="lang"

```go title="Go"
fmt.Println("hi")
```

```python
# hello.py
print("hi")
```

```js
console.log("hi")
```

:::

## Mermaid pass-through

```mermaid
graph TD;
  A-->B;
```

    indented code stays a plain <pre>
"#;

fn main() -> Result<(), kazari::Error> {
    let hl = iro::Highlighter::new().map_err(|e| kazari::Error::Highlight(e.to_string()))?;
    let kz = Kazari::builder(hl)
        .themes("github-light", Some("github-dark"))
        .code_groups(true)
        .build()?;

    let body = render_markdown(&kz, MARKDOWN, Options::empty())?;

    print!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Kazari markdown demo</title>
<style>
{css}
</style>
<style>
body {{ font-family: system-ui, sans-serif; max-width: 48rem; margin: 2rem auto; padding: 0 1rem; }}
</style>
</head>
<body>
{body}
<script>
{js}
</script>
</body>
</html>"#,
        css = kz.css(),
        js = kz.js(),
        body = body,
    );

    Ok(())
}
