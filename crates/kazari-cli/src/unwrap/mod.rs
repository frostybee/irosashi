//! Recovers source code, language and meta information from code blocks in HTML that a
//! static site generator has already built. Each supported generator family has an
//! unwrapper that recognizes its markup shape.

mod chroma;
mod classlist;
mod linehighlight;
mod plain;
mod prism;
mod pygments;
mod rouge;
pub mod scan;
mod shiki;
mod skip;
mod textwalk;
mod zola;

#[cfg(test)]
mod fixtures_tests;

pub use classlist::{attr_value, class_with_prefix, has_class};
pub use skip::{has_reserved_class, skip_reason};
pub use textwalk::{is_void, recover_text};

use crate::html::{Attr, Token, TokenKind};

/// The extracted content of one recognized code block.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Region {
    pub lang: String,
    /// Full meta string with the language token first.
    pub meta: String,
    /// Recovered source text, ready for `render_with_meta`.
    pub code: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unwrapper {
    ChromaDiv,
    ChromaPre,
    RougeTable,
    RougeDiv,
    RougeFigure,
    PygmentsDiv,
    ZolaDiv,
    ZolaPre,
    PrismPre,
    ShikiAstro,
    Plain,
}

/// Tried for regions rooted at a wrapper element such as div.highlight,
/// div.highlighter-rouge, div.highlight-x or figure.highlight. Order matters and first
/// match wins: the Rouge table shape runs before the other Rouge shapes because the table
/// nests inside them; Chroma runs first since its root signature never overlaps theirs.
pub const DIV_WRAPPER_CHAIN: &[Unwrapper] = &[
    Unwrapper::ChromaDiv,
    Unwrapper::RougeTable,
    Unwrapper::RougeDiv,
    Unwrapper::RougeFigure,
    Unwrapper::PygmentsDiv,
    Unwrapper::ZolaDiv,
];

/// Tried for regions rooted at a pre element with no recognized wrapper. The generic
/// fallback runs last so every more specific shape gets first refusal.
pub const BARE_PRE_CHAIN: &[Unwrapper] = &[
    Unwrapper::ChromaPre,
    Unwrapper::PrismPre,
    Unwrapper::ShikiAstro,
    Unwrapper::ZolaPre,
    Unwrapper::Plain,
];

impl Unwrapper {
    pub fn name(self) -> &'static str {
        match self {
            Unwrapper::ChromaDiv => "chroma-div",
            Unwrapper::ChromaPre => "chroma-pre",
            Unwrapper::RougeTable => "rouge-table",
            Unwrapper::RougeDiv => "rouge-div",
            Unwrapper::RougeFigure => "rouge-figure",
            Unwrapper::PygmentsDiv => "pygments-div",
            Unwrapper::ZolaDiv => "zola-div",
            Unwrapper::ZolaPre => "zola-pre",
            Unwrapper::PrismPre => "prism-pre",
            Unwrapper::ShikiAstro => "shiki-astro",
            Unwrapper::Plain => "plain",
        }
    }

    pub fn matches(self, tokens: &[Token]) -> Option<Region> {
        match self {
            Unwrapper::ChromaDiv => chroma::div_wrapper(tokens),
            Unwrapper::ChromaPre => chroma::bare_pre(tokens),
            Unwrapper::RougeTable => rouge::linenos_table(tokens),
            Unwrapper::RougeDiv => rouge::div_wrapper(tokens),
            Unwrapper::RougeFigure => rouge::highlight_figure(tokens),
            Unwrapper::PygmentsDiv => pygments::div_wrapper(tokens),
            Unwrapper::ZolaDiv => zola::div_wrapper(tokens),
            Unwrapper::ZolaPre => zola::bare_pre(tokens),
            Unwrapper::PrismPre => prism::bare_pre(tokens),
            Unwrapper::ShikiAstro => shiki::astro_bare_pre(tokens),
            Unwrapper::Plain => plain::fallback(tokens),
        }
    }
}

