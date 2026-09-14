pub mod html;
mod merge;
mod style;
mod style_class;

use crate::token::TokensResult;

pub use html::{DefaultColor, Dialect, Escape, HtmlOptions, HtmlRenderer};
pub use style_class::StyleClassMap;

/// A minimal output tree shared by renderers. Attributes keep insertion order; the
/// renderer that builds the tree decides the order it wants serialized.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Node {
    Element {
        tag: String,
        attrs: Vec<(String, String)>,
        children: Vec<Node>,
    },
    Text(String),
}

impl Node {
    pub fn element(tag: &str) -> Self {
        Self::Element {
            tag: tag.to_owned(),
            attrs: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn text(text: &str) -> Self {
        Self::Text(text.to_owned())
    }

    /// Sets an attribute, replacing the value in place when the key already exists.
    pub fn attr(mut self, key: &str, value: &str) -> Self {
        if let Self::Element { attrs, .. } = &mut self {
            match attrs.iter_mut().find(|(k, _)| k == key) {
                Some((_, v)) => *v = value.to_owned(),
                None => attrs.push((key.to_owned(), value.to_owned())),
            }
        }
        self
    }

    /// Appends a class token, creating the `class` attribute at the front when absent.
    /// A token already present is not repeated.
    pub fn add_class(mut self, class: &str) -> Self {
        if let Self::Element { attrs, .. } = &mut self {
            match attrs.iter_mut().find(|(k, _)| k == "class") {
                Some((_, v)) => {
                    if !v.split(' ').any(|c| c == class) {
                        if !v.is_empty() {
                            v.push(' ');
                        }
                        v.push_str(class);
                    }
                }
                None => attrs.insert(0, ("class".to_owned(), class.to_owned())),
            }
        }
        self
    }

    pub fn child(mut self, node: Node) -> Self {
        if let Self::Element { children, .. } = &mut self {
            children.push(node);
        }
        self
    }

    pub fn attrs(&self) -> &[(String, String)] {
        match self {
            Self::Element { attrs, .. } => attrs,
            Self::Text(_) => &[],
        }
    }

    pub fn children_mut(&mut self) -> Option<&mut Vec<Node>> {
        match self {
            Self::Element { children, .. } => Some(children),
            Self::Text(_) => None,
        }
    }
}

/// Maps a `TokensResult` to an output format. Implemented by the minimal HTML
/// renderer here and by presentation-layer renderers elsewhere.
pub trait Renderer {
    type Options;
    type Output;

    fn render(&mut self, result: &TokensResult, options: &Self::Options) -> Self::Output;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attr_replaces_in_place_and_add_class_dedupes() {
        let node = Node::element("a")
            .attr("style", "x")
            .attr("class", "one")
            .attr("style", "y")
            .add_class("two")
            .add_class("one");
        assert_eq!(
            node.attrs(),
            [
                ("style".to_owned(), "y".to_owned()),
                ("class".to_owned(), "one two".to_owned())
            ]
        );
        let fresh = Node::element("a").attr("id", "i").add_class("c");
        assert_eq!(fresh.attrs()[0], ("class".to_owned(), "c".to_owned()));
        assert!(Node::text("t").attr("a", "b").attrs().is_empty());
    }
}
