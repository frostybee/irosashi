use std::fmt;

use crate::fixture::FixtureToken;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiffKind {
    /// Different start or end offsets; aborts comparison of the rest of the line.
    BoundaryMismatch,
    /// Same span, different scopes; the style is not compared for this token.
    ScopeMismatch,
    /// Same span and scopes, different color or font style.
    StyleMismatch,
    /// Present in the expected output, absent in the actual one.
    MissingToken,
    /// Present in the actual output, absent in the expected one.
    ExtraToken,
}

impl fmt::Display for DiffKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::BoundaryMismatch => "boundary",
            Self::ScopeMismatch => "scope",
            Self::StyleMismatch => "style",
            Self::MissingToken => "missing",
            Self::ExtraToken => "extra",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenDiff {
    pub line: usize,
    pub kind: DiffKind,
    pub want: Option<FixtureToken>,
    pub got: Option<FixtureToken>,
    pub detail: String,
}

impl fmt::Display for TokenDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}: {}", self.line, self.kind, self.detail)?;
        if let Some(want) = &self.want {
            write!(
                f,
                "\n  want {:?} {:?} {} {}",
                want.text, want.scopes, want.color, want.font_style
            )?;
        }
        if let Some(got) = &self.got {
            write!(
                f,
                "\n  got  {:?} {:?} {} {}",
                got.text, got.scopes, got.color, got.font_style
            )?;
        }
        Ok(())
    }
}

/// Compares two token matrices line by line in lockstep.
pub fn compare_theme_tokens(
    want: &[Vec<FixtureToken>],
    got: &[Vec<FixtureToken>],
) -> Vec<TokenDiff> {
    let mut diffs = Vec::new();
    for line in 0..want.len().max(got.len()) {
        match (want.get(line), got.get(line)) {
            (Some(w), Some(g)) => diffs.extend(compare_line(line, w, g)),
            (Some(_), None) => diffs.push(TokenDiff {
                line,
                kind: DiffKind::MissingToken,
                want: None,
                got: None,
                detail: format!("missing line {line} in actual output"),
            }),
            (None, Some(_)) => diffs.push(TokenDiff {
                line,
                kind: DiffKind::ExtraToken,
                want: None,
                got: None,
                detail: format!("extra line {line} in actual output"),
            }),
            (None, None) => {}
        }
    }
    diffs
}

fn compare_line(line: usize, want: &[FixtureToken], got: &[FixtureToken]) -> Vec<TokenDiff> {
    let mut diffs = Vec::new();
    let mut i = 0;
    while i < want.len() && i < got.len() {
        let (w, g) = (&want[i], &got[i]);
        if w.start != g.start || w.end != g.end {
            diffs.push(TokenDiff {
                line,
                kind: DiffKind::BoundaryMismatch,
                want: Some(w.clone()),
                got: Some(g.clone()),
                detail: format!(
                    "token {i}: boundary [{}:{}] vs [{}:{}]",
                    w.start, w.end, g.start, g.end
                ),
            });
            return diffs;
        }
        if w.scopes != g.scopes {
            diffs.push(TokenDiff {
                line,
                kind: DiffKind::ScopeMismatch,
                want: Some(w.clone()),
                got: Some(g.clone()),
                detail: format!("token {i}: scopes differ"),
            });
            i += 1;
            continue;
        }
        if normalize_color(&w.color) != normalize_color(&g.color) || w.font_style != g.font_style {
            diffs.push(TokenDiff {
                line,
                kind: DiffKind::StyleMismatch,
                want: Some(w.clone()),
                got: Some(g.clone()),
                detail: format!(
                    "token {i}: style differs (color {} vs {}, fontStyle {} vs {})",
                    w.color, g.color, w.font_style, g.font_style
                ),
            });
        }
        i += 1;
    }
    for (j, w) in want.iter().enumerate().skip(i) {
        diffs.push(TokenDiff {
            line,
            kind: DiffKind::MissingToken,
            want: Some(w.clone()),
            got: None,
            detail: format!("missing token {j}"),
        });
    }
    for (j, g) in got.iter().enumerate().skip(i) {
        diffs.push(TokenDiff {
            line,
            kind: DiffKind::ExtraToken,
            want: None,
            got: Some(g.clone()),
            detail: format!("extra token {j}"),
        });
    }
    diffs
}

/// Lowercases and trims a color, expanding `#abc` to `#aabbcc`.
pub fn normalize_color(color: &str) -> String {
    let color = color.trim().to_ascii_lowercase();
    let bytes = color.as_bytes();
    if bytes.len() == 4 && bytes[0] == b'#' {
        let (r, g, b) = (bytes[1] as char, bytes[2] as char, bytes[3] as char);
        return format!("#{r}{r}{g}{g}{b}{b}");
    }
    color
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tok(start: usize, end: usize, scopes: &[&str], color: &str, font_style: u8) -> FixtureToken {
        FixtureToken {
            start,
            end,
            text: String::new(),
            scopes: scopes.iter().map(|s| s.to_string()).collect(),
            color: color.to_owned(),
            font_style,
        }
    }

    #[test]
    fn colors_normalize() {
        assert_eq!(normalize_color(" #ABC "), "#aabbcc");
        assert_eq!(normalize_color("#AABBCC"), "#aabbcc");
        assert_eq!(normalize_color("#abcd"), "#abcd");
    }

    #[test]
    fn boundary_mismatch_aborts_the_line() {
        let want = vec![tok(0, 2, &["a"], "#fff", 0), tok(2, 4, &["b"], "#fff", 0)];
        let got = vec![tok(0, 3, &["a"], "#fff", 0), tok(3, 4, &["zzz"], "#000", 1)];
        let diffs = compare_theme_tokens(&[want], &[got]);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].kind, DiffKind::BoundaryMismatch);
    }

    #[test]
    fn scope_mismatch_suppresses_style_and_continues() {
        let want = vec![tok(0, 2, &["a"], "#fff", 0), tok(2, 4, &["b"], "#fff", 0)];
        let got = vec![tok(0, 2, &["x"], "#000", 0), tok(2, 4, &["b"], "#FFF", 1)];
        let diffs = compare_theme_tokens(&[want], &[got]);
        assert_eq!(
            diffs.iter().map(|d| d.kind).collect::<Vec<_>>(),
            [DiffKind::ScopeMismatch, DiffKind::StyleMismatch]
        );
    }

    #[test]
    fn missing_and_extra_tokens_and_lines() {
        let want = vec![
            vec![tok(0, 2, &["a"], "#fff", 0), tok(2, 4, &["b"], "#fff", 0)],
            vec![],
        ];
        let got = vec![vec![tok(0, 2, &["a"], "#fff", 0)]];
        let diffs = compare_theme_tokens(&want, &got);
        assert_eq!(
            diffs.iter().map(|d| d.kind).collect::<Vec<_>>(),
            [DiffKind::MissingToken, DiffKind::MissingToken]
        );
        let diffs = compare_theme_tokens(&got, &want);
        assert_eq!(
            diffs.iter().map(|d| d.kind).collect::<Vec<_>>(),
            [DiffKind::ExtraToken, DiffKind::ExtraToken]
        );
    }
}
