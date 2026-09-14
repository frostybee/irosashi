mod html;

use std::collections::BTreeMap;

use crate::token::TokensResult;

pub use html::HtmlRenderer;

/// A minimal output tree shared by renderers. Attributes are sorted by key so output
/// is deterministic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Element {
        tag: String,
        attrs: BTreeMap<String, String>,
        children: Vec<Node>,
    },
    Text(String),
}

impl Node {
    pub fn element(tag: &str) -> Self {
        Self::Element {
            tag: tag.to_owned(),
            attrs: BTreeMap::new(),
            children: Vec::new(),
        }
    }

    pub fn text(text: &str) -> Self {
        Self::Text(text.to_owned())
    }

    pub fn attr(mut self, key: &str, value: &str) -> Self {
        if let Self::Element { attrs, .. } = &mut self {
            attrs.insert(key.to_owned(), value.to_owned());
        }
        self
    }

    pub fn child(mut self, node: Node) -> Self {
        if let Self::Element { children, .. } = &mut self {
            children.push(node);
        }
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RenderOptions {
    pub pre_class: Option<String>,
    pub code_class: Option<String>,
}

/// Maps a `TokensResult` to an output format. Implemented by the minimal HTML
/// renderer here and by presentation-layer renderers elsewhere.
pub trait Renderer {
    type Output;

    fn render(&mut self, result: &TokensResult, options: &RenderOptions) -> Self::Output;
}