/// Tries each unwrapper in order and returns the first match with its name. When the
/// region carries a `data-kz-meta` attribute, its value replaces the synthesized meta
/// string verbatim; source recovery and language detection are unaffected.
pub fn run_chain(chain: &[Unwrapper], tokens: &[Token]) -> Option<(Region, &'static str)> {
    for u in chain {
        let Some(mut region) = u.matches(tokens) else {
            continue;
        };
        if let Some(meta) = kz_meta_attr(tokens) {
            region.meta = meta.to_owned();
        }
        return Some((region, u.name()));
    }
    None
}

/// A `data-kz-meta` attribute on the region root or on any pre or code element inside
/// the region: a render hook template stashes the full fence info string there so
/// features beyond the language survive the build.
pub fn kz_meta_attr(tokens: &[Token]) -> Option<&str> {
    tokens.iter().enumerate().find_map(|(i, tok)| {
        if tok.kind != TokenKind::Start {
            return None;
        }
        if i > 0 && tok.name != "pre" && tok.name != "code" {
            return None;
        }
        attr_value(&tok.attrs, "data-kz-meta")
    })
}

/// The index of the first start tag with the given name at or after `from` whose
/// attributes satisfy `pred`.
pub fn find_element(
    tokens: &[Token],
    from: usize,
    tag: &str,
    pred: impl Fn(&[Attr]) -> bool,
) -> Option<usize> {
    (from..tokens.len()).find(|&i| tokens[i].is_start(tag) && pred(&tokens[i].attrs))
}

pub fn any_attrs(_: &[Attr]) -> bool {
    true
}

/// The tokens strictly inside the element opened at index `i`, plus the index just past
/// its matching end tag. Depth is tracked on the element's own tag name only, so
/// unbalanced markup in unrelated tags cannot desync the boundary. An unclosed element
/// yields the rest of the window.
pub fn subtree(tokens: &[Token], i: usize) -> (&[Token], usize) {
    let tag = &tokens[i].name;
    let mut depth = 1;
    for j in i + 1..tokens.len() {
        match tokens[j].kind {
            TokenKind::Start if &tokens[j].name == tag => depth += 1,
            TokenKind::End if &tokens[j].name == tag => {
                depth -= 1;
                if depth == 0 {
                    return (&tokens[i + 1..j], j + 1);
                }
            }
            _ => {}
        }
    }
    (&tokens[i + 1..], tokens.len())
}

pub fn strip_prefix<'a>(class: &'a str, prefix: &str) -> &'a str {
    class.strip_prefix(prefix).unwrap_or(class)
}

pub fn trim_code(text: String) -> String {
    let trimmed = text.trim_end_matches('\n');
    if trimmed.len() == text.len() {
        text
    } else {
        trimmed.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html::tokenize;

    #[test]
    fn subtree_tracks_own_tag_only() {
        let t = tokenize(b"<pre><code><span>a</span><b></code></pre>");
        let (inner, next) = subtree(&t, 1);
        assert_eq!(inner.len(), 4);
        assert_eq!(t[next].kind, TokenKind::End);
        assert_eq!(t[next].name, "pre");
        let (inner, next) = subtree(&t, 5);
        assert_eq!(inner.len(), 2);
        assert_eq!(next, t.len());
    }

    #[test]
    fn kz_meta_only_on_root_pre_or_code() {
        let t = tokenize(
            b"<div><span data-kz-meta=\"x\"></span><code data-kz-meta=\"go {1}\"></code></div>",
        );
        assert_eq!(kz_meta_attr(&t), Some("go {1}"));
        let t = tokenize(b"<div data-kz-meta=\"root\"><code data-kz-meta=\"inner\"></code></div>");
        assert_eq!(kz_meta_attr(&t), Some("root"));
        assert_eq!(kz_meta_attr(&tokenize(b"<pre><code>x</code></pre>")), None);
    }

    #[test]
    fn chain_applies_meta_override() {
        let t = tokenize(b"<pre><code class=\"language-go\" data-kz-meta=\"go title=&quot;m.go&quot;\">x</code></pre>");
        let (r, name) = run_chain(BARE_PRE_CHAIN, &t).unwrap();
        assert_eq!(name, "plain");
        assert_eq!(r.lang, "go");
        assert_eq!(r.meta, "go title=\"m.go\"");
        assert_eq!(r.code, "x");
        assert!(run_chain(DIV_WRAPPER_CHAIN, &t).is_none());
    }
}
