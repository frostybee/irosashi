use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Start,
    End,
    SelfClosing,
    Text,
    Comment,
    Doctype,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attr {
    pub key: String,
    pub value: String,
}

/// One HTML token with the byte range of the source it was read from. Ranges tile the
/// input exactly, so every offset a later phase reports is a position in the original file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    /// Lowercased tag name for tag tokens, empty otherwise.
    pub name: String,
    pub attrs: Vec<Attr>,
    /// Decoded text for text tokens, empty otherwise.
    pub data: String,
    pub raw: Range<usize>,
}

impl Token {
    pub fn is_start(&self, name: &str) -> bool {
        self.kind == TokenKind::Start && self.name == name
    }

    pub fn end(&self) -> usize {
        self.raw.end
    }
}

const RAW_TEXT: [&str; 10] = [
    "iframe",
    "noembed",
    "noframes",
    "noscript",
    "plaintext",
    "script",
    "style",
    "textarea",
    "title",
    "xmp",
];

const RCDATA: [&str; 2] = ["textarea", "title"];

pub fn tokenize(src: &[u8]) -> Vec<Token> {
    Tokenizer {
        src,
        pos: 0,
        tokens: Vec::new(),
    }
    .run()
}

struct Tokenizer<'a> {
    src: &'a [u8],
    pos: usize,
    tokens: Vec<Token>,
}

