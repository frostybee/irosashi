use std::fmt::Write;

use crate::render::{Node, RenderOptions, Renderer};
use crate::token::TokensResult;

/// The batteries-included HTML shape: `pre > code > span.line > span[style]`.
#[derive(Debug, Default)]
pub struct HtmlRenderer;

impl HtmlRenderer {
    pub fn tree(&self, result: &TokensResult, options: &RenderOptions) -> Node {
        let mut code = Node::element("code");
        if let Some(class) = &options.code_class {
            code = code.attr("class", class);
        }
        for (index, line) in result.lines.iter().enumerate() {
            if index > 0 {
                code = code.child(Node::text("\n"));
            }
            let text = result.line_text(line);
            let mut span = Node::element("span").attr("class", "line");
            for token in &line.tokens {
                let mut style = String::new();
                if let Some(color) = token.style.color {
                    let _ = write!(style, "color:{}", result.color(color));
                }
                if let Some(bg) = token.style.bg {
                    if !style.is_empty() {
                        style.push(';');
                    }
                    let _ = write!(style, "background-color:{}", result.color(bg));
                }
                let mut node = Node::element("span");
                if !style.is_empty() {
                    node = node.attr("style", &style);
                }
                span = span.child(node.child(Node::text(token.text(text))));
            }
            code = code.child(span);
        }
        let pre_class = options
            .pre_class
            .clone()
            .unwrap_or_else(|| format!("iro {}", result.theme.name));
        Node::element("pre")
            .attr("class", &pre_class)
            .attr(
                "style",
                &format!("background-color:{};color:{}", result.bg(), result.fg()),
            )
            .attr("tabindex", "0")
            .child(code)
    }
}

impl Renderer for HtmlRenderer {
    type Output = String;

    fn render(&mut self, result: &TokensResult, options: &RenderOptions) -> String {
        let mut out = String::new();
        write_node(&mut out, &self.tree(result, options));
        out
    }
}

pub fn write_node(out: &mut String, node: &Node) {
    match node {
        Node::Text(text) => escape_text(out, text),
        Node::Element {
            tag,
            attrs,
            children,
        } => {
            out.push('<');
            out.push_str(tag);
            for (key, value) in attrs {
                out.push(' ');
                out.push_str(key);
                out.push_str("=\"");
                escape_attr(out, value);
                out.push('"');
            }
            out.push('>');
            for child in children {
                write_node(out, child);
            }
            out.push_str("</");
            out.push_str(tag);
            out.push('>');
        }
    }
}

pub fn escape_text(out: &mut String, text: &str) {
    for c in text.chars() {
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            c => out.push(c),
        }
    }
}

pub fn escape_attr(out: &mut String, text: &str) {
    for c in text.chars() {
        match c {
            '"' => out.push_str("&quot;"),
            '&' => out.push_str("&amp;"),
            c => out.push(c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_text_and_attributes_and_sorts_attrs() {
        let node = Node::element("span")
            .attr("style", "color:#fff")
            .attr("class", "a\"&")
            .child(Node::text("<b>&"));
        let mut out = String::new();
        write_node(&mut out, &node);
        assert_eq!(
            out,
            r#"<span class="a&quot;&amp;" style="color:#fff">&lt;b&gt;&amp;</span>"#
        );
    }
}
