use iro::{ColorId, FontStyle, TokenStyle, TokensResult};

use crate::error::Error;
use crate::types::Themes;

pub struct Tokens {
    result: TokensResult,
    light_slot: usize,
    dark_slot: Option<usize>,
}

#[allow(dead_code)]
impl Tokens {
    pub fn result(&self) -> &TokensResult {
        &self.result
    }

    pub fn is_dual(&self) -> bool {
        self.dark_slot.is_some()
    }

    pub fn line_count(&self) -> usize {
        self.result.lines.len()
    }

    pub fn line_text(&self, line_idx: usize) -> &str {
        self.result.line_text(&self.result.lines[line_idx])
    }

    pub fn tokens(&self, line_idx: usize) -> &[iro::ThemedToken] {
        &self.result.lines[line_idx].tokens
    }

    pub fn token_text(&self, line_idx: usize, token: &iro::ThemedToken) -> &str {
        let line = self.line_text(line_idx);
        token.text(line)
    }

    pub fn light_style(&self, token: &iro::ThemedToken) -> TokenStyle {
        self.result.style_in(token, self.light_slot)
    }

    pub fn dark_style(&self, token: &iro::ThemedToken) -> Option<TokenStyle> {
        self.dark_slot.map(|slot| self.result.style_in(token, slot))
    }

    pub fn light_color(&self, id: ColorId) -> &str {
        self.result.color_in(self.light_slot, id)
    }

    pub fn dark_color(&self, id: ColorId) -> Option<&str> {
        self.dark_slot.map(|slot| self.result.color_in(slot, id))
    }

    pub fn light_fg(&self) -> &str {
        self.result.fg_of(self.light_slot)
    }

    pub fn light_bg(&self) -> &str {
        self.result.bg_of(self.light_slot)
    }

    pub fn dark_fg(&self) -> Option<&str> {
        self.dark_slot.map(|slot| self.result.fg_of(slot))
    }

    pub fn dark_bg(&self) -> Option<&str> {
        self.dark_slot.map(|slot| self.result.bg_of(slot))
    }
}

pub fn tokenize(
    hl: &iro::Highlighter,
    code: &str,
    lang: &str,
    themes: &Themes,
) -> Result<Tokens, Error> {
    let opts = iro::CodeToTokensOptions::new(lang, &themes.light);

    let (result, light_slot, dark_slot) = if themes.dark.is_some() {
        let slot_keys = themes.to_slot_keys();
        let result = hl.code_to_tokens_multi(code, &slot_keys, &opts)?;
        let light_idx = find_slot(&result, "light");
        let dark_idx = find_slot(&result, "dark");
        (result, light_idx, Some(dark_idx))
    } else {
        let result = hl.code_to_tokens(code, &opts)?;
        (result, 0, None)
    };

    Ok(Tokens {
        result,
        light_slot,
        dark_slot,
    })
}

fn find_slot(result: &TokensResult, key: &str) -> usize {
    result.themes.iter().position(|s| s.key == key).unwrap_or(0)
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
    fn themes_slot_keys_single() {
        let t = Themes::single("github-dark");
        let keys = t.to_slot_keys();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys["light"], "github-dark");
    }

    #[test]
    fn themes_slot_keys_dual() {
        let t = Themes::dual("github-light", "github-dark");
        let keys = t.to_slot_keys();
        assert_eq!(keys.len(), 2);
        assert_eq!(keys["dark"], "github-dark");
        assert_eq!(keys["light"], "github-light");
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
