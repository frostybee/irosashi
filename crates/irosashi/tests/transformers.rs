use std::path::Path;
use std::sync::{Arc, Mutex};

use irosashi::transformers::{Meta, Notation, Whitespace};
use irosashi::{
    CodeToHtmlOptions, Highlighter, HighlighterBuilder, HtmlRenderer, LineRange, Node, SpanContext,
    Transformer,
};

fn highlighter() -> Highlighter {
    HighlighterBuilder::from_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("assets"))
        .build()
        .unwrap()
}

fn render(code: &str, lang: &str, transformers: Vec<Box<dyn Transformer>>) -> String {
    highlighter()
        .code_to_html_with(
            code,
            &CodeToHtmlOptions::new(lang, "github-dark"),
            &mut HtmlRenderer::new().with_transformers(transformers),
        )
        .unwrap()
}

fn lines(html: &str) -> Vec<&str> {
    html.split("<span class=\"line").skip(1).collect()
}

fn line_classes(line: &str) -> String {
    format!("line{}", line.split('"').next().unwrap_or(""))
}

fn line_text(line: &str) -> String {
    let body = line.split_once('>').map_or(line, |(_, b)| b);
    let mut out = String::new();
    let mut in_tag = false;
    for c in body.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.trim_end_matches('\n').to_owned()
}

// Line decorations on HtmlOptions

#[test]
fn line_decorations_add_classes_and_dim_the_rest() {
    let h = highlighter();
    let mut options = CodeToHtmlOptions::new("text", "github-dark");
    options.html.highlight_lines = vec![LineRange::single(1)];
    options.html.focus_lines = vec![LineRange::new(2, 3)];
    options.html.inserted_lines = vec![LineRange::single(3)];
    options.html.deleted_lines = vec![LineRange::single(4), LineRange::new(9, 12)];
    let html = h.code_to_html("a\nb\nc\nd\ne", &options).unwrap();
    let l = lines(&html);
    assert_eq!(l.len(), 5);
    assert_eq!(line_classes(l[0]), "line highlighted dimmed");
    assert_eq!(line_classes(l[1]), "line focused");
    assert_eq!(line_classes(l[2]), "line focused diff add");
    assert_eq!(line_classes(l[3]), "line dimmed diff remove");
    assert_eq!(line_classes(l[4]), "line dimmed");
    assert!(html.starts_with("<pre class=\"iro github-dark has-focused\""));

    let plain = h
        .code_to_html("a\nb", &CodeToHtmlOptions::new("text", "github-dark"))
        .unwrap();
    assert!(!plain.contains("has-focused"));
    assert!(!plain.contains("dimmed"));
}

// Custom transformers and hook order

#[derive(Default)]
struct Recorder {
    calls: Arc<Mutex<Vec<String>>>,
}

impl Transformer for Recorder {
    fn name(&self) -> &str {
        "recorder"
    }
    fn preprocess(&mut self, code: &str) -> Option<String> {
        self.calls.lock().unwrap().push("preprocess".into());
        Some(code.replace("ONE", "1"))
    }
    fn tokens(&mut self, result: &mut irosashi::TokensResult) {
        self.calls
            .lock()
            .unwrap()
            .push(format!("tokens:{}", result.lines.len()));
    }
    fn span(&mut self, el: &mut Node, line_el: &mut Node, ctx: &SpanContext<'_>) {
        self.calls
            .lock()
            .unwrap()
            .push(format!("span:{}:{}:{}", ctx.line, ctx.col, ctx.text));
        el.set_attr("data-col", &ctx.col.to_string());
        line_el.push_class("touched");
    }
    fn line(&mut self, el: &mut Node, line: usize) {
        self.calls.lock().unwrap().push(format!("line:{line}"));
        el.push_class(&format!("l{line}"));
    }
    fn code(&mut self, el: &mut Node) {
        self.calls.lock().unwrap().push("code".into());
        el.set_attr("data-code", "yes");
    }
    fn pre(&mut self, el: &mut Node) {
        self.calls.lock().unwrap().push("pre".into());
        el.push_class("from-pre");
    }
    fn root(&mut self, el: &mut Node) {
        self.calls.lock().unwrap().push("root".into());
        assert_eq!(el.tag(), Some("pre"));
    }
    fn postprocess(&mut self, html: &str) -> Option<String> {
        self.calls.lock().unwrap().push("postprocess".into());
        Some(format!("<!-- x -->{html}"))
    }
}

