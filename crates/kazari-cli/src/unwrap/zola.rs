use super::linehighlight::build_meta;
use super::{Region, any_attrs, attr_value, find_element, recover_text, subtree, trim_code};
use crate::html::Token;

// Zola's class names derive from the configured highlight theme, so no literal class can
// be matched. The stable signature is a code element with data-lang inside a pre whose
// inline style sets a background color; both halves are required.

pub fn bare_pre(tokens: &[Token]) -> Option<Region> {
    let root = tokens.first()?;
    if !root.is_start("pre") {
        return None;
    }
    extract(tokens, 0)
}

pub fn div_wrapper(tokens: &[Token]) -> Option<Region> {
    let root = tokens.first()?;
    if !root.is_start("div") {
        return None;
    }
    let pi = find_element(tokens, 1, "pre", any_attrs)?;
    extract(tokens, pi)
}

fn extract(tokens: &[Token], pre_idx: usize) -> Option<Region> {
    let style = attr_value(&tokens[pre_idx].attrs, "style")?;
    if !style.contains("background-color") {
        return None;
    }
    let (inner, _) = subtree(tokens, pre_idx);
    let ci = find_element(inner, 0, "code", |a| attr_value(a, "data-lang").is_some())?;
    let lang = attr_value(&inner[ci].attrs, "data-lang").unwrap_or("");
    let (code_inner, _) = subtree(inner, ci);
    Some(Region {
        lang: lang.to_owned(),
        meta: build_meta(lang, &[]),
        code: trim_code(recover_text(code_inner)),
    })
}
