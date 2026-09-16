//! pulldown-cmark adapter: fenced code blocks in a markdown event stream are rendered
//! through [`Kazari`] and replaced by one raw HTML event each. With
//! `Config::code_groups` on, `:::code-group [sync="key"]` ... `:::` containers around
//! fences render as a tab bar with one panel per fence.

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

use crate::engine::Kazari;
use crate::error::Error;
use crate::escape::{escape_attr, escape_text};
use crate::frame;
use crate::meta;

const GROUP_OPENER: &str = ":::code-group";
const GROUP_CLOSER: &str = ":::";

/// Parses `markdown` with `options`, highlights every fence, and returns the HTML.
pub fn render_markdown(engine: &Kazari, markdown: &str, options: Options) -> Result<String, Error> {
    let events = highlight_events(engine, Parser::new_ext(markdown, options))?;
    let mut html = String::with_capacity(markdown.len() * 2);
    pulldown_cmark::html::push_html(&mut html, events.into_iter());
    Ok(html)
}

/// Replaces every fenced code block (and, when enabled, every code group) in
/// `events` with an [`Event::Html`]; every other event is passed through. Indented
/// code blocks are left to the markdown renderer.
pub fn highlight_events<'a>(
    engine: &Kazari,
    events: impl IntoIterator<Item = Event<'a>>,
) -> Result<Vec<Event<'a>>, Error> {
    let code_groups = engine.config().code_groups;
    let mut input = events.into_iter();
    let mut out = Vec::new();

    while let Some(event) = input.next() {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) => {
                let code = collect_code(&mut input);
                out.push(Event::Html(engine.render_with_meta(&code, &info)?.into()));
            }
            Event::Start(Tag::Paragraph) if code_groups => match paragraph_text(&mut input) {
                Ok(text) if text.trim().starts_with(GROUP_OPENER) => {
                    let sync = extract_sync_key(text.trim()[GROUP_OPENER.len()..].trim());
                    let blocks = collect_group(&mut input);
                    if !blocks.is_empty() {
                        out.push(Event::Html(render_group(engine, &sync, &blocks)?.into()));
                    }
                }
                Ok(text) => {
                    out.push(Event::Start(Tag::Paragraph));
                    out.push(Event::Text(text));
                    out.push(Event::End(TagEnd::Paragraph));
                }
                Err(consumed) => {
                    out.push(Event::Start(Tag::Paragraph));
                    out.extend(consumed);
                }
            },
            other => out.push(other),
        }
    }

    Ok(out)
}

fn collect_code<'a>(input: &mut impl Iterator<Item = Event<'a>>) -> String {
    let mut code = String::new();
    for event in input.by_ref() {
        match event {
            Event::Text(text) => code.push_str(&text),
            Event::End(TagEnd::CodeBlock) => break,
            _ => {}
        }
    }
    code.truncate(code.trim_end_matches('\n').len());
    code
}

/// After a paragraph start: the text of a paragraph made of exactly one text event.
/// Otherwise the events consumed while looking, so the caller can replay them.
fn paragraph_text<'a>(
    input: &mut impl Iterator<Item = Event<'a>>,
) -> Result<pulldown_cmark::CowStr<'a>, Vec<Event<'a>>> {
    match input.next() {
        Some(Event::Text(text)) => match input.next() {
            Some(Event::End(TagEnd::Paragraph)) => Ok(text),
            Some(other) => Err(vec![Event::Text(text), other]),
            None => Err(vec![Event::Text(text)]),
        },
        Some(other) => Err(vec![other]),
        None => Err(Vec::new()),
    }
}

fn extract_sync_key(rest: &str) -> String {
    for prefix in ["sync=\"", "sync='"] {
        if let Some(after) = rest.strip_prefix(prefix) {
            let quote = prefix.chars().last().unwrap_or('"');
            if let Some(end) = after.find(quote) {
                return after[..end].to_owned();
            }
        }
    }
    String::new()
}

/// Collects the fences up to the closing `:::` paragraph or the end of the stream;
/// everything else inside the group is dropped.
fn collect_group<'a>(input: &mut impl Iterator<Item = Event<'a>>) -> Vec<(String, String)> {
    let mut blocks = Vec::new();
    while let Some(event) = input.next() {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) => {
                let code = collect_code(input);
                blocks.push((info.into_string(), code));
            }
            Event::Start(Tag::Paragraph) => {
                if let Ok(text) = paragraph_text(input)
                    && text.trim() == GROUP_CLOSER
                {
                    break;
                }
            }
            _ => {}
        }
    }
    blocks
}

