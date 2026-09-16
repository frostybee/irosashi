use crate::render::Node;
use crate::token::{ThemedToken, TokensResult};

/// What a [`Transformer::span`] hook sees for one token.
#[derive(Debug, Clone, Copy)]
pub struct SpanContext<'a> {
    /// One-based line number, counted after [`Transformer::tokens`] ran.
    pub line: usize,
    /// Byte column of the token in its line.
    pub col: usize,
    pub text: &'a str,
    pub token: &'a ThemedToken,
}

/// Hooks into the HTML pipeline, listed in call order. Every method has a no-op
/// default, so an implementation overrides only what it needs.
///
/// `Highlighter::code_to_html_with` runs the whole sequence: `preprocess` on the
/// source, `tokens` on the styled result, then `span`, `line`, `code`, `pre` and `root`
/// while the tree is built, and `postprocess` on the serialized HTML. Calling
/// `HtmlRenderer::render` directly runs only the tree hooks.
///
/// Hooks take `&mut self` so a transformer can carry state from `tokens` to `line`
/// within one render. A transformer kept on a renderer that is reused across calls
/// must reset that state itself, normally at the start of `tokens`.
pub trait Transformer: Send {
    fn name(&self) -> &str;

    /// Returns the code to tokenize instead of `code`, or `None` to leave it alone.
    fn preprocess(&mut self, _code: &str) -> Option<String> {
        None
    }

    /// Edits the styled tokens before the tree is built. Lines may be removed;
    /// token ranges stay byte offsets into the line.
    fn tokens(&mut self, _result: &mut TokensResult) {}

    /// Called for every token span before it is attached to its line.
    fn span(&mut self, _el: &mut Node, _line_el: &mut Node, _ctx: &SpanContext<'_>) {}

    /// Called for every `span.line` once its tokens are attached.
    fn line(&mut self, _el: &mut Node, _line: usize) {}

    fn code(&mut self, _el: &mut Node) {}

    fn pre(&mut self, _el: &mut Node) {}

    /// Called on the outermost element, which is the `pre`.
    fn root(&mut self, _el: &mut Node) {}

    /// Returns the HTML to emit instead of `html`, or `None` to leave it alone.
    fn postprocess(&mut self, _html: &str) -> Option<String> {
        None
    }
}
