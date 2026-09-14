use std::collections::{BTreeMap, HashMap};

use crate::render::merge::{merge_same_metadata, merge_whitespace};
use crate::render::style::{Props, join_props, slot_props, sort_props, var_name};
use crate::render::{Node, Renderer, StyleClassMap};
use crate::token::{ThemedToken, TokensResult};

/// Which characters are escaped and how.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum Escape {
    /// Named entities: `& < >` in text, `& "` in attributes.
    #[default]
    Named,
    /// Hexadecimal entities as `hast-util-to-html` emits them: `< &` in text and
    /// `\0 " & ' \`` in attributes.
    Hex,
}

/// The output conventions. Both dialects share the tree shape and differ in class
/// names, declaration order, escaping and multi-theme variables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum Dialect {
    #[default]
    Iro,
    /// Byte-identical to Shiki's `codeToHtml`.
    Shiki,
}

/// Which theme provides the inline colors in multi-theme output.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum DefaultColor {
    /// Slot 0, the lexicographically first key.
    #[default]
    First,
    Key(String),
    /// Every theme is emitted as variables only; nothing is inline.
    Off,
    /// Uses the CSS `light-dark()` function: `color: light-dark(#light, #dark)`.
    /// Requires exactly two theme slots with keys `light` and `dark`. Font-style
    /// properties stay as CSS variables because `light-dark()` only wraps color values.
    LightDark,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlOptions {
    pub dialect: Dialect,
    pub escape: Escape,
    /// The first class on `pre`; also used for `{prefix}-themes`.
    pub class_prefix: String,
    /// Prefix of multi-theme CSS variables, including the leading dashes.
    pub var_prefix: String,
    pub pre_class: Option<String>,
    pub code_class: Option<String>,
    pub pre_attrs: BTreeMap<String, String>,
    pub code_attrs: BTreeMap<String, String>,
    pub tabindex: Option<String>,
    pub default_color: DefaultColor,
    /// Merge adjacent tokens whose color, font style and standard token type agree
    /// in every theme, as vscode-textmate's binary tokenization does. The token type
    /// comes from scope names, so the result needs `include_scopes`;
    /// `Highlighter::code_to_html` sets it.
    pub merge_same_metadata: bool,
    /// Fold whitespace-only tokens into the token that follows them.
    pub merge_whitespace: bool,
    /// Use the multi-theme form even for a single theme slot; `None` uses it only
    /// when the result has more than one slot.
    pub multi_theme: Option<bool>,
}

impl Default for HtmlOptions {
    fn default() -> Self {
        Self {
            dialect: Dialect::Iro,
            escape: Escape::Named,
            class_prefix: "iro".to_owned(),
            var_prefix: "--iro-".to_owned(),
            pre_class: None,
            code_class: None,
            pre_attrs: BTreeMap::new(),
            code_attrs: BTreeMap::new(),
            tabindex: Some("0".to_owned()),
            default_color: DefaultColor::First,
            merge_same_metadata: false,
            merge_whitespace: false,
            multi_theme: None,
        }
    }
}

impl HtmlOptions {
    /// Shiki's `codeToHtml` defaults: `shiki` classes, `--shiki-` variables, the
    /// `light` key as default color, whitespace merging on.
    pub fn shiki() -> Self {
        Self {
            dialect: Dialect::Shiki,
            escape: Escape::Hex,
            class_prefix: "shiki".to_owned(),
            var_prefix: "--shiki-".to_owned(),
            default_color: DefaultColor::Key("light".to_owned()),
            merge_same_metadata: true,
            merge_whitespace: true,
            ..Self::default()
        }
    }
}

/// The batteries-included HTML shape: `pre > code > span.line > span[style]`.
#[derive(Debug, Default)]
pub struct HtmlRenderer<'m> {
    class_map: Option<&'m mut StyleClassMap>,
}

