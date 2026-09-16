use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::fmt::Write;

use iro::FontStyle;

use crate::collapsible;
use crate::color;
use crate::config::{
    CollapseRange, CollapseStyle, Config, LangIconMode, MarkerBgs, ResolvedBlock,
    compute_marker_bgs,
};
use crate::escape::{escape_attr, escape_text};
use crate::locale::UIStrings;
use crate::marker::{self, ResolvedLine, Segment};
use crate::tokenize::Tokens;
use crate::types::{Frame, LinkAnnotation, MarkerType, TerminalDotStyle};

const EXTERNAL_LINK_SVG: &str = r#"<svg class="kz-link-icon" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"/></svg>"#;

const COPY_SVG: &str = r#"<svg class="kz-copy-icon" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/></svg>"#;

const WRAP_SVG: &str = r#"<svg class="kz-wrap-icon" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 6h18M3 12h15a3 3 0 110 6h-4m0 0l2-2m-2 2l2 2"/></svg>"#;

const WRAP_OFF_SVG: &str = r#"<svg class="kz-wrap-off-icon" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 6h18M3 12h18M3 18h18"/></svg>"#;

const FULLSCREEN_SVG: &str = r#"<svg class="kz-fs-icon" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5v-4m0 4h-4m4 0l-5-5"/></svg>"#;

const FULLSCREEN_EXIT_SVG: &str = r#"<svg class="kz-fs-exit-icon" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 4v5H4m5 0L4 4M15 4v5h5m-5 0l5-5M9 20v-5H4m5 0l-5 5M15 20v-5h5m-5 0l5 5"/></svg>"#;

const FONT_INCREASE_SVG: &str = r#"<svg class="kz-font-icon" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M12 6v12m-6-6h12"/></svg>"#;

const FONT_DECREASE_SVG: &str = r#"<svg class="kz-font-icon" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M6 12h12"/></svg>"#;

const THEME_TOGGLE_LIGHT_SVG: &str = r#"<svg class="kz-theme-toggle-light-icon" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><circle cx="12" cy="12" r="5" stroke-width="2"/><path stroke-linecap="round" stroke-width="2" d="M12 1v2m0 18v2M4.22 4.22l1.42 1.42m12.72 12.72l1.42 1.42M1 12h2m18 0h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/></svg>"#;

const THEME_TOGGLE_DARK_SVG: &str = r#"<svg class="kz-theme-toggle-dark-icon" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z"/></svg>"#;

const CHEVRON_SVG: &str = r#"<svg class="kz-collapse-toggle-icon" aria-hidden="true" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/></svg>"#;

fn theme_toggle_active(cfg: &Config) -> bool {
    cfg.theme_toggle && cfg.dark_theme.is_some()
}

fn initially_collapsed(cfg: &Config) -> bool {
    cfg.collapsible.as_ref().is_none_or(|c| c.default_collapsed)
}

struct CollapseTexts<'a> {
    expand: &'a str,
    collapse: &'a str,
    expanded: &'a str,
    collapsed: &'a str,
}

fn collapse_texts<'a>(cfg: &'a Config, strings: &'a UIStrings) -> CollapseTexts<'a> {
    let pick = |custom: Option<&'a String>, default: &'a String| -> &'a str {
        match custom {
            Some(s) if !s.is_empty() => s,
            _ => default,
        }
    };
    let c = cfg.collapsible.as_ref();
    CollapseTexts {
        expand: pick(
            c.map(|c| &c.expand_button_text),
            &strings.expand_button_text,
        ),
        collapse: pick(
            c.map(|c| &c.collapse_button_text),
            &strings.collapse_button_text,
        ),
        expanded: pick(
            c.map(|c| &c.expanded_announcement),
            &strings.expanded_announcement,
        ),
        collapsed: pick(
            c.map(|c| &c.collapsed_announcement),
            &strings.collapsed_announcement,
        ),
    }
}

struct LineCtx<'a> {
    resolved: &'a ResolvedBlock,
    cfg: &'a Config,
    is_dual: bool,
    resolved_markers: Option<HashMap<usize, ResolvedLine>>,
    focus_set: Option<HashSet<usize>>,
    has_focus: bool,
    collapse_range_map: HashMap<usize, usize>,
    threshold_visible: Option<HashSet<usize>>,
    contrast: Option<ContrastCtx>,
}

/// Backgrounds the token colours are adjusted against when `min_contrast` is on;
/// the block's override theme wins over the page theme.
struct ContrastCtx {
    light_bg: String,
    dark_bg: String,
    light_marker_bgs: Option<MarkerBgs>,
    dark_marker_bgs: Option<MarkerBgs>,
    current_marker: Cell<Option<MarkerType>>,
    cache: RefCell<HashMap<String, String>>,
}

impl ContrastCtx {
    fn new(resolved: &ResolvedBlock, cfg: &Config) -> Option<Self> {
        if cfg.min_contrast <= 0.0 {
            return None;
        }
        let pick = |info: Option<&crate::types::ThemeInfo>| match info {
            Some(info) if !info.bg.is_empty() => {
                (info.bg.clone(), Some(compute_marker_bgs(&info.bg)))
            }
            _ => (String::new(), None),
        };
        let (light_bg, light_marker_bgs) = pick(resolved.contrast_light.as_ref());
        let (dark_bg, dark_marker_bgs) = pick(resolved.contrast_dark.as_ref());
        Some(Self {
            light_bg,
            dark_bg,
            light_marker_bgs,
            dark_marker_bgs,
            current_marker: Cell::new(None),
            cache: RefCell::new(HashMap::new()),
        })
    }

