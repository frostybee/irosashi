use super::linehighlight::build_meta;
use super::{
    Region, any_attrs, class_with_prefix, find_element, recover_text, strip_prefix, subtree,
    trim_code,
};
use crate::html::Token;

/// The Eleventy Prism plugin shape: the language class duplicated on both the pre and the
/// code element. Requiring both keeps a hand written pre.language-x around a plain code
/// element in the generic fallback. Prism's line numbers gutter sits inside the pre but
/// outside the code element, so walking only the code subtree already excludes it.
pub fn bare_pre(tokens: &[Token]) -> Option<Region> {
    let root = tokens.first()?;
    if !root.is_start("pre") {
        return None;
    }
    class_with_prefix(&root.attrs, "language-")?;
    let ci = find_element(tokens, 1, "code", any_attrs)?;
    let lang = strip_prefix(
        class_with_prefix(&tokens[ci].attrs, "language-")?,
        "language-",
    );
    let (code_inner, _) = subtree(tokens, ci);
    Some(Region {
        lang: lang.to_owned(),
        meta: build_meta(lang, &[]),
        code: trim_code(recover_text(code_inner)),
    })
}
