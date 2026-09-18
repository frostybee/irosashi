use super::linehighlight::build_meta;
use super::{
    Region, any_attrs, attr_value, find_element, has_class, recover_text, subtree, trim_code,
};
use crate::html::Token;

/// Astro's built in Shiki output: a pre with the astro-code class token, the language in
/// data-language on the pre, lines as span.line, and class free token spans styled inline.
/// The astro-code requirement keeps this from claiming the Expressive Code shape used by
/// Starlight, which also carries data-language but structures lines as div.ec-line.
pub fn astro_bare_pre(tokens: &[Token]) -> Option<Region> {
    let root = tokens.first()?;
    if !root.is_start("pre") || !has_class(&root.attrs, "astro-code") {
        return None;
    }
    let lang = attr_value(&root.attrs, "data-language").unwrap_or("");
    let ci = find_element(tokens, 1, "code", any_attrs)?;
    let (code_inner, _) = subtree(tokens, ci);
    Some(Region {
        lang: lang.to_owned(),
        meta: build_meta(lang, &[]),
        code: trim_code(recover_text(code_inner)),
    })
}
