pub fn escape_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

pub fn escape_attr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

/// Escapes `s` for the inside of a Typst string literal. Code mode strings keep
/// whitespace verbatim and know no markup, so only the backslash, the quote and
/// control characters need attention.
pub fn escape_typst_string(s: &str) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 || c == '\u{7f}' => {
                write!(out, "\\u{{{:x}}}", c as u32).unwrap();
            }
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_typst_string_backslash_and_quote() {
        assert_eq!(escape_typst_string(r"a\b"), r"a\\b");
        assert_eq!(escape_typst_string(r#"say "hi""#), r#"say \"hi\""#);
        assert_eq!(
            escape_typst_string(r#"path = "C:\\dir""#),
            r#"path = \"C:\\\\dir\""#
        );
    }

    #[test]
    fn escape_typst_string_control_chars() {
        assert_eq!(escape_typst_string("a\nb"), r"a\nb");
        assert_eq!(escape_typst_string("a\rb"), r"a\rb");
        assert_eq!(escape_typst_string("a\tb"), r"a\tb");
        assert_eq!(escape_typst_string("a\u{1}b"), r"a\u{1}b");
        assert_eq!(escape_typst_string("a\u{7f}b"), r"a\u{7f}b");
    }

    #[test]
    fn escape_typst_string_markup_chars_untouched() {
        let code = "x -- y // #tag $z @me ~ 'q' *b* _i_ `c` [a] <b> = h";
        assert_eq!(escape_typst_string(code), code);
        assert_eq!(escape_typst_string("    indented"), "    indented");
    }

    #[test]
    fn escape_typst_string_empty() {
        assert_eq!(escape_typst_string(""), "");
    }
}
