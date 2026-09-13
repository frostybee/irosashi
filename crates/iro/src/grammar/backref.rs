use std::borrow::Cow;

/// Whether `pattern` contains a `\N` group reference that must be substituted with
/// captured text before the pattern is compiled.
pub fn has_backref_marker(pattern: &str) -> bool {
    let bytes = pattern.as_bytes();
    bytes
        .windows(2)
        .any(|pair| pair[0] == b'\\' && pair[1].is_ascii_digit())
}

/// Replaces every `\N` in an end or while pattern with the escaped text of capture
/// group `N`; groups that did not participate contribute an empty string.
pub fn resolve_backrefs(pattern: &str, captures: &[&str]) -> String {
    let mut out = String::with_capacity(pattern.len());
    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        let mut digits = String::new();
        while let Some(d) = chars.peek().filter(|d| d.is_ascii_digit()) {
            digits.push(*d);
            chars.next();
        }
        if digits.is_empty() {
            out.push('\\');
            continue;
        }
        let text = digits
            .parse::<usize>()
            .ok()
            .and_then(|index| captures.get(index).copied())
            .unwrap_or("");
        out.push_str(&escape_regex(text));
    }
    out
}

/// Escapes vscode-textmate's set of regex metacharacters, including whitespace so the
/// result survives `(?x)` patterns.
pub fn escape_regex(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if matches!(
            c,
            '-' | '\\'
                | '{'
                | '}'
                | '*'
                | '+'
                | '?'
                | '|'
                | '^'
                | '$'
                | '.'
                | ','
                | '['
                | ']'
                | '('
                | ')'
                | '#'
        ) || c.is_whitespace()
        {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// Substitutes `$N`, `${N:/downcase}` and `${N:/upcase}` in a scope name with the text
/// of capture group `N`, leading dots stripped. References to groups beyond the capture
/// list are left as written.
pub fn resolve_scope_backrefs<'a>(name: &'a str, capture_texts: &[&str]) -> Cow<'a, str> {
    if !name.contains('$') {
        return Cow::Borrowed(name);
    }
    let mut out = String::with_capacity(name.len());
    let mut rest = name;
    while let Some(dollar) = rest.find('$') {
        out.push_str(&rest[..dollar]);
        let after = &rest[dollar + 1..];
        match parse_reference(after) {
            Some((index, case, consumed)) => {
                match capture_texts.get(index) {
                    Some(text) => {
                        let trimmed = text.trim_start_matches('.');
                        match case {
                            Case::Keep => out.push_str(trimmed),
                            Case::Down => out.push_str(&trimmed.to_lowercase()),
                            Case::Up => out.push_str(&trimmed.to_uppercase()),
                        }
                    }
                    None => out.push_str(&rest[dollar..dollar + 1 + consumed]),
                }
                rest = &after[consumed..];
            }
            None => {
                out.push('$');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    Cow::Owned(out)
}

#[derive(Clone, Copy)]
enum Case {
    Keep,
    Down,
    Up,
}

/// Parses the part after `$`: `N` or `{N:/downcase}` / `{N:/upcase}`.
/// Returns the group index, the case command, and the number of bytes consumed.
fn parse_reference(text: &str) -> Option<(usize, Case, usize)> {
    let digits_len = |s: &str| s.bytes().take_while(u8::is_ascii_digit).count();
    if let Some(inner) = text.strip_prefix('{') {
        let len = digits_len(inner);
        if len == 0 {
            return None;
        }
        let index = inner[..len].parse().ok()?;
        let after_digits = &inner[len..];
        let (case, command_len) = if after_digits.starts_with(":/downcase}") {
            (Case::Down, ":/downcase}".len())
        } else if after_digits.starts_with(":/upcase}") {
            (Case::Up, ":/upcase}".len())
        } else {
            return None;
        };
        return Some((index, case, 1 + len + command_len));
    }
    let len = digits_len(text);
    if len == 0 {
        return None;
    }
    Some((text[..len].parse().ok()?, Case::Keep, len))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_backref_markers() {
        assert!(has_backref_marker(r"\1"));
        assert!(has_backref_marker(r"(?<=\2)"));
        assert!(has_backref_marker(r"\\1"));
        assert!(!has_backref_marker(r"\w+"));
        assert!(!has_backref_marker("plain"));
        assert!(!has_backref_marker(""));
    }

    #[test]
    fn resolves_backrefs_with_escaping() {
        let captures = ["whole", "a.b", "", "x y", "#c-d"];
        assert_eq!(resolve_backrefs(r"\1", &captures), r"a\.b");
        assert_eq!(resolve_backrefs(r"end\1end", &captures), r"enda\.bend");
        assert_eq!(resolve_backrefs(r"\2", &captures), "");
        assert_eq!(resolve_backrefs(r"\3", &captures), r"x\ y");
        assert_eq!(resolve_backrefs(r"\4", &captures), r"\#c\-d");
        assert_eq!(resolve_backrefs(r"\9", &captures), "");
        assert_eq!(resolve_backrefs(r"\10", &captures), "");
        assert_eq!(resolve_backrefs(r"\w+\s", &captures), r"\w+\s");
        assert_eq!(resolve_backrefs(r"trailing\", &captures), r"trailing\");
    }

    #[test]
    fn escapes_full_vscode_set() {
        assert_eq!(
            escape_regex(r"a-b\c{d}e*f+g?h|i^j$k.l,m[n]o(p)q#r s"),
            r"a\-b\\c\{d\}e\*f\+g\?h\|i\^j\$k\.l\,m\[n\]o\(p\)q\#r\ s"
        );
        assert_eq!(escape_regex("plain_text/"), "plain_text/");
    }

    #[test]
    fn resolves_scope_backrefs() {
        let texts = ["Whole", "Foo", "..method", ""];
        assert_eq!(resolve_scope_backrefs("plain.scope", &texts), "plain.scope");
        assert_eq!(resolve_scope_backrefs("meta.$1.x", &texts), "meta.Foo.x");
        assert_eq!(
            resolve_scope_backrefs("meta.${1:/downcase}", &texts),
            "meta.foo"
        );
        assert_eq!(
            resolve_scope_backrefs("meta.${1:/upcase}", &texts),
            "meta.FOO"
        );
        assert_eq!(resolve_scope_backrefs("a.$0", &texts), "a.Whole");
        assert_eq!(resolve_scope_backrefs("a.$2", &texts), "a.method");
        assert_eq!(resolve_scope_backrefs("a.$3.b", &texts), "a..b");
        assert_eq!(resolve_scope_backrefs("a.$5.b", &texts), "a.$5.b");
        assert_eq!(
            resolve_scope_backrefs("a.${7:/downcase}", &texts),
            "a.${7:/downcase}"
        );
        assert_eq!(
            resolve_scope_backrefs("a.${1:/titlecase}", &texts),
            "a.${1:/titlecase}"
        );
        assert_eq!(resolve_scope_backrefs("$1$1", &texts), "FooFoo");
        assert_eq!(resolve_scope_backrefs("cost.$", &texts), "cost.$");
        assert_eq!(resolve_scope_backrefs("$x", &texts), "$x");
    }
}
