use super::classlist::{attr_value, class_list, has_class};
use crate::html::{Attr, Token, TokenKind};

pub const SKIP_KZ_CLASS: &str = "kz-class";
pub const SKIP_KAZARI_IGNORE: &str = "data-kazari-ignore";
pub const SKIP_MERMAID: &str = "mermaid";

/// Why a candidate region must be left untouched, or `None` when it should proceed to
/// unwrapping. These run before any chain dispatch because an already processed site hits
/// them on nearly every candidate during a check run.
pub fn skip_reason(tokens: &[Token]) -> Option<&'static str> {
    let root = tokens.first()?;
    if has_reserved_class(&root.attrs) {
        return Some(SKIP_KZ_CLASS);
    }
    if attr_value(&root.attrs, "data-kazari") == Some("ignore") {
        return Some(SKIP_KAZARI_IGNORE);
    }
    if is_mermaid(tokens) {
        return Some(SKIP_MERMAID);
    }
    None
}

/// Kazari's own output roots at class kazari-block with kz- prefixed classes inside, so
/// both prefixes mark an element as already processed.
pub fn has_reserved_class(attrs: &[Attr]) -> bool {
    class_list(attrs).any(|c| c.starts_with("kz-") || c.starts_with("kazari-"))
}

fn is_mermaid(tokens: &[Token]) -> bool {
    tokens.iter().any(|tok| {
        tok.kind == TokenKind::Start
            && matches!(tok.name.as_str(), "pre" | "code" | "div")
            && (has_class(&tok.attrs, "mermaid")
                || has_class(&tok.attrs, "language-mermaid")
                || attr_value(&tok.attrs, "data-lang") == Some("mermaid"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html::tokenize;

    #[test]
    fn reasons() {
        assert_eq!(skip_reason(&[]), None);
        assert_eq!(
            skip_reason(&tokenize(b"<div class=\"kazari-block\"><pre></pre></div>")),
            Some(SKIP_KZ_CLASS)
        );
        assert_eq!(
            skip_reason(&tokenize(b"<pre class=\"x kz-line\"></pre>")),
            Some(SKIP_KZ_CLASS)
        );
        assert_eq!(
            skip_reason(&tokenize(
                b"<pre data-kazari=\"ignore\"><code>x</code></pre>"
            )),
            Some(SKIP_KAZARI_IGNORE)
        );
        assert_eq!(
            skip_reason(&tokenize(
                b"<pre><code class=\"language-mermaid\">g</code></pre>"
            )),
            Some(SKIP_MERMAID)
        );
        assert_eq!(
            skip_reason(&tokenize(
                b"<div class=\"highlight\"><pre class=\"mermaid\">g</pre></div>"
            )),
            Some(SKIP_MERMAID)
        );
        assert_eq!(
            skip_reason(&tokenize(
                b"<pre><code data-lang=\"mermaid\">g</code></pre>"
            )),
            Some(SKIP_MERMAID)
        );
        assert_eq!(
            skip_reason(&tokenize(b"<pre><span class=\"mermaid\">g</span></pre>")),
            None
        );
        assert_eq!(
            skip_reason(&tokenize(
                b"<pre><code class=\"language-go\">x</code></pre>"
            )),
            None
        );
    }
}
