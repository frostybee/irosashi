use std::collections::{BTreeMap, HashMap};

use crate::meta::BlockOptions;
use crate::types::{DarkMode, Frame, InlineMarker, LineMarker, LineRange, TerminalDotStyle};

#[derive(Debug, Clone)]
pub struct BlockDefaults {
    pub wrap: bool,
    pub preserve_indent: bool,
    pub hanging_indent: usize,
    pub line_numbers: bool,
    pub frame: Frame,
}

impl Default for BlockDefaults {
    fn default() -> Self {
        Self {
            wrap: false,
            preserve_indent: true,
            hanging_indent: 0,
            line_numbers: false,
            frame: Frame::Auto,
        }
    }
}

pub struct Config {
    pub light_theme: String,
    pub dark_theme: Option<String>,
    pub dark_mode: DarkMode,
    pub copy_button: bool,
    pub wrap_button: bool,
    pub frame_detection: bool,
    pub file_name_extraction: bool,
    pub language_badge: bool,
    pub style_reset: bool,
    pub themed_scrollbars: bool,
    pub themed_selection: bool,
    pub terminal_dot_style: TerminalDotStyle,
    pub terminal_comment_stripping: bool,
    pub data_line_count: bool,
    pub tab_width: usize,
    pub defaults: BlockDefaults,
    pub language_defaults: BTreeMap<String, BlockDefaults>,
    pub language_aliases: HashMap<String, String>,
    #[allow(clippy::type_complexity)]
    pub warning_handler: Option<Box<dyn Fn(&str) + Send + Sync>>,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("light_theme", &self.light_theme)
            .field("dark_theme", &self.dark_theme)
            .finish_non_exhaustive()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            light_theme: "github-light".to_owned(),
            dark_theme: Some("github-dark".to_owned()),
            dark_mode: DarkMode::default(),
            copy_button: true,
            wrap_button: true,
            frame_detection: true,
            file_name_extraction: true,
            language_badge: true,
            style_reset: true,
            themed_scrollbars: true,
            themed_selection: false,
            terminal_dot_style: TerminalDotStyle::Colored,
            terminal_comment_stripping: true,
            data_line_count: true,
            tab_width: 2,
            defaults: BlockDefaults::default(),
            language_defaults: BTreeMap::new(),
            language_aliases: HashMap::new(),
            warning_handler: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ResolvedBlock {
    pub lang: String,
    pub title: String,
    pub theme: String,
    pub diff_lang: String,
    pub frame: Frame,
    pub line_numbers: bool,
    pub start_line_number: usize,
    pub wrap: bool,
    pub preserve_indent: bool,
    pub hanging_indent: usize,
    pub raw_code: String,
    pub line_markers: Vec<LineMarker>,
    pub inline_markers: Vec<InlineMarker>,
    pub focus_lines: Vec<LineRange>,
}

impl Config {
    pub fn resolve(&self, lang: &str, block_opts: Option<&BlockOptions>) -> ResolvedBlock {
        let mut resolved = ResolvedBlock {
            lang: lang.to_owned(),
            frame: self.defaults.frame,
            line_numbers: self.defaults.line_numbers,
            start_line_number: 1,
            wrap: self.defaults.wrap,
            preserve_indent: self.defaults.preserve_indent,
            hanging_indent: self.defaults.hanging_indent,
            ..Default::default()
        };

        for (key, lang_def) in &self.language_defaults {
            if matches_language_key(key, lang) {
                resolved.frame = lang_def.frame;
                resolved.line_numbers = lang_def.line_numbers;
                resolved.wrap = lang_def.wrap;
                resolved.preserve_indent = lang_def.preserve_indent;
                resolved.hanging_indent = lang_def.hanging_indent;
                break;
            }
        }

        if let Some(opts) = block_opts {
            if !opts.lang.is_empty() {
                resolved.lang = opts.lang.clone();
            }
            if !opts.title.is_empty() {
                resolved.title = opts.title.clone();
            }
            if !opts.theme.is_empty() {
                resolved.theme = opts.theme.clone();
            }
            if let Some(frame) = opts.frame {
                resolved.frame = frame;
            }
            if let Some(ln) = opts.line_numbers {
                resolved.line_numbers = ln;
            }
            if let Some(n) = opts.start_line_number {
                resolved.start_line_number = n;
            }
            if let Some(w) = opts.wrap {
                resolved.wrap = w;
            }
            if let Some(pi) = opts.preserve_indent {
                resolved.preserve_indent = pi;
            }
            if let Some(hi) = opts.hanging_indent {
                resolved.hanging_indent = hi;
            }
        }

        resolved
    }

