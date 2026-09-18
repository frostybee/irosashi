use super::{attr_value, class_with_prefix, has_class, has_reserved_class, is_void};
use crate::html::{Token, TokenKind};

/// The two discovery entry points, which map to the two unwrapper chains.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateKind {
    Wrapper,
    BarePre,
}

/// One potential code block region. `tokens` spans the root start tag through its
/// matching end tag, so the root tag sits at index 0. A malformed candidate (unclosed root
/// element) carries only the root token and must never be spliced.
#[derive(Debug, Clone)]
pub struct Candidate<'a> {
    pub kind: CandidateKind,
    pub tokens: &'a [Token],
    pub byte_start: usize,
    pub byte_end: usize,
    pub malformed: bool,
}

/// An existing link or script tag injected by a previous run, identified by its
/// `data-kazari="assets"` marker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetTag {
    pub byte_start: usize,
    pub byte_end: usize,
    pub kind: AssetKind,
    pub reference: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetKind {
    Link,
    Script,
}

/// The result of discovering one HTML file. All byte offsets are positions in the original
/// source bytes.
#[derive(Debug, Default)]
pub struct Page<'a> {
    pub candidates: Vec<Candidate<'a>>,
    pub asset_tags: Vec<AssetTag>,
    /// Content of `meta name="generator"`, empty if absent.
    pub generator: String,
    /// Where a stylesheet link belongs: before the head close tag, else right after the
    /// body start tag, else the start of content (byte 3 after a UTF-8 BOM).
    pub link_insert: usize,
    /// Where the script tag belongs: before the body close tag, else before the html close
    /// tag, else EOF.
    pub script_insert: usize,
    /// Candidates not emitted because they sit inside an element carrying a kz- or
    /// kazari- prefixed class. That namespace is reserved for Kazari's own output.
    pub suppressed: usize,
    /// A reserved class scope was still open at the end of the file, so suppression ran
    /// to EOF (fail closed).
    pub suppressed_unclosed: bool,
}

pub fn discover<'a>(src: &[u8], tokens: &'a [Token]) -> Page<'a> {
    let mut page = Page::default();
    let (mut head_end, mut body_after, mut body_end, mut html_end) = (None, None, None, None);

    let mut depth: usize = 0;
    let mut suppress_at: Option<usize> = None;

    let mut i = 0;
    while i < tokens.len() {
        let tok = &tokens[i];
        match tok.kind {
            TokenKind::Start | TokenKind::SelfClosing => {
                let void = is_void(&tok.name) || tok.kind == TokenKind::SelfClosing;

                match tok.name.as_str() {
                    "meta" => {
                        if attr_value(&tok.attrs, "name") == Some("generator")
                            && page.generator.is_empty()
                        {
                            page.generator =
                                attr_value(&tok.attrs, "content").unwrap_or("").to_owned();
                        }
                    }
                    "body" => {
                        if body_after.is_none() && tok.kind == TokenKind::Start {
                            body_after = Some(tok.end());
                        }
                    }
                    "link" => {
                        if attr_value(&tok.attrs, "data-kazari") == Some("assets") {
                            page.asset_tags.push(AssetTag {
                                byte_start: tok.raw.start,
                                byte_end: tok.end(),
                                kind: AssetKind::Link,
                                reference: attr_value(&tok.attrs, "href").unwrap_or("").to_owned(),
                            });
                        }
                    }
                    "script" => {
                        if attr_value(&tok.attrs, "data-kazari") == Some("assets") {
                            let end = match find_matching_end(tokens, i) {
                                Some(j) => tokens[j].end(),
                                None => tok.end(),
                            };
                            page.asset_tags.push(AssetTag {
                                byte_start: tok.raw.start,
                                byte_end: end,
                                kind: AssetKind::Script,
                                reference: attr_value(&tok.attrs, "src").unwrap_or("").to_owned(),
                            });
                        }
                    }
                    _ => {}
                }

                if suppress_at.is_none() && !void && has_reserved_class(&tok.attrs) {
                    suppress_at = Some(depth);
                }

                if suppress_at.is_some() {
                    if is_wrapper_trigger(tok) || (tok.name == "pre" && !void) {
                        page.suppressed += 1;
                    }
                    if !void {
                        depth += 1;
                    }
                    i += 1;
                    continue;
                }

                if !void && is_wrapper_trigger(tok) {
                    let (cand, next, closed) = make_candidate(tokens, i, CandidateKind::Wrapper);
                    page.candidates.push(cand);
                    if closed {
                        // The subtree is balanced, so depth is unchanged and nested
                        // candidates are consumed: outer wins.
                        i = next;
                        continue;
                    }
                    // Unclosed root: resume at the very next token so one malformed
                    // wrapper never blinds the rest of the file.
                    depth += 1;
                    i += 1;
                    continue;
                }
                if !void && tok.name == "pre" {
                    let (cand, next, closed) = make_candidate(tokens, i, CandidateKind::BarePre);
                    page.candidates.push(cand);
                    if closed {
                        i = next;
                        continue;
                    }
                    depth += 1;
                    i += 1;
                    continue;
                }

                if !void {
                    depth += 1;
                }
                i += 1;
            }
            TokenKind::End => {
                match tok.name.as_str() {
                    "head" => head_end.get_or_insert(tok.raw.start),
                    "body" => body_end.get_or_insert(tok.raw.start),
                    "html" => html_end.get_or_insert(tok.raw.start),
                    _ => &mut 0,
                };
                if !is_void(&tok.name) {
                    depth = depth.saturating_sub(1);
                    if suppress_at.is_some_and(|s| depth <= s) {
                        suppress_at = None;
                    }
                }
                i += 1;
            }
            _ => i += 1,
        }
    }
    if suppress_at.is_some() {
        page.suppressed_unclosed = true;
    }

    let content_start = if src.starts_with(&[0xEF, 0xBB, 0xBF]) {
        3
    } else {
        0
    };
    page.link_insert = head_end.or(body_after).unwrap_or(content_start);
    page.script_insert = body_end.or(html_end).unwrap_or(src.len());
    page
}

