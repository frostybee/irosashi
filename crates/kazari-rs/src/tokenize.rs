use crate::error::Error;
use crate::highlighter::{FontStyle, Highlighted, Highlighter, Token};
use crate::types::Themes;

pub struct Tokens(Highlighted);

impl Tokens {
    pub fn is_dual(&self) -> bool {
        self.0.dark.is_some()
    }

    pub fn line_count(&self) -> usize {
        self.0.lines.len()
    }

    pub fn line_text(&self, line_idx: usize) -> &str {
        &self.0.lines[line_idx].text
    }

    pub fn tokens(&self, line_idx: usize) -> &[Token] {
        &self.0.lines[line_idx].tokens
    }

    pub fn light_fg(&self) -> &str {
        &self.0.light.fg
    }

    pub fn light_bg(&self) -> &str {
        &self.0.light.bg
    }
}

pub fn tokenize(
    hl: &dyn Highlighter,
    code: &str,
    lang: &str,
    themes: &Themes,
) -> Result<Tokens, Error> {
    hl.tokenize(code, lang, &themes.light, themes.dark.as_deref())
        .map(Tokens)
}

pub fn expand_tabs(code: &str, tab_width: usize) -> String {
    if !code.contains('\t') {
        return code.to_owned();
    }
    let spaces = " ".repeat(tab_width);
    code.replace('\t', &spaces)
}

#[allow(dead_code)]
pub fn format_font_style(fs: FontStyle) -> (&'static str, &'static str, &'static str) {
    let font_style = if fs.contains(FontStyle::ITALIC) {
        "italic"
    } else {
        ""
    };
    let font_weight = if fs.contains(FontStyle::BOLD) {
        "bold"
    } else {
        ""
    };
    let text_decoration = match (
        fs.contains(FontStyle::UNDERLINE),
        fs.contains(FontStyle::STRIKETHROUGH),
    ) {
        (true, true) => "underline line-through",
        (true, false) => "underline",
        (false, true) => "line-through",
        (false, false) => "",
    };
    (font_style, font_weight, text_decoration)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_tabs_replaces_with_spaces() {
        assert_eq!(expand_tabs("no tabs", 4), "no tabs");
        assert_eq!(expand_tabs("\tfoo\tbar", 4), "    foo    bar");
        assert_eq!(expand_tabs("\t", 2), "  ");
    }

    #[test]
    fn expand_tabs_no_tab_returns_same() {
        let code = "just text";
        let result = expand_tabs(code, 4);
        assert_eq!(result, code);
    }

    #[test]
    fn format_font_style_combinations() {
        assert_eq!(format_font_style(FontStyle::empty()), ("", "", ""));
        assert_eq!(format_font_style(FontStyle::ITALIC), ("italic", "", ""));
        assert_eq!(
            format_font_style(FontStyle::BOLD | FontStyle::UNDERLINE),
            ("", "bold", "underline")
        );
        assert_eq!(
            format_font_style(FontStyle::UNDERLINE | FontStyle::STRIKETHROUGH),
            ("", "", "underline line-through")
        );
    }
}
