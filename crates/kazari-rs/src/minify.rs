//! Comment and whitespace stripping for the generated stylesheet and scripts. Not a
//! parser: values, selectors and string literals are copied as written.

/// Drops comments, collapses whitespace runs, removes the spaces that CSS never
/// needs (around `{`, `}`, `;`, `,` and after `:`) and the `;` before a `}`.
/// Quoted strings are kept verbatim, so data URIs survive.
pub fn css(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut pending_space = false;

    while let Some(c) = chars.next() {
        match c {
            '/' if chars.peek() == Some(&'*') => {
                chars.next();
                let mut prev = '\0';
                for ch in chars.by_ref() {
                    if prev == '*' && ch == '/' {
                        break;
                    }
                    prev = ch;
                }
                pending_space = true;
            }
            '"' | '\'' => {
                flush_space(&mut out, &mut pending_space, c);
                out.push(c);
                while let Some(ch) = chars.next() {
                    out.push(ch);
                    if ch == '\\' {
                        if let Some(escaped) = chars.next() {
                            out.push(escaped);
                        }
                    } else if ch == c {
                        break;
                    }
                }
            }
            c if c.is_whitespace() => pending_space = true,
            '}' => {
                pending_space = false;
                while out.ends_with(';') {
                    out.pop();
                }
                out.push('}');
            }
            _ => {
                flush_space(&mut out, &mut pending_space, c);
                out.push(c);
            }
        }
    }
    out
}

fn flush_space(out: &mut String, pending: &mut bool, next: char) {
    if !*pending {
        return;
    }
    *pending = false;
    let after_separator = matches!(out.chars().last(), Some('{' | '}' | ';' | ':' | ','));
    let before_separator = matches!(next, '{' | ';' | ',');
    if !out.is_empty() && !after_separator && !before_separator {
        out.push(' ');
    }
}

/// Drops `//` and `/* */` comments outside strings, template literals and regex
/// literals, then strips indentation and blank lines. Newlines are kept, so
/// automatic semicolon insertion is unaffected.
pub fn js(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::with_capacity(input.len());
    let mut last_significant: Option<char> = None;
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        match c {
            '/' if next == Some('/') => {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            '/' if next == Some('*') => {
                i += 2;
                while i < chars.len() && !(chars[i] == '*' && chars.get(i + 1) == Some(&'/')) {
                    i += 1;
                }
                i += 2;
                out.push(' ');
            }
            '"' | '\'' | '`' => {
                out.push(c);
                i += 1;
                while i < chars.len() {
                    let ch = chars[i];
                    out.push(ch);
                    i += 1;
                    if ch == '\\' {
                        if let Some(&escaped) = chars.get(i) {
                            out.push(escaped);
                            i += 1;
                        }
                    } else if ch == c {
                        break;
                    }
                }
                last_significant = Some(c);
            }
            '/' if starts_regex(last_significant) => {
                out.push(c);
                i += 1;
                let mut in_class = false;
                while i < chars.len() {
                    let ch = chars[i];
                    out.push(ch);
                    i += 1;
                    match ch {
                        '\\' => {
                            if let Some(&escaped) = chars.get(i) {
                                out.push(escaped);
                                i += 1;
                            }
                        }
                        '[' => in_class = true,
                        ']' => in_class = false,
                        '/' if !in_class => break,
                        '\n' => break,
                        _ => {}
                    }
                }
                last_significant = Some('/');
            }
            _ => {
                out.push(c);
                if !c.is_whitespace() {
                    last_significant = Some(c);
                }
                i += 1;
            }
        }
    }

    let mut result = String::with_capacity(out.len());
    for line in out.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        result.push_str(line);
        result.push('\n');
    }
    result
}

/// A `/` after one of these begins a regex literal rather than a division.
fn starts_regex(last_significant: Option<char>) -> bool {
    match last_significant {
        None => true,
        Some(c) => "(,=:[!&|?{};+-*%<>~^".contains(c),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn css_empty_and_already_minified() {
        assert_eq!(css(""), "");
        let tight = "a{color:red}b{margin:0 1px}";
        assert_eq!(css(tight), tight);
    }

    #[test]
    fn css_strips_comments_and_whitespace() {
        let input = "/* header */\n.a {\n  color: red;\n  margin: 0 1px;\n}\n\n.b > .c, .d {\n  border: 1px solid transparent;\n}\n";
        let out = css(input);
        assert_eq!(
            out,
            ".a{color:red;margin:0 1px}.b > .c,.d{border:1px solid transparent}"
        );
        assert!(out.len() < input.len());
    }

    #[test]
    fn css_keeps_strings_and_data_uris() {
        let input = ":root {\n  --icon: url(\"data:image/svg+xml,%3Csvg viewBox='0 0 16 16'%3E%3C/svg%3E\");\n  content: ' a  b ';\n}\n";
        let out = css(input);
        assert_eq!(
            out,
            ":root{--icon:url(\"data:image/svg+xml,%3Csvg viewBox='0 0 16 16'%3E%3C/svg%3E\");content:' a  b '}"
        );
    }

    #[test]
    fn css_media_and_layer() {
        let input =
            "@layer kazari {\n@media (prefers-color-scheme: dark) {\n  :root { --x: 1; }\n}\n}\n";
        assert_eq!(
            css(input),
            "@layer kazari{@media (prefers-color-scheme:dark){:root{--x:1}}}"
        );
    }

    #[test]
    fn js_empty_and_already_minified() {
        assert_eq!(js(""), "");
        assert_eq!(js("var a=1;\n"), "var a=1;\n");
    }

    #[test]
    fn js_strips_comments_and_indentation() {
        let input = "(function () {\n  // line comment\n  var url = 'http://x//y'; /* block */\n  var re = /\\x7f\\//g;\n\n  var t = `a // b`;\n})();\n";
        let out = js(input);
        assert_eq!(
            out,
            "(function () {\nvar url = 'http://x//y';\nvar re = /\\x7f\\//g;\nvar t = `a // b`;\n})();\n"
        );
        assert!(out.len() < input.len());
    }

    #[test]
    fn js_division_is_not_a_regex() {
        assert_eq!(
            js("var x = a / b; // c\nvar y = 2;\n"),
            "var x = a / b;\nvar y = 2;\n"
        );
    }
}