impl Tokenizer<'_> {
    fn run(mut self) -> Vec<Token> {
        while self.pos < self.src.len() {
            let start = self.pos;
            if self.src[start] == b'<'
                && let Some(raw_tag) = self.read_markup(start)
            {
                if let Some(tag) = raw_tag {
                    self.read_raw_text(&tag);
                }
                continue;
            }
            self.read_text(start);
        }
        self.tokens
    }

    fn peek(&self, at: usize) -> Option<u8> {
        self.src.get(at).copied()
    }

    /// Text runs until a `<` that opens a tag, comment, declaration or processing
    /// instruction; a bare `<` stays text.
    fn read_text(&mut self, start: usize) {
        let mut i = start + 1;
        while i < self.src.len() {
            if self.src[i] == b'<' && self.starts_markup(i) {
                break;
            }
            i += 1;
        }
        self.push_text(start..i, true);
        self.pos = i;
    }

    fn starts_markup(&self, lt: usize) -> bool {
        matches!(self.peek(lt + 1), Some(c) if c.is_ascii_alphabetic() || matches!(c, b'/' | b'!' | b'?'))
    }

    /// Reads whatever starts at a `<`. Returns `None` when it is plain text, otherwise the
    /// raw-text element name to consume after a start tag, if any.
    fn read_markup(&mut self, lt: usize) -> Option<Option<String>> {
        match self.peek(lt + 1) {
            Some(c) if c.is_ascii_alphabetic() => Some(self.read_start_tag(lt)),
            Some(b'/') => {
                match self.peek(lt + 2) {
                    Some(c) if c.is_ascii_alphabetic() => self.read_end_tag(lt),
                    Some(b'>') => self.push(TokenKind::Comment, "", lt..lt + 3),
                    _ => self.read_bogus_comment(lt, lt + 2),
                }
                Some(None)
            }
            Some(b'!') => {
                if self.src[lt + 2..].starts_with(b"--") {
                    self.read_comment(lt);
                } else if self.src[lt + 2..]
                    .get(..7)
                    .is_some_and(|s| s.eq_ignore_ascii_case(b"doctype"))
                {
                    self.read_doctype(lt);
                } else {
                    self.read_bogus_comment(lt, lt + 2);
                }
                Some(None)
            }
            Some(b'?') => {
                self.read_bogus_comment(lt, lt + 2);
                Some(None)
            }
            _ => None,
        }
    }

    fn push(&mut self, kind: TokenKind, name: &str, raw: Range<usize>) {
        self.tokens.push(Token {
            kind,
            name: name.to_owned(),
            attrs: Vec::new(),
            data: String::new(),
            raw: raw.clone(),
        });
        self.pos = raw.end;
    }

    fn push_text(&mut self, raw: Range<usize>, decode: bool) {
        let text = String::from_utf8_lossy(&self.src[raw.clone()]);
        let text = convert_newlines(&text);
        let data = if decode {
            html_escape::decode_html_entities(&text).into_owned()
        } else {
            text
        };
        self.tokens.push(Token {
            kind: TokenKind::Text,
            name: String::new(),
            attrs: Vec::new(),
            data,
            raw: raw.clone(),
        });
        self.pos = raw.end;
    }

    fn read_comment(&mut self, lt: usize) {
        let body = lt + 4;
        let end = find(self.src, body, b"-->")
            .map(|i| i + 3)
            .unwrap_or(self.src.len());
        self.push(TokenKind::Comment, "", lt..end);
    }

    fn read_bogus_comment(&mut self, lt: usize, body: usize) {
        let end = find(self.src, body, b">")
            .map(|i| i + 1)
            .unwrap_or(self.src.len());
        self.push(TokenKind::Comment, "", lt..end);
    }

    fn read_doctype(&mut self, lt: usize) {
        let end = find(self.src, lt + 9, b">")
            .map(|i| i + 1)
            .unwrap_or(self.src.len());
        self.push(TokenKind::Doctype, "", lt..end);
    }

    fn read_end_tag(&mut self, lt: usize) {
        let mut i = lt + 2;
        let name_start = i;
        while i < self.src.len() && !is_space(self.src[i]) && self.src[i] != b'>' {
            i += 1;
        }
        let name = ascii_lower(&self.src[name_start..i]);
        let Some(gt) = find(self.src, i, b">") else {
            self.push_text(lt..self.src.len(), false);
            return;
        };
        self.push(TokenKind::End, &name, lt..gt + 1);
    }

    fn read_start_tag(&mut self, lt: usize) -> Option<String> {
        let mut i = lt + 1;
        let name_start = i;
        while i < self.src.len() && !is_space(self.src[i]) && !matches!(self.src[i], b'>' | b'/') {
            i += 1;
        }
        let name = ascii_lower(&self.src[name_start..i]);
        let mut attrs = Vec::new();
        let mut self_closing = false;
        loop {
            while i < self.src.len() && (is_space(self.src[i]) || self.src[i] == b'/') {
                if self.src[i] == b'/' && self.peek(i + 1) == Some(b'>') {
                    self_closing = true;
                    i += 1;
                    break;
                }
                i += 1;
            }
            match self.peek(i) {
                None => {
                    self.push_text(lt..self.src.len(), false);
                    return None;
                }
                Some(b'>') => {
                    i += 1;
                    break;
                }
                _ => {}
            }
            let key_start = i;
            while i < self.src.len()
                && !is_space(self.src[i])
                && !matches!(self.src[i], b'=' | b'>' | b'/')
            {
                i += 1;
            }
            let key = ascii_lower(&self.src[key_start..i]);
            while i < self.src.len() && is_space(self.src[i]) {
                i += 1;
            }
            let mut value = Vec::new();
            if self.peek(i) == Some(b'=') {
                i += 1;
                while i < self.src.len() && is_space(self.src[i]) {
                    i += 1;
                }
                match self.peek(i) {
                    Some(q @ (b'"' | b'\'')) => {
                        let vs = i + 1;
                        let ve = find(self.src, vs, &[q]).unwrap_or(self.src.len());
                        value.extend_from_slice(&self.src[vs..ve]);
                        i = (ve + 1).min(self.src.len());
                    }
                    Some(_) => {
                        let vs = i;
                        while i < self.src.len() && !is_space(self.src[i]) && self.src[i] != b'>' {
                            i += 1;
                        }
                        value.extend_from_slice(&self.src[vs..i]);
                    }
                    None => {}
                }
            }
            if !key.is_empty() {
                let value = convert_newlines(&String::from_utf8_lossy(&value));
                attrs.push(Attr {
                    key,
                    value: html_escape::decode_html_entities(&value).into_owned(),
                });
            }
        }
        let kind = if self_closing {
            TokenKind::SelfClosing
        } else {
            TokenKind::Start
        };
        self.tokens.push(Token {
            kind,
            name: name.clone(),
            attrs,
            data: String::new(),
            raw: lt..i,
        });
        self.pos = i;
        (kind == TokenKind::Start && RAW_TEXT.contains(&name.as_str())).then_some(name)
    }

    fn read_raw_text(&mut self, tag: &str) {
        let start = self.pos;
        let mut i = start;
        let end = loop {
            if tag == "plaintext" {
                break self.src.len();
            }
            match find(self.src, i, b"</") {
                None => break self.src.len(),
                Some(at) => {
                    let after = at + 2 + tag.len();
                    let matches = self.src[at + 2..]
                        .get(..tag.len())
                        .is_some_and(|s| s.eq_ignore_ascii_case(tag.as_bytes()))
                        && self
                            .peek(after)
                            .is_none_or(|c| is_space(c) || matches!(c, b'/' | b'>'));
                    if matches {
                        break at;
                    }
                    i = at + 2;
                }
            }
        };
        if end > start {
            self.push_text(start..end, RCDATA.contains(&tag));
        }
        self.pos = end;
    }
}