impl<'m> HtmlRenderer<'m> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Emits hashed classes instead of `style` attributes, collecting the rules in `map`.
    pub fn with_class_map(map: &'m mut StyleClassMap) -> Self {
        Self {
            class_map: Some(map),
        }
    }

    pub fn tree(&mut self, result: &TokensResult, options: &HtmlOptions) -> Node {
        let multi = options.multi_theme.unwrap_or(result.themes.len() > 1);
        let order = variant_order(result, options);
        let light_dark = options.default_color == DefaultColor::LightDark;
        let emit_default = options.default_color != DefaultColor::Off;
        let light_slot = light_dark
            .then(|| result.themes.iter().position(|s| s.key == "light"))
            .flatten();
        let dark_slot = light_dark
            .then(|| result.themes.iter().position(|s| s.key == "dark"))
            .flatten();
        let ctx = Ctx {
            result,
            options,
            multi,
            order: &order,
            emit_default,
            light_dark,
            light_slot,
            dark_slot,
        };

        let mut code = Node::element("code");
        if let Some(class) = &options.code_class {
            code = code.attr("class", class);
        }
        for (key, value) in &options.code_attrs {
            code = code.attr(key, value);
        }
        let empty_line = Vec::new();
        let lines: Vec<(&str, &Vec<ThemedToken>)> =
            if result.lines.is_empty() && options.dialect == Dialect::Shiki {
                vec![("", &empty_line)]
            } else {
                result
                    .lines
                    .iter()
                    .map(|line| (result.line_text(line), &line.tokens))
                    .collect()
            };
        let mut types = HashMap::new();
        let mut by_metadata;
        let mut by_whitespace;
        for (index, (text, tokens)) in lines.into_iter().enumerate() {
            if index > 0 {
                code = code.child(Node::text("\n"));
            }
            let tokens: &[ThemedToken] = if options.merge_same_metadata {
                by_metadata = merge_same_metadata(result, tokens, &order, &mut types);
                &by_metadata
            } else {
                tokens
            };
            let tokens: &[ThemedToken] = if options.merge_whitespace {
                by_whitespace = merge_whitespace(text, tokens, !multi);
                &by_whitespace
            } else {
                tokens
            };
            let mut line = Node::element("span").attr("class", "line");
            for token in tokens {
                let props = ctx.token_props(token);
                let node = self.styled(Node::element("span"), &props);
                line = line.child(node.child(Node::text(token.text(text))));
            }
            code = code.child(line);
        }

        let pre_props = ctx.pre_props();
        let mut classes = ctx.pre_classes();
        let mut pre = Node::element("pre");
        let mut style = None;
        if let Some(map) = &mut self.class_map {
            if !pre_props.is_empty() {
                classes.push(map.get(&pre_props).to_owned());
            }
        } else if !pre_props.is_empty() {
            style = Some(join_props(&pre_props));
        }
        if let Some(class) = &options.pre_class {
            classes.push(class.clone());
        }
        pre = pre.attr("class", &classes.join(" "));
        if let Some(style) = style {
            pre = pre.attr("style", &style);
        }
        match options.dialect {
            Dialect::Iro => {
                let mut attrs = BTreeMap::new();
                if let Some(tabindex) = &options.tabindex {
                    attrs.insert("tabindex".to_owned(), tabindex.clone());
                }
                attrs.extend(options.pre_attrs.clone());
                for (key, value) in &attrs {
                    pre = pre.attr(key, value);
                }
            }
            Dialect::Shiki => {
                if let Some(tabindex) = &options.tabindex {
                    pre = pre.attr("tabindex", tabindex);
                }
                for (key, value) in &options.pre_attrs {
                    pre = pre.attr(key, value);
                }
            }
        }
        pre.child(code)
    }

    fn styled(&mut self, node: Node, props: &Props) -> Node {
        if props.is_empty() {
            return node;
        }
        match &mut self.class_map {
            Some(map) => node.attr("class", map.get(props)),
            None => node.attr("style", &join_props(props)),
        }
    }
}

impl Renderer for HtmlRenderer<'_> {
    type Options = HtmlOptions;
    type Output = String;

    fn render(&mut self, result: &TokensResult, options: &HtmlOptions) -> String {
        let mut out = String::new();
        write_node(&mut out, &self.tree(result, options), options.escape);
        out
    }
}

