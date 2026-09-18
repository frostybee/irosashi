use std::collections::{BTreeMap, HashMap};

use serde::Deserialize;

use crate::error::Error;
use crate::meta::BlockOptions;
use crate::types::{
    DarkMode, Frame, InlineMarker, LineMarker, LineRange, LinkAnnotation, MarkerType,
    TerminalDotStyle, ThemeInfo,
};

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
    /// Fullscreen toggle plus font size controls in the toolbar.
    pub fullscreen_button: bool,
    /// Per-block light/dark toggle; rendered only when a dark theme is set.
    pub theme_toggle: bool,
    /// Allow `withOutput` blocks to split at the output separator and render the
    /// remainder in an output panel.
    pub output_panel: bool,
    pub output_default_collapsed: bool,
    /// Line that separates code from output; empty means `---output---`.
    pub output_separator: String,
    /// Turn `@[text](url)` in the source into links.
    pub inline_links: bool,
    /// Ship the tab bar CSS and JS for `:::code-group` containers and let the
    /// markdown adapter parse them.
    pub code_groups: bool,
    /// Render `mermaid` fences as a bare `<pre class="mermaid">` for a client-side
    /// Mermaid script instead of highlighting them.
    pub mermaid_pass_through: bool,
    /// Minimum WCAG contrast ratio of token colours against the editor background
    /// (or the marker background on marked lines); 0 disables the adjustment.
    pub min_contrast: f64,
    /// Strip comments and whitespace from the generated CSS and JS.
    pub minify: bool,
    /// Threshold collapsing of long blocks. Range collapse (`collapse={3-5}`) works
    /// without it.
    pub collapsible: Option<CollapsibleConfig>,
    /// Emit a `kz-file-icon` placeholder (with `data-ext`) before titles that have an
    /// extension; the site's CSS supplies the image.
    pub file_icons: bool,
    /// Replaces the placeholder with custom markup for the given extension.
    #[allow(clippy::type_complexity)]
    pub file_icon_resolver: Option<Box<dyn Fn(&str) -> String + Send + Sync>>,
    pub lang_icon_mode: LangIconMode,
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
    /// Apply `[!code ++]`, `[!code highlight]`, `[!code focus]`, `[!code word:x]` and
    /// the other comment annotations found in the source.
    pub notation_comments: bool,
    /// Render every tab and space as a visible symbol.
    pub visible_whitespace: bool,
    pub whitespace_tab: String,
    pub whitespace_space: String,
    /// Locale for the UI strings (`en-US`, `fr-FR`, `ja-JP`; unknown falls back to
    /// `en-US`), resolved once when the engine is built.
    pub locale: String,
    /// Per-key overrides of the UI strings, keyed like `copy.label`.
    pub ui_string_overrides: HashMap<String, String>,
    /// Name of the CSS cascade layer the generated stylesheet is wrapped in; empty
    /// disables the wrapper.
    pub cascade_layer: String,
    /// Selector that carries the theme variables.
    pub theme_css_root: String,
    /// CSS variable overrides emitted inside the theme scopes, sorted by name.
    pub style_overrides: BTreeMap<String, StyleValue>,
    pub defaults: BlockDefaults,
    pub language_defaults: BTreeMap<String, BlockDefaults>,
    pub language_aliases: HashMap<String, String>,
    #[allow(clippy::type_complexity)]
    pub warning_handler: Option<Box<dyn Fn(&str) + Send + Sync>>,
}

/// The translucent line marker backgrounds, the same values as the static
/// `--kz-mark-bg`, `--kz-ins-bg` and `--kz-del-bg` variables.
pub(crate) const MARKER_BG_COLORS: [(MarkerType, &str); 3] = [
    (MarkerType::Mark, "rgba(255,200,0,0.12)"),
    (MarkerType::Ins, "rgba(46,160,67,0.12)"),
    (MarkerType::Del, "rgba(248,81,73,0.12)"),
];

/// Opaque marker backgrounds after compositing on an editor background.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct MarkerBgs {
    pub mark: String,
    pub ins: String,
    pub del: String,
}

impl MarkerBgs {
    /// Warning and error lines have no dedicated background.
    pub fn bg(&self, mt: MarkerType) -> Option<&str> {
        match mt {
            MarkerType::Mark => Some(&self.mark),
            MarkerType::Ins => Some(&self.ins),
            MarkerType::Del => Some(&self.del),
            MarkerType::Warning | MarkerType::Error => None,
        }
    }
}

