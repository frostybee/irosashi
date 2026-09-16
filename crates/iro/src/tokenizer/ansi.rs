use std::ops::Range;

use crate::theme::FontStyle;

/// The 16 standard colours as VS Code's terminal draws them.
pub const ANSI_STANDARD_COLORS: [&str; 16] = [
    "#000000", "#cd3131", "#0dbc79", "#e5e510", "#2472c8", "#bc3fbc", "#11a8cd", "#e5e5e5",
    "#666666", "#f14c4c", "#23d18b", "#f5f543", "#3b8eea", "#d670d6", "#29b8db", "#e5e5e5",
];

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub(crate) struct AnsiStyle {
    pub fg: Option<String>,
    pub bg: Option<String>,
    pub font_style: FontStyle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AnsiToken {
    /// Byte range within the line's text.
    pub range: Range<usize>,
    pub style: AnsiStyle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AnsiLine {
    /// Byte range within the cleaned text.
    pub range: Range<usize>,
    pub tokens: Vec<AnsiToken>,
}

struct Scanner<'a> {
    code: &'a str,
    cleaned: String,
    lines: Vec<AnsiLine>,
    current: Vec<AnsiToken>,
    line_start: usize,
    token_start: usize,
    style: AnsiStyle,
}

impl Scanner<'_> {
    fn push_run(&mut self, range: Range<usize>) {
        self.cleaned.push_str(&self.code[range]);
    }

    fn flush(&mut self) {
        if self.cleaned.len() > self.token_start {
            self.current.push(AnsiToken {
                range: (self.token_start - self.line_start)..(self.cleaned.len() - self.line_start),
                style: self.style.clone(),
            });
        }
        self.token_start = self.cleaned.len();
    }

    fn end_line(&mut self) {
        self.flush();
        self.lines.push(AnsiLine {
            range: self.line_start..self.cleaned.len(),
            tokens: std::mem::take(&mut self.current),
        });
    }
}

/// Splits `code` into lines of tokens by its SGR escape sequences (`ESC [ ... m`).
/// Returns the text with the sequences removed and, per line, byte ranges into it.
/// Style carries across lines; empty and escape-only input yield no lines.
pub(crate) fn tokenize_ansi(code: &str) -> (String, Vec<AnsiLine>) {
    let bytes = code.as_bytes();
    let mut s = Scanner {
        code,
        cleaned: String::with_capacity(code.len()),
        lines: Vec::new(),
        current: Vec::new(),
        line_start: 0,
        token_start: 0,
        style: AnsiStyle::default(),
    };
    let mut run_start = 0;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\n' => {
                s.push_run(run_start..i);
                s.end_line();
                s.cleaned.push('\n');
                s.line_start = s.cleaned.len();
                s.token_start = s.line_start;
                i += 1;
                run_start = i;
            }
            0x1b if bytes.get(i + 1) == Some(&b'[') => {
                s.push_run(run_start..i);
                s.flush();
                match code[i + 2..].find('m') {
                    Some(end) => {
                        apply_sgr(&code[i + 2..i + 2 + end], &mut s.style);
                        i += 2 + end + 1;
                    }
                    None => i += 1,
                }
                run_start = i;
            }
            _ => i += 1,
        }
    }
    s.push_run(run_start..bytes.len());
    s.flush();
    if !s.current.is_empty() || !s.lines.is_empty() {
        s.end_line();
    }
    (s.cleaned, s.lines)
}

fn color256(index: u32) -> Option<String> {
    let index = index as usize;
    if index < 16 {
        Some(ANSI_STANDARD_COLORS[index].to_owned())
    } else if index < 232 {
        let n = index - 16;
        Some(format!(
            "#{:02x}{:02x}{:02x}",
            n / 36 * 51,
            (n % 36) / 6 * 51,
            (n % 6) * 51
        ))
    } else if index < 256 {
        let v = (index - 232) * 10 + 8;
        Some(format!("#{v:02x}{v:02x}{v:02x}"))
    } else {
        None
    }
}

