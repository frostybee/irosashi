use super::classlist::has_class;
use crate::html::{Token, TokenKind};

/// The 1-based numbers of lines whose Chroma line wrapper also carries the hl class. The
/// counter gates on the `line` class because Chroma always pairs hl with line on the same
/// span. Hugo's default inline styles mode carries no classes, so nothing is recoverable
/// there and the result is empty.
pub fn collect_highlighted_lines(tokens: &[Token]) -> Vec<usize> {
    let mut line = 0;
    let mut highlighted = Vec::new();
    for tok in tokens {
        if tok.kind != TokenKind::Start || !has_class(&tok.attrs, "line") {
            continue;
        }
        line += 1;
        if has_class(&tok.attrs, "hl") {
            highlighted.push(line);
        }
    }
    highlighted
}

/// Ascending line numbers as the brace range syntax the meta grammar parses, with
/// consecutive runs collapsed: 3,4,5,9 becomes `3-5,9`.
pub fn collapse_ranges(lines: &[usize]) -> String {
    let Some(&first) = lines.first() else {
        return String::new();
    };
    let mut parts = Vec::new();
    let (mut start, mut prev) = (first, first);
    let flush = |parts: &mut Vec<String>, start: usize, prev: usize| {
        if start == prev {
            parts.push(start.to_string());
        } else {
            parts.push(format!("{start}-{prev}"));
        }
    };
    for &n in &lines[1..] {
        if n == prev + 1 {
            prev = n;
            continue;
        }
        flush(&mut parts, start, prev);
        start = n;
        prev = n;
    }
    flush(&mut parts, start, prev);
    parts.join(",")
}

/// The full meta string: the language token first, then a bare brace range token when
/// any lines are highlighted.
pub fn build_meta(lang: &str, highlighted: &[usize]) -> String {
    let ranges = collapse_ranges(highlighted);
    match (lang.is_empty(), ranges.is_empty()) {
        (true, true) => String::new(),
        (false, true) => lang.to_owned(),
        (true, false) => format!("{{{ranges}}}"),
        (false, false) => format!("{lang} {{{ranges}}}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html::tokenize;

    #[test]
    fn ranges_collapse() {
        assert_eq!(collapse_ranges(&[]), "");
        assert_eq!(collapse_ranges(&[1]), "1");
        assert_eq!(collapse_ranges(&[3, 4, 5, 9]), "3-5,9");
        assert_eq!(collapse_ranges(&[1, 3, 4]), "1,3-4");
    }

    #[test]
    fn meta_shapes() {
        assert_eq!(build_meta("", &[]), "");
        assert_eq!(build_meta("go", &[]), "go");
        assert_eq!(build_meta("", &[2]), "{2}");
        assert_eq!(build_meta("go", &[2, 3]), "go {2-3}");
    }

    #[test]
    fn counts_line_wrappers() {
        let t = tokenize(
            b"<span class=\"line\"><span class=\"cl\">a</span></span><span class=\"line hl\"><span class=\"cl\">b</span></span><span class=\"hl\">not a line</span><span class=\"line hl\">c</span>",
        );
        assert_eq!(collect_highlighted_lines(&t), vec![2, 3]);
    }
}
