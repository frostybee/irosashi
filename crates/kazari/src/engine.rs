use std::collections::{BTreeMap, HashMap};

use crate::collapsible;
use crate::config::{
    CollapseSpec, CollapsibleConfig, Config, LangIconMode, ResolvedBlock, StyleValue,
};
use crate::diff;
use crate::error::Error;
use crate::frame;
use crate::link;
use crate::locale::{self, UIStrings};
use crate::meta;
use crate::notation;
use crate::render;
use crate::render_typst;
use crate::tokenize::{self, Tokens, expand_tabs};
use crate::types::{Frame, InlineMarker, LineMarker, LineRange, ThemeInfo, Themes};

#[derive(Default)]
pub struct Options {
    pub lang: String,
    pub title: String,
    pub theme: String,
    pub frame: Option<Frame>,
    pub line_numbers: Option<bool>,
    pub start_line_number: Option<usize>,
    pub wrap: Option<bool>,
    pub preserve_indent: Option<bool>,
    pub hanging_indent: Option<usize>,
    pub line_markers: Vec<LineMarker>,
    pub inline_markers: Vec<InlineMarker>,
    pub focus_lines: Vec<LineRange>,
    pub with_output: Option<bool>,
    pub output_collapsed: Option<bool>,
    pub output_label: String,
    pub collapse: Option<CollapseSpec>,
}

pub struct Kazari {
    highlighter: iro::Highlighter,
    config: Config,
    strings: UIStrings,
    light_info: ThemeInfo,
    dark_info: Option<ThemeInfo>,
}

impl Kazari {
    pub fn builder(highlighter: iro::Highlighter) -> KazariBuilder {
        KazariBuilder {
            highlighter,
            config: Config::default(),
        }
    }

    pub fn render_with_meta(&self, code: &str, meta_str: &str) -> Result<String, Error> {
        let mut resolved = self.resolve_meta(meta_str);
        if self.is_mermaid(&resolved) {
            return Ok(render_mermaid_block(code));
        }
        let tokens = self.prepare_and_tokenize(code, &mut resolved, false)?;
        Ok(render::render_block(
            &tokens,
            &resolved,
            &self.config,
            &self.strings,
        ))
    }

    pub fn render(&self, code: &str, options: &Options) -> Result<String, Error> {
        let mut resolved = self.resolve_options(options);
        if self.is_mermaid(&resolved) {
            return Ok(render_mermaid_block(code));
        }
        let tokens = self.prepare_and_tokenize(code, &mut resolved, false)?;
        Ok(render::render_block(
            &tokens,
            &resolved,
            &self.config,
            &self.strings,
        ))
    }

    /// Renders the block as a `#code-block(...)` call for the functions defined by
    /// [`typst_preamble`]. Only the light theme is used.
    pub fn render_with_meta_typst(&self, code: &str, meta_str: &str) -> Result<String, Error> {
        let mut resolved = self.resolve_meta(meta_str);
        let tokens = self.prepare_and_tokenize(code, &mut resolved, true)?;
        Ok(render_typst::render_block(&tokens, &resolved))
    }

    /// Same as [`Kazari::render_with_meta_typst`] with programmatic options.
    pub fn render_typst(&self, code: &str, options: &Options) -> Result<String, Error> {
        let mut resolved = self.resolve_options(options);
        let tokens = self.prepare_and_tokenize(code, &mut resolved, true)?;
        Ok(render_typst::render_block(&tokens, &resolved))
    }

    fn is_mermaid(&self, resolved: &ResolvedBlock) -> bool {
        self.config.mermaid_pass_through && resolved.lang == "mermaid"
    }

    fn resolve_meta(&self, meta_str: &str) -> ResolvedBlock {
        let parsed = meta::parse(meta_str);

        let lang = if !parsed.block_options.lang.is_empty() {
            self.config.resolve_language(&parsed.block_options.lang)
        } else {
            String::new()
        };

        let mut resolved = self.config.resolve(&lang, Some(&parsed.block_options));
        resolved.line_markers = parsed.line_markers;
        resolved.inline_markers = parsed.inline_markers;
        resolved.focus_lines = parsed.focus_lines;
        resolved.diff_lang = parsed.diff_lang;
        resolved.collapse_spec = parsed.collapse;
        resolved
    }

    fn resolve_options(&self, options: &Options) -> ResolvedBlock {
        let lang = if !options.lang.is_empty() {
            self.config.resolve_language(&options.lang)
        } else {
            String::new()
        };

        let block_opts = meta::BlockOptions {
            lang: lang.clone(),
            title: options.title.clone(),
            theme: options.theme.clone(),
            frame: options.frame,
            line_numbers: options.line_numbers,
            start_line_number: options.start_line_number,
            wrap: options.wrap,
            preserve_indent: options.preserve_indent,
            hanging_indent: options.hanging_indent,
            with_output: options.with_output,
            output_collapsed: options.output_collapsed,
            output_label: options.output_label.clone(),
        };

        let mut resolved = self.config.resolve(&lang, Some(&block_opts));
        resolved.line_markers = options.line_markers.clone();
        resolved.inline_markers = options.inline_markers.clone();
        resolved.focus_lines = options.focus_lines.clone();
        resolved.collapse_spec = options.collapse.clone();
        resolved
    }

    pub fn css(&self) -> String {
        crate::css::generate(&self.config, &self.light_info, self.dark_info.as_ref())
    }