/// Re-runs candidate discovery on the inner tokens of a wrapper candidate that no
/// unwrapper matched, so a site component using a trigger class does not swallow a
/// genuine code block nested inside it. Only `candidates` and `suppressed` are meaningful
/// in the result; the caller applies this one level deep only.
pub fn discover_within(inner: &[Token]) -> Page<'_> {
    discover(&[], inner)
}

/// A div or figure whose class list carries the highlight token, the highlighter-rouge
/// token, or a highlight- prefixed token. The bare chroma token is deliberately not a
/// trigger: no generator roots a block there and unrelated widget libraries use it.
fn is_wrapper_trigger(tok: &Token) -> bool {
    if tok.kind != TokenKind::Start || !matches!(tok.name.as_str(), "div" | "figure") {
        return false;
    }
    has_class(&tok.attrs, "highlight")
        || has_class(&tok.attrs, "highlighter-rouge")
        || class_with_prefix(&tok.attrs, "highlight-").is_some()
}

/// Builds a candidate for the element opened at index `i`. Returns where the outer walk
/// resumes and whether the element's end tag was found. An unclosed element is detected
/// explicitly rather than yielding the rest of the window, because a same named nested
/// element could otherwise fake a balanced close.
fn make_candidate(tokens: &[Token], i: usize, kind: CandidateKind) -> (Candidate<'_>, usize, bool) {
    let root = &tokens[i];
    match find_matching_end(tokens, i) {
        None => (
            Candidate {
                kind,
                tokens: &tokens[i..i + 1],
                byte_start: root.raw.start,
                byte_end: root.end(),
                malformed: true,
            },
            i + 1,
            false,
        ),
        Some(j) => (
            Candidate {
                kind,
                tokens: &tokens[i..=j],
                byte_start: root.raw.start,
                byte_end: tokens[j].end(),
                malformed: false,
            },
            j + 1,
            true,
        ),
    }
}

