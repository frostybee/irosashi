use std::collections::BTreeMap;
use std::path::Path;

use iro::{
    CodeToHtmlOptions, DefaultColor, Highlighter, HighlighterBuilder, HtmlRenderer, StyleClassMap,
};

fn highlighter() -> Highlighter {
    HighlighterBuilder::from_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("assets"))
        .build()
        .unwrap()
}

fn dual() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("light".to_owned(), "github-light".to_owned()),
        ("dark".to_owned(), "github-dark".to_owned()),
    ])
}

fn code_of(html: &str) -> &str {
    let start = html.find("<code").unwrap();
    let end = html.rfind("</code>").unwrap();
    &html[start..end]
}

#[test]
fn single_theme_structure() {
    let h = highlighter();
    let html = h
        .code_to_html(
            "package main\n\nfunc main() {}",
            &CodeToHtmlOptions::new("go", "github-dark"),
        )
        .unwrap();
    assert!(html.starts_with(
        r#"<pre class="iro github-dark" style="background-color:#24292e;color:#e1e4e8" tabindex="0"><code><span class="line">"#
    ));
    assert!(html.ends_with("</span></code></pre>"));
    assert!(html.contains(r#"<span style="color:#f97583">package</span>"#));
    assert!(html.contains("</span>\n<span class=\"line\"></span>\n<span class=\"line\">"));
    assert!(!html.ends_with("\n</code></pre>"));
    assert!(!html.contains("iro-themes"));
    assert!(!html.contains("--iro-"));
}

#[test]
fn plaintext_full_string_and_empty_input() {
    let h = highlighter();
    let html = h
        .code_to_html("a b", &CodeToHtmlOptions::new("text", "github-dark"))
        .unwrap();
    assert_eq!(
        html,
        r#"<pre class="iro github-dark" style="background-color:#24292e;color:#e1e4e8" tabindex="0"><code><span class="line"><span style="color:#e1e4e8">a b</span></span></code></pre>"#
    );
    let empty = h
        .code_to_html("", &CodeToHtmlOptions::new("go", "github-dark"))
        .unwrap();
    assert!(empty.contains("<code></code>"));
    let unknown = h
        .code_to_html("x", &CodeToHtmlOptions::new("nope", "github-dark"))
        .unwrap();
    assert!(unknown.contains(r#"<span style="color:#e1e4e8">x</span>"#));
}

#[test]
fn source_is_escaped() {
    let h = highlighter();
    let html = h
        .code_to_html(
            "<script>alert(\"&amp;\")</script>",
            &CodeToHtmlOptions::new("text", "github-dark"),
        )
        .unwrap();
    assert!(html.contains("&lt;script&gt;alert(\"&amp;amp;\")&lt;/script&gt;"));
    assert!(!code_of(&html).contains("<script>"));
}

#[test]
fn pre_and_code_classes_and_attributes() {
    let h = highlighter();
    let mut options = CodeToHtmlOptions::new("go", "github-dark");
    options.html.pre_class = Some("my-pre".to_owned());
    options.html.code_class = Some("my-code".to_owned());
    options.html.pre_attrs = BTreeMap::from([
        ("tabindex".to_owned(), "-1".to_owned()),
        ("data-lang".to_owned(), "go".to_owned()),
    ]);
    options.html.code_attrs = BTreeMap::from([("data-x".to_owned(), "1".to_owned())]);
    let html = h.code_to_html("x", &options).unwrap();
    assert!(html.starts_with(
        r#"<pre class="iro github-dark my-pre" style="background-color:#24292e;color:#e1e4e8" data-lang="go" tabindex="-1"><code class="my-code" data-x="1">"#
    ));
    options.html.tabindex = None;
    options.html.pre_attrs.clear();
    let html = h.code_to_html("x", &options).unwrap();
    assert!(html.contains(r#"color:#e1e4e8"><code"#));
}

#[test]
fn font_styles_are_emitted() {
    let h = highlighter();
    let html = h
        .code_to_html(
            "*italic* and **bold**",
            &CodeToHtmlOptions::new("markdown", "github-dark"),
        )
        .unwrap();
    assert!(html.contains("font-style:italic"));
    assert!(html.contains("font-weight:bold"));
}

#[test]
fn multi_theme_variables_and_classes() {
    let h = highlighter();
    let html = h
        .code_to_html("package main", &CodeToHtmlOptions::multi("go", dual()))
        .unwrap();
    assert!(html.starts_with(
        r#"<pre class="iro iro-themes dark light" style="--iro-light:#24292e;--iro-light-bg:#fff;background-color:#24292e;color:#e1e4e8" tabindex="0">"#
    ));
    assert!(html.contains(r#"<span style="--iro-light:#d73a49;color:#f97583">package</span>"#));
    assert!(!html.contains("--iro-dark"));
}

#[test]
fn multi_theme_default_color_key_and_off() {
    let h = highlighter();
    let mut options = CodeToHtmlOptions::multi("go", dual());
    options.html.default_color = DefaultColor::Key("light".to_owned());
    let html = h.code_to_html("package main", &options).unwrap();
    assert!(html.contains(
        r#"style="--iro-dark:#e1e4e8;--iro-dark-bg:#24292e;background-color:#fff;color:#24292e""#
    ));
    assert!(html.contains(r#"<span style="--iro-dark:#f97583;color:#d73a49">package</span>"#));

    options.html.default_color = DefaultColor::Off;
    let html = h.code_to_html("package main", &options).unwrap();
    assert!(html.contains(r#"style="--iro-light:#24292e;--iro-light-bg:#fff""#));
    assert!(!code_of(&html).contains("\"color:"));
    assert!(!code_of(&html).contains(";color:"));
    assert!(html.contains(r#"<span style="--iro-light:#d73a49">package</span>"#));

    options.html.default_color = DefaultColor::Key("nope".to_owned());
    assert!(matches!(
        h.code_to_html("x", &options),
        Err(iro::Error::ThemeNotFound(_))
    ));
}

#[test]
fn three_themes_and_font_style_variables() {
    let h = highlighter();
    let mut themes = dual();
    themes.insert("dim".to_owned(), "github-dark-dimmed".to_owned());
    let html = h
        .code_to_html("*italic*", &CodeToHtmlOptions::multi("markdown", themes))
        .unwrap();
    assert!(html.contains(r#"class="iro iro-themes dark dim light""#));
    assert!(html.contains("--iro-dim:"));
    assert!(html.contains("--iro-light-font-style:italic"));
    assert!(html.contains("font-style:italic"));
}

#[test]
fn one_key_themes_map_uses_the_multi_form() {
    let h = highlighter();
    let themes = BTreeMap::from([("dark".to_owned(), "github-dark".to_owned())]);
    let html = h
        .code_to_html("x", &CodeToHtmlOptions::multi("go", themes))
        .unwrap();
    assert!(html.starts_with(
        r#"<pre class="iro iro-themes dark" style="background-color:#24292e;color:#e1e4e8""#
    ));
}

#[test]
fn style_to_class_mode_shares_a_map_across_blocks() {
    let h = highlighter();
    let mut map = StyleClassMap::new();
    let options = CodeToHtmlOptions::new("go", "github-dark");
    let first = h
        .code_to_html_with(
            "package main",
            &options,
            &mut HtmlRenderer::with_class_map(&mut map),
        )
        .unwrap();
    assert!(!first.contains("style=\""));
    assert!(first.contains(r#"<span class="_s_"#));
    assert!(first.starts_with(r#"<pre class="iro github-dark _s_"#));
    let before = map.len();
    let second = h
        .code_to_html_with(
            "package other",
            &options,
            &mut HtmlRenderer::with_class_map(&mut map),
        )
        .unwrap();
    assert!(second.contains(r#"<span class="_s_"#));
    assert!(map.len() >= before);
    let css = map.css();
    assert!(css.contains("color: #f97583"));
    assert!(css.contains("background-color: #24292e; color: #e1e4e8"));

    let multi = h
        .code_to_html_with(
            "package main",
            &CodeToHtmlOptions::multi("go", dual()),
            &mut HtmlRenderer::with_class_map(&mut map),
        )
        .unwrap();
    assert!(!multi.contains("style=\""));
    assert!(map.css().contains("--iro-light: #d73a49"));

    let plain = h.code_to_html("package main", &options).unwrap();
    assert!(plain.contains("style=\""));
}

#[test]
fn shiki_preset_shape_escaping_and_empty_source() {
    let h = highlighter();
    let html = h
        .code_to_html(
            "let s = \"<a>\"; // & 'q'",
            &CodeToHtmlOptions::new("javascript", "github-dark").shiki(),
        )
        .unwrap();
    assert_eq!(
        html,
        r#"<pre class="shiki github-dark" style="background-color:#24292e;color:#e1e4e8" tabindex="0"><code><span class="line"><span style="color:#F97583">let</span><span style="color:#E1E4E8"> s </span><span style="color:#F97583">=</span><span style="color:#9ECBFF"> "&#x3C;a>"</span><span style="color:#E1E4E8">; </span><span style="color:#6A737D">// &#x26; 'q'</span></span></code></pre>"#
    );
    let empty = h
        .code_to_html(
            "",
            &CodeToHtmlOptions::new("javascript", "github-dark").shiki(),
        )
        .unwrap();
    assert!(empty.contains(r#"<code><span class="line"></span></code>"#));
}

#[test]
fn output_is_deterministic_across_highlighters() {
    let code = "fn main() {\n    let x = \"<a & b>\";\n}\n";
    let mut outputs = Vec::new();
    for _ in 0..2 {
        let h = highlighter();
        let mut map = StyleClassMap::new();
        let single = h
            .code_to_html(code, &CodeToHtmlOptions::new("rust", "github-dark"))
            .unwrap();
        let multi = h
            .code_to_html(code, &CodeToHtmlOptions::multi("rust", dual()))
            .unwrap();
        let classed = h
            .code_to_html_with(
                code,
                &CodeToHtmlOptions::new("rust", "github-dark"),
                &mut HtmlRenderer::with_class_map(&mut map),
            )
            .unwrap();
        outputs.push((single, multi, classed, map.css()));
    }
    assert_eq!(outputs[0], outputs[1]);
}
