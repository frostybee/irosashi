use std::collections::HashMap;

use crate::scope::ScopeListId;
use crate::theme::{ColorId, FontStyle};
use crate::token::{ThemedToken, TokenStyle, TokensResult};

/// The token type vscode-textmate encodes from scope names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum TokenType {
    #[default]
    Other,
    Comment,
    String,
    Regex,
}

/// The innermost scope that names one of the standard types decides; `meta.embedded`
/// resets to `Other`.
pub(crate) fn token_type(scopes: &[impl AsRef<str>]) -> TokenType {
    scopes
        .iter()
        .rev()
        .find_map(|scope| standard_type(scope.as_ref()))
        .unwrap_or_default()
}

fn standard_type(scope: &str) -> Option<TokenType> {
    const WORDS: [(&str, TokenType); 4] = [
        ("comment", TokenType::Comment),
        ("string", TokenType::String),
        ("regex", TokenType::Regex),
        ("meta.embedded", TokenType::Other),
    ];
    let bytes = scope.as_bytes();
    let is_word = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    for start in 0..bytes.len() {
        if !is_word(bytes[start]) || (start > 0 && is_word(bytes[start - 1])) {
            continue;
        }
        for (word, kind) in WORDS {
            let end = start + word.len();
            if bytes[start..].starts_with(word.as_bytes())
                && bytes.get(end).is_none_or(|&b| !is_word(b))
            {
                return Some(kind);
            }
        }
    }
    None
}

#[derive(PartialEq, Eq)]
struct Metadata {
    kind: TokenType,
    styles: Vec<(ColorId, ColorId, FontStyle)>,
}

/// Merges adjacent tokens whose encoded attributes agree in every theme slot, as
/// `tokenizeLine2` does. Needs the result's scope table for the token type; without
/// it every token counts as `Other`.
pub(crate) fn merge_same_metadata(
    result: &TokensResult,
    tokens: &[ThemedToken],
    slots: &[usize],
    types: &mut HashMap<ScopeListId, TokenType>,
) -> Vec<ThemedToken> {
    let mut metadata = |token: &ThemedToken| {
        let kind = *types
            .entry(token.scopes)
            .or_insert_with(|| result.scopes_of(token).map_or(TokenType::Other, token_type));
        let styles = slots
            .iter()
            .map(|&slot| {
                let theme = &result.themes[slot].theme;
                let style = result.style_in(token, slot);
                (
                    style.color.unwrap_or(theme.default_foreground_id()),
                    style.bg.unwrap_or(theme.default_background_id()),
                    style.font_style,
                )
            })
            .collect();
        Metadata { kind, styles }
    };
    let mut out: Vec<ThemedToken> = Vec::with_capacity(tokens.len());
    let mut last = None;
    for token in tokens {
        let current = metadata(token);
        match (&last, out.last_mut()) {
            (Some(previous), Some(merged)) if *previous == current => merged.end = token.end,
            _ => {
                out.push(*token);
                last = Some(current);
            }
        }
    }
    out
}

/// Folds whitespace-only tokens into the token that follows them, which keeps its own
/// style. A run that precedes an underlined or struck-through token is emitted on its
/// own without a style, and a run at the end of the line is left alone.
pub(crate) fn merge_whitespace(
    text: &str,
    tokens: &[ThemedToken],
    check_decoration: bool,
) -> Vec<ThemedToken> {
    let mut out = Vec::with_capacity(tokens.len());
    let mut carry_start = None;
    for (i, token) in tokens.iter().enumerate() {
        let decorated = check_decoration
            && token
                .style
                .font_style
                .intersects(FontStyle::UNDERLINE | FontStyle::STRIKETHROUGH);
        let could_merge = !decorated;
        if could_merge && i + 1 < tokens.len() && is_js_whitespace_only(token.text(text)) {
            carry_start.get_or_insert(token.start);
            continue;
        }
        match carry_start.take() {
            Some(start) if could_merge => out.push(ThemedToken { start, ..*token }),
            Some(start) => {
                out.push(ThemedToken {
                    start,
                    end: token.start,
                    style: TokenStyle::default(),
                    scopes: ScopeListId::EMPTY,
                });
                out.push(*token);
            }
            None => out.push(*token),
        }
    }
    out
}

