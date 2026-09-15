//! Renders several code blocks as a standalone Typst document.
//! `cargo run -p kazari --example demo_typst > demo.typ`, then `typst compile demo.typ`.

use kazari::Kazari;

fn main() -> Result<(), kazari::Error> {
    let hl = iro::Highlighter::new().map_err(|e| kazari::Error::Highlight(e.to_string()))?;
    let kz = Kazari::builder(hl)
        .themes("github-light", None)
        .notation_comments(true)
        .build()?;

    let blocks: Vec<(&str, &str, &str)> = vec![
        (
            "Code block with title",
            r#"rust title="hello.rs""#,
            "fn main() {\n    println!(\"Hello, world!\");\n}\n",
        ),
        (
            "Line numbers and markers",
            "javascript showLineNumbers {2-3} ins={5}",
            "function greet(name) {\n  const msg = `Hello, ${name}!`;\n  console.log(msg);\n\n  return msg;\n}\n",
        ),
        (
            "Diff with language highlighting",
            r#"diff lang="go""#,
            " func main() {\n-    fmt.Println(\"old\")\n+    fmt.Println(\"new\")\n }\n",
        ),
        (
            "Inline markers",
            r#"python "print" del=/greet/"#,
            "def greet(name):\n    print(f\"Hello, {name}!\")\n    print(\"Done.\")\n",
        ),
        (
            "Focus lines",
            "typescript focus={2}",
            "const a = 1;\nconst b = a + 1; // the focused line\nconst c = b * 2;\n",
        ),
        (
            "Comment notation and labels",
            r#"go showLineNumbers startLineNumber=98 {"API":100}"#,
            "package main\n\nimport \"fmt\" // [!code ++]\n\nfunc main() { fmt.Println(\"hi\") } // [!code highlight]\n",
        ),
        (
            "Long lines wrap with hanging indent",
            "rust hangingIndent=4",
            "    let result = some_function_with_a_long_name(first_argument, second_argument, third_argument, fourth_argument);\n",
        ),
    ];

    println!("{}", kazari::typst_preamble());
    println!("#set page(width: 16cm, height: auto, margin: 1cm)\n");
    for (title, meta, code) in blocks {
        println!("== {title}\n");
        println!(
            "#raw(\"{}\")\n",
            meta.replace('\\', "\\\\").replace('"', "\\\"")
        );
        println!("{}\n", kz.render_with_meta_typst(code, meta)?);
    }
    Ok(())
}