    /// The marker background on mark, ins and del lines, the editor background
    /// otherwise; `None` when the theme has no usable background.
    fn effective_bg<'b>(
        &self,
        editor_bg: &'b str,
        marker_bgs: Option<&'b MarkerBgs>,
    ) -> Option<&'b str> {
        if editor_bg.is_empty() {
            return None;
        }
        match self.current_marker.get() {
            Some(mt) => Some(marker_bgs.and_then(|m| m.bg(mt)).unwrap_or(editor_bg)),
            None => Some(editor_bg),
        }
    }

    fn adjust(&self, color: &str, bg: &str, min_contrast: f64) -> String {
        let key = format!("{color}|{bg}");
        if let Some(hit) = self.cache.borrow().get(&key) {
            return hit.clone();
        }
        let adjusted = color::ensure_contrast_on_background(color, bg, min_contrast);
        self.cache.borrow_mut().insert(key, adjusted.clone());
        adjusted
    }
}

impl LineCtx<'_> {
    fn in_collapse_range_end(&self, line_num: usize) -> bool {
        self.collapse_range_map
            .get(&line_num)
            .is_some_and(|&idx| self.resolved.collapse_ranges[idx].end == line_num)
    }

    fn set_current_line(&self, line_num: usize) {
        if let Some(ctx) = &self.contrast {
            let mt = self
                .resolved_markers
                .as_ref()
                .and_then(|m| m.get(&line_num))
                .map(|entry| entry.marker_type);
            ctx.current_marker.set(mt);
        }
    }

    /// A token colour as written in the style attribute: ANSI blocks resolve the
    /// standard palette through `var(--kz-ansi-*)`, everything else is adjusted for
    /// contrast when that is on.
    fn token_color(&self, color: &str, dark: bool) -> String {
        if self.resolved.lang == "ansi"
            && let Some(var) = ansi_var(color)
        {
            return var;
        }
        let Some(ctx) = &self.contrast else {
            return color.to_owned();
        };
        let bg = if dark {
            ctx.effective_bg(&ctx.dark_bg, ctx.dark_marker_bgs.as_ref())
        } else {
            ctx.effective_bg(&ctx.light_bg, ctx.light_marker_bgs.as_ref())
        };
        match bg {
            Some(bg) => ctx.adjust(color, bg, self.cfg.min_contrast),
            None => color.to_owned(),
        }
    }

    /// Background colours are never contrast-adjusted; ANSI blocks still map them.
    fn token_bg(&self, color: &str) -> String {
        if self.resolved.lang == "ansi"
            && let Some(var) = ansi_var(color)
        {
            return var;
        }
        color.to_owned()
    }
}

pub fn render_block(
    tokens: &Tokens,
    resolved: &ResolvedBlock,
    cfg: &Config,
    strings: &UIStrings,
) -> String {
    let mut sb = String::with_capacity(4096);

    let mut wrapper_class = String::from("kazari-block");
    if !resolved.theme_override_style.is_empty() {
        wrapper_class.push_str(" kz-themed");
    }
    if resolved.collapse_threshold && initially_collapsed(cfg) {
        wrapper_class.push_str(" kz-collapsed");
    }
    wrapper_class.push_str(" not-content");
    write!(sb, "<div class=\"{}\"", wrapper_class).unwrap();
    if cfg.data_line_count {
        write!(sb, " data-lines=\"{}\"", tokens.line_count()).unwrap();
    }
    if theme_toggle_active(cfg) {
        write!(sb, " data-kz-id=\"{}\"", block_id(&resolved.raw_code)).unwrap();
    }
    if !resolved.theme_override_style.is_empty() {
        write!(
            sb,
            " style=\"{}\"",
            escape_attr(&resolved.theme_override_style)
        )
        .unwrap();
    }
    sb.push_str(">\n");

    match resolved.frame {
        Frame::None => render_no_frame(&mut sb, tokens, resolved, cfg, strings),
        Frame::Terminal => render_terminal_frame(&mut sb, tokens, resolved, cfg, strings),
        _ => render_framed_block(&mut sb, tokens, resolved, cfg, strings),
    }

    sb.push_str("</div>");
    sb
}

fn render_framed_block(
    sb: &mut String,
    tokens: &Tokens,
    resolved: &ResolvedBlock,
    cfg: &Config,
    strings: &UIStrings,
) {
    let mut classes = String::from("frame");
    if !resolved.title.is_empty() {
        classes.push_str(" has-title");
    }
    write!(
        sb,
        "<figure class=\"{}\" data-lang=\"{}\">",
        classes,
        escape_attr(&resolved.lang)
    )
    .unwrap();

    render_toolbar(sb, resolved, cfg, strings);
    render_code_area(sb, tokens, resolved, cfg, strings);
    render_output_panel(sb, resolved, strings);

    sb.push_str("</figure>\n");
}

/// The `pre` with its threshold-collapse wrapper, gradient and bar when the block
/// is collapsed by length.
fn render_code_area(
    sb: &mut String,
    tokens: &Tokens,
    resolved: &ResolvedBlock,
    cfg: &Config,
    strings: &UIStrings,
) {
    if resolved.collapse_threshold {
        sb.push_str("<div class=\"kz-collapse-content\">");
    }
    render_pre_code(sb, tokens, resolved, cfg, strings);
    if resolved.collapse_threshold {
        sb.push_str("<div class=\"kz-collapse-gradient\"></div></div>");
        render_collapse_bar(sb, resolved, cfg, strings);
    }
}

fn render_collapse_bar(
    sb: &mut String,
    resolved: &ResolvedBlock,
    cfg: &Config,
    strings: &UIStrings,
) {
    let texts = collapse_texts(cfg, strings);
    let expand = if resolved.collapse_beyond_cap > 0 {
        format!(
            "{} (+{} highlighted)",
            texts.expand, resolved.collapse_beyond_cap
        )
    } else {
        texts.expand.to_owned()
    };
    write!(
        sb,
        "<div class=\"kz-collapse-bar\"><button class=\"kz-collapse-btn\" aria-expanded=\"false\" data-expand=\"{}\" data-collapse=\"{}\" data-expanded-msg=\"{}\" data-collapsed-msg=\"{}\">{}</button></div><div class=\"kz-sr-announce\" aria-live=\"polite\"></div>",
        escape_attr(&expand),
        escape_attr(texts.collapse),
        escape_attr(texts.expanded),
        escape_attr(texts.collapsed),
        escape_text(&expand),
    )
    .unwrap();
}