#[test]
fn hooks_run_in_nuri_order_and_can_edit_every_level() {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let recorder = Recorder {
        calls: Arc::clone(&calls),
    };
    let html = render("a ONE\nb", "text", vec![Box::new(recorder)]);
    assert_eq!(
        *calls.lock().unwrap(),
        [
            "preprocess",
            "tokens:2",
            "span:1:0:a 1",
            "line:1",
            "span:2:0:b",
            "line:2",
            "code",
            "pre",
            "root",
            "postprocess",
        ]
    );
    assert!(html.starts_with("<!-- x --><pre class=\"iro github-dark from-pre\""));
    assert!(html.contains("<code data-code=\"yes\">"));
    assert!(html.contains("<span class=\"line touched l1\"><span style=\"color:#"));
    assert!(html.contains("\" data-col=\"0\">a 1</span>"));
    assert!(html.contains("<span class=\"line touched l2\">"));
}

struct Upper;

impl Transformer for Upper {
    fn name(&self) -> &str {
        "upper"
    }
    fn preprocess(&mut self, code: &str) -> Option<String> {
        Some(code.to_uppercase())
    }
    fn postprocess(&mut self, html: &str) -> Option<String> {
        Some(html.replace("</pre>", "</pre><!-- upper -->"))
    }
}

struct Suffix;

impl Transformer for Suffix {
    fn name(&self) -> &str {
        "suffix"
    }
    fn preprocess(&mut self, code: &str) -> Option<String> {
        Some(format!("{code}!"))
    }
    fn postprocess(&mut self, html: &str) -> Option<String> {
        Some(format!("{html}<!-- suffix -->"))
    }
}

#[test]
fn preprocess_and_postprocess_chain_in_list_order() {
    let html = render("ab", "text", vec![Box::new(Upper), Box::new(Suffix)]);
    assert!(html.contains(">AB!</span>"));
    assert!(html.ends_with("</pre><!-- upper --><!-- suffix -->"));
}

#[test]
fn an_empty_transformer_list_changes_nothing() {
    let h = highlighter();
    let options = CodeToHtmlOptions::new("rust", "github-light").shiki();
    let code = "fn main() {\n    println!(\"hi\");\n}\n";
    let plain = h.code_to_html(code, &options).unwrap();
    let with = h
        .code_to_html_with(
            code,
            &options,
            &mut HtmlRenderer::new().with_transformers(vec![]),
        )
        .unwrap();
    assert_eq!(plain, with);
    assert!(!plain.contains("highlighted"));
}

// Notation

#[test]
fn notation_strips_comments_and_classes_lines() {
    let code = "a = 1 // [!code ++]\nb = 2 // [!code --]\nc = 3 // [!code highlight]\nd = 4";
    let html = render(code, "javascript", vec![Box::new(Notation::new())]);
    let l = lines(&html);
    assert_eq!(l.len(), 4);
    assert_eq!(line_classes(l[0]), "line diff add");
    assert_eq!(line_classes(l[1]), "line diff remove");
    assert_eq!(line_classes(l[2]), "line highlighted");
    assert_eq!(line_classes(l[3]), "line");
    assert!(!html.contains("[!code"));
    assert!(!html.contains("//"));
    assert_eq!(line_text(l[0]), "a = 1");
    assert_eq!(line_text(l[2]), "c = 3");
}

#[test]
fn notation_focus_dims_the_others_and_marks_pre() {
    let code = "a // [!code focus]\nb\nc // [!code focus]";
    let html = render(code, "javascript", vec![Box::new(Notation::new())]);
    let l = lines(&html);
    assert_eq!(line_classes(l[0]), "line focused");
    assert_eq!(line_classes(l[1]), "line dimmed");
    assert_eq!(line_classes(l[2]), "line focused");
    assert!(html.starts_with("<pre class=\"iro github-dark has-focused\""));
}

#[test]
fn notation_error_and_warning() {
    let code = "a # [!code error]\nb # [!code warning]";
    let html = render(code, "python", vec![Box::new(Notation::new())]);
    let l = lines(&html);
    assert_eq!(line_classes(l[0]), "line highlighted error");
    assert_eq!(line_classes(l[1]), "line highlighted warning");
    assert_eq!(line_text(l[0]), "a");
}

