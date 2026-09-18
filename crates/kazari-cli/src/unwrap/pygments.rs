use super::linehighlight::build_meta;
use super::{
    Region, any_attrs, class_with_prefix, find_element, has_class, recover_text, strip_prefix,
    subtree, trim_code,
};
use crate::html::Token;

/// The Pygments family (Sphinx, Pelican): an outer div whose class carries the highlight-
/// prefix with the language as suffix, wrapping div.highlight > pre with no code element
/// at all. The absence of a code element is part of the signature; it is what separates
/// this family from Rouge. The language suffix is kept verbatim (python3 stays python3).
pub fn div_wrapper(tokens: &[Token]) -> Option<Region> {
    let root = tokens.first()?;
    if !root.is_start("div") {
        return None;
    }
    let lang = strip_prefix(class_with_prefix(&root.attrs, "highlight-")?, "highlight-");

    // Sphinx linenos mode wraps everything in table.highlighttable with a td.linenos gutter
    // cell and the real div.highlight inside td.code.
    let mut window = &tokens[1..];
    if let Some(ti) = find_element(tokens, 1, "table", |a| has_class(a, "highlighttable")) {
        let (inner, _) = subtree(tokens, ti);
        let di = find_element(inner, 0, "td", |a| has_class(a, "code"))?;
        window = subtree(inner, di).0;
    }
    let code = extract(window)?;
    Some(Region {
        lang: lang.to_owned(),
        meta: build_meta(lang, &[]),
        code,
    })
}

fn extract(tokens: &[Token]) -> Option<String> {
    let hi = find_element(tokens, 0, "div", |a| has_class(a, "highlight"))?;
    let (inner, _) = subtree(tokens, hi);
    let pi = find_element(inner, 0, "pre", any_attrs)?;
    let (pre_inner, _) = subtree(inner, pi);
    if find_element(pre_inner, 0, "code", any_attrs).is_some() {
        return None;
    }
    Some(trim_code(recover_text(pre_inner)))
}