fn render_terminal_frame(
    sb: &mut String,
    tokens: &Tokens,
    resolved: &ResolvedBlock,
    cfg: &Config,
    strings: &UIStrings,
) {
    let mut classes = String::from("frame is-terminal");
    if !resolved.title.is_empty() {
        classes.push_str(" has-title");
    }
    write!(
        sb,
        "<figure class=\"{}\" data-lang=\"{}\">",
        classes,
        escape_attr(&resolved.lang)
    )
    .unwrap();

    if cfg.terminal_dot_style == TerminalDotStyle::Minimal {
        sb.push_str("<div class=\"kz-terminal-header kz-dots-minimal\">");
    } else {
        sb.push_str("<div class=\"kz-terminal-header\">");
        sb.push_str("<span class=\"kz-terminal-dots\" aria-hidden=\"true\"><span></span><span></span><span></span></span>");
    }
    if !resolved.title.is_empty() {
        write!(
            sb,
            "<span class=\"kz-title\">{}</span>",
            escape_text(&resolved.title)
        )
        .unwrap();
    } else {
        write!(
            sb,
            "<span class=\"sr-only\">{}</span>",
            escape_text(&strings.terminal_window_label)
        )
        .unwrap();
    }
    if cfg.copy_button || cfg.wrap_button || cfg.fullscreen_button || theme_toggle_active(cfg) {
        sb.push_str("<div class=\"kz-terminal-actions\">");
        render_action_buttons(sb, resolved, cfg, strings);
        sb.push_str("</div>");
    }
    sb.push_str("</div>");

    render_code_area(sb, tokens, resolved, cfg, strings);
    render_output_panel(sb, resolved, strings);

    sb.push_str("</figure>\n");
}

fn render_no_frame(
    sb: &mut String,
    tokens: &Tokens,
    resolved: &ResolvedBlock,
    cfg: &Config,
    strings: &UIStrings,
) {
    render_code_area(sb, tokens, resolved, cfg, strings);
    render_output_panel(sb, resolved, strings);
    if cfg.copy_button {
        render_copy_button(sb, &resolved.raw_code, strings);
    }
}

fn render_output_panel(sb: &mut String, resolved: &ResolvedBlock, strings: &UIStrings) {
    if resolved.output_text.is_empty() {
        return;
    }
    let (class, expanded) = if resolved.output_collapsed {
        ("kz-output kz-output-hidden", "false")
    } else {
        ("kz-output", "true")
    };
    let label = if resolved.output_label.is_empty() {
        strings.output_label.as_str()
    } else {
        resolved.output_label.as_str()
    };
    write!(
        sb,
        "<div class=\"{}\"><div class=\"kz-output-header\"><button class=\"kz-output-toggle\" aria-expanded=\"{}\">{}</button></div><pre class=\"kz-output-pre\">{}</pre></div>",
        class,
        expanded,
        escape_text(label),
        escape_text(&resolved.output_text)
    )
    .unwrap();
}

/// FNV-1a 32-bit hash of the raw code, eight hex digits; keys the persisted
/// per-block theme choice.
fn block_id(code: &str) -> String {
    crate::hash::fnv1a32(code)
}

/// The `var(--kz-ansi-*)` reference for one of the 16 standard terminal colours,
/// matched by Iro's palette value. VS Code draws white and bright white the same,
/// so both resolve to `white`.
fn ansi_var(hex: &str) -> Option<String> {
    let index = iro::ANSI_STANDARD_COLORS
        .iter()
        .position(|c| c.eq_ignore_ascii_case(hex))?;
    Some(format!(
        "var(--kz-ansi-{})",
        crate::theme_css::ANSI_PALETTE[index].0
    ))
}

fn render_toolbar(sb: &mut String, resolved: &ResolvedBlock, cfg: &Config, strings: &UIStrings) {
    sb.push_str("<div class=\"kz-toolbar\">");

    sb.push_str("<div class=\"kz-toolbar-left\">");
    if cfg.language_badge && !resolved.lang.is_empty() {
        render_lang_badge(sb, &resolved.lang, cfg);
    }
    if !resolved.title.is_empty() {
        if cfg.file_icons
            && let Some(ext) = file_ext(&resolved.title)
        {
            match &cfg.file_icon_resolver {
                Some(resolver) => sb.push_str(&resolver(ext)),
                None => write!(
                    sb,
                    "<span class=\"kz-file-icon\" data-ext=\"{}\"></span>",
                    escape_attr(ext)
                )
                .unwrap(),
            }
        }
        write!(
            sb,
            "<span class=\"kz-title\">{}</span>",
            escape_text(&resolved.title)
        )
        .unwrap();
    }
    sb.push_str("</div>");

    sb.push_str("<div class=\"kz-toolbar-right\">");
    render_action_buttons(sb, resolved, cfg, strings);
    if resolved.collapse_threshold {
        let texts = collapse_texts(cfg, strings);
        let (expanded, tooltip) = if initially_collapsed(cfg) {
            ("false", texts.expand)
        } else {
            ("true", texts.collapse)
        };
        write!(
            sb,
            "<button class=\"kz-collapse-toggle\" aria-expanded=\"{}\" aria-label=\"{}\" data-tooltip=\"{}\" data-expand=\"{}\" data-collapse=\"{}\">",
            expanded,
            escape_attr(tooltip),
            escape_attr(tooltip),
            escape_attr(texts.expand),
            escape_attr(texts.collapse),
        )
        .unwrap();
        sb.push_str(CHEVRON_SVG);
        sb.push_str("</button>");
    }
    sb.push_str("</div>");

    sb.push_str("</div>");
}