/// Theme slots in emission order: the default color's slot first, then the rest.
/// `LightDark` does not reorder: the `light-dark()` function itself picks values
/// by key, and the slot order stays as the caller provided it.
fn variant_order(result: &TokensResult, options: &HtmlOptions) -> Vec<usize> {
    let mut order: Vec<usize> = (0..result.themes.len()).collect();
    if let DefaultColor::Key(key) = &options.default_color
        && let Some(pos) = result.themes.iter().position(|slot| &slot.key == key)
    {
        order.remove(pos);
        order.insert(0, pos);
    }
    order
}

struct Ctx<'a> {
    result: &'a TokensResult,
    options: &'a HtmlOptions,
    multi: bool,
    order: &'a [usize],
    emit_default: bool,
    light_dark: bool,
    light_slot: Option<usize>,
    dark_slot: Option<usize>,
}

fn light_dark_value(light: &str, dark: &str) -> String {
    format!("light-dark({light}, {dark})")
}

const COLOR_KEYS: [&str; 2] = ["color", "background-color"];

impl Ctx<'_> {
    fn key(&self, slot: usize) -> &str {
        &self.result.themes[slot].key
    }

    fn var(&self, slot: usize, prop: &str) -> String {
        var_name(&self.options.var_prefix, self.key(slot), prop)
    }

    fn token_props(&self, token: &ThemedToken) -> Props {
        let dialect = self.options.dialect;
        if !self.multi {
            let mut props = slot_props(self.result, token, 0, dialect);
            if dialect == Dialect::Iro {
                sort_props(&mut props);
            }
            return props;
        }
        match dialect {
            Dialect::Iro => {
                let mut props = Props::new();
                if self.light_dark
                    && let (Some(ls), Some(ds)) = (self.light_slot, self.dark_slot)
                {
                    let light = slot_props(self.result, token, ls, dialect);
                    let dark = slot_props(self.result, token, ds, dialect);
                    for key in COLOR_KEYS {
                        let l = light
                            .iter()
                            .find(|(k, _)| k == key)
                            .map(|(_, v)| v.as_str());
                        let d = dark.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str());
                        if let (Some(l), Some(d)) = (l, d) {
                            props.push((key.to_owned(), light_dark_value(l, d)));
                        } else if let Some(v) = l.or(d) {
                            props.push((key.to_owned(), v.to_owned()));
                        }
                    }
                    for (prop, value) in &dark {
                        if !COLOR_KEYS.contains(&prop.as_str()) {
                            props.push((self.var(ds, prop), value.clone()));
                        }
                    }
                    for (prop, value) in &light {
                        if !COLOR_KEYS.contains(&prop.as_str()) {
                            props.push((self.var(ls, prop), value.clone()));
                        }
                    }
                } else {
                    for (i, &slot) in self.order.iter().enumerate() {
                        let slot_props = slot_props(self.result, token, slot, dialect);
                        if i == 0 {
                            if self.emit_default {
                                props.extend(slot_props);
                            }
                        } else {
                            for (prop, value) in slot_props {
                                props.push((self.var(slot, &prop), value));
                            }
                        }
                    }
                }
                sort_props(&mut props);
                props
            }
            Dialect::Shiki => {
                let styles: Vec<Props> = self
                    .order
                    .iter()
                    .map(|&slot| slot_props(self.result, token, slot, dialect))
                    .collect();
                let mut keys: Vec<&str> = Vec::new();
                for style in &styles {
                    for (key, _) in style {
                        if !keys.contains(&key.as_str()) {
                            keys.push(key);
                        }
                    }
                }
                let mut props = Props::new();
                if self.light_dark
                    && let (Some(ls), Some(ds)) = (self.light_slot, self.dark_slot)
                {
                    let ls_idx = self.order.iter().position(|&s| s == ls);
                    let ds_idx = self.order.iter().position(|&s| s == ds);
                    for &key in &keys {
                        let l = ls_idx.and_then(|i| {
                            styles[i]
                                .iter()
                                .find(|(k, _)| k == key)
                                .map(|(_, v)| v.as_str())
                        });
                        let d = ds_idx.and_then(|i| {
                            styles[i]
                                .iter()
                                .find(|(k, _)| k == key)
                                .map(|(_, v)| v.as_str())
                        });
                        if COLOR_KEYS.contains(&key) {
                            let lv = l.unwrap_or("inherit");
                            let dv = d.unwrap_or("inherit");
                            props.push((key.to_owned(), light_dark_value(lv, dv)));
                        }
                        for (i, style) in styles.iter().enumerate() {
                            let value = style
                                .iter()
                                .find(|(k, _)| k == key)
                                .map_or("inherit", |(_, v)| v.as_str());
                            props.push((self.var(self.order[i], key), value.to_owned()));
                        }
                    }
                } else {
                    for (i, style) in styles.iter().enumerate() {
                        for &key in &keys {
                            let value = style
                                .iter()
                                .find(|(k, _)| k == key)
                                .map_or("inherit", |(_, v)| v.as_str());
                            if i == 0
                                && self.emit_default
                                && (key == "color" || key == "background-color")
                            {
                                props.push((key.to_owned(), value.to_owned()));
                            } else {
                                props.push((self.var(self.order[i], key), value.to_owned()));
                            }
                        }
                    }
                }
                props
            }
        }
    }

    fn pre_props(&self) -> Props {
        let result = self.result;
        if !self.multi {
            return vec![
                ("background-color".to_owned(), result.bg().to_owned()),
                ("color".to_owned(), result.fg().to_owned()),
            ];
        }
        let mut props = Props::new();
        if self.light_dark
            && let (Some(light), Some(dark)) = (self.light_slot, self.dark_slot)
        {
            match self.options.dialect {
                Dialect::Iro => {
                    props.push((
                        "background-color".to_owned(),
                        light_dark_value(result.bg_of(light), result.bg_of(dark)),
                    ));
                    props.push((
                        "color".to_owned(),
                        light_dark_value(result.fg_of(light), result.fg_of(dark)),
                    ));
                    sort_props(&mut props);
                }
                Dialect::Shiki => {
                    let ld_chain =
                        |prop: &str, light_val: &str, dark_val: &str, props: &mut Props| {
                            props.push((prop.to_owned(), light_dark_value(light_val, dark_val)));
                            props.push((self.var(dark, prop), dark_val.to_owned()));
                            props.push((self.var(light, prop), light_val.to_owned()));
                        };
                    ld_chain(
                        "background-color",
                        result.bg_of(light),
                        result.bg_of(dark),
                        &mut props,
                    );
                    ld_chain("color", result.fg_of(light), result.fg_of(dark), &mut props);
                }
            }
        } else {
            match self.options.dialect {
                Dialect::Iro => {
                    for (i, &slot) in self.order.iter().enumerate() {
                        if i == 0 {
                            if self.emit_default {
                                props.push((
                                    "background-color".to_owned(),
                                    result.bg_of(slot).to_owned(),
                                ));
                                props.push(("color".to_owned(), result.fg_of(slot).to_owned()));
                            }
                        } else {
                            props.push((
                                self.var(slot, "background-color"),
                                result.bg_of(slot).to_owned(),
                            ));
                            props.push((self.var(slot, "color"), result.fg_of(slot).to_owned()));
                        }
                    }
                    sort_props(&mut props);
                }
                Dialect::Shiki => {
                    let chain = |prop: &str, props: &mut Props| {
                        for (i, &slot) in self.order.iter().enumerate() {
                            let value = if prop == "color" {
                                result.fg_of(slot)
                            } else {
                                result.bg_of(slot)
                            };
                            if i == 0 && self.emit_default {
                                props.push((prop.to_owned(), value.to_owned()));
                            } else {
                                props.push((self.var(slot, prop), value.to_owned()));
                            }
                        }
                    };
                    if self.emit_default {
                        chain("background-color", &mut props);
                        chain("color", &mut props);
                    } else {
                        chain("color", &mut props);
                        chain("background-color", &mut props);
                    }
                }
            }
        }
        props
    }

    fn pre_classes(&self) -> Vec<String> {
        let prefix = &self.options.class_prefix;
        let mut classes = vec![prefix.clone()];
        if !self.multi {
            classes.push(self.result.theme().name.clone());
            return classes;
        }
        classes.push(format!("{prefix}-themes"));
        match self.options.dialect {
            Dialect::Iro => classes.extend(self.result.themes.iter().map(|s| s.key.clone())),
            Dialect::Shiki => classes.extend(
                self.order
                    .iter()
                    .map(|&slot| self.result.themes[slot].theme.name.clone()),
            ),
        }
        classes
    }
}

