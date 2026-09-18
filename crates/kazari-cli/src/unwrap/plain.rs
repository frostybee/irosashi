use super::linehighlight::build_meta;
use super::{
    Region, any_attrs, attr_value, class_with_prefix, find_element, has_class, recover_text,
    strip_prefix, subtree, trim_code,
};
use crate::html::Token;

/// The generic pre > code shape emitted by mdBook, markdown-it, Hugo with codeFences
/// disabled, and hand written pages. Runs last in the bare pre chain and matches even
/// without a language class; the unlabeled block policy belongs to the processor. A pre
/// without any code element falls through unclaimed. Expressive Code output is declined:
/// its div.ec-line structure carries no newline text nodes, so generic recovery would glue
/// every line together.
pub fn fallback(tokens: &[Token]) -> Option<Region> {
    let root = tokens.first()?;
    if !root.is_start("pre") {
        return None;
    }
    let ci = find_element(tokens, 1, "code", any_attrs)?;
    let (code_inner, _) = subtree(tokens, ci);
    if find_element(code_inner, 0, "div", |a| has_class(a, "ec-line")).is_some() {
        return None;
    }
    let lang = if let Some(c) = class_with_prefix(&tokens[ci].attrs, "language-") {
        strip_prefix(c, "language-")
    } else if let Some(v) = attr_value(&tokens[ci].attrs, "data-lang") {
        v
    } else if let Some(c) = class_with_prefix(&root.attrs, "language-") {
        strip_prefix(c, "language-")
    } else {
        ""
    };
    Some(Region {
        lang: lang.to_owned(),
        meta: build_meta(lang, &[]),
        code: trim_code(recover_text(code_inner)),
    })
}