fn render_action_buttons(
    sb: &mut String,
    resolved: &ResolvedBlock,
    cfg: &Config,
    strings: &UIStrings,
) {
    if cfg.copy_button {
        render_copy_button(sb, &resolved.raw_code, strings);
    }
    if cfg.wrap_button {
        render_wrap_button(sb, resolved, strings);
    }
    if theme_toggle_active(cfg) {
        render_theme_toggle_button(sb, cfg, strings);
    }
    if cfg.fullscreen_button {
        render_font_controls(sb, strings);
        render_fullscreen_button(sb, strings);
    }
}

fn render_fullscreen_button(sb: &mut String, strings: &UIStrings) {
    let label = escape_attr(&strings.fullscreen_label);
    write!(
        sb,
        "<button class=\"kz-fs-btn\" aria-label=\"{label}\" data-tooltip=\"{label}\" aria-expanded=\"false\">"
    )
    .unwrap();
    sb.push_str(FULLSCREEN_SVG);
    sb.push_str(FULLSCREEN_EXIT_SVG);
    sb.push_str("</button>");
}

fn render_font_controls(sb: &mut String, strings: &UIStrings) {
    sb.push_str("<div class=\"kz-font-controls\">");
    let dec = escape_attr(&strings.font_decrease_label);
    write!(
        sb,
        "<button class=\"kz-font-dec\" aria-label=\"{dec}\" data-tooltip=\"{dec}\">"
    )
    .unwrap();
    sb.push_str(FONT_DECREASE_SVG);
    sb.push_str("</button>");
    let inc = escape_attr(&strings.font_increase_label);
    write!(
        sb,
        "<button class=\"kz-font-inc\" aria-label=\"{inc}\" data-tooltip=\"{inc}\">"
    )
    .unwrap();
    sb.push_str(FONT_INCREASE_SVG);
    sb.push_str("</button>");
    sb.push_str("</div>");
}

fn render_theme_toggle_button(sb: &mut String, cfg: &Config, strings: &UIStrings) {
    let (mode, selector) = match &cfg.dark_mode {
        crate::types::DarkMode::Selector(sel) => ("selector", sel.as_str()),
        crate::types::DarkMode::MediaQuery => ("media", ""),
        crate::types::DarkMode::Both(sel) => ("both", sel.as_str()),
    };
    let label = escape_attr(&strings.theme_toggle_label);
    write!(
        sb,
        "<button class=\"kz-theme-toggle-btn\" aria-pressed=\"false\" aria-label=\"{label}\" data-tooltip=\"{label}\" data-label=\"{label}\" data-toggled=\"{label}\" data-announcement=\"{}\" data-kz-dark-selector=\"{}\" data-kz-dark-mode=\"{}\">",
        escape_attr(&strings.theme_toggle_announcement),
        escape_attr(selector),
        mode,
    )
    .unwrap();
    sb.push_str(THEME_TOGGLE_LIGHT_SVG);
    sb.push_str(THEME_TOGGLE_DARK_SVG);
    sb.push_str("</button>");
}

fn render_copy_button(sb: &mut String, raw_code: &str, strings: &UIStrings) {
    let encoded = encode_for_data_code(raw_code);
    let label = escape_attr(&strings.copy_label);
    write!(
        sb,
        "<button class=\"kz-copy-btn\" aria-label=\"{}\" data-tooltip=\"{}\" data-copied=\"{}\" data-code=\"{}\">",
        label,
        label,
        escape_attr(&strings.copy_success),
        escape_attr(&encoded)
    )
    .unwrap();
    sb.push_str(COPY_SVG);
    sb.push_str("</button>");
    sb.push_str("<span class=\"kz-sr-announce\" aria-live=\"polite\"></span>");
}

fn render_wrap_button(sb: &mut String, resolved: &ResolvedBlock, strings: &UIStrings) {
    let (pressed, title) = if resolved.wrap {
        ("true", strings.wrap_disable_label.as_str())
    } else {
        ("false", strings.wrap_enable_label.as_str())
    };
    write!(
        sb,
        "<button class=\"kz-wrap-btn\" aria-pressed=\"{}\" aria-label=\"{}\" data-tooltip=\"{}\" data-enable=\"{}\" data-disable=\"{}\">",
        pressed,
        escape_attr(title),
        escape_attr(title),
        escape_attr(&strings.wrap_enable_label),
        escape_attr(&strings.wrap_disable_label),
    )
    .unwrap();
    sb.push_str(WRAP_SVG);
    sb.push_str(WRAP_OFF_SVG);
    sb.push_str("</button>");
}

fn render_lang_badge(sb: &mut String, lang: &str, cfg: &Config) {
    let mode = cfg.lang_icon_mode;
    if mode != LangIconMode::None {
        write!(
            sb,
            "<span class=\"kz-lang-icon\" data-lang=\"{}\"></span>",
            escape_attr(lang)
        )
        .unwrap();
    }
    if mode != LangIconMode::IconOnly {
        write!(
            sb,
            "<span class=\"kz-lang\">{}</span>",
            escape_text(&display_lang(lang))
        )
        .unwrap();
    }
}

/// The text after the last dot, unless the dot is missing or trailing.
fn file_ext(title: &str) -> Option<&str> {
    let idx = title.rfind('.')?;
    let ext = &title[idx + 1..];
    (!ext.is_empty()).then_some(ext)
}

fn display_lang(lang: &str) -> String {
    match lang.to_lowercase().as_str() {
        "javascript" => "JavaScript".to_owned(),
        "typescript" => "TypeScript".to_owned(),
        "css" => "CSS".to_owned(),
        "html" => "HTML".to_owned(),
        "json" => "JSON".to_owned(),
        "yaml" => "YAML".to_owned(),
        "sql" => "SQL".to_owned(),
        "php" => "PHP".to_owned(),
        "xml" => "XML".to_owned(),
        "svg" => "SVG".to_owned(),
        "jsx" => "JSX".to_owned(),
        "tsx" => "TSX".to_owned(),
        "graphql" => "GraphQL".to_owned(),
        _ => {
            let mut chars = lang.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        }
    }
}

