use super::linehighlight::{build_meta, collect_highlighted_lines};
use super::{
    Region, any_attrs, attr_value, class_with_prefix, find_element, has_class, recover_text,
    strip_prefix, subtree, trim_code,
};
use crate::html::Token;

/// Hugo's wrapped Chroma output. Two shapes are accepted under the div with the highlight
/// class token: the classes mode shape whose pre carries the chroma class, and Hugo's
/// default inline styles mode, which carries no Chroma classes at all but still has the
/// language class and data-lang on the code element.
pub fn div_wrapper(tokens: &[Token]) -> Option<Region> {
    let root = tokens.first()?;
    if !root.is_start("div") || !has_class(&root.attrs, "highlight") {
        return None;
    }

    // Table line number mode: div.chroma > table.lntable with two td.lntd cells. The first
    // cell is the gutter; recovery must run inside the second cell only.
    if let Some(ti) = find_element(tokens, 1, "table", |a| has_class(a, "lntable")) {
        let (inner, _) = subtree(tokens, ti);
        let first = find_element(inner, 0, "td", |a| has_class(a, "lntd"))?;
        let (_, after) = subtree(inner, first);
        let second = find_element(inner, after, "td", |a| has_class(a, "lntd"))?;
        let (cell, _) = subtree(inner, second);
        let pi = find_element(cell, 0, "pre", any_attrs)?;
        return extract(cell, pi);
    }

    let pi = find_element(tokens, 1, "pre", any_attrs)?;
    if !has_class(&tokens[pi].attrs, "chroma") {
        let (inner, _) = subtree(tokens, pi);
        let ci = find_element(inner, 0, "code", any_attrs)?;
        if attr_value(&inner[ci].attrs, "data-lang").is_none()
            && class_with_prefix(&inner[ci].attrs, "language-").is_none()
        {
            return None;
        }
    }
    extract(tokens, pi)
}

/// Chroma classes mode output without Hugo's wrapper div. The chroma class is required so
/// that plain fenced output from codeFences=false falls through to the generic fallback.
pub fn bare_pre(tokens: &[Token]) -> Option<Region> {
    let root = tokens.first()?;
    if !root.is_start("pre") || !has_class(&root.attrs, "chroma") {
        return None;
    }
    extract(tokens, 0)
}

fn extract(tokens: &[Token], pre_idx: usize) -> Option<Region> {
    let pre = &tokens[pre_idx];
    let (inner, _) = subtree(tokens, pre_idx);
    let ci = find_element(inner, 0, "code", any_attrs)?;
    let lang = if let Some(v) = attr_value(&inner[ci].attrs, "data-lang") {
        v
    } else if let Some(c) = class_with_prefix(&inner[ci].attrs, "language-") {
        strip_prefix(c, "language-")
    } else {
        attr_value(&pre.attrs, "data-lang").unwrap_or("")
    };
    let (code_inner, _) = subtree(inner, ci);
    let code = trim_code(recover_text(code_inner));
    let highlighted = collect_highlighted_lines(code_inner);
    Some(Region {
        lang: lang.to_owned(),
        meta: build_meta(lang, &highlighted),
        code,
    })
}
