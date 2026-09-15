use std::collections::{BTreeMap, HashMap};

use serde::Deserialize;

use crate::error::Error;
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

// ---------------------------------------------------------------------------
// YAML file config
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ThemesFile {
    pub light: String,
    #[serde(default)]
    pub dark: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DarkModeFile {
    pub kind: String,
    #[serde(default)]
    pub selector: String,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct BlockDefaultsFile {
    pub wrap: Option<bool>,
    pub preserve_indent: Option<bool>,
    pub hanging_indent: Option<usize>,
    pub line_numbers: Option<bool>,
    pub frame: Option<Frame>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FileConfig {
    pub themes: Option<ThemesFile>,
    pub dark_mode: Option<DarkModeFile>,
    pub copy_button: Option<bool>,
    pub wrap_button: Option<bool>,
    pub line_numbers: Option<bool>,
    pub frame_detection: Option<bool>,
    pub file_name_extraction: Option<bool>,
    pub language_badge: Option<bool>,
    pub themed_scrollbars: Option<bool>,
    pub themed_selection: Option<bool>,
    pub terminal_comment_stripping: Option<bool>,
    pub data_line_count: Option<bool>,
    pub style_reset: Option<bool>,
    pub tab_width: Option<usize>,
    pub terminal_dot_style: Option<TerminalDotStyle>,
    pub defaults: Option<BlockDefaultsFile>,
    pub language_defaults: Option<BTreeMap<String, BlockDefaultsFile>>,
    pub language_aliases: Option<HashMap<String, String>>,
}

impl FileConfig {
    pub fn from_yaml(yaml_str: &str) -> Result<Self, Error> {
        let fc: Self =
            serde_yaml_ng::from_str(yaml_str).map_err(|e| Error::Config(e.to_string()))?;
        fc.validate()?;
        Ok(fc)
    }

    pub fn apply(self, cfg: &mut Config) -> Result<(), Error> {
        if let Some(themes) = self.themes {
            cfg.light_theme = themes.light;
            cfg.dark_theme = themes.dark;
        }
        if let Some(dm) = self.dark_mode {
            cfg.dark_mode = parse_dark_mode_file(&dm)?;
        }
        if let Some(v) = self.copy_button {
            cfg.copy_button = v;
        }
        if let Some(v) = self.wrap_button {
            cfg.wrap_button = v;
        }
        if let Some(v) = self.line_numbers {
            cfg.defaults.line_numbers = v;
        }
        if let Some(v) = self.frame_detection {
            cfg.frame_detection = v;
        }
        if let Some(v) = self.file_name_extraction {
            cfg.file_name_extraction = v;
        }
        if let Some(v) = self.language_badge {
            cfg.language_badge = v;
        }
        if let Some(v) = self.themed_scrollbars {
            cfg.themed_scrollbars = v;
        }
        if let Some(v) = self.themed_selection {
            cfg.themed_selection = v;
        }
        if let Some(v) = self.terminal_comment_stripping {
            cfg.terminal_comment_stripping = v;
        }
        if let Some(v) = self.data_line_count {
            cfg.data_line_count = v;
        }
        if let Some(v) = self.style_reset {
            cfg.style_reset = v;
        }
        if let Some(v) = self.tab_width {
            cfg.tab_width = v;
        }
        if let Some(v) = self.terminal_dot_style {
            cfg.terminal_dot_style = v;
        }
        if let Some(defaults) = self.defaults {
            apply_block_defaults_file(&defaults, &mut cfg.defaults);
        }
        if let Some(lang_defs) = self.language_defaults {
            for (composite_key, bd_file) in lang_defs {
                for lang in split_comma_key(&composite_key) {
                    let entry = cfg.language_defaults.entry(lang).or_default();
                    apply_block_defaults_file(&bd_file, entry);
                }
            }
        }
        if let Some(aliases) = self.language_aliases {
            cfg.language_aliases.extend(aliases);
        }
        Ok(())
    }

    fn validate(&self) -> Result<(), Error> {
        if let Some(dm) = &self.dark_mode {
            let valid = ["selector", "mediaQuery", "both"];
            if !valid.contains(&dm.kind.as_str()) {
                return Err(Error::Config(format!(
                    "darkMode.kind must be one of {valid:?}, got {:?}",
                    dm.kind
                )));
            }
            if (dm.kind == "selector" || dm.kind == "both") && dm.selector.is_empty() {
                return Err(Error::Config(format!(
                    "darkMode.selector is required when kind is {:?}",
                    dm.kind
                )));
            }
        }
        if let Some(tw) = self.tab_width
            && tw == 0
        {
            return Err(Error::Config("tabWidth must be at least 1".into()));
        }
        if let Some(themes) = &self.themes
            && themes.light.is_empty()
        {
            return Err(Error::Config("themes.light is required".into()));
        }
        Ok(())
    }
}

fn parse_dark_mode_file(dm: &DarkModeFile) -> Result<DarkMode, Error> {
    match dm.kind.as_str() {
        "selector" => Ok(DarkMode::Selector(dm.selector.clone())),
        "mediaQuery" => Ok(DarkMode::MediaQuery),
        "both" => Ok(DarkMode::Both(dm.selector.clone())),
        other => Err(Error::Config(format!("unknown darkMode kind: {other}"))),
    }
}

fn apply_block_defaults_file(file: &BlockDefaultsFile, target: &mut BlockDefaults) {
    if let Some(v) = file.wrap {
        target.wrap = v;
    }
    if let Some(v) = file.preserve_indent {
        target.preserve_indent = v;
    }
    if let Some(v) = file.hanging_indent {
        target.hanging_indent = v;
    }
    if let Some(v) = file.line_numbers {
        target.line_numbers = v;
    }
    if let Some(v) = file.frame {
        target.frame = v;
    }
}

fn split_comma_key(key: &str) -> Vec<String> {
    key.split(',')
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .collect()
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

    // -- FileConfig tests --

    #[test]
    fn file_config_full_yaml() {
        let yaml = r#"
themes:
  light: github-light
  dark: github-dark
darkMode:
  kind: selector
  selector: ".dark"
copyButton: false
wrapButton: true
lineNumbers: true
frameDetection: false
fileNameExtraction: true
languageBadge: false
themedScrollbars: true
themedSelection: true
terminalCommentStripping: false
dataLineCount: false
styleReset: true
tabWidth: 4
terminalDotStyle: minimal
defaults:
  wrap: true
  preserveIndent: false
  hangingIndent: 2
  lineNumbers: false
  frame: code
languageDefaults:
  "bash,sh":
    frame: terminal
    lineNumbers: false
languageAliases:
  ts: typescript
  js: javascript
"#;
        let fc = FileConfig::from_yaml(yaml).unwrap();
        let mut cfg = Config::default();
        fc.apply(&mut cfg).unwrap();

        assert_eq!(cfg.light_theme, "github-light");
        assert_eq!(cfg.dark_theme.as_deref(), Some("github-dark"));
        assert_eq!(cfg.dark_mode, DarkMode::Selector(".dark".into()));
        assert!(!cfg.copy_button);
        assert!(cfg.wrap_button);
        assert!(!cfg.frame_detection);
        assert!(cfg.file_name_extraction);
        assert!(!cfg.language_badge);
        assert!(cfg.themed_scrollbars);
        assert!(cfg.themed_selection);
        assert!(!cfg.terminal_comment_stripping);
        assert!(!cfg.data_line_count);
        assert!(cfg.style_reset);
        assert_eq!(cfg.tab_width, 4);
        assert_eq!(cfg.terminal_dot_style, TerminalDotStyle::Minimal);
        assert!(cfg.defaults.wrap);
        assert!(!cfg.defaults.preserve_indent);
        assert_eq!(cfg.defaults.hanging_indent, 2);
        assert!(!cfg.defaults.line_numbers);
        assert_eq!(cfg.defaults.frame, Frame::Code);
        assert!(cfg.language_defaults.contains_key("bash"));
        assert!(cfg.language_defaults.contains_key("sh"));
        assert_eq!(cfg.language_defaults["bash"].frame, Frame::Terminal);
        assert_eq!(cfg.language_aliases.get("ts").unwrap(), "typescript");
    }

    #[test]
    fn file_config_minimal_yaml() {
        let yaml = "themes:\n  light: dracula\n";
        let fc = FileConfig::from_yaml(yaml).unwrap();
        let mut cfg = Config::default();
        let orig_copy = cfg.copy_button;
        fc.apply(&mut cfg).unwrap();
        assert_eq!(cfg.light_theme, "dracula");
        assert!(cfg.dark_theme.is_none());
        assert_eq!(cfg.copy_button, orig_copy);
    }

    #[test]
    fn file_config_apply_merges() {
        let mut cfg = Config::default();
        cfg.copy_button = false;
        cfg.tab_width = 8;
        let yaml = "tabWidth: 4\n";
        let fc = FileConfig::from_yaml(yaml).unwrap();
        fc.apply(&mut cfg).unwrap();
        assert_eq!(cfg.tab_width, 4);
        assert!(!cfg.copy_button);
    }

    #[test]
    fn file_config_dark_mode_variants() {
        let selector_yaml = "darkMode:\n  kind: selector\n  selector: '[data-theme=\"dark\"]'\n";
        let fc = FileConfig::from_yaml(selector_yaml).unwrap();
        let mut cfg = Config::default();
        fc.apply(&mut cfg).unwrap();
        assert_eq!(
            cfg.dark_mode,
            DarkMode::Selector("[data-theme=\"dark\"]".into())
        );

        let mq_yaml = "darkMode:\n  kind: mediaQuery\n";
        let fc = FileConfig::from_yaml(mq_yaml).unwrap();
        let mut cfg = Config::default();
        fc.apply(&mut cfg).unwrap();
        assert_eq!(cfg.dark_mode, DarkMode::MediaQuery);

        let both_yaml = "darkMode:\n  kind: both\n  selector: .dark\n";
        let fc = FileConfig::from_yaml(both_yaml).unwrap();
        let mut cfg = Config::default();
        fc.apply(&mut cfg).unwrap();
        assert_eq!(cfg.dark_mode, DarkMode::Both(".dark".into()));
    }

    #[test]
    fn file_config_language_defaults_comma_split() {
        let yaml = "languageDefaults:\n  \"bash,sh,zsh\":\n    frame: terminal\n";
        let fc = FileConfig::from_yaml(yaml).unwrap();
        let mut cfg = Config::default();
        fc.apply(&mut cfg).unwrap();
        assert!(cfg.language_defaults.contains_key("bash"));
        assert!(cfg.language_defaults.contains_key("sh"));
        assert!(cfg.language_defaults.contains_key("zsh"));
        assert_eq!(cfg.language_defaults["bash"].frame, Frame::Terminal);
    }

    #[test]
    fn file_config_language_defaults_merge() {
        let mut cfg = Config::default();
        cfg.language_defaults.insert(
            "go".into(),
            BlockDefaults {
                wrap: true,
                line_numbers: false,
                ..Default::default()
            },
        );
        let yaml = "languageDefaults:\n  go:\n    lineNumbers: true\n";
        let fc = FileConfig::from_yaml(yaml).unwrap();
        fc.apply(&mut cfg).unwrap();
        assert!(cfg.language_defaults["go"].line_numbers);
        assert!(cfg.language_defaults["go"].wrap);
    }

    #[test]
    fn file_config_unknown_key_rejected() {
        let yaml = "unknownKey: true\n";
        assert!(FileConfig::from_yaml(yaml).is_err());
    }

    #[test]
    fn file_config_invalid_dark_mode_kind() {
        let yaml = "darkMode:\n  kind: invalid\n  selector: .dark\n";
        assert!(FileConfig::from_yaml(yaml).is_err());
    }

    #[test]
    fn file_config_tab_width_zero_rejected() {
        let yaml = "tabWidth: 0\n";
        assert!(FileConfig::from_yaml(yaml).is_err());
    }

    #[test]
    fn file_config_dark_mode_selector_required() {
        let yaml = "darkMode:\n  kind: selector\n";
        assert!(FileConfig::from_yaml(yaml).is_err());
    }

    #[test]
    fn file_config_themes_light_required() {
        let yaml = "themes:\n  light: \"\"\n";
        assert!(FileConfig::from_yaml(yaml).is_err());
    }
}