/// The end tag matching the element opened at index `i`, tracking depth on that tag name
/// only.
pub fn find_matching_end(tokens: &[Token], i: usize) -> Option<usize> {
    let tag = &tokens[i].name;
    let mut depth = 1;
    for (j, tok) in tokens.iter().enumerate().skip(i + 1) {
        match tok.kind {
            TokenKind::Start if &tok.name == tag => depth += 1,
            TokenKind::End if &tok.name == tag => {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html::tokenize;

    fn page(src: &str) -> (Vec<Token>, String) {
        (tokenize(src.as_bytes()), src.to_owned())
    }

    #[test]
    fn finds_wrapper_and_bare_pre_candidates() {
        let src = "<html><head></head><body><div class=\"highlight\"><pre><code>a</code></pre></div><pre><code>b</code></pre><div class=\"x\"><pre>c</pre></div></body></html>";
        let (t, s) = page(src);
        let p = discover(s.as_bytes(), &t);
        assert_eq!(p.candidates.len(), 3);
        assert_eq!(p.candidates[0].kind, CandidateKind::Wrapper);
        assert_eq!(
            &s[p.candidates[0].byte_start..p.candidates[0].byte_end],
            "<div class=\"highlight\"><pre><code>a</code></pre></div>"
        );
        assert_eq!(p.candidates[1].kind, CandidateKind::BarePre);
        assert_eq!(
            &s[p.candidates[1].byte_start..p.candidates[1].byte_end],
            "<pre><code>b</code></pre>"
        );
        assert_eq!(
            &s[p.candidates[2].byte_start..p.candidates[2].byte_end],
            "<pre>c</pre>"
        );
        assert!(p.candidates.iter().all(|c| !c.malformed));
        assert_eq!(
            &s[p.link_insert..],
            "</head><body><div class=\"highlight\"><pre><code>a</code></pre></div><pre><code>b</code></pre><div class=\"x\"><pre>c</pre></div></body></html>"
        );
        assert_eq!(&s[p.script_insert..], "</body></html>");
    }

    #[test]
    fn nested_candidates_are_consumed_by_the_outer() {
        let (t, s) =
            page("<div class=\"highlight\"><div class=\"highlight\"><pre>x</pre></div></div>");
        let p = discover(s.as_bytes(), &t);
        assert_eq!(p.candidates.len(), 1);
        assert_eq!(p.candidates[0].byte_end, s.len());
    }

    #[test]
    fn unclosed_root_is_malformed_and_does_not_blind_the_rest() {
        let (t, s) = page("<div class=\"highlight\"><pre><code>a</code></pre>");
        let p = discover(s.as_bytes(), &t);
        assert_eq!(p.candidates.len(), 2);
        assert!(p.candidates[0].malformed);
        assert_eq!(p.candidates[0].tokens.len(), 1);
        assert!(!p.candidates[1].malformed);
    }

    #[test]
    fn reserved_class_scope_suppresses_and_ends_with_ancestor() {
        let (t, s) = page(
            "<div class=\"kazari-block\"><pre>a</pre><div class=\"highlight\"></div></div><pre>b</pre>",
        );
        let p = discover(s.as_bytes(), &t);
        assert_eq!(p.suppressed, 2);
        assert!(!p.suppressed_unclosed);
        assert_eq!(p.candidates.len(), 1);
        assert_eq!(
            &s[p.candidates[0].byte_start..p.candidates[0].byte_end],
            "<pre>b</pre>"
        );

        let (t, s) = page("<section><div class=\"kz-x\"><pre>a</pre></section><pre>b</pre>");
        let p = discover(s.as_bytes(), &t);
        assert_eq!(p.suppressed, 1);
        assert_eq!(p.candidates.len(), 1);

        let (t, s) = page("<div class=\"kz-x\"><pre>a</pre><pre>b</pre>");
        let p = discover(s.as_bytes(), &t);
        assert!(p.suppressed_unclosed);
        assert_eq!(p.suppressed, 2);
        assert!(p.candidates.is_empty());
    }

    #[test]
    fn asset_tags_generator_and_fallback_offsets() {
        let src = "\u{feff}<meta name=\"generator\" content=\"Hugo 0.1\"><link rel=\"stylesheet\" href=\"a.css\" data-kazari=\"assets\"><p>x</p><script src=\"a.js\" data-kazari=\"assets\"></script>";
        let (t, s) = page(src);
        let p = discover(s.as_bytes(), &t);
        assert_eq!(p.generator, "Hugo 0.1");
        assert_eq!(p.asset_tags.len(), 2);
        assert_eq!(p.asset_tags[0].kind, AssetKind::Link);
        assert_eq!(p.asset_tags[0].reference, "a.css");
        assert_eq!(
            &s[p.asset_tags[1].byte_start..p.asset_tags[1].byte_end],
            "<script src=\"a.js\" data-kazari=\"assets\"></script>"
        );
        assert_eq!(p.link_insert, 3);
        assert_eq!(p.script_insert, s.len());

        let (t, s) = page("<body><p>x</p></html>");
        let p = discover(s.as_bytes(), &t);
        assert_eq!(p.link_insert, 6);
        assert_eq!(&s[p.script_insert..], "</html>");
    }

    #[test]
    fn unclosed_asset_script_ends_at_its_start_tag() {
        let (t, s) = page("<script src=\"a.js\" data-kazari=\"assets\">");
        let p = discover(s.as_bytes(), &t);
        assert_eq!(p.asset_tags[0].byte_end, s.len());
    }

    #[test]
    fn discover_within_finds_inner_blocks() {
        let (t, _) =
            page("<div class=\"highlight-widget\"><span></span><pre><code>x</code></pre></div>");
        let inner = discover_within(&t[1..t.len() - 1]);
        assert_eq!(inner.candidates.len(), 1);
        assert_eq!(inner.candidates[0].kind, CandidateKind::BarePre);
    }
}