fn apply_sgr(params: &str, style: &mut AnsiStyle) {
    if params.is_empty() {
        *style = AnsiStyle::default();
        return;
    }
    let parts: Vec<&str> = params.split(';').collect();
    let number = |i: usize| parts.get(i).and_then(|p| p.parse::<u32>().ok());
    let mut i = 0;
    while i < parts.len() {
        let Some(code) = number(i) else {
            i += 1;
            continue;
        };
        match code {
            0 => *style = AnsiStyle::default(),
            1 => style.font_style |= FontStyle::BOLD,
            3 => style.font_style |= FontStyle::ITALIC,
            4 => style.font_style |= FontStyle::UNDERLINE,
            9 => style.font_style |= FontStyle::STRIKETHROUGH,
            22 => style.font_style.remove(FontStyle::BOLD),
            23 => style.font_style.remove(FontStyle::ITALIC),
            24 => style.font_style.remove(FontStyle::UNDERLINE),
            29 => style.font_style.remove(FontStyle::STRIKETHROUGH),
            30..=37 => style.fg = Some(ANSI_STANDARD_COLORS[(code - 30) as usize].to_owned()),
            39 => style.fg = None,
            40..=47 => style.bg = Some(ANSI_STANDARD_COLORS[(code - 40) as usize].to_owned()),
            49 => style.bg = None,
            90..=97 => style.fg = Some(ANSI_STANDARD_COLORS[(code - 90 + 8) as usize].to_owned()),
            100..=107 => {
                style.bg = Some(ANSI_STANDARD_COLORS[(code - 100 + 8) as usize].to_owned())
            }
            38 | 48 => {
                let value = match number(i + 1) {
                    Some(5) => {
                        let color = number(i + 2).and_then(color256);
                        i += 2;
                        color
                    }
                    Some(2) => {
                        let channel = |k: usize| number(i + k).unwrap_or(0).min(255);
                        let color =
                            format!("#{:02x}{:02x}{:02x}", channel(2), channel(3), channel(4));
                        i += 4;
                        Some(color)
                    }
                    _ => {
                        i += 1;
                        continue;
                    }
                };
                if code == 38 {
                    style.fg = value;
                } else {
                    style.bg = value;
                }
            }
            _ => {}
        }
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(code: &str) -> (String, Vec<Vec<(String, AnsiStyle)>>) {
        let (cleaned, lines) = tokenize_ansi(code);
        let view = lines
            .iter()
            .map(|line| {
                let text = &cleaned[line.range.clone()];
                line.tokens
                    .iter()
                    .map(|t| (text[t.range.clone()].to_owned(), t.style.clone()))
                    .collect()
            })
            .collect();
        (cleaned, view)
    }

    fn fg(hex: &str) -> AnsiStyle {
        AnsiStyle {
            fg: Some(hex.to_owned()),
            ..Default::default()
        }
    }

    #[test]
    fn colors_and_styles() {
        let (cleaned, lines) = tokens("\x1b[31mred\x1b[0m");
        assert_eq!(cleaned, "red");
        assert_eq!(lines, [vec![("red".to_owned(), fg("#cd3131"))]]);

        let (_, lines) = tokens("\x1b[1;32mbold green\x1b[0m");
        let mut expected = fg("#0dbc79");
        expected.font_style = FontStyle::BOLD;
        assert_eq!(lines[0][0], ("bold green".to_owned(), expected));

        assert_eq!(tokens("\x1b[91mx").1[0][0].1, fg("#f14c4c"));
        assert_eq!(
            tokens("\x1b[41mx").1[0][0].1,
            AnsiStyle {
                bg: Some("#cd3131".to_owned()),
                ..Default::default()
            }
        );
        assert_eq!(tokens("\x1b[38;5;196mx").1[0][0].1, fg("#ff0000"));
        assert_eq!(tokens("\x1b[38;5;240mx").1[0][0].1, fg("#585858"));
        assert_eq!(tokens("\x1b[38;2;255;128;0mx").1[0][0].1, fg("#ff8000"));
        assert_eq!(
            tokens("\x1b[48;2;0;128;255mx").1[0][0].1.bg.as_deref(),
            Some("#0080ff")
        );
        assert_eq!(tokens("\x1b[38;2;999;0;0mx").1[0][0].1, fg("#ff0000"));
        assert_eq!(
            tokens("\x1b[1;22;4mx").1[0][0].1.font_style,
            FontStyle::UNDERLINE
        );
        assert_eq!(tokens("\x1b[31;39mx").1[0][0].1, AnsiStyle::default());
        assert_eq!(tokens("\x1b[mx").1[0][0].1, AnsiStyle::default());
        assert_eq!(tokens("\x1b[38;5;300mx").1[0][0].1, AnsiStyle::default());
    }

    #[test]
    fn state_and_line_handling() {
        let (cleaned, lines) = tokens("\x1b[31mred\nstill red");
        assert_eq!(cleaned, "red\nstill red");
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0][0].1, fg("#cd3131"));
        assert_eq!(lines[1][0].1, fg("#cd3131"));

        let (_, lines) = tokens("\x1b[31mred\x1b[0mnormal");
        assert_eq!(lines[0].len(), 2);
        assert_eq!(lines[0][1], ("normal".to_owned(), AnsiStyle::default()));

        let (_, lines) = tokens("hello \x1b[32mworld\x1b[0m!");
        assert_eq!(
            lines[0].iter().map(|(t, _)| t.as_str()).collect::<Vec<_>>(),
            ["hello ", "world", "!"]
        );

        let (cleaned, lines) = tokens("\x1b[999mtext");
        assert_eq!(cleaned, "text");
        assert_eq!(lines[0], [("text".to_owned(), AnsiStyle::default())]);

        assert_eq!(tokens("").1.len(), 0);
        assert_eq!(tokens("\x1b[31m\x1b[0m").1.len(), 0);
        assert_eq!(tokens("line1\nline2\nline3").1.len(), 3);
        assert_eq!(tokens("a\n").1.len(), 2);

        // Without a closing `m` the ESC byte is dropped and the rest is text.
        let (cleaned, lines) = tokens("a\x1b[31 no end");
        assert_eq!(cleaned, "a[31 no end");
        assert_eq!(lines[0].len(), 2);
        // The next `m` anywhere closes the sequence, as in Nuri.
        assert_eq!(tokens("a\x1b[31 no m").0, "a");
    }
}