/// JavaScript's `\s`, which is not Rust's `char::is_whitespace`.
fn is_js_whitespace_only(text: &str) -> bool {
    !text.is_empty()
        && text.chars().all(|c| {
            matches!(
                c,
                '\t' | '\n' | '\u{0B}' | '\u{0C}' | '\r' | ' ' | '\u{A0}' | '\u{1680}' | '\u{2000}'
                    ..='\u{200A}'
                        | '\u{2028}'
                        | '\u{2029}'
                        | '\u{202F}'
                        | '\u{205F}'
                        | '\u{3000}'
                        | '\u{FEFF}'
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ColorId;

    fn tok(start: usize, end: usize, font_style: FontStyle) -> ThemedToken {
        ThemedToken {
            start,
            end,
            style: TokenStyle {
                color: Some(ColorId(1)),
                bg: None,
                font_style,
            },
            scopes: ScopeListId(7),
        }
    }

    #[test]
    fn indent_merges_into_the_next_token_with_its_style() {
        let text = "  \tab  ";
        let tokens = [
            tok(0, 2, FontStyle::empty()),
            tok(2, 3, FontStyle::empty()),
            tok(3, 5, FontStyle::BOLD),
            tok(5, 7, FontStyle::empty()),
        ];
        let merged = merge_whitespace(text, &tokens, true);
        assert_eq!(
            merged,
            [tok(0, 5, FontStyle::BOLD), tok(5, 7, FontStyle::empty())]
        );
    }

    #[test]
    fn decorated_next_token_gets_a_bare_run_before_it() {
        let text = " a";
        let tokens = [
            tok(0, 1, FontStyle::empty()),
            tok(1, 2, FontStyle::UNDERLINE),
        ];
        let merged = merge_whitespace(text, &tokens, true);
        assert_eq!(merged.len(), 2);
        assert_eq!((merged[0].start, merged[0].end), (0, 1));
        assert_eq!(merged[0].style, TokenStyle::default());
        assert_eq!(merged[0].scopes, ScopeListId::EMPTY);
        assert_eq!(merged[1], tokens[1]);
        let ignored = merge_whitespace(text, &tokens, false);
        assert_eq!(ignored, [tok(0, 2, FontStyle::UNDERLINE)]);
    }

    #[test]
    fn decorated_whitespace_is_not_merged() {
        let text = " a";
        let tokens = [
            tok(0, 1, FontStyle::STRIKETHROUGH),
            tok(1, 2, FontStyle::empty()),
        ];
        assert_eq!(merge_whitespace(text, &tokens, true), tokens);
    }

    #[test]
    fn standard_token_types_follow_the_innermost_match() {
        assert_eq!(
            token_type(&["source.js", "string.quoted.js"]),
            TokenType::String
        );
        assert_eq!(
            token_type(&[
                "source.js",
                "string.quoted.js",
                "punctuation.definition.string.begin.js"
            ]),
            TokenType::String
        );
        assert_eq!(
            token_type(&["source.js", "comment.block.js", "storage.type.class.jsdoc"]),
            TokenType::Comment
        );
        assert_eq!(
            token_type(&["source.js", "string.regexp.js"]),
            TokenType::String
        );
        assert_eq!(
            token_type(&["source.js", "keyword.other.regex"]),
            TokenType::Regex
        );
        assert_eq!(
            token_type(&[
                "text.html",
                "string.x",
                "meta.embedded.block.js",
                "keyword.js"
            ]),
            TokenType::Other
        );
        assert_eq!(
            token_type(&["source.js", "strings.x", "commentary.y"]),
            TokenType::Other
        );
        assert_eq!(token_type(&["a_string.x"]), TokenType::Other);
        assert_eq!(token_type(&[] as &[&str]), TokenType::Other);
    }

    #[test]
    fn js_whitespace_set() {
        assert!(is_js_whitespace_only("\u{FEFF} \u{3000}"));
        assert!(!is_js_whitespace_only("\u{85}"));
        assert!(!is_js_whitespace_only(""));
        assert!(!is_js_whitespace_only(" x"));
    }
}