pub(crate) fn compute_marker_bgs(editor_bg: &str) -> MarkerBgs {
    let composite = |mt: MarkerType| {
        let rgba = MARKER_BG_COLORS
            .iter()
            .find(|(m, _)| *m == mt)
            .map(|(_, v)| *v)
            .unwrap_or_default();
        match crate::color::rgba_to_hex(rgba) {
            Some(hex) => crate::color::on_background(&hex, editor_bg),
            None => editor_bg.to_owned(),
        }
    };
    MarkerBgs {
        mark: composite(MarkerType::Mark),
        ins: composite(MarkerType::Ins),
        del: composite(MarkerType::Del),
    }
}

/// How the language badge is shown: text only, an icon placeholder, or both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LangIconMode {
    #[default]
    None,
    IconOnly,
    IconAndText,
}

/// Visual style of a range-based collapsible section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CollapseStyle {
    /// One-way expand; the summary disappears once opened.
    #[default]
    Github,
    /// Re-collapsible, summary above the content.
    CollapsibleStart,
    /// Re-collapsible, summary below the content.
    CollapsibleEnd,
    /// `CollapsibleEnd` when the range reaches the last line, else `CollapsibleStart`.
    CollapsibleAuto,
}

/// Threshold-based collapsing of long blocks; `Some` on `Config` enables it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollapsibleConfig {
    /// Blocks longer than this collapse (0 means 15).
    pub line_threshold: usize,
    /// Lines shown while collapsed (0 means 8).
    pub preview_lines: usize,
    pub default_collapsed: bool,
    /// Give range summaries the indentation of their content.
    pub preserve_indent: bool,
    pub style: CollapseStyle,
    /// Empty strings fall back to the locale.
    pub expand_button_text: String,
    pub collapse_button_text: String,
    pub expanded_announcement: String,
    pub collapsed_announcement: String,
}

impl Default for CollapsibleConfig {
    fn default() -> Self {
        Self {
            line_threshold: 15,
            preview_lines: 8,
            default_collapsed: true,
            preserve_indent: true,
            style: CollapseStyle::Github,
            expand_button_text: String::new(),
            collapse_button_text: String::new(),
            expanded_announcement: String::new(),
            collapsed_announcement: String::new(),
        }
    }
}

/// Per-block collapse directives from the fence meta or `Options`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CollapseSpec {
    pub enabled: bool,
    pub disabled: bool,
    pub ranges: Vec<LineRange>,
    pub style: Option<CollapseStyle>,
    pub threshold: Option<usize>,
}

/// A validated range-based section, ready to render.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollapseRange {
    pub start: usize,
    pub end: usize,
    pub line_count: usize,
    pub min_indent: usize,
    pub style: CollapseStyle,
}

/// A contiguous run of lines that stays visible in a threshold preview.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PreviewSegment {
    pub start: usize,
    pub end: usize,
}

/// A CSS variable override that is either the same in both themes or split per
/// theme.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StyleValue {
    pub value: String,
    pub light: String,
    pub dark: String,
}

impl StyleValue {
    pub fn plain(value: &str) -> Self {
        Self {
            value: value.to_owned(),
            ..Default::default()
        }
    }

    pub fn themed(light: &str, dark: &str) -> Self {
        Self {
            value: String::new(),
            light: light.to_owned(),
            dark: dark.to_owned(),
        }
    }

    pub fn is_themed(&self) -> bool {
        !self.light.is_empty() || !self.dark.is_empty()
    }

    pub fn light_value(&self) -> &str {
        if self.is_themed() {
            &self.light
        } else {
            &self.value
        }
    }