fn render_pre_code(
    sb: &mut String,
    tokens: &Tokens,
    resolved: &ResolvedBlock,
    cfg: &Config,
    strings: &UIStrings,
) {
    let mut collapse_range_map = HashMap::new();
    for (idx, cr) in resolved.collapse_ranges.iter().enumerate() {
        for line in cr.start..=cr.end {
            collapse_range_map.insert(line, idx);
        }
    }
    let threshold_visible = if resolved.collapse_threshold && !resolved.collapse_segments.is_empty()
    {
        Some(
            resolved
                .collapse_segments
                .iter()
                .flat_map(|s| s.start..=s.end)
                .collect::<HashSet<usize>>(),
        )
    } else {
        None
    };
    let lctx = LineCtx {
        resolved,
        cfg,
        is_dual: tokens.is_dual(),
        resolved_markers: marker::resolve_line_markers(&resolved.line_markers),
        focus_set: marker::resolve_focus_set(&resolved.focus_lines),
        has_focus: !resolved.focus_lines.is_empty(),
        collapse_range_map,
        threshold_visible,
        contrast: ContrastCtx::new(resolved, cfg),
    };

    if resolved.wrap {
        sb.push_str("<pre class=\"wrap\"");
    } else {
        sb.push_str("<pre");
    }
    write!(sb, " data-language=\"{}\">", escape_attr(&resolved.lang)).unwrap();

    let ln_width = if resolved.line_numbers {
        let end_num = resolved.start_line_number + tokens.line_count().saturating_sub(1);
        let max_digits = digit_count(resolved.start_line_number).max(digit_count(end_num));
        if max_digits > 2 { max_digits } else { 0 }
    } else {
        0
    };

    if lctx.has_focus {
        if ln_width > 0 {
            write!(
                sb,
                "<code class=\"has-focus\" style=\"--kz-ln-width:{}ch\">",
                ln_width
            )
            .unwrap();
        } else {
            sb.push_str("<code class=\"has-focus\">");
        }
    } else if ln_width > 0 {
        write!(sb, "<code style=\"--kz-ln-width:{}ch\">", ln_width).unwrap();
    } else {
        sb.push_str("<code>");
    }

    for i in 0..tokens.line_count() {
        let line_num = resolved.start_line_number + i;

        if let Some(&idx) = lctx.collapse_range_map.get(&line_num) {
            let cr = &resolved.collapse_ranges[idx];
            if line_num == cr.start {
                render_collapse_range_open(sb, resolved, cr, cfg, strings);
            }
            render_line(sb, tokens, i, line_num, &lctx);
            if line_num == cr.end {
                render_collapse_range_close(sb, cr);
            }
            continue;
        }

        if let Some(visible) = &lctx.threshold_visible
            && !visible.contains(&line_num)
        {
            if resolved.collapse_segments.len() > 1 {
                let prev_visible = line_num
                    .checked_sub(1)
                    .is_some_and(|p| visible.contains(&p) || lctx.in_collapse_range_end(p));
                let next_visible_exists = resolved
                    .collapse_segments
                    .iter()
                    .any(|s| s.start > line_num);
                if prev_visible && next_visible_exists {
                    render_gap_indicator(sb, resolved);
                }
            }
            render_hidden_line(sb, tokens, i, line_num, &lctx);
            continue;
        }

        render_line(sb, tokens, i, line_num, &lctx);
    }

    sb.push_str("</code></pre>");
}

fn render_empty_gutter(sb: &mut String, resolved: &ResolvedBlock) {
    if resolved.line_numbers {
        sb.push_str("<div class=\"kz-gutter\"><div class=\"kz-ln\"></div></div>");
    }
}

fn render_gap_indicator(sb: &mut String, resolved: &ResolvedBlock) {
    sb.push_str("<div class=\"kz-line kz-gap\">");
    render_empty_gutter(sb, resolved);
    sb.push_str("<div class=\"kz-code\"><span class=\"kz-gap-indicator\" aria-hidden=\"true\">\u{22ee}</span><span class=\"sr-only\">Lines hidden</span></div>");
    sb.push_str("</div>");
}

/// A line outside the threshold preview: same markup as a visible line minus the
/// wrap indent and inline annotations, so expanding it needs no re-render.
fn render_hidden_line(
    sb: &mut String,
    tokens: &Tokens,
    line_idx: usize,
    line_num: usize,
    lctx: &LineCtx<'_>,
) {
    lctx.set_current_line(line_num);
    let (extra, _label_attr) = marker_and_focus_classes(line_num, lctx);
    write!(sb, "<div class=\"kz-line kz-hidden{}\">", extra).unwrap();
    if lctx.resolved.line_numbers {
        write!(
            sb,
            "<div class=\"kz-gutter\"><div class=\"kz-ln\" aria-hidden=\"true\">{}</div></div>",
            line_num
        )
        .unwrap();
    }
    sb.push_str("<div class=\"kz-code\">");
    let line_text = tokens.line_text(line_idx);
    render_plain_tokens(sb, tokens.tokens(line_idx), line_text, 0, tokens, lctx);
    sb.push_str("</div></div>");
}

fn render_summary_line(
    sb: &mut String,
    resolved: &ResolvedBlock,
    cr: &CollapseRange,
    cfg: &Config,
    strings: &UIStrings,
) {
    sb.push_str("<summary><div class=\"kz-line\">");
    render_empty_gutter(sb, resolved);
    let preserve = cfg.collapsible.as_ref().is_some_and(|c| c.preserve_indent);
    if cr.min_indent > 0 && preserve {
        write!(
            sb,
            "<div class=\"kz-code\" style=\"--kz-indent:{}ch\">",
            cr.min_indent
        )
        .unwrap();
    } else {
        sb.push_str("<div class=\"kz-code\">");
    }
    sb.push_str("<span class=\"expand\" aria-hidden=\"true\"></span><span class=\"collapse\" aria-hidden=\"true\"></span>");
    write!(
        sb,
        "<span class=\"text\">{}</span>",
        escape_text(&collapsible::summary_text(cr.line_count, strings))
    )
    .unwrap();
    sb.push_str("</div></div></summary>");
}