fn render_group(engine: &Kazari, sync: &str, blocks: &[(String, String)]) -> Result<String, Error> {
    let mut html = String::new();
    if sync.is_empty() {
        html.push_str("<div class=\"kazari-block kz-group not-content\">");
    } else {
        html.push_str(&format!(
            "<div class=\"kazari-block kz-group not-content\" data-sync=\"{}\">",
            escape_attr(sync)
        ));
    }

    html.push_str("<div class=\"kz-group-tabs\" role=\"tablist\" aria-label=\"Code variants\">");
    for (i, (info, code)) in blocks.iter().enumerate() {
        let (selected, tabindex) = if i == 0 {
            ("true", "0")
        } else {
            ("false", "-1")
        };
        html.push_str(&format!(
            "<button role=\"tab\" aria-selected=\"{selected}\" tabindex=\"{tabindex}\">{}</button>",
            escape_text(&tab_label(engine, info, code))
        ));
    }
    html.push_str("</div>");

    html.push_str("<div class=\"kz-group-panels\">");
    for (i, (info, code)) in blocks.iter().enumerate() {
        let rendered = engine.render_with_meta(code, info)?;
        if i == 0 {
            html.push_str(&format!("<div role=\"tabpanel\">{rendered}</div>"));
        } else {
            html.push_str(&format!("<div role=\"tabpanel\" hidden>{rendered}</div>"));
        }
    }
    html.push_str("</div>");

    html.push_str("</div>");
    Ok(html)
}

