use crate::config::{Config, ResolvedBlock};
use crate::diff;
use crate::error::Error;
use crate::frame;
use crate::meta;
use crate::notation;
use crate::render;
use crate::tokenize::{self, expand_tabs};
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
}

pub struct Kazari {
    highlighter: iro::Highlighter,
    config: Config,
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

        self.render_resolved(code, &mut resolved)
    }

    pub fn render(&self, code: &str, options: &Options) -> Result<String, Error> {
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
        };

        let mut resolved = self.config.resolve(&lang, Some(&block_opts));
        resolved.line_markers = options.line_markers.clone();
        resolved.inline_markers = options.inline_markers.clone();
        resolved.focus_lines = options.focus_lines.clone();

        self.render_resolved(code, &mut resolved)
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

    pub fn light_theme_info(&self) -> &ThemeInfo {
        &self.light_info
    }

    pub fn dark_theme_info(&self) -> Option<&ThemeInfo> {
        self.dark_info.as_ref()
    }

    fn render_resolved(&self, code: &str, resolved: &mut ResolvedBlock) -> Result<String, Error> {
        let code = expand_tabs(code, self.config.tab_width);

        if self.config.file_name_extraction
            && resolved.title.is_empty()
            && let Some((title, modified)) = frame::extract_file_name(&code, &resolved.lang)
        {
            resolved.title = title;
            resolved.raw_code =
                if self.config.terminal_comment_stripping && resolved.frame == Frame::Terminal {
                    frame::strip_terminal_comments(&modified)
                } else {
                    modified.clone()
                };
            return self.tokenize_and_render(&modified, resolved);
        }

        resolved.raw_code =
            if self.config.terminal_comment_stripping && resolved.frame == Frame::Terminal {
                frame::strip_terminal_comments(&code)
            } else {
                code.clone()
            };

        self.tokenize_and_render(&code, resolved)
    }

    fn tokenize_and_render(
        &self,
        code: &str,
        resolved: &mut ResolvedBlock,
    ) -> Result<String, Error> {
        let (code, lang) = if resolved.lang == "diff" && !resolved.diff_lang.is_empty() {
            let (stripped, markers) = diff::process_diff_block(code);
            resolved.line_markers.extend(markers);
            resolved.lang = self.config.resolve_language(&resolved.diff_lang);
            (stripped, resolved.lang.clone())
        } else {
            (code.to_owned(), resolved.lang.clone())
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

        if self.config.frame_detection {
            resolved.frame = frame::detect_frame_type(&code, &lang, resolved.frame);
        }

        let themes = if !resolved.theme.is_empty() {
            Themes::parse_override(&resolved.theme)
        } else {
            Themes {
                light: self.config.light_theme.clone(),
                dark: self.config.dark_theme.clone(),
            }
        };

        let tokens = tokenize::tokenize(&self.highlighter, &code, &lang, &themes)?;

        Ok(render::render_block(&tokens, resolved, &self.config))
    }
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

        Ok(Kazari {
            highlighter: self.highlighter,
            config: self.config,
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
