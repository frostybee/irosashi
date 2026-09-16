use crate::render::Node;
use crate::render::transformer::{SpanContext, Transformer};

const DEFAULT_TAB: &str = "\u{2192}";
const DEFAULT_SPACE: &str = "\u{b7}";

/// Renders tabs and spaces as visible symbols: each becomes a `span.ws-tab` or
/// `span.ws-space` inside the token span.
#[derive(Debug, Clone)]
pub struct Whitespace {
    tab: String,
    space: String,
}

impl Whitespace {
    /// `→` for tabs and `·` for spaces.
    pub fn new() -> Self {
        Self::with(DEFAULT_TAB, DEFAULT_SPACE)
    }

    /// Custom symbols; an empty string falls back to the default.
    pub fn with(tab: &str, space: &str) -> Self {
        let or_default = |s: &str, default: &str| {
            if s.is_empty() {
                default.to_owned()
            } else {
                s.to_owned()
            }
        };
        Self {
            tab: or_default(tab, DEFAULT_TAB),
            space: or_default(space, DEFAULT_SPACE),
        }
    }
}

impl Default for Whitespace {
    fn default() -> Self {
        Self::new()
    }
}

impl Transformer for Whitespace {
    fn name(&self) -> &str {
        "whitespace"
    }

    fn span(&mut self, el: &mut Node, _line_el: &mut Node, ctx: &SpanContext<'_>) {
        if !ctx.text.contains(['\t', ' ']) {
            return;
        }
        let mut children = Vec::new();
        let mut run = String::new();
        for c in ctx.text.chars() {
            let symbol = match c {
                '\t' => Some(("ws-tab", &self.tab)),
                ' ' => Some(("ws-space", &self.space)),
                _ => None,
            };
            match symbol {
                Some((class, symbol)) => {
                    if !run.is_empty() {
                        children.push(Node::text(&run));
                        run.clear();
                    }
                    children.push(
                        Node::element("span")
                            .attr("class", class)
                            .child(Node::text(symbol)),
                    );
                }
                None => run.push(c),
            }
        }
        if !run.is_empty() {
            children.push(Node::text(&run));
        }
        if let Some(existing) = el.children_mut() {
            *existing = children;
        }
    }
}