    pub fn js(&self) -> String {
        crate::js::generate(&self.config)
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    /// The UI strings resolved from the configured locale and overrides.
    pub fn ui_strings(&self) -> &UIStrings {
        &self.strings
    }

    pub fn light_theme_info(&self) -> &ThemeInfo {
        &self.light_info
    }

    pub fn dark_theme_info(&self) -> Option<&ThemeInfo> {
        self.dark_info.as_ref()
    }

    /// Runs every source-level step (tab expansion, filename extraction, terminal
    /// comment stripping, diff, notation, frame detection) and tokenizes. With
    /// `single_theme` only the light theme is resolved.
    fn prepare_and_tokenize(
        &self,
        code: &str,
        resolved: &mut ResolvedBlock,
        single_theme: bool,
    ) -> Result<Tokens, Error> {
        let mut code = expand_tabs(code, self.config.tab_width);

        if self.config.file_name_extraction
            && resolved.title.is_empty()
            && let Some((title, modified)) = frame::extract_file_name(&code, &resolved.lang)
        {
            resolved.title = title;
            code = modified;
        }

        if resolved.with_output && self.config.output_panel {
            code = self.split_output_section(&code, resolved);
        }

        resolved.raw_code =
            if self.config.terminal_comment_stripping && resolved.frame == Frame::Terminal {
                frame::strip_terminal_comments(&code)
            } else {
                code.clone()
            };

        let (code, lang) = if resolved.lang == "diff" && !resolved.diff_lang.is_empty() {
            let (stripped, markers) = diff::process_diff_block(&code);
            resolved.line_markers.extend(markers);
            resolved.lang = self.config.resolve_language(&resolved.diff_lang);
            (stripped, resolved.lang.clone())
        } else {
            (code, resolved.lang.clone())
        };

        let code = if self.config.notation_comments {
            let n = notation::process_notation(&code);
            for warning in &n.warnings {
                self.config.warn(warning);
            }
            resolved.line_markers.extend(n.line_markers);
            resolved.focus_lines.extend(n.focus_lines);
            resolved.inline_markers.extend(n.inline_markers);
            resolved.raw_code = notation::process_notation(&resolved.raw_code).code;
            n.code
        } else {
            code
        };

        let code = if self.config.inline_links {
            let (cleaned, links) = link::extract_links(&code);
            resolved.links = links;
            resolved.raw_code = link::strip_links(&resolved.raw_code);
            cleaned
        } else {
            code
        };

        if self.config.frame_detection {
            resolved.frame = frame::detect_frame_type(&code, &lang, resolved.frame);
        }

        let mut themes = if !resolved.theme.is_empty() {
            Themes::parse_override(&resolved.theme)
        } else {
            Themes {
                light: self.config.light_theme.clone(),
                dark: self.config.dark_theme.clone(),
            }
        };
        if single_theme {
            themes.dark = None;
        }

        let tokens = tokenize::tokenize(&self.highlighter, &code, &lang, &themes)?;
        if lang == "ansi" {
            resolved.raw_code = (0..tokens.line_count())
                .map(|i| tokens.line_text(i))
                .collect::<Vec<_>>()
                .join("\n");
        }

        let collapse = collapsible::resolve_collapse(
            tokens.line_count(),
            resolved.collapse_spec.as_ref(),
            self.config.collapsible.as_ref(),
            &code,
            &resolved.line_markers,
            &resolved.focus_lines,
        );
        resolved.collapse_threshold = collapse.threshold;
        resolved.collapse_segments = collapse.preview_segments;
        resolved.collapse_beyond_cap = collapse.beyond_cap_count;
        resolved.collapse_ranges = collapse.ranges;

        Ok(tokens)
    }

    /// Splits `code` at the first line equal to the output separator; the rest
    /// becomes the output panel text.
    fn split_output_section(&self, code: &str, resolved: &mut ResolvedBlock) -> String {
        let sep = if self.config.output_separator.is_empty() {
            "---output---"
        } else {
            self.config.output_separator.as_str()
        };
        let lines: Vec<&str> = code.split('\n').collect();
        for (i, line) in lines.iter().enumerate() {
            if line.trim() == sep {
                resolved.output_text = lines[i + 1..].join("\n");
                return lines[..i].join("\n");
            }
        }
        code.to_owned()
    }
}

fn render_mermaid_block(code: &str) -> String {
    format!(
        "<pre class=\"mermaid\">{}</pre>\n",
        crate::escape::escape_text(code)
    )
}

pub struct KazariBuilder {
    highlighter: iro::Highlighter,
    config: Config,
}

impl KazariBuilder {
    pub fn themes(mut self, light: &str, dark: Option<&str>) -> Self {
        self.config.light_theme = light.to_owned();
        self.config.dark_theme = dark.map(|d| d.to_owned());
        self
    }

    pub fn dark_mode(mut self, mode: crate::types::DarkMode) -> Self {
        self.config.dark_mode = mode;
        self
    }

    pub fn copy_button(mut self, enabled: bool) -> Self {
        self.config.copy_button = enabled;
        self
    }

    pub fn wrap_button(mut self, enabled: bool) -> Self {
        self.config.wrap_button = enabled;
        self
    }

    pub fn fullscreen_button(mut self, enabled: bool) -> Self {
        self.config.fullscreen_button = enabled;
        self
    }

    pub fn theme_toggle(mut self, enabled: bool) -> Self {
        self.config.theme_toggle = enabled;
        self
    }

    pub fn output_panel(mut self, enabled: bool) -> Self {
        self.config.output_panel = enabled;
        self
    }

    pub fn output_collapsed(mut self, collapsed: bool) -> Self {
        self.config.output_default_collapsed = collapsed;
        self
    }

    pub fn output_separator(mut self, separator: &str) -> Self {
        self.config.output_separator = separator.to_owned();
        self
    }

    pub fn inline_links(mut self, enabled: bool) -> Self {
        self.config.inline_links = enabled;
        self
    }

    /// Ships the tab bar assets for `:::code-group` containers and lets the markdown
    /// adapter parse them.
    pub fn code_groups(mut self, enabled: bool) -> Self {
        self.config.code_groups = enabled;
        self
    }

    pub fn mermaid_pass_through(mut self, enabled: bool) -> Self {
        self.config.mermaid_pass_through = enabled;
        self
    }

    /// Enables threshold collapsing of long blocks.
    pub fn collapsible(mut self, config: CollapsibleConfig) -> Self {
        self.config.collapsible = Some(config);
        self
    }

    pub fn file_icons(mut self, enabled: bool) -> Self {
        self.config.file_icons = enabled;
        self
    }

    /// Custom markup for the file icon of an extension, replacing the placeholder
    /// span.
    pub fn file_icon_resolver(
        mut self,
        resolver: impl Fn(&str) -> String + Send + Sync + 'static,
    ) -> Self {
        self.config.file_icon_resolver = Some(Box::new(resolver));
        self
    }

    pub fn lang_icon_mode(mut self, mode: LangIconMode) -> Self {
        self.config.lang_icon_mode = mode;
        self
    }

    pub fn line_numbers(mut self, enabled: bool) -> Self {
        self.config.defaults.line_numbers = enabled;
        self
    }

    pub fn frame_detection(mut self, enabled: bool) -> Self {
        self.config.frame_detection = enabled;
        self
    }

    pub fn file_name_extraction(mut self, enabled: bool) -> Self {
        self.config.file_name_extraction = enabled;
        self
    }

    pub fn language_badge(mut self, enabled: bool) -> Self {
        self.config.language_badge = enabled;
        self
    }

    pub fn tab_width(mut self, width: usize) -> Self {
        self.config.tab_width = width;
        self
    }

    pub fn notation_comments(mut self, enabled: bool) -> Self {
        self.config.notation_comments = enabled;
        self
    }

    pub fn visible_whitespace(mut self, enabled: bool) -> Self {
        self.config.visible_whitespace = enabled;
        self
    }

    /// The symbols shown for tabs and spaces when whitespace is visible; an empty
    /// string keeps the default.
    pub fn whitespace_symbols(mut self, tab: &str, space: &str) -> Self {
        if !tab.is_empty() {
            self.config.whitespace_tab = tab.to_owned();
        }
        if !space.is_empty() {
            self.config.whitespace_space = space.to_owned();
        }
        self
    }

    pub fn locale(mut self, locale: &str) -> Self {
        self.config.locale = locale.to_owned();
        self
    }

    /// Overrides single UI strings by their dotted key (`copy.label`, ...).
    pub fn ui_strings(mut self, overrides: HashMap<String, String>) -> Self {
        self.config.ui_string_overrides.extend(overrides);
        self
    }