#[test]
fn notation_word_highlights_every_occurrence_on_every_line() {
    let code = "foo(foo) // [!code word:foo]\nbar foo";
    let html = render(code, "javascript", vec![Box::new(Notation::new())]);
    assert_eq!(
        html.matches("<span class=\"highlighted-word\">foo</span>")
            .count(),
        3
    );
    assert!(!html.contains("[!code"));
    assert_eq!(line_text(lines(&html)[0]), "foo(foo)");
}

#[test]
fn notation_drops_a_block_comment_wrapper() {
    let code = "x /* [!code ++] */\n/* [!code highlight] */\ny";
    let html = render(code, "javascript", vec![Box::new(Notation::new())]);
    let l = lines(&html);
    assert_eq!(l.len(), 2);
    assert_eq!(line_classes(l[0]), "line diff add");
    assert_eq!(line_text(l[0]), "x");
    assert_eq!(line_text(l[1]), "y");
}

#[test]
fn notation_removes_a_comment_only_line_and_renumbers() {
    let code = "x\n// [!code ++]\ny // [!code highlight]";
    let html = render(code, "javascript", vec![Box::new(Notation::new())]);
    let l = lines(&html);
    assert_eq!(l.len(), 2);
    assert_eq!(line_classes(l[0]), "line");
    assert_eq!(line_classes(l[1]), "line highlighted");
    assert_eq!(line_text(l[1]), "y");
    assert!(!html.contains("diff"));
}

#[test]
fn notation_inside_a_token_keeps_the_text_around_it() {
    let code = "x // [!code ++] tail";
    let html = render(code, "javascript", vec![Box::new(Notation::new())]);
    let l = lines(&html);
    assert_eq!(line_classes(l[0]), "line diff add");
    assert_eq!(line_text(l[0]), "x // tail");
}

#[test]
fn notation_in_plain_text_shrinks_the_token() {
    let html = render(
        "a b [!code highlight]",
        "text",
        vec![Box::new(Notation::new())],
    );
    let l = lines(&html);
    assert_eq!(line_classes(l[0]), "line highlighted");
    assert_eq!(line_text(l[0]), "a b");
}

#[test]
fn notation_state_resets_between_renders() {
    let h = highlighter();
    let mut renderer = HtmlRenderer::new().with_transformers(vec![Box::new(Notation::new())]);
    let options = CodeToHtmlOptions::new("text", "github-dark");
    let first = h
        .code_to_html_with("a [!code focus]\nb", &options, &mut renderer)
        .unwrap();
    assert!(first.contains("has-focused"));
    let second = h
        .code_to_html_with("a\nb", &options, &mut renderer)
        .unwrap();
    assert!(!second.contains("has-focused"));
    assert!(!second.contains("dimmed"));
}

// Meta

#[test]
fn meta_highlights_ranges_from_the_fence_string() {
    let html = render(
        "a\nb\nc\nd",
        "text",
        vec![Box::new(Meta::new("js {1,3-4}"))],
    );
    let l = lines(&html);
    assert_eq!(line_classes(l[0]), "line highlighted");
    assert_eq!(line_classes(l[1]), "line");
    assert_eq!(line_classes(l[2]), "line highlighted");
    assert_eq!(line_classes(l[3]), "line highlighted");
}

// Whitespace

#[test]
fn whitespace_wraps_tabs_and_spaces() {
    let html = render("a b\tc", "text", vec![Box::new(Whitespace::new())]);
    assert!(html.contains(
        "a<span class=\"ws-space\">\u{b7}</span>b<span class=\"ws-tab\">\u{2192}</span>c</span>"
    ));
    let custom = render("a b", "text", vec![Box::new(Whitespace::with("", "_"))]);
    assert!(custom.contains("a<span class=\"ws-space\">_</span>b"));
    let untouched = render("abc", "text", vec![Box::new(Whitespace::new())]);
    assert!(!untouched.contains("ws-"));
}

#[test]
fn whitespace_and_notation_compose() {
    let html = render(
        "a b // [!code ++]",
        "javascript",
        vec![Box::new(Notation::new()), Box::new(Whitespace::new())],
    );
    let l = lines(&html);
    assert_eq!(line_classes(l[0]), "line diff add");
    assert_eq!(line_text(l[0]), "a\u{b7}b");
}