fn render_collapse_range_open(
    sb: &mut String,
    resolved: &ResolvedBlock,
    cr: &CollapseRange,
    cfg: &Config,
    strings: &UIStrings,
) {
    match cr.style {
        CollapseStyle::CollapsibleStart | CollapseStyle::CollapsibleEnd => {
            let class = if cr.style == CollapseStyle::CollapsibleEnd {
                "collapsible-end"
            } else {
                "collapsible-start"
            };
            write!(sb, "<div class=\"kz-section {}\"><details>", class).unwrap();
            render_summary_line(sb, resolved, cr, cfg, strings);
            sb.push_str("</details><div class=\"content-lines\">");
        }
        _ => {
            sb.push_str("<details class=\"kz-section\">");
            render_summary_line(sb, resolved, cr, cfg, strings);
        }
    }
}

fn render_collapse_range_close(sb: &mut String, cr: &CollapseRange) {
    match cr.style {
        CollapseStyle::CollapsibleStart | CollapseStyle::CollapsibleEnd => {
            sb.push_str("</div></div>");
        }
        _ => sb.push_str("</details>"),
    }
}

/// The marker and focus classes of a line (with leading spaces) and its
/// `data-label` attribute.
fn marker_and_focus_classes(line_num: usize, lctx: &LineCtx<'_>) -> (String, String) {
    let mut classes = String::new();
    let mut label_attr = String::new();

    if let Some(entry) = lctx
        .resolved_markers
        .as_ref()
        .and_then(|m| m.get(&line_num))
        && entry.has_mark
    {
        classes.push_str(" highlight");
        match entry.marker_type {
            MarkerType::Mark => classes.push_str(" mark"),
            MarkerType::Warning => classes.push_str(" warning"),
            MarkerType::Error => classes.push_str(" error"),
            MarkerType::Del => classes.push_str(" del"),
            MarkerType::Ins => classes.push_str(" ins"),
        }
        if !entry.label.is_empty() {
            classes.push_str(" tm-label");
            label_attr = format!(" data-label=\"{}\"", escape_attr(&entry.label));
        }
    }

    if lctx.has_focus
        && lctx
            .focus_set
            .as_ref()
            .is_some_and(|f| f.contains(&line_num))
    {
        classes.push_str(" focused");
    }

    (classes, label_attr)
}

fn render_line(
    sb: &mut String,
    tokens: &Tokens,
    line_idx: usize,
    line_num: usize,
    lctx: &LineCtx<'_>,
) {
    lctx.set_current_line(line_num);
    let (extra, label_attr) = marker_and_focus_classes(line_num, lctx);
    let classes = format!("kz-line{extra}");

    let line_text = tokens.line_text(line_idx);
    let line_tokens = tokens.tokens(line_idx);

    let mut indent_attr = String::new();
    let mut indent_ws = String::new();
    let mut skip_ws_tokens = 0;

    if lctx.resolved.wrap {
        let (ws, first_non_ws_idx) = split_leading_whitespace(line_text, line_tokens);
        let mut indent = lctx.resolved.hanging_indent;
        if lctx.resolved.preserve_indent {
            indent += ws.len();
        }
        if indent > 0 {
            indent_attr = format!(" style=\"--kz-indent:{}ch\"", indent);
            indent_ws = ws.to_owned();
            skip_ws_tokens = first_non_ws_idx;
        }
    }

    write!(sb, "<div class=\"{}\">", classes).unwrap();

    if lctx.resolved.line_numbers {
        write!(
            sb,
            "<div class=\"kz-gutter\"><div class=\"kz-ln\" aria-hidden=\"true\">{}</div></div>",
            line_num
        )
        .unwrap();
    }

    write!(sb, "<div class=\"kz-code\"{}{}>", label_attr, indent_attr).unwrap();

    if !indent_ws.is_empty() {
        write!(sb, "<span class=\"indent\">{}</span>", indent_ws).unwrap();
    }

    let line_links = lctx
        .resolved
        .links
        .get(line_idx)
        .map(Vec::as_slice)
        .unwrap_or(&[]);

    if !lctx.resolved.inline_markers.is_empty() || !line_links.is_empty() {
        let mut plain_text = String::new();
        let mut token_ranges: Vec<(usize, usize)> = Vec::new();
        let mut renderable_indices: Vec<usize> = Vec::new();

        for (tok_idx, token) in line_tokens.iter().enumerate() {
            if tok_idx < skip_ws_tokens {
                continue;
            }
            let text = if tok_idx == skip_ws_tokens && skip_ws_tokens > 0 {
                let full = token.text(line_text);
                let trimmed = full.trim_start_matches([' ', '\t']);
                if trimmed.is_empty() {
                    continue;
                }
                trimmed
            } else {
                token.text(line_text)
            };
            if text.is_empty() {
                continue;
            }
            let start = plain_text.len();
            plain_text.push_str(text);
            token_ranges.push((start, plain_text.len()));
            renderable_indices.push(tok_idx);
        }

        let links = shift_links(line_links, indent_ws.len());
        if let Some(annotated) = marker::process_inline_markers_and_links(
            &plain_text,
            &token_ranges,
            &lctx.resolved.inline_markers,
            &links,
        ) {
            for at in &annotated {
                let real_idx = renderable_indices[at.token_idx];
                let token = &line_tokens[real_idx];
                render_annotated_token(sb, &plain_text, token, &at.segments, tokens, lctx);
            }
        } else {
            render_plain_tokens(sb, line_tokens, line_text, skip_ws_tokens, tokens, lctx);
        }
    } else {
        render_plain_tokens(sb, line_tokens, line_text, skip_ws_tokens, tokens, lctx);
    }

    sb.push_str("</div></div>");
}

