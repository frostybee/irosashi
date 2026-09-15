//! Renders several code blocks as a standalone HTML page showcasing Kazari features.
//! `cargo run -p kazari --example demo > demo.html`, then open the file in a browser.

use kazari::Kazari;

fn main() -> Result<(), kazari::Error> {
    let hl = iro::Highlighter::new().map_err(|e| kazari::Error::Highlight(e.to_string()))?;
    let kz = Kazari::builder(hl)
        .themes("github-light", Some("github-dark"))
        .notation_comments(true)
        .build()?;

    let blocks: Vec<(&str, &str, &str)> = vec![
        // (section title, meta string, code)
        (
            "Code frame with title",
            r#"rust title="hello.rs""#,
            "fn main() {\n    println!(\"Hello, world!\");\n}\n",
        ),
        (
            "Line numbers and markers",
            "javascript showLineNumbers {2-3} ins={5}",
            "function greet(name) {\n  const msg = `Hello, ${name}!`;\n  console.log(msg);\n\n  return msg;\n}\n",
        ),
        (
            "Terminal frame",
            "bash",
            "# Install dependencies\ncargo build --workspace\ncargo test --workspace\n",
        ),
        (
            "Diff with language highlighting",
            r#"diff lang="go""#,
            " func main() {\n-    fmt.Println(\"old\")\n+    fmt.Println(\"new\")\n }\n",
        ),
        (
            "Inline markers",
            r#"python "print""#,
            "def greet(name):\n    print(f\"Hello, {name}!\")\n    print(\"Done.\")\n",
        ),
        (
            "Focus lines",
            "typescript focus={2-3}",
            "interface User {\n  name: string;\n  email: string;\n  age?: number;\n}\n",
        ),
        (
            "Word wrap with preserved indent",
            r#"css wrap preserveIndent title="styles.css""#,
            ".kazari-block .kz-line .kz-code span[style] { transition: color 0.2s ease-in-out, background-color 0.2s ease-in-out; }\n.kazari-block pre { overflow-x: auto; }\n",
        ),
        (
            "Comment notation (engine built with notation_comments)",
            "javascript showLineNumbers",
            "const total = 0; // [!code --]\nlet total = 0; // [!code ++]\nfor (const n of items) total += n; // [!code highlight]\nreturn totl; // [!code error]\n// [!code word:total]\n",
        ),
    ];

    let mut rendered = String::new();
    for (title, meta, code) in &blocks {
        let html = kz.render_with_meta(code, meta)?;
        rendered.push_str(&format!(
            "<section>\n<h2>{title}</h2>\n<p class=\"meta\"><code>{meta}</code></p>\n{html}\n</section>\n",
        ));
    }

    let hl = iro::Highlighter::new().map_err(|e| kazari::Error::Highlight(e.to_string()))?;
    let visible = Kazari::builder(hl)
        .themes("github-light", Some("github-dark"))
        .visible_whitespace(true)
        .build()?;
    let meta = r#"python title="visible whitespace""#;
    let html = visible.render_with_meta("def greet(name):\n    return f\"Hi {name}\"\n", meta)?;
    rendered.push_str(&format!(
        "<section>\n<h2>Visible whitespace (engine built with visible_whitespace)</h2>\n<p class=\"meta\"><code>{meta}</code></p>\n{html}\n</section>\n",
    ));

    print!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Kazari Demo</title>
<style>
{css}
</style>
<style>
body {{
  font-family: system-ui, -apple-system, sans-serif;
  max-width: 48rem;
  margin: 2rem auto;
  padding: 0 1rem;
  background: #f6f8fa;
  color: #1f2328;
  transition: background-color 0.2s, color 0.2s;
}}
.dark body {{
  background: #0d1117;
  color: #e6edf3;
}}
header {{
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 0.5rem;
}}
h1 {{
  font-size: 1.75rem;
  margin: 0;
}}
h2 {{
  font-size: 1.1rem;
  margin-top: 2rem;
  margin-bottom: 0.25rem;
}}
p.meta {{
  font-size: 0.85rem;
  color: #656d76;
  margin: 0 0 0.5rem;
}}
.dark p.meta {{
  color: #8b949e;
}}
p.meta code {{
  background: #eff1f3;
  padding: 0.15em 0.4em;
  border-radius: 4px;
  font-size: 0.85em;
}}
.dark p.meta code {{
  background: #21262d;
}}
.theme-toggle {{
  border: 1px solid #d0d7de;
  background: #fff;
  color: #1f2328;
  padding: 0.35em 0.75em;
  border-radius: 6px;
  cursor: pointer;
  font-size: 0.85rem;
  font-family: inherit;
}}
.dark .theme-toggle {{
  border-color: #30363d;
  background: #21262d;
  color: #e6edf3;
}}
</style>
</head>
<body>
<header>
<h1>Kazari Demo</h1>
<button class="theme-toggle" onclick="document.documentElement.classList.toggle('dark');this.textContent=document.documentElement.classList.contains('dark')?'Light':'Dark'">Dark</button>
</header>
<p>Each block shows a different feature. The <code>meta</code> line is the fence meta string passed to <code>render_with_meta</code>.</p>
{rendered}
<script>
{js}
</script>
</body>
</html>"#,
        css = kz.css(),
        js = kz.js(),
        rendered = rendered,
    );

    Ok(())
}