fn tab_label(engine: &Kazari, info: &str, code: &str) -> String {
    let parsed = meta::parse(info);
    if !parsed.block_options.title.is_empty() {
        return parsed.block_options.title;
    }
    let lang = parsed.block_options.lang;
    if let Some((title, _)) = frame::extract_file_name(code, &lang) {
        return title;
    }
    let mut chars = lang.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => engine.ui_strings().code_group_fallback.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> Kazari {
        Kazari::builder(irosashi::Highlighter::new().unwrap())
            .themes("github-light", Some("github-dark"))
            .build()
            .unwrap()
    }

    fn group_engine() -> Kazari {
        Kazari::builder(irosashi::Highlighter::new().unwrap())
            .themes("github-light", Some("github-dark"))
            .code_groups(true)
            .build()
            .unwrap()
    }

    fn render(kz: &Kazari, md: &str) -> String {
        render_markdown(kz, md, Options::empty()).unwrap()
    }

    #[test]
    fn fence_becomes_kazari_block() {
        let html = render(&engine(), "# Title\n\n```go\npackage main\n```\n\nAfter.\n");
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("class=\"kazari-block not-content"));
        assert!(html.contains("data-lang=\"go\""));
        assert!(html.contains("package"));
        assert!(html.contains("<p>After.</p>"));
        assert!(!html.contains("<pre><code"));
    }

    #[test]
    fn prose_only_is_untouched() {
        let html = render(&engine(), "# Hello\n\nJust *text*.\n");
        assert_eq!(html, "<h1>Hello</h1>\n<p>Just <em>text</em>.</p>\n");
    }

    #[test]
    fn meta_reaches_the_engine() {
        let html = render(
            &engine(),
            "```go title=\"main.go\" showLineNumbers\npackage main\n```\n",
        );
        assert!(html.contains("main.go"));
        assert!(html.contains("has-title"));
        assert!(html.contains("kz-ln"));
    }

    #[test]
    fn bare_and_empty_fences_are_wrapped() {
        let bare = render(&engine(), "```\nplain text\n```\n");
        assert!(bare.contains("kazari-block"));
        assert!(bare.contains("plain text"));
        let empty = render(&engine(), "```\n```\n");
        assert!(empty.contains("kazari-block"));
    }

    #[test]
    fn several_fences_and_tilde_fences() {
        let html = render(&engine(), "```js\na\n```\n\ntext\n\n~~~py\nb\n~~~\n");
        assert_eq!(html.matches("class=\"kazari-block").count(), 2);
        assert!(html.contains("data-lang=\"js\""));
        assert!(html.contains("data-lang=\"py\""));
    }

    #[test]
    fn trailing_newlines_are_trimmed() {
        let html = render(&engine(), "```text\na\n\n\n```\n");
        assert!(html.contains("data-lines=\"1\""), "{html}");
    }

    #[test]
    fn indented_block_passes_through() {
        let html = render(&engine(), "para\n\n    indented\n");
        assert!(html.contains("<pre><code>indented\n</code></pre>"));
        assert!(!html.contains("kazari-block"));
    }

    #[test]
    fn unknown_language_renders_without_error() {
        let html = render(&engine(), "```plantuml\n@startuml\n```\n");
        assert!(html.contains("kazari-block"));
        assert!(html.contains("@startuml"));
    }

    #[test]
    fn engine_error_aborts_conversion() {
        let err = render_markdown(
            &engine(),
            "```js theme=\"no-such-theme\"\nx\n```\n",
            Options::empty(),
        );
        assert!(err.is_err());
    }

    #[test]
    fn mermaid_passes_through_by_default() {
        let html = render(&engine(), "```mermaid\ngraph TD; A-->B\n```\n");
        assert_eq!(html, "<pre class=\"mermaid\">graph TD; A--&gt;B</pre>\n");

        let kz = Kazari::builder(irosashi::Highlighter::new().unwrap())
            .themes("github-light", None)
            .mermaid_pass_through(false)
            .build()
            .unwrap();
        let html = render(&kz, "```mermaid\ngraph TD; A-->B\n```\n");
        assert!(html.contains("kazari-block"));
        assert!(!html.contains("class=\"mermaid\""));
    }

    const GROUP: &str =
        ":::code-group\n\n```go\nfmt.Println()\n```\n\n```python\nprint()\n```\n\n:::\n";

    #[test]
    fn code_group_renders_tabs_and_panels() {
        let html = render(&group_engine(), GROUP);
        assert!(html.contains("class=\"kazari-block kz-group not-content\">"));
        assert!(html.contains("role=\"tablist\""));
        assert_eq!(html.matches("role=\"tab\"").count(), 2);
        assert_eq!(html.matches("aria-selected=\"true\"").count(), 1);
        assert_eq!(html.matches("tabindex=\"0\"").count(), 1);
        assert_eq!(html.matches("role=\"tabpanel\"").count(), 2);
        assert_eq!(html.matches("role=\"tabpanel\" hidden>").count(), 1);
        assert!(html.contains(">Go</button>"));
        assert!(html.contains(">Python</button>"));
        assert!(!html.contains(":::"));
        assert!(!html.contains("data-sync"));
    }

    #[test]
    fn code_group_labels() {
        let md = ":::code-group\n```go title=\"Server\"\nx\n```\n```js\n// app.js\nx\n```\n```\nx\n```\n:::\n";
        let html = render(&group_engine(), md);
        assert!(html.contains(">Server</button>"));
        assert!(html.contains(">app.js</button>"));
        assert!(html.contains(">Code</button>"));
    }

    #[test]
    fn code_group_sync_keys() {
        let kz = group_engine();
        let double = render(
            &kz,
            &GROUP.replacen(":::code-group", ":::code-group sync=\"lang\"", 1),
        );
        assert!(double.contains("data-sync=\"lang\""));
        let single = render(
            &kz,
            &GROUP.replacen(":::code-group", ":::code-group sync='lang'", 1),
        );
        assert!(single.contains("data-sync=\"lang\""));
        let escaped = render(
            &kz,
            &GROUP.replacen(":::code-group", ":::code-group sync=\"a&b\"", 1),
        );
        assert!(escaped.contains("data-sync=\"a&amp;b\""));
    }

    #[test]
    fn code_group_edge_cases() {
        let kz = group_engine();
        let single = render(&kz, ":::code-group\n```go\nx\n```\n:::\n");
        assert_eq!(single.matches("role=\"tab\"").count(), 1);

        let empty = render(&kz, "before\n\n:::code-group\n\nprose\n\n:::\n\nafter\n");
        assert!(!empty.contains("kz-group"));
        assert!(empty.contains("<p>before</p>"));
        assert!(empty.contains("<p>after</p>"));
        assert!(!empty.contains("prose"));

        let unterminated = render(&kz, ":::code-group\n```go\nx\n```\n");
        assert_eq!(unterminated.matches("role=\"tab\"").count(), 1);

        let outside = render(
            &kz,
            "```rust\nx\n```\n\n:::code-group\n```go\nx\n```\n:::\n",
        );
        assert!(outside.contains("class=\"kazari-block not-content"));
        assert!(outside.contains("kz-group"));
    }

    #[test]
    fn code_group_syntax_is_plain_text_when_disabled() {
        let html = render(&engine(), GROUP);
        assert!(!html.contains("kz-group"));
        assert!(html.contains("<p>:::code-group</p>"));
        assert!(html.contains("<p>:::</p>"));
        assert_eq!(html.matches("class=\"kazari-block").count(), 2);
    }

    #[test]
    fn paragraph_lookahead_replays_events() {
        let html = render(&group_engine(), "a *b* c\n\nd\ne\n");
        assert_eq!(html, "<p>a <em>b</em> c</p>\n<p>d\ne</p>\n");
    }

    #[test]
    fn assets_follow_the_flag() {
        assert!(!engine().css().contains(".kz-group"));
        assert!(!engine().js().contains(".kz-group"));
        assert!(group_engine().css().contains(".kz-group"));
        assert!(group_engine().js().contains(".kz-group"));
    }

    #[test]
    fn highlight_events_keeps_borrowed_events() {
        let kz = engine();
        let md = "text\n\n```js\nlet x;\n```\n";
        let events = highlight_events(&kz, Parser::new(md)).unwrap();
        assert!(matches!(events[0], Event::Start(Tag::Paragraph)));
        assert!(events.iter().any(|e| matches!(e, Event::Html(_))));
    }
}