    /// Name of the CSS cascade layer; an empty name disables the wrapper.
    pub fn cascade_layer(mut self, name: &str) -> Self {
        self.config.cascade_layer = name.to_owned();
        self
    }

    pub fn theme_css_root(mut self, selector: &str) -> Self {
        if !selector.is_empty() {
            self.config.theme_css_root = selector.to_owned();
        }
        self
    }

    /// Overrides CSS variables with the same value in both themes.
    pub fn style_overrides(mut self, overrides: BTreeMap<String, String>) -> Self {
        for (name, value) in overrides {
            self.config
                .style_overrides
                .insert(name, StyleValue::plain(&value));
        }
        self
    }

    /// Overrides CSS variables with separate light and dark values.
    pub fn themed_style_overrides(mut self, overrides: BTreeMap<String, (String, String)>) -> Self {
        for (name, (light, dark)) in overrides {
            self.config
                .style_overrides
                .insert(name, StyleValue::themed(&light, &dark));
        }
        self
    }

    pub fn config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    pub fn config_file(mut self, yaml_str: &str) -> Result<Self, Error> {
        let file_config = crate::config::FileConfig::from_yaml(yaml_str)?;
        file_config.apply(&mut self.config)?;
        Ok(self)
    }

    pub fn build(self) -> Result<Kazari, Error> {
        let light_colors = self.highlighter.theme_colors(&self.config.light_theme)?;
        let light_info = ThemeInfo::from_iro(&light_colors);

        let dark_info = if let Some(ref dark) = self.config.dark_theme {
            let dark_colors = self.highlighter.theme_colors(dark)?;
            Some(ThemeInfo::from_iro(&dark_colors))
        } else {
            None
        };

        let strings = locale::resolve(&self.config.locale, &self.config.ui_string_overrides);

        Ok(Kazari {
            highlighter: self.highlighter,
            config: self.config,
            strings,
            light_info,
            dark_info,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::MarkerType;

    fn test_engine() -> Kazari {
        let hl = iro::Highlighter::new().unwrap();
        Kazari::builder(hl)
            .themes("github-light", Some("github-dark"))
            .build()
            .unwrap()
    }

    #[test]
    fn render_with_meta_basic() {
        let kz = test_engine();
        let html = kz.render_with_meta("let x = 1;", "javascript").unwrap();
        assert!(html.contains("kazari-block"));
        assert!(html.contains("kz-line"));
        assert!(html.contains("--sl:"));
    }

    #[test]
    fn render_with_meta_line_numbers() {
        let kz = test_engine();
        let html = kz
            .render_with_meta("let x = 1;\nlet y = 2;", "js showLineNumbers")
            .unwrap();
        assert!(html.contains("kz-gutter"));
        assert!(html.contains("kz-ln"));
    }

    #[test]
    fn render_with_meta_markers() {
        let kz = test_engine();
        let html = kz
            .render_with_meta("line1\nline2\nline3", "text {2}")
            .unwrap();
        assert!(html.contains("highlight"));
        assert!(html.contains("mark"));
    }

    #[test]
    fn render_with_meta_title() {
        let kz = test_engine();
        let html = kz
            .render_with_meta("fn main() {}", "rust title=\"app.rs\"")
            .unwrap();
        assert!(html.contains("kz-title"));
        assert!(html.contains("app.rs"));
    }

    #[test]
    fn render_with_meta_terminal_frame() {
        let kz = test_engine();
        let html = kz.render_with_meta("echo hello", "bash").unwrap();
        assert!(html.contains("is-terminal"));
        assert!(html.contains("kz-terminal-dots"));
    }

    #[test]
    fn render_with_meta_focus() {
        let kz = test_engine();
        let html = kz.render_with_meta("a\nb\nc", "text focus={2}").unwrap();
        assert!(html.contains("has-focus"));
        assert!(html.contains("focused"));
    }

    #[test]
    fn render_with_meta_diff() {
        let kz = test_engine();
        let html = kz
            .render_with_meta("+ added\n- removed\n context", "diff lang=\"go\"")
            .unwrap();
        assert!(html.contains("ins"));
        assert!(html.contains("del"));
    }

    #[test]
    fn render_with_options() {
        let kz = test_engine();
        let html = kz
            .render(
                "let x = 1;",
                &Options {
                    lang: "javascript".into(),
                    line_numbers: Some(true),
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(html.contains("kz-gutter"));
    }

    #[test]
    fn css_output_contains_vars_and_rules() {
        let kz = test_engine();
        let css = kz.css();
        assert!(css.contains(":root {"));
        assert!(css.contains("--kz-editor-bg"));
        assert!(css.contains(".kazari-block"));
        assert!(css.contains("kz-line span[style^=\"--\"]"));
    }

    #[test]
    fn js_output_non_empty() {
        let kz = test_engine();
        let js = kz.js();
        assert!(!js.is_empty());
    }

    #[test]
    fn file_name_extraction_works() {
        let kz = test_engine();
        let html = kz
            .render_with_meta("// main.go\npackage main", "go")
            .unwrap();
        assert!(html.contains("main.go"));
        assert!(html.contains("kz-title"));
    }

    #[test]
    fn language_badge_displayed() {
        let kz = test_engine();
        let html = kz.render_with_meta("let x = 1;", "javascript").unwrap();
        assert!(html.contains("kz-lang"));
        assert!(html.contains("JavaScript"));
    }

    #[test]
    fn copy_button_present() {
        let kz = test_engine();
        let html = kz.render_with_meta("hello", "text").unwrap();
        assert!(html.contains("kz-copy-btn"));
        assert!(html.contains("data-code="));
    }

    #[test]
    fn data_line_count_attribute() {
        let kz = test_engine();
        let html = kz.render_with_meta("a\nb\nc", "text").unwrap();
        assert!(html.contains("data-lines=\"3\""));
    }

    #[test]
    fn single_theme_no_dark_vars() {
        let hl = iro::Highlighter::new().unwrap();
        let kz = Kazari::builder(hl)
            .themes("github-light", None)
            .build()
            .unwrap();
        let html = kz.render_with_meta("let x = 1;", "js").unwrap();
        assert!(html.contains("--sl:"));
        assert!(!html.contains("--sd:"));
    }

    #[test]
    fn render_inline_marker_literal() {
        let kz = test_engine();
        let html = kz
            .render(
                "let x = 1;",
                &Options {
                    lang: "javascript".into(),
                    inline_markers: vec![InlineMarker {
                        marker_type: MarkerType::Mark,
                        text: "x".into(),
                        is_regex: false,
                    }],
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(html.contains("<mark>"));
        assert!(html.contains("</mark>"));
    }

    #[test]
    fn render_inline_marker_ins_del() {
        let kz = test_engine();
        let html = kz
            .render(
                "old_fn new_fn",
                &Options {
                    lang: "text".into(),
                    inline_markers: vec![
                        InlineMarker {
                            marker_type: MarkerType::Del,
                            text: "old_fn".into(),
                            is_regex: false,
                        },
                        InlineMarker {
                            marker_type: MarkerType::Ins,
                            text: "new_fn".into(),
                            is_regex: false,
                        },
                    ],
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(html.contains("<del>"));
        assert!(html.contains("<ins>"));
    }

    #[test]
    fn render_inline_marker_regex() {
        let kz = test_engine();
        let html = kz
            .render(
                "let count = 42;",
                &Options {
                    lang: "javascript".into(),
                    inline_markers: vec![InlineMarker {
                        marker_type: MarkerType::Mark,
                        text: r"\d+".into(),
                        is_regex: true,
                    }],
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(html.contains("<mark>"));
    }

    #[test]
    fn render_inline_marker_via_meta() {
        let kz = test_engine();
        let html = kz
            .render_with_meta("let x = 1;", "javascript \"x\"")
            .unwrap();
        assert!(html.contains("<mark>"));
    }

    fn notation_engine() -> Kazari {
        let hl = iro::Highlighter::new().unwrap();
        Kazari::builder(hl)
            .themes("github-light", Some("github-dark"))
            .notation_comments(true)
            .build()
            .unwrap()
    }

    #[test]
    fn notation_is_off_by_default() {
        let kz = test_engine();
        let html = kz
            .render_with_meta("a // [!code ++]", "javascript")
            .unwrap();
        assert!(html.contains("[!code"));
        assert!(!html.contains("highlight"));
    }

    #[test]
    fn notation_diff_highlight_error_warning() {
        let kz = notation_engine();
        let html = kz
            .render_with_meta(
                "a // [!code ++]\nb // [!code --]\nc // [!code highlight]\nd // [!code error]\ne // [!code warning]\nf",
                "javascript",
            )
            .unwrap();
        assert!(!html.contains("[!code"));
        assert!(html.contains("kz-line highlight ins"));
        assert!(html.contains("kz-line highlight del"));
        assert!(html.contains("kz-line highlight mark"));
        assert!(html.contains("kz-line highlight error"));
        assert!(html.contains("kz-line highlight warning"));
        assert!(html.contains("data-lines=\"6\""));
    }

    #[test]
    fn notation_focus_and_word() {
        let kz = notation_engine();
        let html = kz
            .render_with_meta(
                "foo // [!code focus]\nbar foo // [!code word:foo]",
                "javascript",
            )
            .unwrap();
        assert!(html.contains("has-focus"));
        assert!(html.contains("kz-line focused"));
        assert_eq!(html.matches("<mark>foo</mark>").count(), 2);
    }

    #[test]
    fn notation_removes_blank_lines_and_combines_with_meta_markers() {
        let kz = notation_engine();
        let html = kz
            .render_with_meta("x\n// [!code ++]\ny // [!code highlight]", "javascript {1}")
            .unwrap();
        assert!(html.contains("data-lines=\"2\""));
        assert_eq!(html.matches("kz-line highlight mark").count(), 2);
        assert!(!html.contains("highlight ins"));
    }

    #[test]
    fn notation_unknown_annotation_warns() {
        use std::sync::{Arc, Mutex};
        let seen = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&seen);
        let hl = iro::Highlighter::new().unwrap();
        let config = Config {
            notation_comments: true,
            warning_handler: Some(Box::new(move |msg: &str| {
                sink.lock().unwrap().push(msg.to_owned());
            })),
            ..Default::default()
        };
        let kz = Kazari::builder(hl).config(config).build().unwrap();
        kz.render_with_meta("a // [!code nope]", "javascript")
            .unwrap();
        assert_eq!(
            *seen.lock().unwrap(),
            ["unknown code notation [!code nope]"]
        );
    }

    #[test]
    fn visible_whitespace_wraps_spaces_and_tabs() {
        let hl = iro::Highlighter::new().unwrap();
        let kz = Kazari::builder(hl)
            .themes("github-light", None)
            .visible_whitespace(true)
            .build()
            .unwrap();
        let html = kz.render_with_meta("a b", "text").unwrap();
        assert!(html.contains("a<span class=\"ws-space\">\u{b7}</span>b"));

        let hl = iro::Highlighter::new().unwrap();
        let kz = Kazari::builder(hl)
            .themes("github-light", None)
            .visible_whitespace(true)
            .whitespace_symbols(">", "_")
            .build()
            .unwrap();
        let html = kz
            .render(
                "a b c",
                &Options {
                    lang: "text".into(),
                    inline_markers: vec![InlineMarker {
                        marker_type: MarkerType::Mark,
                        text: "b c".into(),
                        is_regex: false,
                    }],
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(
            html.contains(
                "a<span class=\"ws-space\">_</span><mark>b<span class=\"ws-space\">_</span>c</mark>"
            ),
            "{html}"
        );

        let plain = test_engine().render_with_meta("a b", "text").unwrap();
        assert!(!plain.contains("ws-space"));
    }

    #[test]
    fn ansi_input_renders_through_kazari() {
        let kz = test_engine();
        let html = kz
            .render_with_meta("\x1b[31mred\x1b[0m plain", "ansi")
            .unwrap();
        assert!(html.contains("--sl:#cd3131"));
        assert!(html.contains(">red</span>"));
        assert!(!html.contains('\x1b'));
    }

    #[test]
    fn render_with_meta_typst_basic() {
        let kz = test_engine();
        let out = kz
            .render_with_meta_typst("let x = 1;", "javascript")
            .unwrap();
        assert!(
            out.starts_with("#code-block(lang: \"javascript\", fg: rgb(\""),
            "{out}"
        );
        assert!(
            out.contains("#code-line[#text(fill: rgb(\"#d73a49\"), \"let\")"),
            "{out}"
        );
        assert!(out.ends_with("]\n]"), "{out}");
    }

    #[test]
    fn render_with_meta_typst_line_numbers() {
        let kz = test_engine();
        let out = kz
            .render_with_meta_typst("a\nb", "text showLineNumbers startLineNumber=9")
            .unwrap();
        assert!(
            out.contains("numbers: true, gutter-width: 2 * 0.65em"),
            "{out}"
        );
        assert!(out.contains("#code-line(num: 10)[#text(\"b\")]"), "{out}");
    }

    #[test]
    fn render_with_meta_typst_markers() {
        let kz = test_engine();
        let out = kz
            .render_with_meta_typst("a\nb\nc", "text {1} ins={2} del={3}")
            .unwrap();
        assert!(
            out.contains("#code-line(mark: \"mark\")[#text(\"a\")]"),
            "{out}"
        );
        assert!(
            out.contains("#code-line(mark: \"ins\")[#text(\"b\")]"),
            "{out}"
        );
        assert!(
            out.contains("#code-line(mark: \"del\")[#text(\"c\")]"),
            "{out}"
        );
    }

    #[test]
    fn render_with_meta_typst_focus() {
        let kz = test_engine();
        let out = kz.render_with_meta_typst("a\nb", "text focus={1}").unwrap();
        assert!(out.contains("#code-line[#text(\"a\")]"), "{out}");
        assert!(
            out.contains("#code-line[#text(fill: rgb(\"#24292e\").transparentize(60%), \"b\")]"),
            "{out}"
        );
    }

    #[test]
    fn render_with_meta_typst_title() {
        let kz = test_engine();
        let out = kz
            .render_with_meta_typst("x", r#"rust title="app.rs""#)
            .unwrap();
        assert!(out.contains("title: \"app.rs\", "), "{out}");
    }

    #[test]
    fn render_typst_uses_light_theme_only() {
        let kz = test_engine();
        let code = "fn main() { let s = \"x\"; }";
        let html = kz.render_with_meta(code, "rust").unwrap();
        let typ = kz.render_with_meta_typst(code, "rust").unwrap();
        for color in ["#d73a49", "#6f42c1", "#032f62"] {
            assert!(html.contains(&format!("--sl:{color}")), "{html}");
            assert!(typ.contains(&format!("fill: rgb(\"{color}\")")), "{typ}");
        }
        let dark_only = "#f97583";
        assert!(html.contains(&format!("--sd:{dark_only}")), "{html}");
        assert!(!typ.contains(dark_only), "{typ}");
    }

    #[test]
    fn render_typst_with_options() {
        let kz = test_engine();
        let out = kz
            .render_typst(
                "a",
                &Options {
                    lang: "text".into(),
                    title: "t".into(),
                    line_numbers: Some(true),
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(out.contains("title: \"t\", "), "{out}");
        assert!(out.contains("#code-line(num: 1)[#text(\"a\")]"), "{out}");
    }

    #[test]
    fn typst_preamble_defines_functions() {
        let p = crate::typst_preamble();
        assert!(p.contains("#let code-block("));
        assert!(p.contains("#let code-line("));
        assert!(p.contains("#let kz-marker-colors"));
    }

    #[test]
    fn locale_and_overrides_reach_the_buttons() {
        let hl = iro::Highlighter::new().unwrap();
        let mut overrides = HashMap::new();
        overrides.insert("copy.success".to_owned(), "Fait".to_owned());
        let kz = Kazari::builder(hl)
            .locale("fr-FR")
            .ui_strings(overrides)
            .build()
            .unwrap();
        assert_eq!(kz.ui_strings().copy_label, "Copier");
        let html = kz.render_with_meta("x", "text").unwrap();
        assert!(html.contains("aria-label=\"Copier\""), "{html}");
        assert!(html.contains("data-copied=\"Fait\""), "{html}");
        assert!(
            html.contains("data-enable=\"Activer le retour à la ligne\""),
            "{html}"
        );
        let term = kz.render_with_meta("ls", "bash").unwrap();
        assert!(
            term.contains("<span class=\"sr-only\">Fenêtre de terminal</span>"),
            "{term}"
        );
    }

    #[test]
    fn fullscreen_and_font_controls_default_on() {
        let kz = test_engine();
        let html = kz.render_with_meta("x", "rust").unwrap();
        let copy = html.find("kz-copy-btn").unwrap();
        let wrap = html.find("kz-wrap-btn").unwrap();
        let font = html.find("kz-font-controls").unwrap();
        let fs = html.find("kz-fs-btn").unwrap();
        assert!(copy < wrap && wrap < font && font < fs, "{html}");
        assert!(html.contains("class=\"kz-font-dec\" aria-label=\"Decrease font size\""));
        assert!(html.contains("class=\"kz-font-inc\" aria-label=\"Increase font size\""));
        assert!(html.contains("class=\"kz-fs-btn\" aria-label=\"Fullscreen\" data-tooltip=\"Fullscreen\" aria-expanded=\"false\""));
        assert!(kz.css().contains("--kz-fs-font-scale: 1;"));
        assert!(kz.css().contains(".kz-fs-btn"));
        assert!(kz.js().contains("kz-fs-btn"));

        let hl = iro::Highlighter::new().unwrap();
        let off = Kazari::builder(hl)
            .fullscreen_button(false)
            .build()
            .unwrap();
        let html = off.render_with_meta("x", "rust").unwrap();
        assert!(!html.contains("kz-fs-btn") && !html.contains("kz-font-controls"));
        assert!(!off.css().contains("--kz-fs-font-scale"));
        assert!(!off.css().contains(".kz-fs-btn"));
        assert!(!off.js().contains("kz-fs-btn"));
    }

    #[test]
    fn theme_toggle_button_and_block_id() {
        let hl = iro::Highlighter::new().unwrap();
        let kz = Kazari::builder(hl)
            .themes("github-light", Some("github-dark"))
            .theme_toggle(true)
            .build()
            .unwrap();
        let html = kz.render_with_meta("x = 1", "python").unwrap();
        assert!(html.contains("data-kz-id=\""), "{html}");
        let again = kz.render_with_meta("x = 1", "python").unwrap();
        assert_eq!(html, again);
        assert!(html.contains("<button class=\"kz-theme-toggle-btn\" aria-pressed=\"false\" aria-label=\"Toggle theme\" data-tooltip=\"Toggle theme\" data-label=\"Toggle theme\" data-toggled=\"Toggle theme\" data-announcement=\"Theme toggled\" data-kz-dark-selector=\".dark\" data-kz-dark-mode=\"selector\">"), "{html}");
        let wrap = html.find("kz-wrap-btn").unwrap();
        let toggle = html.find("kz-theme-toggle-btn").unwrap();
        let font = html.find("kz-font-controls").unwrap();
        assert!(wrap < toggle && toggle < font);
        let css = kz.css();
        assert!(
            css.contains(".kazari-block[data-kz-theme=\"dark\"] { --kz-editor-bg: #24292e; "),
            "{css}"
        );
        assert!(
            css.contains(".kazari-block[data-kz-theme=\"light\"] { --kz-editor-bg: #fff; "),
            "{css}"
        );
        assert!(css.contains(".kazari-block[data-kz-theme] { --kz-terminal-bg: var(--kz-editor-bg); --kz-terminal-titlebar-bg: var(--kz-toolbar-bg); }"));
        assert!(css.contains(".kazari-block[data-kz-theme=\"dark\"] .kz-line span[style^=\"--\"] { color: var(--sd, inherit)"));
        assert!(css.contains(".kz-theme-toggle-btn"));
        assert!(kz.js().contains("kz-theme-toggle-btn"));

        let hl = iro::Highlighter::new().unwrap();
        let single = Kazari::builder(hl)
            .themes("github-light", None)
            .theme_toggle(true)
            .build()
            .unwrap();
        let html = single.render_with_meta("x = 1", "python").unwrap();
        assert!(!html.contains("data-kz-id"));
        assert!(!html.contains("kz-theme-toggle-btn"));
        assert!(!single.css().contains("[data-kz-theme=\"dark\"] {"));
        assert!(!single.css().contains(".kz-theme-toggle-btn"));
        assert!(!single.js().contains("kz-theme-toggle-btn"));

        let plain = test_engine().render_with_meta("x = 1", "python").unwrap();
        assert!(!plain.contains("data-kz-id"));
    }

    #[test]
    fn output_panel_splits_code_and_renders_in_every_frame() {
        let hl = iro::Highlighter::new().unwrap();
        let kz = Kazari::builder(hl).output_panel(true).build().unwrap();
        let src = "print(1)\n---output---\n1\n<done>";
        let html = kz.render_with_meta(src, "python withOutput").unwrap();
        assert!(html.contains("data-lines=\"1\""), "{html}");
        assert!(html.contains("<div class=\"kz-output\"><div class=\"kz-output-header\"><button class=\"kz-output-toggle\" aria-expanded=\"true\">Output</button></div><pre class=\"kz-output-pre\">1\n&lt;done&gt;</pre></div>"), "{html}");
        assert!(
            html.contains("data-code=\"print(1)\""),
            "copy text excludes output: {html}"
        );
        assert!(kz.css().contains(".kz-output"));
        assert!(kz.js().contains("kz-output-toggle"));

        let collapsed = kz
            .render_with_meta(
                src,
                "python withOutput outputCollapsed outputLabel=\"Result\"",
            )
            .unwrap();
        assert!(collapsed.contains("<div class=\"kz-output kz-output-hidden\"><div class=\"kz-output-header\"><button class=\"kz-output-toggle\" aria-expanded=\"false\">Result</button>"), "{collapsed}");

        let none = kz
            .render_with_meta(src, "python withOutput frame=none")
            .unwrap();
        assert!(none.contains("kz-output-pre"), "{none}");
        let term = kz
            .render_with_meta("ls\n---output---\na b", "bash withOutput")
            .unwrap();
        assert!(
            term.contains("is-terminal") && term.contains("kz-output-pre"),
            "{term}"
        );

        let no_meta = kz.render_with_meta(src, "python").unwrap();
        assert!(!no_meta.contains("kz-output"));
        assert!(no_meta.contains("data-lines=\"4\""));

        let off = test_engine()
            .render_with_meta(src, "python withOutput")
            .unwrap();
        assert!(!off.contains("kz-output"));
        assert!(!test_engine().css().contains(".kz-output"));
    }

    #[test]
    fn output_separator_is_trimmed_exact_and_configurable() {
        let hl = iro::Highlighter::new().unwrap();
        let kz = Kazari::builder(hl)
            .output_panel(true)
            .output_separator("===")
            .build()
            .unwrap();
        let html = kz
            .render_with_meta(
                "a\n  ===  \nout\n---output---\nstill out",
                "text withOutput",
            )
            .unwrap();
        assert!(
            html.contains("<pre class=\"kz-output-pre\">out\n---output---\nstill out</pre>"),
            "{html}"
        );
        let html = kz
            .render_with_meta("a\n====\nb", "text withOutput")
            .unwrap();
        assert!(!html.contains("kz-output"), "{html}");
    }

    fn links_engine() -> Kazari {
        let hl = iro::Highlighter::new().unwrap();
        Kazari::builder(hl)
            .themes("github-light", None)
            .inline_links(true)
            .notation_comments(true)
            .build()
            .unwrap()
    }

    #[test]
    fn inline_links_render_anchors_and_clean_the_copy_text() {
        let kz = links_engine();
        let html = kz
            .render_with_meta("see @[the docs](https://x.y/d) here", "text")
            .unwrap();
        assert!(html.contains("<a class=\"kz-link\" href=\"https://x.y/d\" target=\"_blank\" rel=\"noopener noreferrer\">the docs<svg class=\"kz-link-icon\""), "{html}");
        assert!(!html.contains("@["), "{html}");
        assert!(html.contains("data-code=\"see the docs here\""), "{html}");
        assert!(kz.css().contains(".kz-link"));

        let off = test_engine()
            .render_with_meta("see @[the docs](https://x.y/d)", "text")
            .unwrap();
        assert!(off.contains("@[the docs](https://x.y/d)"), "{off}");
        assert!(!test_engine().css().contains(".kz-link"));
    }

    #[test]
    fn inline_links_across_tokens_and_over_markers() {
        let kz = links_engine();
        let html = kz
            .render_with_meta("let @[x = 1](/one);", "javascript")
            .unwrap();
        let anchors = html.matches("<a class=\"kz-link\"").count();
        assert!(anchors >= 3, "one anchor per token under the link: {html}");
        assert_eq!(
            html.matches("kz-link-icon").count(),
            1,
            "icon only at the end: {html}"
        );

        let html = kz
            .render_with_meta("ab @[cd](/x) ef", "text \"b cd e\"")
            .unwrap();
        assert!(html.contains("<mark>b </mark>"), "{html}");
        assert!(html.contains("<a class=\"kz-link\" href=\"/x\" target=\"_blank\" rel=\"noopener noreferrer\"><mark>cd</mark><svg class=\"kz-link-icon\""), "{html}");
        assert!(html.contains("</a><mark> e</mark>"), "{html}");
    }

    #[test]
    fn inline_links_after_notation_and_diff_keep_offsets() {
        let kz = links_engine();
        let html = kz
            .render_with_meta(
                "// [!code ++]\nx @[y](/y) z // [!code highlight]",
                "javascript",
            )
            .unwrap();
        assert!(html.contains("data-lines=\"1\""), "{html}");
        assert!(
            html.contains("href=\"/y\" target=\"_blank\" rel=\"noopener noreferrer\">y<svg"),
            "{html}"
        );
        let html = kz
            .render_with_meta("+a @[b](/b) c", "diff lang=\"text\"")
            .unwrap();
        assert!(html.contains("rel=\"noopener noreferrer\">b<svg"), "{html}");
        assert!(html.contains(">a <a class=\"kz-link\""), "{html}");
    }

    #[test]
    fn inline_links_unsafe_scheme_stays_literal() {
        let kz = links_engine();
        let html = kz
            .render_with_meta("@[x](javascript:alert(1))", "text")
            .unwrap();
        assert!(!html.contains("<a "), "{html}");
        assert!(html.contains("@[x](javascript:alert(1))"), "{html}");
    }

    #[test]
    fn inline_links_in_typst_output() {
        let kz = links_engine();
        let typ = kz
            .render_with_meta_typst("see @[docs](https://x.y/d) now", "text")
            .unwrap();
        assert!(
            typ.contains("#text(\"see \")#link(\"https://x.y/d\")[#text(\"docs\")]#text(\" now\")"),
            "{typ}"
        );
    }

    fn numbered(n: usize) -> String {
        (1..=n)
            .map(|i| format!("line {i}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn collapsible_engine(cfg: CollapsibleConfig) -> Kazari {
        let hl = iro::Highlighter::new().unwrap();
        Kazari::builder(hl)
            .themes("github-light", None)
            .collapsible(cfg)
            .build()
            .unwrap()
    }

    #[test]
    fn threshold_collapse_markup() {
        let kz = collapsible_engine(CollapsibleConfig::default());
        let html = kz
            .render_with_meta(&numbered(20), "text showLineNumbers")
            .unwrap();
        assert!(
            html.starts_with("<div class=\"kazari-block kz-collapsed not-content\""),
            "{html}"
        );
        assert!(html.contains("<button class=\"kz-collapse-toggle\" aria-expanded=\"false\" aria-label=\"Show more\" data-tooltip=\"Show more\" data-expand=\"Show more\" data-collapse=\"Show less\">"), "{html}");
        assert!(
            html.contains("<div class=\"kz-collapse-content\"><pre"),
            "{html}"
        );
        assert!(html.contains("</pre><div class=\"kz-collapse-gradient\"></div></div><div class=\"kz-collapse-bar\"><button class=\"kz-collapse-btn\" aria-expanded=\"false\" data-expand=\"Show more\" data-collapse=\"Show less\" data-expanded-msg=\"Code block expanded\" data-collapsed-msg=\"Code block collapsed\">Show more</button></div><div class=\"kz-sr-announce\" aria-live=\"polite\"></div>"), "{html}");
        assert_eq!(html.matches("kz-line kz-hidden").count(), 12, "{html}");
        assert!(html.contains("<div class=\"kz-line kz-hidden\"><div class=\"kz-gutter\"><div class=\"kz-ln\" aria-hidden=\"true\">9</div></div><div class=\"kz-code\"><span"), "{html}");
        assert!(!html.contains("kz-gap"), "{html}");
        assert!(kz.js().contains("kz-collapse-btn"));
        assert!(kz.css().contains(".kz-collapse-bar") || kz.css().contains("kz-collapse"));

        let short = kz.render_with_meta(&numbered(15), "text").unwrap();
        assert!(
            !short.contains("kz-collapse") && !short.contains("kz-hidden"),
            "{short}"
        );
        let off = kz
            .render_with_meta(&numbered(20), "text nocollapse")
            .unwrap();
        assert!(!off.contains("kz-collapse"), "{off}");
        let forced = kz.render_with_meta(&numbered(3), "text collapse").unwrap();
        assert!(forced.contains("kz-collapse-bar"), "{forced}");
        assert!(
            !forced.contains("kz-hidden"),
            "everything fits the preview: {forced}"
        );

        let plain = test_engine()
            .render_with_meta(&numbered(40), "text")
            .unwrap();
        assert!(
            !plain.contains("kz-collapse") && !plain.contains("kz-hidden"),
            "{plain}"
        );
        assert!(!test_engine().js().contains("kz-collapse-btn"));
        assert!(
            test_engine().css().contains("kz-section"),
            "range CSS always ships"
        );
    }

    #[test]
    fn threshold_preview_segments_gap_and_badge() {
        let kz = collapsible_engine(CollapsibleConfig::default());
        let html = kz
            .render_with_meta(&numbered(30), "text ins={12} {20}")
            .unwrap();
        assert!(html.contains("<div class=\"kz-line kz-gap\"><div class=\"kz-code\"><span class=\"kz-gap-indicator\" aria-hidden=\"true\">⋮</span><span class=\"sr-only\">Lines hidden</span></div></div>"), "{html}");
        assert_eq!(html.matches("kz-gap\"").count(), 1, "{html}");
        assert!(
            html.contains("<div class=\"kz-line highlight ins\">"),
            "line 12 visible and marked: {html}"
        );
        assert!(
            html.contains("<div class=\"kz-line kz-hidden highlight mark\">"),
            "line 20 hidden but marked: {html}"
        );
        assert!(
            html.contains(">Show more (+1 highlighted)</button>"),
            "{html}"
        );
        assert_eq!(
            html.matches("kz-line kz-hidden").count(),
            30 - 8 - 3,
            "{html}"
        );

        let expanded = collapsible_engine(CollapsibleConfig {
            default_collapsed: false,
            expand_button_text: "Plus".into(),
            collapse_button_text: "Moins".into(),
            ..Default::default()
        });
        let html = expanded.render_with_meta(&numbered(20), "text").unwrap();
        assert!(
            html.starts_with("<div class=\"kazari-block not-content\""),
            "{html}"
        );
        assert!(html.contains("<button class=\"kz-collapse-toggle\" aria-expanded=\"true\" aria-label=\"Moins\" data-tooltip=\"Moins\" data-expand=\"Plus\" data-collapse=\"Moins\">"), "{html}");
        assert!(html.contains("data-expand=\"Plus\" data-collapse=\"Moins\" data-expanded-msg=\"Code block expanded\""), "{html}");
    }

    #[test]
    fn labeled_markers_disable_threshold_unless_forced() {
        let kz = collapsible_engine(CollapsibleConfig::default());
        let html = kz
            .render_with_meta(&numbered(20), "text {\"API\":3}")
            .unwrap();
        assert!(!html.contains("kz-collapse"), "{html}");
        let html = kz
            .render_with_meta(&numbered(20), "text {\"API\":3} collapse")
            .unwrap();
        assert!(html.contains("kz-collapse-bar"), "{html}");
    }

    #[test]
    fn range_collapse_works_without_config() {
        let kz = test_engine();
        let code = "a\n  b\n  c\nd\ne";
        let html = kz
            .render_with_meta(code, "text collapse={2-3} showLineNumbers")
            .unwrap();
        assert!(html.contains("<details class=\"kz-section\"><summary><div class=\"kz-line\"><div class=\"kz-gutter\"><div class=\"kz-ln\"></div></div><div class=\"kz-code\"><span class=\"expand\" aria-hidden=\"true\"></span><span class=\"collapse\" aria-hidden=\"true\"></span><span class=\"text\">2 collapsed lines</span></div></div></summary>"), "{html}");
        let details_start = html.find("<details class=\"kz-section\">").unwrap();
        let details_end = html.find("</details>").unwrap();
        let inner = &html[details_start..details_end];
        assert_eq!(
            inner.matches("<div class=\"kz-line\">").count(),
            3,
            "summary + two lines: {inner}"
        );
        assert!(
            !html.contains("--kz-indent"),
            "no indent without a config: {html}"
        );
        assert!(!html.contains("kz-collapse-bar"));

        let kz = collapsible_engine(CollapsibleConfig::default());
        let html = kz.render_with_meta(code, "text collapse={2-3}").unwrap();
        assert!(
            html.contains(
                "<div class=\"kz-code\" style=\"--kz-indent:2ch\"><span class=\"expand\""
            ),
            "{html}"
        );
    }

    #[test]
    fn range_collapse_styles() {
        let kz = test_engine();
        let code = "a\nb\nc\nd";
        let html = kz
            .render_with_meta(code, "text collapse={2-3} collapseStyle=collapsible-start")
            .unwrap();
        assert!(
            html.contains("<div class=\"kz-section collapsible-start\"><details><summary>"),
            "{html}"
        );
        assert!(
            html.contains("</details><div class=\"content-lines\"><div class=\"kz-line\">"),
            "{html}"
        );
        assert!(html.contains("</div></div></div></div>"), "{html}");

        let html = kz
            .render_with_meta(code, "text collapse={3-4} collapseStyle=collapsible-auto")
            .unwrap();
        assert!(
            html.contains("kz-section collapsible-end"),
            "reaches the last line: {html}"
        );
        let html = kz
            .render_with_meta(code, "text collapse={2-3} collapseStyle=collapsible-auto")
            .unwrap();
        assert!(html.contains("kz-section collapsible-start"), "{html}");

        let html = kz
            .render_with_meta(code, "text collapse={9-12} collapse={3-1}")
            .unwrap();
        assert!(
            !html.contains("kz-section"),
            "invalid ranges dropped: {html}"
        );
        let one = kz.render_with_meta(code, "text collapse={2}").unwrap();
        assert!(
            one.contains("<span class=\"text\">1 collapsed line</span>"),
            "{one}"
        );
    }

    #[test]
    fn range_collapse_via_options() {
        let kz = test_engine();
        let html = kz
            .render(
                "a\nb\nc",
                &Options {
                    lang: "text".into(),
                    collapse: Some(CollapseSpec {
                        ranges: vec![LineRange::new(2, 3)],
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )
            .unwrap();
        assert!(html.contains("<details class=\"kz-section\">"), "{html}");
    }

    #[test]
    fn collapsible_theme_vars_and_toggle_gradient() {
        let kz = collapsible_engine(CollapsibleConfig::default());
        let css = kz.css();
        assert!(css.contains("--kz-collapse-btn-fg: #4b5563;"), "{css}");
        assert!(css.contains("--kz-collapse-gradient-end: var(--kz-editor-bg);"));
        let plain = test_engine().css();
        assert!(
            plain.contains("--kz-collapse-closed-bg:"),
            "static collapse vars always present"
        );
        assert!(!plain.contains("--kz-collapse-btn-border:"), "{plain}");

        let hl = iro::Highlighter::new().unwrap();
        let both = Kazari::builder(hl)
            .themes("github-light", Some("github-dark"))
            .theme_toggle(true)
            .collapsible(CollapsibleConfig::default())
            .build()
            .unwrap();
        let css = both.css();
        assert!(css.contains("--kz-collapse-btn-fg: #d4d4d8; "), "{css}");
        assert!(
            css.contains("--kz-collapse-gradient-end: var(--kz-editor-bg); }"),
            "{css}"
        );
    }

    #[test]
    fn file_icons_default_on_and_resolver() {
        let kz = test_engine();
        let html = kz.render_with_meta("x", "rust title=\"app.rs\"").unwrap();
        assert!(html.contains("<span class=\"kz-file-icon\" data-ext=\"rs\"></span><span class=\"kz-title\">app.rs</span>"), "{html}");
        assert!(kz.css().contains("--kz-file-icon-size: 1rem;"));
        assert!(kz.css().contains(".kz-file-icon"));
        let no_ext = kz.render_with_meta("x", "rust title=\"Makefile\"").unwrap();
        assert!(!no_ext.contains("kz-file-icon"), "{no_ext}");
        let no_title = kz.render_with_meta("x", "rust").unwrap();
        assert!(!no_title.contains("kz-file-icon"), "{no_title}");

        let hl = iro::Highlighter::new().unwrap();
        let custom = Kazari::builder(hl)
            .file_icon_resolver(|ext| format!("<i class=\"icon-{ext}\"></i>"))
            .build()
            .unwrap();
        let html = custom
            .render_with_meta("x", "go title=\"main.go\"")
            .unwrap();
        assert!(
            html.contains("<i class=\"icon-go\"></i><span class=\"kz-title\">main.go</span>"),
            "{html}"
        );

        let hl = iro::Highlighter::new().unwrap();
        let off = Kazari::builder(hl).file_icons(false).build().unwrap();
        let html = off.render_with_meta("x", "rust title=\"app.rs\"").unwrap();
        assert!(!html.contains("kz-file-icon"), "{html}");
        assert!(!off.css().contains("--kz-file-icon-size"));
        assert!(!off.css().contains(".kz-file-icon"));
    }

    #[test]
    fn lang_icon_modes() {
        let plain = test_engine().render_with_meta("x", "javascript").unwrap();
        assert!(plain.contains("<span class=\"kz-lang\">JavaScript</span>"));
        assert!(!plain.contains("kz-lang-icon"));
        assert!(!test_engine().css().contains("--kz-lang-icon-size"));

        let hl = iro::Highlighter::new().unwrap();
        let both = Kazari::builder(hl)
            .lang_icon_mode(LangIconMode::IconAndText)
            .build()
            .unwrap();
        let html = both.render_with_meta("x", "javascript").unwrap();
        assert!(html.contains("<span class=\"kz-lang-icon\" data-lang=\"javascript\"></span><span class=\"kz-lang\">JavaScript</span>"), "{html}");
        assert!(both.css().contains("--kz-lang-icon-size: 1.25rem;"));
        assert!(both.css().contains(".kz-lang-icon"));

        let hl = iro::Highlighter::new().unwrap();
        let icon = Kazari::builder(hl)
            .lang_icon_mode(LangIconMode::IconOnly)
            .build()
            .unwrap();
        let html = icon.render_with_meta("x", "javascript").unwrap();
        assert!(
            html.contains("<span class=\"kz-lang-icon\" data-lang=\"javascript\"></span></div>"),
            "{html}"
        );
        assert!(!html.contains("<span class=\"kz-lang\">"), "{html}");
    }

    #[test]
    fn css_builder_options() {
        let hl = iro::Highlighter::new().unwrap();
        let mut plain = BTreeMap::new();
        plain.insert("--kz-radius".to_owned(), "0".to_owned());
        let mut themed = BTreeMap::new();
        themed.insert(
            "--kz-editor-bg".to_owned(),
            ("#fafafa".to_owned(), "#101010".to_owned()),
        );
        let kz = Kazari::builder(hl)
            .cascade_layer("")
            .theme_css_root("[data-docs]")
            .style_overrides(plain)
            .themed_style_overrides(themed)
            .build()
            .unwrap();
        let css = kz.css();
        assert!(!css.contains("@layer"));
        assert!(css.starts_with("[data-docs] {\n"), "{}", &css[..40]);
        assert!(css.contains("[data-docs].dark {\n"));
        assert!(css.contains("  --kz-radius: 0;\n"));
        assert!(css.contains("  --kz-editor-bg: #fafafa;\n"));
        assert!(css.contains("  --kz-editor-bg: #101010;\n"));
    }

    #[test]
    fn builder_config_file() {
        let yaml = r#"
themes:
  light: github-light
  dark: github-dark
copyButton: false
tabWidth: 4
"#;
        let hl = iro::Highlighter::new().unwrap();
        let kz = Kazari::builder(hl)
            .config_file(yaml)
            .unwrap()
            .build()
            .unwrap();
        assert!(!kz.config().copy_button);
        assert_eq!(kz.config().tab_width, 4);
        let html = kz.render_with_meta("let x = 1;", "javascript").unwrap();
        assert!(html.contains("kazari-block"));
    }
}
