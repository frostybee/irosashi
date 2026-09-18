use super::classlist::class_list;
use crate::html::{Attr, Token, TokenKind};

/// Class tokens that mark line number gutters across the supported generator families:
/// Chroma (ln inline, lnt and lntd in table mode), Rouge (gutter, gl, lineno), Pygments
/// (linenos, linenodiv) and the Prism line numbers plugin.
const GUTTER_CLASSES: [&str; 10] = [
    "ln",
    "lnt",
    "lntd",
    "gutter",
    "gl",
    "linenos",
    "lineno",
    "linenodiv",
    "line-numbers",
    "line-numbers-rows",
];

const VOID_ELEMENTS: [&str; 14] = [
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

pub fn is_void(name: &str) -> bool {
    VOID_ELEMENTS.contains(&name)
}

fn is_gutter_element(attrs: &[Attr]) -> bool {
    class_list(attrs).any(|c| GUTTER_CLASSES.contains(&c))
}

/// Concatenates the text content of a token window, excluding every subtree rooted at a
/// gutter element. A br element becomes a newline. Comments are ignored. Callers trim
/// trailing newlines once.
pub fn recover_text(tokens: &[Token]) -> String {
    let mut out = String::new();
    let mut depth: usize = 0;
    let mut skip_above: Option<usize> = None;
    for tok in tokens {
        match tok.kind {
            TokenKind::Start => {
                if tok.name == "br" {
                    if skip_above.is_none() {
                        out.push('\n');
                    }
                    continue;
                }
                if is_void(&tok.name) {
                    continue;
                }
                depth += 1;
                if skip_above.is_none() && is_gutter_element(&tok.attrs) {
                    skip_above = Some(depth - 1);
                }
            }
            TokenKind::SelfClosing => {
                if tok.name == "br" && skip_above.is_none() {
                    out.push('\n');
                }
            }
            TokenKind::End => {
                if is_void(&tok.name) {
                    continue;
                }
                depth = depth.saturating_sub(1);
                if skip_above.is_some_and(|s| depth <= s) {
                    skip_above = None;
                }
            }
            TokenKind::Text => {
                if skip_above.is_none() {
                    out.push_str(&tok.data);
                }
            }
            TokenKind::Comment | TokenKind::Doctype => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html::tokenize;

    fn text(src: &str) -> String {
        recover_text(&tokenize(src.as_bytes()))
    }

    #[test]
    fn concatenates_text_and_decodes() {
        assert_eq!(text("<span>a</span> &lt;b&gt;<!-- c -->\n"), "a <b>\n");
    }

    #[test]
    fn excludes_gutter_subtrees() {
        assert_eq!(
            text(
                "<span class=\"ln\">1</span><span class=\"cl\">x</span><span class=\"lnt\"><span>2</span></span>y"
            ),
            "xy"
        );
        assert_eq!(
            text(
                "<td class=\"gutter gl\"><pre>1\n2</pre></td><td class=\"code\"><pre>a\nb</pre></td>"
            ),
            "a\nb"
        );
    }

    #[test]
    fn br_is_newline_unless_inside_gutter() {
        assert_eq!(text("a<br>b<br/>c"), "a\nb\nc");
        assert_eq!(text("<span class=\"ln\">1<br>2</span>x"), "x");
    }

    #[test]
    fn unbalanced_end_tags_do_not_underflow() {
        assert_eq!(text("</span></span>a<img>b"), "ab");
    }
}
