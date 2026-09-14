use crate::render::html::Dialect;
use crate::theme::FontStyle;
use crate::token::{ThemedToken, TokensResult};

pub(crate) type Props = Vec<(String, String)>;

pub(crate) fn font_props(font_style: FontStyle, out: &mut Props) {
    if font_style.is_italic() {
        out.push(("font-style".to_owned(), "italic".to_owned()));
    }
    if font_style.is_bold() {
        out.push(("font-weight".to_owned(), "bold".to_owned()));
    }
    let mut decorations = Vec::new();
    if font_style.is_underline() {
        decorations.push("underline");
    }
    if font_style.is_strikethrough() {
        decorations.push("line-through");
    }
    if !decorations.is_empty() {
        out.push(("text-decoration".to_owned(), decorations.join(" ")));
    }
}

/// The declarations one theme slot contributes for a token, in the order the dialect
/// lists them before any sorting.
pub(crate) fn slot_props(
    result: &TokensResult,
    token: &ThemedToken,
    slot: usize,
    dialect: Dialect,
) -> Props {
    let style = result.style_in(token, slot);
    let mut props = Props::new();
    if let Some(color) = style.color {
        let color = result.color_in(slot, color);
        let color = match dialect {
            Dialect::Iro => color.to_owned(),
            Dialect::Shiki => color.to_ascii_uppercase(),
        };
        props.push(("color".to_owned(), color));
    }
    if dialect == Dialect::Iro
        && let Some(bg) = style.bg
    {
        props.push((
            "background-color".to_owned(),
            result.color_in(slot, bg).to_owned(),
        ));
    }
    font_props(style.font_style, &mut props);
    props
}

pub(crate) fn var_name(prefix: &str, key: &str, prop: &str) -> String {
    match prop {
        "color" => format!("{prefix}{key}"),
        "background-color" => format!("{prefix}{key}-bg"),
        other => format!("{prefix}{key}-{other}"),
    }
}

pub(crate) fn sort_props(props: &mut Props) {
    props.sort_by(|a, b| a.0.cmp(&b.0));
}

pub(crate) fn join_props(props: &Props) -> String {
    let mut out = String::new();
    for (i, (key, value)) in props.iter().enumerate() {
        if i > 0 {
            out.push(';');
        }
        out.push_str(key);
        out.push(':');
        out.push_str(value);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_props_follow_css_names_with_underline_first() {
        let mut props = Props::new();
        font_props(FontStyle::all(), &mut props);
        assert_eq!(
            join_props(&props),
            "font-style:italic;font-weight:bold;text-decoration:underline line-through"
        );
        let mut none = Props::new();
        font_props(FontStyle::empty(), &mut none);
        assert!(none.is_empty());
    }

    #[test]
    fn var_names() {
        assert_eq!(var_name("--iro-", "dark", "color"), "--iro-dark");
        assert_eq!(
            var_name("--iro-", "dark", "background-color"),
            "--iro-dark-bg"
        );
        assert_eq!(
            var_name("--iro-", "dark", "font-style"),
            "--iro-dark-font-style"
        );
    }
}