pub fn write_node(out: &mut String, node: &Node, escape: Escape) {
    match node {
        Node::Text(text) => escape_text(out, text, escape),
        Node::Element {
            tag,
            attrs,
            children,
        } => {
            out.push('<');
            out.push_str(tag);
            for (key, value) in attrs {
                out.push(' ');
                out.push_str(key);
                out.push_str("=\"");
                escape_attr(out, value, escape);
                out.push('"');
            }
            out.push('>');
            for child in children {
                write_node(out, child, escape);
            }
            out.push_str("</");
            out.push_str(tag);
            out.push('>');
        }
    }
}

pub fn escape_text(out: &mut String, text: &str, escape: Escape) {
    for c in text.chars() {
        match (escape, c) {
            (Escape::Named, '<') => out.push_str("&lt;"),
            (Escape::Named, '>') => out.push_str("&gt;"),
            (Escape::Named, '&') => out.push_str("&amp;"),
            (Escape::Hex, '<') => out.push_str("&#x3C;"),
            (Escape::Hex, '&') => out.push_str("&#x26;"),
            (_, c) => out.push(c),
        }
    }
}

pub fn escape_attr(out: &mut String, text: &str, escape: Escape) {
    for c in text.chars() {
        match (escape, c) {
            (Escape::Named, '"') => out.push_str("&quot;"),
            (Escape::Named, '&') => out.push_str("&amp;"),
            (Escape::Hex, '\0') => out.push_str("&#x0;"),
            (Escape::Hex, '"') => out.push_str("&#x22;"),
            (Escape::Hex, '&') => out.push_str("&#x26;"),
            (Escape::Hex, '\'') => out.push_str("&#x27;"),
            (Escape::Hex, '`') => out.push_str("&#x60;"),
            (_, c) => out.push(c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn html(node: &Node, escape: Escape) -> String {
        let mut out = String::new();
        write_node(&mut out, node, escape);
        out
    }

    #[test]
    fn named_escaping_in_text_and_attributes() {
        let node = Node::element("span")
            .attr("class", "a\"&<>'")
            .attr("style", "color:#fff")
            .child(Node::text("<b>&\"'`"));
        assert_eq!(
            html(&node, Escape::Named),
            r#"<span class="a&quot;&amp;<>'" style="color:#fff">&lt;b&gt;&amp;"'`</span>"#
        );
    }

    #[test]
    fn hex_escaping_in_text_and_attributes() {
        let node = Node::element("span")
            .attr("data-x", "\0\"&'`<>")
            .child(Node::text("<b>&\"'`"));
        assert_eq!(
            html(&node, Escape::Hex),
            "<span data-x=\"&#x0;&#x22;&#x26;&#x27;&#x60;<>\">&#x3C;b>&#x26;\"'`</span>"
        );
    }

    #[test]
    fn script_close_tag_is_neutralized() {
        let node = Node::element("span").child(Node::text("</script>"));
        assert_eq!(html(&node, Escape::Named), "<span>&lt;/script&gt;</span>");
        assert_eq!(html(&node, Escape::Hex), "<span>&#x3C;/script></span>");
    }

    #[test]
    fn nested_elements_text_nodes_and_empty_elements() {
        let node = Node::element("div")
            .child(Node::element("span").child(Node::text("a")))
            .child(Node::text("\n"))
            .child(Node::element("span").child(Node::text("b")))
            .child(Node::element("span"));
        assert_eq!(
            html(&node, Escape::Named),
            "<div><span>a</span>\n<span>b</span><span></span></div>"
        );
    }

    #[test]
    fn attributes_keep_insertion_order_and_serializing_is_deterministic() {
        let node = Node::element("pre")
            .attr("class", "x")
            .attr("style", "color:#000")
            .attr("data-lang", "go")
            .attr("tabindex", "0");
        let first = html(&node, Escape::Named);
        assert_eq!(
            first,
            r#"<pre class="x" style="color:#000" data-lang="go" tabindex="0"></pre>"#
        );
        for _ in 0..100 {
            assert_eq!(html(&node, Escape::Named), first);
        }
    }

    #[test]
    fn shiki_multi_theme_fills_missing_keys_with_inherit() {
        use std::collections::HashMap;
        use std::sync::Arc;

        use crate::scope::ScopeListId;
        use crate::theme::{FontStyle, Theme};
        use crate::token::{ThemeSlot, ThemedLine, ThemedToken, TokenStyle, TokensResult};

        let theme = |name: &str, fg: &str, bg: &str, font_style: &str| {
            let json = format!(
                r#"{{"name":"{name}","type":"dark","colors":{{"editor.foreground":"{fg}","editor.background":"{bg}"}},"tokenColors":[{{"scope":"x","settings":{{"fontStyle":"{font_style}"}}}}]}}"#
            );
            Arc::new(Theme::parse(json.as_bytes()).unwrap())
        };
        let a = theme("a", "#111111", "#222222", "italic");
        let b = theme("b", "#333333", "#444444", "");
        let style_a = TokenStyle {
            color: Some(a.default_foreground_id()),
            bg: None,
            font_style: FontStyle::ITALIC,
        };
        let style_b = TokenStyle {
            color: Some(b.default_foreground_id()),
            bg: None,
            font_style: FontStyle::empty(),
        };
        let token = ThemedToken {
            start: 0,
            end: 1,
            style: style_a,
            scopes: ScopeListId(1),
        };
        let result = TokensResult::new(
            "x".to_owned(),
            vec![ThemedLine::new(0..1, vec![token])],
            vec![
                ThemeSlot {
                    key: "a".to_owned(),
                    theme: a,
                },
                ThemeSlot {
                    key: "b".to_owned(),
                    theme: b,
                },
            ],
            HashMap::from([(ScopeListId(1), vec![style_a, style_b].into_boxed_slice())]),
            None,
            Vec::new(),
        );
        let mut options = HtmlOptions::shiki();
        options.default_color = DefaultColor::Key("a".to_owned());
        let html = HtmlRenderer::new().render(&result, &options);
        assert_eq!(
            html,
            r#"<pre class="shiki shiki-themes a b" style="background-color:#222222;--shiki-b-bg:#444444;color:#111111;--shiki-b:#333333" tabindex="0"><code><span class="line"><span style="color:#111111;--shiki-a-font-style:italic;--shiki-b:#333333;--shiki-b-font-style:inherit">x</span></span></code></pre>"#
        );
        options.default_color = DefaultColor::Off;
        let html = HtmlRenderer::new().render(&result, &options);
        assert!(html.starts_with(
            r#"<pre class="shiki shiki-themes a b" style="--shiki-a:#111111;--shiki-b:#333333;--shiki-a-bg:#222222;--shiki-b-bg:#444444" tabindex="0">"#
        ));
        assert!(html.contains(
            r#"<span style="--shiki-a:#111111;--shiki-a-font-style:italic;--shiki-b:#333333;--shiki-b-font-style:inherit">x</span>"#
        ));
    }

    #[test]
    fn shiki_options_differ_from_the_default_only_where_documented() {
        let iro = HtmlOptions::default();
        let shiki = HtmlOptions::shiki();
        assert_eq!(iro.class_prefix, "iro");
        assert_eq!(iro.var_prefix, "--iro-");
        assert_eq!(iro.tabindex.as_deref(), Some("0"));
        assert!(!iro.merge_whitespace);
        assert_eq!(shiki.dialect, Dialect::Shiki);
        assert_eq!(shiki.escape, Escape::Hex);
        assert_eq!(shiki.class_prefix, "shiki");
        assert_eq!(shiki.var_prefix, "--shiki-");
        assert_eq!(shiki.default_color, DefaultColor::Key("light".to_owned()));
        assert!(shiki.merge_whitespace);
        assert_eq!(shiki.tabindex, iro.tabindex);
    }
}