fn render_plain_tokens(
    sb: &mut String,
    line_tokens: &[iro::ThemedToken],
    line_text: &str,
    skip_ws_tokens: usize,
    tokens: &Tokens,
    lctx: &LineCtx<'_>,
) {
    for (tok_idx, token) in line_tokens.iter().enumerate() {
        if tok_idx < skip_ws_tokens {
            continue;
        }
        let text = if tok_idx == skip_ws_tokens && skip_ws_tokens > 0 {
            let full = token.text(line_text);
            let trimmed = full.trim_start_matches([' ', '\t']);
            if trimmed.is_empty() {
                continue;
            }
            trimmed
        } else {
            token.text(line_text)
        };
        if text.is_empty() {
            continue;
        }
        open_styled_span(sb, tokens, token, lctx);
        write_text(sb, text, lctx.cfg);
        sb.push_str("</span>");
    }
}

fn open_styled_span(
    sb: &mut String,
    tokens: &Tokens,
    token: &iro::ThemedToken,
    lctx: &LineCtx<'_>,
) {
    let style = build_token_style(tokens, token, lctx);
    if style.is_empty() {
        sb.push_str("<span>");
    } else {
        write!(sb, "<span style=\"{}\">", style).unwrap();
    }
}

/// Escapes `text` into `sb`; with visible whitespace on, every tab and space
/// becomes a `span.ws-tab` or `span.ws-space` holding the configured symbol.
fn write_text(sb: &mut String, text: &str, cfg: &Config) {
    if !cfg.visible_whitespace || !text.contains(['\t', ' ']) {
        sb.push_str(&escape_text(text));
        return;
    }
    let mut run = String::new();
    for c in text.chars() {
        let symbol = match c {
            '\t' => Some(("ws-tab", cfg.whitespace_tab.as_str())),
            ' ' => Some(("ws-space", cfg.whitespace_space.as_str())),
            _ => None,
        };
        match symbol {
            Some((class, symbol)) => {
                if !run.is_empty() {
                    sb.push_str(&escape_text(&run));
                    run.clear();
                }
                write!(
                    sb,
                    "<span class=\"{}\">{}</span>",
                    class,
                    escape_text(symbol)
                )
                .unwrap();
            }
            None => run.push(c),
        }
    }
    if !run.is_empty() {
        sb.push_str(&escape_text(&run));
    }
}

/// Link offsets are relative to the full line; when the leading whitespace was
/// pulled out into the indent span, the annotated text starts that much later.
fn shift_links(links: &[LinkAnnotation], stripped: usize) -> Vec<LinkAnnotation> {
    if stripped == 0 {
        return links.to_vec();
    }
    links
        .iter()
        .filter_map(|l| {
            let start = l.start.saturating_sub(stripped);
            let end = l.end.saturating_sub(stripped);
            (end > start).then(|| LinkAnnotation {
                start,
                end,
                url: l.url.clone(),
            })
        })
        .collect()
}

fn open_anchor(sb: &mut String, url: &str) {
    write!(
        sb,
        "<a class=\"kz-link\" href=\"{}\" target=\"_blank\" rel=\"noopener noreferrer\">",
        escape_attr(url)
    )
    .unwrap();
}

fn close_anchor(sb: &mut String, with_icon: bool) {
    if with_icon {
        sb.push_str(EXTERNAL_LINK_SVG);
    }
    sb.push_str("</a>");
}

fn marker_element(mt: MarkerType) -> &'static str {
    match mt {
        MarkerType::Ins => "ins",
        MarkerType::Del => "del",
        MarkerType::Mark => "mark",
        MarkerType::Error => "mark class=\"error\"",
        MarkerType::Warning => "mark class=\"warning\"",
    }
}

fn marker_close(mt: MarkerType) -> &'static str {
    match mt {
        MarkerType::Ins => "ins",
        MarkerType::Del => "del",
        MarkerType::Mark | MarkerType::Error | MarkerType::Warning => "mark",
    }
}

fn render_annotated_token(
    sb: &mut String,
    plain_text: &str,
    token: &iro::ThemedToken,
    segments: &[Segment],
    tokens: &Tokens,
    lctx: &LineCtx<'_>,
) {
    let has_inline_marker = segments.iter().any(|s| s.marker.is_some());

    if !has_inline_marker {
        let text: String = segments
            .iter()
            .map(|s| &plain_text[s.start..s.end])
            .collect();
        open_styled_span(sb, tokens, token, lctx);
        write_text(sb, &text, lctx.cfg);
        sb.push_str("</span>");
        return;
    }

    let single_spanning = segments.len() == 1
        && segments[0]
            .marker
            .as_ref()
            .is_some_and(|m| m.open_start || m.open_end);

    if single_spanning {
        let seg = &segments[0];
        let ann = seg.marker.as_ref().unwrap();
        if let Some(url) = &ann.link {
            open_anchor(sb, url);
        }
        match ann.kind {
            Some(kind) => {
                let mut classes = String::new();
                if ann.open_start {
                    classes.push_str("open-start");
                }
                if ann.open_end {
                    if !classes.is_empty() {
                        classes.push(' ');
                    }
                    classes.push_str("open-end");
                }
                match kind {
                    MarkerType::Error => classes.insert_str(0, "error "),
                    MarkerType::Warning => classes.insert_str(0, "warning "),
                    _ => {}
                }
                let elem = marker_close(kind);
                write!(sb, "<{} class=\"{}\">", elem, classes.trim_end()).unwrap();
                open_styled_span(sb, tokens, token, lctx);
                write_text(sb, &plain_text[seg.start..seg.end], lctx.cfg);
                sb.push_str("</span>");
                write!(sb, "</{}>", elem).unwrap();
            }
            None => {
                open_styled_span(sb, tokens, token, lctx);
                write_text(sb, &plain_text[seg.start..seg.end], lctx.cfg);
                sb.push_str("</span>");
            }
        }
        if ann.link.is_some() {
            close_anchor(sb, !ann.open_end);
        }
        return;
    }

    open_styled_span(sb, tokens, token, lctx);
    for seg in segments {
        let text = &plain_text[seg.start..seg.end];
        match &seg.marker {
            Some(ann) => {
                if let Some(url) = &ann.link {
                    open_anchor(sb, url);
                }
                match ann.kind {
                    Some(kind) => {
                        write!(sb, "<{}>", marker_element(kind)).unwrap();
                        write_text(sb, text, lctx.cfg);
                        write!(sb, "</{}>", marker_close(kind)).unwrap();
                    }
                    None => write_text(sb, text, lctx.cfg),
                }
                if ann.link.is_some() {
                    close_anchor(sb, !ann.open_end);
                }
            }
            None => write_text(sb, text, lctx.cfg),
        }
    }
    sb.push_str("</span>");
}