fn is_space(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | b'\r' | b'\x0c')
}

fn ascii_lower(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_ascii_lowercase()
}

fn find(hay: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
    if from > hay.len() {
        return None;
    }
    hay[from..]
        .windows(needle.len())
        .position(|w| w == needle)
        .map(|i| i + from)
}

fn convert_newlines(s: &str) -> String {
    if !s.contains('\r') {
        return s.to_owned();
    }
    s.replace("\r\n", "\n").replace('\r', "\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<(TokenKind, String)> {
        tokenize(src.as_bytes())
            .into_iter()
            .map(|t| {
                (
                    t.kind,
                    if t.kind == TokenKind::Text {
                        t.data
                    } else {
                        t.name
                    },
                )
            })
            .collect()
    }

    fn assert_tiles(src: &[u8]) {
        let tokens = tokenize(src);
        let mut pos = 0;
        for t in &tokens {
            assert_eq!(t.raw.start, pos, "gap before {t:?}");
            pos = t.raw.end;
        }
        assert_eq!(pos, src.len());
    }

    #[test]
    fn ranges_tile_the_input() {
        for src in [
            "",
            "plain",
            "<p>a</p>",
            "<pre><code class=\"x\">a &lt; b</code></pre>",
            "<!DOCTYPE html><html><head><title>t</title><script>if (a<b) {}</script></head></html>",
            "<!-- c --><br/><img src=x><a href='y'>z",
            "<div",
            "<div class=\"a",
            "</",
            "</>",
            "<?xml ?><![CDATA[x]]><!bogus>",
            "a < b > c",
            "<script>never closed",
        ] {
            assert_tiles(src.as_bytes());
        }
    }

    #[test]
    fn tags_and_text() {
        assert_eq!(
            kinds("<P Class=a>x<br/>y</P>"),
            vec![
                (TokenKind::Start, "p".into()),
                (TokenKind::Text, "x".into()),
                (TokenKind::SelfClosing, "br".into()),
                (TokenKind::Text, "y".into()),
                (TokenKind::End, "p".into()),
            ]
        );
    }

    #[test]
    fn attributes_are_decoded_and_lowercased() {
        let t = tokenize(
            br#"<code CLASS="language-go" data-kz-meta="go title=&#34;m.go&#34;" x=y z 'q'=1>"#,
        );
        assert_eq!(
            t[0].attrs[0],
            Attr {
                key: "class".into(),
                value: "language-go".into()
            }
        );
        assert_eq!(t[0].attrs[1].value, "go title=\"m.go\"");
        assert_eq!(
            t[0].attrs[2],
            Attr {
                key: "x".into(),
                value: "y".into()
            }
        );
        assert_eq!(
            t[0].attrs[3],
            Attr {
                key: "z".into(),
                value: String::new()
            }
        );
        assert_eq!(
            t[0].attrs[4],
            Attr {
                key: "'q'".into(),
                value: "1".into()
            }
        );
    }

    #[test]
    fn text_is_decoded_and_newlines_converted() {
        let t = tokenize(b"a &amp;&nbsp;b\r\nc\rd");
        assert_eq!(t[0].data, "a &\u{a0}b\nc\nd");
        assert_eq!(t[0].raw, 0..19);
    }

    #[test]
    fn bare_lt_is_text() {
        assert_eq!(kinds("a < b"), vec![(TokenKind::Text, "a < b".into())]);
    }

    #[test]
    fn raw_text_elements() {
        let t = tokenize(
            b"<script>if (a<b) </scriptx> x</script ><style>a>b</style><title>&amp;</title>",
        );
        assert_eq!(t[1].kind, TokenKind::Text);
        assert_eq!(t[1].data, "if (a<b) </scriptx> x");
        assert_eq!(t[2].kind, TokenKind::End);
        assert_eq!(t[4].data, "a>b");
        assert_eq!(t[7].data, "&");
    }

    #[test]
    fn comments_and_doctype() {
        assert_eq!(
            kinds("<!DOCTYPE html><!-- a > b --><!bogus><?pi?></>"),
            vec![
                (TokenKind::Doctype, String::new()),
                (TokenKind::Comment, String::new()),
                (TokenKind::Comment, String::new()),
                (TokenKind::Comment, String::new()),
                (TokenKind::Comment, String::new()),
            ]
        );
    }

    #[test]
    fn bom_is_text() {
        let t = tokenize("\u{feff}<p>x</p>".as_bytes());
        assert_eq!(t[0].kind, TokenKind::Text);
        assert_eq!(t[0].raw, 0..3);
        assert_eq!(t[1].raw.start, 3);
    }
}