    pub fn dark_value(&self) -> &str {
        if self.is_themed() {
            &self.dark
        } else {
            &self.value
        }
    }
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
            fullscreen_button: true,
            theme_toggle: false,
            output_panel: false,
            output_default_collapsed: false,
            output_separator: String::new(),
            inline_links: false,
            code_groups: false,
            mermaid_pass_through: true,
            min_contrast: 0.0,
            minify: true,
            collapsible: None,
            file_icons: true,
            file_icon_resolver: None,
            lang_icon_mode: LangIconMode::None,
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
            notation_comments: false,
            visible_whitespace: false,
            whitespace_tab: "\u{2192}".to_owned(),
            whitespace_space: "\u{b7}".to_owned(),
            locale: "en-US".to_owned(),
            ui_string_overrides: HashMap::new(),
            cascade_layer: "kazari".to_owned(),
            theme_css_root: ":root".to_owned(),
            style_overrides: BTreeMap::new(),
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
    /// Inline `--kz-ovl-*` / `--kz-ovd-*` declarations for a per-block theme
    /// override; empty when the block uses the page themes.
    pub theme_override_style: String,
    /// The theme colours token contrast is measured against (the override theme
    /// when one applies, else the page theme); set only when `min_contrast` is on.
    pub contrast_light: Option<ThemeInfo>,
    pub contrast_dark: Option<ThemeInfo>,
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
    pub with_output: bool,
    pub output_collapsed: bool,
    pub output_label: String,
    pub output_text: String,
    /// Link annotations per source line, byte offsets into the cleaned line.
    pub links: Vec<Vec<LinkAnnotation>>,
    pub collapse_spec: Option<CollapseSpec>,
    pub collapse_threshold: bool,
    pub collapse_ranges: Vec<CollapseRange>,
    pub collapse_segments: Vec<PreviewSegment>,
    /// Marked lines past the preview cap, shown as a badge on the expand button.
    pub collapse_beyond_cap: usize,
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
            output_collapsed: self.output_default_collapsed,
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
            if let Some(v) = opts.with_output {
                resolved.with_output = v;
            }
            if let Some(v) = opts.output_collapsed {
                resolved.output_collapsed = v;
            }
            if !opts.output_label.is_empty() {
                resolved.output_label = opts.output_label.clone();
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

/// A `styleOverrides` entry: a bare string or a `{ light, dark }` map.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum StyleValueFile {
    Themed {
        #[serde(default)]
        light: String,
        #[serde(default)]
        dark: String,
    },
    Plain(String),
    Int(i64),
    Float(f64),
}

impl From<StyleValueFile> for StyleValue {
    fn from(v: StyleValueFile) -> Self {
        match v {
            StyleValueFile::Themed { light, dark } => StyleValue::themed(&light, &dark),
            StyleValueFile::Plain(value) => StyleValue::plain(&value),
            StyleValueFile::Int(n) => StyleValue::plain(&n.to_string()),
            StyleValueFile::Float(n) => StyleValue::plain(&n.to_string()),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CollapsibleFile {
    pub line_threshold: Option<usize>,
    pub preview_lines: Option<usize>,
    pub default_collapsed: Option<bool>,
    pub preserve_indent: Option<bool>,
    pub style: Option<CollapseStyle>,
    pub expand_button_text: Option<String>,
    pub collapse_button_text: Option<String>,
    pub expanded_announcement: Option<String>,
    pub collapsed_announcement: Option<String>,
}

/// The `process` section of a config file. It configures the `kazari process` command
/// rather than the engine, so `apply` ignores it and the CLI reads it directly.
#[derive(Debug, Deserialize, Default, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProcessFile {
    pub skip_unlabeled: Option<bool>,
    pub assets_base: Option<String>,
    pub hashed_assets: Option<bool>,
    pub concurrency: Option<usize>,
    pub max_file_bytes: Option<u64>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FileConfig {
    pub themes: Option<ThemesFile>,
    pub collapsible: Option<CollapsibleFile>,
    pub locale: Option<String>,
    pub ui_strings: Option<HashMap<String, String>>,
    pub cascade_layer: Option<String>,
    pub theme_css_root: Option<String>,
    pub style_overrides: Option<BTreeMap<String, StyleValueFile>>,
    pub dark_mode: Option<DarkModeFile>,
    pub copy_button: Option<bool>,
    pub wrap_button: Option<bool>,
    pub fullscreen_button: Option<bool>,
    pub theme_toggle: Option<bool>,
    pub output_panel: Option<bool>,
    pub output_default_collapsed: Option<bool>,
    pub output_separator: Option<String>,
    pub inline_links: Option<bool>,
    pub code_groups: Option<bool>,
    pub mermaid_pass_through: Option<bool>,
    pub min_contrast: Option<f64>,
    pub minify: Option<bool>,
    pub file_icons: Option<bool>,
    pub lang_icon_mode: Option<LangIconMode>,
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
    pub notation_comments: Option<bool>,
    pub visible_whitespace: Option<bool>,
    pub whitespace_tab: Option<String>,
    pub whitespace_space: Option<String>,
    pub defaults: Option<BlockDefaultsFile>,
    pub language_defaults: Option<BTreeMap<String, BlockDefaultsFile>>,
    pub language_aliases: Option<HashMap<String, String>>,
    pub process: Option<ProcessFile>,
}

impl FileConfig {
    pub fn from_yaml(yaml_str: &str) -> Result<Self, Error> {
        let fc: Self =
            serde_yaml_ng::from_str(yaml_str).map_err(|e| Error::Config(e.to_string()))?;
        fc.validate()?;
        Ok(fc)
    }

    pub fn from_json(json_str: &str) -> Result<Self, Error> {
        let fc: Self = serde_json::from_str(json_str).map_err(|e| Error::Config(e.to_string()))?;
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
        if let Some(v) = self.fullscreen_button {
            cfg.fullscreen_button = v;
        }
        if let Some(v) = self.theme_toggle {
            cfg.theme_toggle = v;
        }
        if let Some(v) = self.output_panel {
            cfg.output_panel = v;
        }
        if let Some(v) = self.output_default_collapsed {
            cfg.output_default_collapsed = v;
        }
        if let Some(v) = self.output_separator {
            cfg.output_separator = v;
        }
        if let Some(v) = self.inline_links {
            cfg.inline_links = v;
        }
        if let Some(v) = self.code_groups {
            cfg.code_groups = v;
        }
        if let Some(v) = self.mermaid_pass_through {
            cfg.mermaid_pass_through = v;
        }
        if let Some(v) = self.min_contrast {
            cfg.min_contrast = v;
        }
        if let Some(v) = self.minify {
            cfg.minify = v;
        }
        if let Some(v) = self.file_icons {
            cfg.file_icons = v;
        }
        if let Some(v) = self.lang_icon_mode {
            cfg.lang_icon_mode = v;
        }
        if let Some(file) = self.collapsible {
            let mut c = cfg.collapsible.take().unwrap_or_default();
            if let Some(v) = file.line_threshold {
                c.line_threshold = v;
            }
            if let Some(v) = file.preview_lines {
                c.preview_lines = v;
            }
            if let Some(v) = file.default_collapsed {
                c.default_collapsed = v;
            }
            if let Some(v) = file.preserve_indent {
                c.preserve_indent = v;
            }
            if let Some(v) = file.style {
                c.style = v;
            }
            if let Some(v) = file.expand_button_text {
                c.expand_button_text = v;
            }
            if let Some(v) = file.collapse_button_text {
                c.collapse_button_text = v;
            }
            if let Some(v) = file.expanded_announcement {
                c.expanded_announcement = v;
            }
            if let Some(v) = file.collapsed_announcement {
                c.collapsed_announcement = v;
            }
            cfg.collapsible = Some(c);
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
        if let Some(v) = self.notation_comments {
            cfg.notation_comments = v;
        }
        if let Some(v) = self.visible_whitespace {
            cfg.visible_whitespace = v;
        }
        if let Some(v) = self.whitespace_tab.filter(|s| !s.is_empty()) {
            cfg.whitespace_tab = v;
        }
        if let Some(v) = self.whitespace_space.filter(|s| !s.is_empty()) {
            cfg.whitespace_space = v;
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
        if let Some(v) = self.locale.filter(|s| !s.is_empty()) {
            cfg.locale = v;
        }
        if let Some(overrides) = self.ui_strings {
            cfg.ui_string_overrides.extend(overrides);
        }
        if let Some(v) = self.cascade_layer {
            cfg.cascade_layer = v;
        }
        if let Some(v) = self.theme_css_root.filter(|s| !s.is_empty()) {
            cfg.theme_css_root = v;
        }
        if let Some(overrides) = self.style_overrides {
            cfg.style_overrides
                .extend(overrides.into_iter().map(|(k, v)| (k, v.into())));
        }
        Ok(())
    }

    fn validate(&self) -> Result<(), Error> {
        if let Some(p) = &self.process {
            if p.concurrency == Some(0) {
                return Err(Error::Config(
                    "process.concurrency must be at least 1".into(),
                ));
            }
            if p.max_file_bytes == Some(0) {
                return Err(Error::Config(
                    "process.maxFileBytes must be at least 1".into(),
                ));
            }
        }
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
        if let Some(mc) = self.min_contrast
            && !(0.0..=21.0).contains(&mc)
        {
            return Err(Error::Config(format!(
                "minContrast must be between 0 and 21, got {mc}"
            )));
        }
        if let Some(themes) = &self.themes
            && themes.light.is_empty()
        {
            return Err(Error::Config("themes.light is required".into()));
        }
        if let Some(c) = &self.collapsible {
            if c.line_threshold == Some(0) {
                return Err(Error::Config(
                    "collapsible.lineThreshold must be at least 1".into(),
                ));
            }
            if c.preview_lines == Some(0) {
                return Err(Error::Config(
                    "collapsible.previewLines must be at least 1".into(),
                ));
            }
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
    fn file_config_notation_and_whitespace() {
        let yaml = "notationComments: true\nvisibleWhitespace: true\nwhitespaceTab: '>'\nwhitespaceSpace: ''\n";
        let fc = FileConfig::from_yaml(yaml).unwrap();
        let mut cfg = Config::default();
        fc.apply(&mut cfg).unwrap();
        assert!(cfg.notation_comments);
        assert!(cfg.visible_whitespace);
        assert_eq!(cfg.whitespace_tab, ">");
        assert_eq!(cfg.whitespace_space, "\u{b7}");
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
    fn file_config_process_section() {
        let yaml = "process:
  skipUnlabeled: true
  assetsBase: /static
  hashedAssets: true
  concurrency: 4
  maxFileBytes: 1024
";
        let fc = FileConfig::from_yaml(yaml).unwrap();
        let p = fc.process.clone().unwrap();
        assert_eq!(p.skip_unlabeled, Some(true));
        assert_eq!(p.assets_base.as_deref(), Some("/static"));
        assert_eq!(p.hashed_assets, Some(true));
        assert_eq!(p.concurrency, Some(4));
        assert_eq!(p.max_file_bytes, Some(1024));
        let mut cfg = Config::default();
        fc.apply(&mut cfg).unwrap();
        assert!(
            FileConfig::from_yaml(
                "process:
  concurrency: 0
"
            )
            .is_err()
        );
        assert!(
            FileConfig::from_yaml(
                "process:
  maxFileBytes: 0
"
            )
            .is_err()
        );
        assert!(
            FileConfig::from_yaml(
                "process:
  bogus: 1
"
            )
            .is_err()
        );
    }

    #[test]
    fn file_config_from_json() {
        let json = r#"{"themes":{"light":"dracula"},"process":{"concurrency":2}}"#;
        let fc = FileConfig::from_json(json).unwrap();
        assert_eq!(fc.process.as_ref().unwrap().concurrency, Some(2));
        let mut cfg = Config::default();
        fc.apply(&mut cfg).unwrap();
        assert_eq!(cfg.light_theme, "dracula");
        assert!(FileConfig::from_json("{").is_err());
    }

    #[test]
    fn file_config_apply_merges() {
        let mut cfg = Config {
            copy_button: false,
            tab_width: 8,
            ..Default::default()
        };
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
    fn file_config_locale_and_css_fields() {
        let yaml = r##"
locale: fr-FR
uiStrings:
  copy.label: Kopieren
cascadeLayer: ""
themeCssRoot: ".docs"
styleOverrides:
  --kz-radius: 0
  --kz-editor-bg:
    light: "#fff"
    dark: "#000"
  --kz-font-size:
    dark: 1rem
"##;
        let fc = FileConfig::from_yaml(yaml).unwrap();
        let mut cfg = Config::default();
        fc.apply(&mut cfg).unwrap();
        assert_eq!(cfg.locale, "fr-FR");
        assert_eq!(cfg.ui_string_overrides["copy.label"], "Kopieren");
        assert_eq!(cfg.cascade_layer, "");
        assert_eq!(cfg.theme_css_root, ".docs");
        assert_eq!(cfg.style_overrides["--kz-radius"], StyleValue::plain("0"));
        assert_eq!(
            cfg.style_overrides["--kz-editor-bg"],
            StyleValue::themed("#fff", "#000")
        );
        assert_eq!(cfg.style_overrides["--kz-font-size"].light_value(), "");
        assert_eq!(cfg.style_overrides["--kz-font-size"].dark_value(), "1rem");
    }

    #[test]
    fn file_config_toolbar_and_output_fields() {
        let yaml = "fullscreenButton: false\nthemeToggle: true\noutputPanel: true\noutputDefaultCollapsed: true\noutputSeparator: '==='\ninlineLinks: true\nfileIcons: false\nlangIconMode: iconAndText\n";
        let fc = FileConfig::from_yaml(yaml).unwrap();
        let mut cfg = Config::default();
        fc.apply(&mut cfg).unwrap();
        assert!(!cfg.fullscreen_button);
        assert!(cfg.theme_toggle);
        assert!(cfg.output_panel);
        assert!(cfg.output_default_collapsed);
        assert_eq!(cfg.output_separator, "===");
        assert!(cfg.inline_links);
        assert!(!cfg.file_icons);
        assert_eq!(cfg.lang_icon_mode, LangIconMode::IconAndText);
        assert!(FileConfig::from_yaml("langIconMode: pictures\n").is_err());
    }

    #[test]
    fn output_options_cascade() {
        let cfg = Config {
            output_default_collapsed: true,
            ..Default::default()
        };
        let r = cfg.resolve("py", None);
        assert!(!r.with_output);
        assert!(r.output_collapsed);
        let opts = BlockOptions {
            with_output: Some(true),
            output_collapsed: Some(false),
            output_label: "Result".into(),
            ..Default::default()
        };
        let r = cfg.resolve("py", Some(&opts));
        assert!(r.with_output);
        assert!(!r.output_collapsed);
        assert_eq!(r.output_label, "Result");
    }

    #[test]
    fn file_config_collapsible() {
        let yaml = "collapsible:\n  lineThreshold: 20\n  style: collapsibleAuto\n  expandButtonText: More\n";
        let fc = FileConfig::from_yaml(yaml).unwrap();
        let mut cfg = Config::default();
        fc.apply(&mut cfg).unwrap();
        let c = cfg.collapsible.as_ref().unwrap();
        assert_eq!(c.line_threshold, 20);
        assert_eq!(c.preview_lines, 8);
        assert!(c.default_collapsed);
        assert_eq!(c.style, CollapseStyle::CollapsibleAuto);
        assert_eq!(c.expand_button_text, "More");

        let fc = FileConfig::from_yaml("collapsible: {}\n").unwrap();
        let mut cfg = Config::default();
        fc.apply(&mut cfg).unwrap();
        assert_eq!(cfg.collapsible, Some(CollapsibleConfig::default()));

        assert!(FileConfig::from_yaml("collapsible:\n  style: sideways\n").is_err());
        assert!(FileConfig::from_yaml("collapsible:\n  lineThreshold: 0\n").is_err());
        assert!(FileConfig::from_yaml("collapsible:\n  previewLines: 0\n").is_err());
    }

    #[test]
    fn style_value_accessors() {
        let plain = StyleValue::plain("1px");
        assert!(!plain.is_themed());
        assert_eq!(plain.light_value(), "1px");
        assert_eq!(plain.dark_value(), "1px");
        let themed = StyleValue::themed("#fff", "#000");
        assert!(themed.is_themed());
        assert_eq!(themed.light_value(), "#fff");
        assert_eq!(themed.dark_value(), "#000");
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
    fn file_config_minify_key() {
        let mut cfg = Config::default();
        assert!(cfg.minify);
        FileConfig::from_yaml("minify: false\n")
            .unwrap()
            .apply(&mut cfg)
            .unwrap();
        assert!(!cfg.minify);
    }

    #[test]
    fn file_config_min_contrast_range() {
        assert!(FileConfig::from_yaml("minContrast: 22\n").is_err());
        assert!(FileConfig::from_yaml("minContrast: -1\n").is_err());
        let mut cfg = Config::default();
        FileConfig::from_yaml("minContrast: 5.5\n")
            .unwrap()
            .apply(&mut cfg)
            .unwrap();
        assert_eq!(cfg.min_contrast, 5.5);
    }

    #[test]
    fn marker_bgs_composite_on_editor_background() {
        let light = compute_marker_bgs("#ffffff");
        assert_eq!(light.mark, "#fff8e0");
        assert_ne!(light.ins, light.del);
        assert_eq!(light.bg(MarkerType::Ins), Some(light.ins.as_str()));
        assert_eq!(light.bg(MarkerType::Warning), None);
        let dark = compute_marker_bgs("#24292e");
        assert_ne!(dark.mark, light.mark);
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