    pub fn resolve_language(&self, lang: &str) -> String {
        let lower = lang.to_lowercase();
        if let Some(canonical) = self.language_aliases.get(&lower) {
            return canonical.clone();
        }
        lower
    }

    pub fn warn(&self, msg: &str) {
        if let Some(handler) = &self.warning_handler {
            handler(msg);
        }
    }
}

fn matches_language_key(key: &str, lang: &str) -> bool {
    let lower = lang.to_lowercase();
    key.split(',').any(|k| k.trim().to_lowercase() == lower)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_resolves_with_defaults() {
        let cfg = Config::default();
        let r = cfg.resolve("js", None);
        assert_eq!(r.lang, "js");
        assert_eq!(r.frame, Frame::Auto);
        assert!(!r.line_numbers);
        assert!(!r.wrap);
        assert!(r.preserve_indent);
        assert_eq!(r.hanging_indent, 0);
        assert_eq!(r.start_line_number, 1);
    }

    #[test]
    fn language_defaults_override_engine_defaults() {
        let mut cfg = Config::default();
        cfg.defaults.line_numbers = false;
        cfg.language_defaults.insert(
            "bash,sh,zsh".to_owned(),
            BlockDefaults {
                line_numbers: true,
                frame: Frame::Terminal,
                ..Default::default()
            },
        );
        let r = cfg.resolve("sh", None);
        assert!(r.line_numbers);
        assert_eq!(r.frame, Frame::Terminal);
    }

    #[test]
    fn language_defaults_first_match_wins() {
        let mut cfg = Config::default();
        cfg.language_defaults.insert(
            "js".to_owned(),
            BlockDefaults {
                line_numbers: true,
                ..Default::default()
            },
        );
        cfg.language_defaults.insert(
            "js,ts".to_owned(),
            BlockDefaults {
                line_numbers: false,
                wrap: true,
                ..Default::default()
            },
        );
        let r = cfg.resolve("js", None);
        assert!(r.line_numbers);
        assert!(!r.wrap);
    }

    #[test]
    fn block_options_override_language_defaults() {
        let mut cfg = Config::default();
        cfg.language_defaults.insert(
            "go".to_owned(),
            BlockDefaults {
                line_numbers: true,
                ..Default::default()
            },
        );
        let opts = BlockOptions {
            line_numbers: Some(false),
            wrap: Some(true),
            ..Default::default()
        };
        let r = cfg.resolve("go", Some(&opts));
        assert!(!r.line_numbers);
        assert!(r.wrap);
    }

    #[test]
    fn block_options_none_fields_keep_defaults() {
        let mut cfg = Config::default();
        cfg.defaults.line_numbers = true;
        let opts = BlockOptions::default();
        let r = cfg.resolve("js", Some(&opts));
        assert!(r.line_numbers);
    }

    #[test]
    fn resolve_language_applies_aliases() {
        let mut cfg = Config::default();
        cfg.language_aliases
            .insert("typescript".to_owned(), "ts".to_owned());
        assert_eq!(cfg.resolve_language("TypeScript"), "ts");
        assert_eq!(cfg.resolve_language("go"), "go");
    }

    #[test]
    fn matches_language_key_case_insensitive() {
        assert!(matches_language_key("bash,sh,zsh", "SH"));
        assert!(matches_language_key("JavaScript", "javascript"));
        assert!(!matches_language_key("bash,sh", "zsh"));
    }

    #[test]
    fn block_options_title_and_theme() {
        let cfg = Config::default();
        let opts = BlockOptions {
            title: "app.rs".to_owned(),
            theme: "monokai".to_owned(),
            start_line_number: Some(10),
            ..Default::default()
        };
        let r = cfg.resolve("rust", Some(&opts));
        assert_eq!(r.title, "app.rs");
        assert_eq!(r.theme, "monokai");
        assert_eq!(r.start_line_number, 10);
    }

    #[test]
    fn full_cascade() {
        let mut cfg = Config {
            defaults: BlockDefaults {
                wrap: false,
                preserve_indent: true,
                hanging_indent: 0,
                line_numbers: false,
                frame: Frame::Auto,
            },
            ..Default::default()
        };
        cfg.language_defaults.insert(
            "bash,sh".to_owned(),
            BlockDefaults {
                frame: Frame::Terminal,
                line_numbers: true,
                ..Default::default()
            },
        );
        let opts = BlockOptions {
            frame: Some(Frame::Code),
            hanging_indent: Some(4),
            ..Default::default()
        };
        let r = cfg.resolve("bash", Some(&opts));
        assert_eq!(r.frame, Frame::Code);
        assert!(r.line_numbers);
        assert_eq!(r.hanging_indent, 4);
        assert!(!r.wrap);
    }
}
