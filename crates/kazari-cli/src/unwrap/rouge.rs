use super::linehighlight::build_meta;
use super::{
    Region, any_attrs, attr_value, class_with_prefix, find_element, has_class, recover_text,
    strip_prefix, subtree, trim_code,
};
use crate::html::{Token, TokenKind};

/// Jekyll's kramdown output: an outer div carrying both a language class token and the
/// highlighter-rouge token, wrapping div.highlight > pre.highlight > code. The language
/// lives only in the outer class list.
pub fn div_wrapper(tokens: &[Token]) -> Option<Region> {
    let root = tokens.first()?;
    if !root.is_start("div") {
        return None;
    }
    let lang_token = class_with_prefix(&root.attrs, "language-")?;
    if !has_class(&root.attrs, "highlighter-rouge") {
        return None;
    }
    let pi = find_element(tokens, 1, "pre", |a| has_class(a, "highlight"))?;
    let (inner, _) = subtree(tokens, pi);
    // The code element is what separates Rouge's div.highlight from the Pygments family,
    // which never emits one.
    let ci = find_element(inner, 0, "code", any_attrs)?;
    let lang = strip_prefix(lang_token, "language-");
    let (code_inner, _) = subtree(inner, ci);
    Some(Region {
        lang: lang.to_owned(),
        meta: build_meta(lang, &[]),
        code: trim_code(recover_text(code_inner)),
    })
}

/// The legacy Liquid highlight tag output: figure.highlight > pre > code with a language
/// class and data-lang.
pub fn highlight_figure(tokens: &[Token]) -> Option<Region> {
    let root = tokens.first()?;
    if !root.is_start("figure") || !has_class(&root.attrs, "highlight") {
        return None;
    }
    let pi = find_element(tokens, 1, "pre", any_attrs)?;
    let (inner, _) = subtree(tokens, pi);
    let ci = find_element(inner, 0, "code", any_attrs)?;
    let lang = match attr_value(&inner[ci].attrs, "data-lang") {
        Some(v) => v,
        None => strip_prefix(
            class_with_prefix(&inner[ci].attrs, "language-")?,
            "language-",
        ),
    };
    let (code_inner, _) = subtree(inner, ci);
    Some(Region {
        lang: lang.to_owned(),
        meta: build_meta(lang, &[]),
        code: trim_code(recover_text(code_inner)),
    })
}

/// Rouge's line number table: a rouge-table with a gutter cell and a td.code cell holding
/// the real content. Tried before the other Rouge shapes because the table nests inside
/// them.
pub fn linenos_table(tokens: &[Token]) -> Option<Region> {
    let root = tokens.first()?;
    if root.kind != TokenKind::Start || !matches!(root.name.as_str(), "div" | "figure") {
        return None;
    }
    let ti = find_element(tokens, 1, "table", |a| has_class(a, "rouge-table"))?;
    let lang = table_lang(&tokens[..ti]);
    let (inner, _) = subtree(tokens, ti);
    let di = find_element(inner, 0, "td", |a| has_class(a, "code"))?;
    let (cell, _) = subtree(inner, di);
    let pi = find_element(cell, 0, "pre", any_attrs)?;
    let (pre_inner, _) = subtree(cell, pi);
    Some(Region {
        lang: lang.to_owned(),
        meta: build_meta(lang, &[]),
        code: trim_code(recover_text(pre_inner)),
    })
}

/// The language among the elements that wrap the table: a data-lang or language class on
/// a code element, or a language token on the region root.
fn table_lang(tokens: &[Token]) -> &str {
    for tok in tokens {
        if !tok.is_start("code") {
            continue;
        }
        if let Some(v) = attr_value(&tok.attrs, "data-lang") {
            return v;
        }
        if let Some(c) = class_with_prefix(&tok.attrs, "language-") {
            return strip_prefix(c, "language-");
        }
    }
    tokens
        .first()
        .and_then(|root| class_with_prefix(&root.attrs, "language-"))
        .map(|c| strip_prefix(c, "language-"))
        .unwrap_or("")
}