fn split_leading_whitespace<'a>(
    line_text: &'a str,
    tokens: &[iro::ThemedToken],
) -> (&'a str, usize) {
    let mut ws_end = 0;
    let mut first_non_ws_idx = 0;

    for (i, token) in tokens.iter().enumerate() {
        let text = token.text(line_text);
        let trimmed = text.trim_start_matches([' ', '\t']);
        ws_end += text.len() - trimmed.len();
        if !trimmed.is_empty() {
            first_non_ws_idx = i;
            break;
        }
        first_non_ws_idx = i + 1;
    }

    (&line_text[..ws_end], first_non_ws_idx)
}

fn build_token_style(tokens: &Tokens, token: &iro::ThemedToken, lctx: &LineCtx<'_>) -> String {
    let light = tokens.light_style(token);
    let dark = tokens.dark_style(token);

    let mut parts = Vec::new();

    if let Some(color_id) = light.color {
        parts.push(format!(
            "--sl:{}",
            lctx.token_color(tokens.light_color(color_id), false)
        ));
    }
    if let Some(bg_id) = light.bg {
        let bg = tokens.light_color(bg_id);
        if !bg.is_empty() {
            parts.push(format!("--slbg:{}", lctx.token_bg(bg)));
        }
    }

    if lctx.is_dual
        && let Some(dark_style) = dark
    {
        if let Some(color_id) = dark_style.color {
            let color = tokens.dark_color(color_id).unwrap_or("");
            if !color.is_empty() {
                parts.push(format!("--sd:{}", lctx.token_color(color, true)));
            }
        }
        if let Some(bg_id) = dark_style.bg {
            let bg = tokens.dark_color(bg_id).unwrap_or("");
            if !bg.is_empty() {
                parts.push(format!("--sdbg:{}", lctx.token_bg(bg)));
            }
        }
    }

    let fs = light.font_style;
    if fs.contains(FontStyle::ITALIC) {
        parts.push("--sfs:italic".to_owned());
    }
    if fs.contains(FontStyle::BOLD) {
        parts.push("--sfw:bold".to_owned());
    }
    let has_underline = fs.contains(FontStyle::UNDERLINE);
    let has_strike = fs.contains(FontStyle::STRIKETHROUGH);
    if has_underline || has_strike {
        let dec = match (has_underline, has_strike) {
            (true, true) => "underline line-through",
            (true, false) => "underline",
            (false, true) => "line-through",
            _ => unreachable!(),
        };
        parts.push(format!("--std:{}", dec));
    }

    if parts.is_empty() {
        return String::new();
    }
    escape_attr(&parts.join(";"))
}

fn encode_for_data_code(code: &str) -> String {
    code.replace('\n', "\x7f")
}

pub(crate) fn digit_count(n: usize) -> usize {
    if n == 0 {
        return 1;
    }
    let mut count = 0;
    let mut n = n;
    while n > 0 {
        count += 1;
        n /= 10;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_lang_known() {
        assert_eq!(display_lang("javascript"), "JavaScript");
        assert_eq!(display_lang("css"), "CSS");
        assert_eq!(display_lang("graphql"), "GraphQL");
    }

    #[test]
    fn display_lang_capitalizes_first() {
        assert_eq!(display_lang("rust"), "Rust");
        assert_eq!(display_lang("go"), "Go");
    }

    #[test]
    fn display_lang_empty() {
        assert_eq!(display_lang(""), "");
    }

    #[test]
    fn digit_count_values() {
        assert_eq!(digit_count(0), 1);
        assert_eq!(digit_count(1), 1);
        assert_eq!(digit_count(9), 1);
        assert_eq!(digit_count(10), 2);
        assert_eq!(digit_count(99), 2);
        assert_eq!(digit_count(100), 3);
        assert_eq!(digit_count(1000), 4);
    }

    #[test]
    fn file_ext_cases() {
        assert_eq!(file_ext("app.rs"), Some("rs"));
        assert_eq!(file_ext("archive.tar.gz"), Some("gz"));
        assert_eq!(file_ext("Makefile"), None);
        assert_eq!(file_ext("trailing."), None);
        assert_eq!(file_ext(".env"), Some("env"));
    }

    #[test]
    fn block_id_is_fnv1a_32() {
        assert_eq!(block_id(""), "811c9dc5");
        assert_eq!(block_id("a"), "e40c292c");
        assert_eq!(block_id("foobar"), "bf9cf968");
    }

    #[test]
    fn encode_for_data_code_replaces_newlines() {
        assert_eq!(encode_for_data_code("a\nb\nc"), "a\x7fb\x7fc");
        assert_eq!(encode_for_data_code("no newlines"), "no newlines");
    }
}
