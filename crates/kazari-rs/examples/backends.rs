//! Renders the same markdown through a backend chosen at runtime, the way a site
//! generator would from a config key.
//!
//! `cargo run -p kazari-rs --features markdown,syntect --example backends -- syntect`
//! `cargo run -p kazari-rs --features markdown,syntect --example backends -- all`
//!
//! The backend is the first argument or `KAZARI_ENGINE`: `irosashi` (default),
//! `syntect`, or `all` for one page per backend the binary was built with.

use kazari_rs::markdown::render_markdown;
use kazari_rs::{Error, Kazari, KazariBuilder};
use pulldown_cmark::Options;

const MARKDOWN: &str = r#"# Backends

```rust title="hello.rs" showLineNumbers {2} ins={3}
fn main() {
    println!("Hello, world!");
    println!("Added line");
}
```

```bash
cargo build --workspace
```

```python
def greet(name: str) -> str:
    return f"Hello, {name}"  # [!code highlight]
```
"#;

fn builder(engine: &str) -> Result<KazariBuilder, Error> {
    match engine {
        #[cfg(feature = "irosashi")]
        "irosashi" => {
            let hl = irosashi::Highlighter::new().map_err(|e| Error::Highlight(e.to_string()))?;
            Ok(Kazari::builder(hl))
        }
        #[cfg(feature = "syntect")]
        "syntect" => Ok(Kazari::builder(
            kazari_rs::backends::syntect::SyntectHighlighter::new(),
        )),
        #[allow(unreachable_patterns)]
        "irosashi" | "syntect" => Err(Error::Config(format!(
            "this binary was built without the `{engine}` feature"
        ))),
        other => Err(Error::Config(format!("unknown engine {other:?}"))),
    }
}

fn page(engine: &str) -> Result<String, Error> {
    let kz = builder(engine)?
        .themes("github-light", Some("github-dark"))
        .build()?;
    let body = render_markdown(&kz, MARKDOWN, Options::empty())?;
    Ok(format!(
        "<!-- {engine} -->\n<style>\n{css}\n</style>\n{body}\n<script>\n{js}\n</script>\n",
        css = kz.css(),
        js = kz.js(),
    ))
}

fn main() -> Result<(), Error> {
    let engine = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("KAZARI_ENGINE").ok())
        .unwrap_or_else(|| "irosashi".to_owned());

    let engines: Vec<&str> = if engine == "all" {
        let mut all = Vec::new();
        if cfg!(feature = "irosashi") {
            all.push("irosashi");
        }
        if cfg!(feature = "syntect") {
            all.push("syntect");
        }
        all
    } else {
        vec![engine.as_str()]
    };

    for engine in engines {
        print!("{}", page(engine)?);
    }
    Ok(())
}
