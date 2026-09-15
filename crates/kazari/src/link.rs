use std::sync::LazyLock;

use regex::Regex;

use crate::types::LinkAnnotation;

static LINK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"@\[([^\]]+)\]\(([^)]+)\)").unwrap());

/// Finds `@[text](url)` in every line, removes the syntax leaving the text, and
/// returns the cleaned code with one annotation list per line (byte offsets into
/// the cleaned line). Links with an unsafe target keep their literal syntax.
pub fn extract_links(code: &str) -> (String, Vec<Vec<LinkAnnotation>>) {
    let mut cleaned_lines = Vec::new();
    let mut all_links = Vec::new();
    for line in code.split('\n') {
        let (cleaned, links) = extract_line_links(line);
        cleaned_lines.push(cleaned);
        all_links.push(links);
    }
    (cleaned_lines.join("\n"), all_links)
}

/// Strips the link syntax only, for text that is copied rather than rendered.
pub fn strip_links(code: &str) -> String {
    extract_links(code).0
}

fn extract_line_links(line: &str) -> (String, Vec<LinkAnnotation>) {
    let mut links = Vec::new();
    let mut out = String::with_capacity(line.len());
    let mut prev = 0;
    for caps in LINK_RE.captures_iter(line) {
        let full = caps.get(0).unwrap();
        let text = caps.get(1).unwrap().as_str();
        let url = caps.get(2).unwrap().as_str();
        if !is_safe_url(url) {
            out.push_str(&line[prev..full.end()]);
            prev = full.end();
            continue;
        }
        out.push_str(&line[prev..full.start()]);
        let start = out.len();
        out.push_str(text);
        links.push(LinkAnnotation {
            start,
            end: out.len(),
            url: url.to_owned(),
        });
        prev = full.end();
    }
    if links.is_empty() {
        return (line.to_owned(), links);
    }
    out.push_str(&line[prev..]);
    (out, links)
}

/// Accepts `http`, `https` and `mailto` targets and absolute paths; rejects every
/// other scheme (`javascript:` above all) and protocol-relative `//host` paths.
fn is_safe_url(raw: &str) -> bool {
    let trimmed = raw.trim();
    match scheme_of(trimmed) {
        Some(scheme) => {
            let s = scheme.to_ascii_lowercase();
            s == "http" || s == "https" || s == "mailto"
        }
        None => trimmed.starts_with('/') && !trimmed.starts_with("//"),
    }
}

fn scheme_of(s: &str) -> Option<&str> {
    let colon = s.find(':')?;
    let scheme = &s[..colon];
    let mut chars = scheme.chars();
    let first = chars.next()?;
    if !first.is_ascii_alphabetic() {
        return None;
    }
    if chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.')) {
        Some(scheme)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_and_cleans() {
        let (code, links) = extract_links("see @[docs](https://x.y/d) now\nplain");
        assert_eq!(code, "see docs now\nplain");
        assert_eq!(links.len(), 2);
        assert_eq!(
            links[0],
            [LinkAnnotation {
                start: 4,
                end: 8,
                url: "https://x.y/d".into()
            }]
        );
        assert!(links[1].is_empty());
    }

    #[test]
    fn offsets_after_an_earlier_link_on_the_same_line() {
        let (code, links) = extract_links("@[a](/one) and @[bcd](/two)!");
        assert_eq!(code, "a and bcd!");
        assert_eq!((links[0][0].start, links[0][0].end), (0, 1));
        assert_eq!((links[0][1].start, links[0][1].end), (6, 9));
        assert_eq!(&code[links[0][1].start..links[0][1].end], "bcd");
    }

    #[test]
    fn unsafe_targets_keep_the_literal_syntax() {
        let src = "x @[click](javascript:alert(1)) y @[ok](https://a) z";
        let (code, links) = extract_links(src);
        assert_eq!(code, "x @[click](javascript:alert(1)) y ok z");
        assert_eq!(links[0].len(), 1);
        assert_eq!(&code[links[0][0].start..links[0][0].end], "ok");
    }

    #[test]
    fn url_safety() {
        assert!(is_safe_url("https://example.com"));
        assert!(is_safe_url("HTTP://example.com"));
        assert!(is_safe_url("mailto:me@example.com"));
        assert!(is_safe_url("/docs/page"));
        assert!(is_safe_url("  /docs "));
        assert!(!is_safe_url("//evil.com"));
        assert!(!is_safe_url("javascript:alert(1)"));
        assert!(!is_safe_url("data:text/html,x"));
        assert!(!is_safe_url("relative/path"));
        assert!(!is_safe_url("C:\\path"));
    }

    #[test]
    fn no_links_returns_input_unchanged() {
        let src = "fn main() {}\n";
        let (code, links) = extract_links(src);
        assert_eq!(code, src);
        assert_eq!(links.len(), 2);
        assert!(links.iter().all(Vec::is_empty));
        assert_eq!(strip_links("@[a](/b) c"), "a c");
    }
}
