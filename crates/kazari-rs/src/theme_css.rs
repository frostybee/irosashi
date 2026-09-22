use std::fmt::Write;

use crate::color::{is_light, set_alpha};
use crate::config::Config;
use crate::types::{DarkMode, ThemeInfo};

const COLLAPSE_EXPAND_ICON: &str = r#"url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16'%3E%3Cpath d='m8.177.677 2.896 2.896a.25.25 0 0 1-.177.427H8.75v1.25a.75.75 0 0 1-1.5 0V4H5.104a.25.25 0 0 1-.177-.427L7.823.677a.25.25 0 0 1 .354 0ZM7.25 10.75a.75.75 0 0 1 1.5 0V12h2.146a.25.25 0 0 1 .177.427l-2.896 2.896a.25.25 0 0 1-.354 0l-2.896-2.896A.25.25 0 0 1 5.104 12H7.25v-1.25Zm-5-2a.75.75 0 0 0 0-1.5h-.5a.75.75 0 0 0 0 1.5h.5ZM6 8a.75.75 0 0 1-.75.75h-.5a.75.75 0 0 1 0-1.5h.5A.75.75 0 0 1 6 8Zm2.25.75a.75.75 0 0 0 0-1.5h-.5a.75.75 0 0 0 0 1.5h.5ZM12 8a.75.75 0 0 1-.75.75h-.5a.75.75 0 0 1 0-1.5h.5A.75.75 0 0 1 12 8Zm2.25.75a.75.75 0 0 0 0-1.5h-.5a.75.75 0 0 0 0 1.5h.5Z'/%3E%3C/svg%3E")"#;

const COLLAPSE_COLLAPSE_ICON: &str = r#"url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16'%3E%3Cpath d='M10.896 2H8.75V.75a.75.75 0 0 0-1.5 0V2H5.104a.25.25 0 0 0-.177.427l2.896 2.896a.25.25 0 0 0 .354 0l2.896-2.896A.25.25 0 0 0 10.896 2ZM8.75 15.25a.75.75 0 0 1-1.5 0V14H5.104a.25.25 0 0 1-.177-.427l2.896-2.896a.25.25 0 0 1 .354 0l2.896 2.896a.25.25 0 0 1-.177.427H8.75v1.25Zm-6.5-6.5a.75.75 0 0 0 0-1.5h-.5a.75.75 0 0 0 0 1.5h.5ZM6 8a.75.75 0 0 1-.75.75h-.5a.75.75 0 0 1 0-1.5h.5A.75.75 0 0 1 6 8Zm2.25.75a.75.75 0 0 0 0-1.5h-.5a.75.75 0 0 0 0 1.5h.5ZM12 8a.75.75 0 0 1-.75.75h-.5a.75.75 0 0 1 0-1.5h.5A.75.75 0 0 1 12 8Zm2.25.75a.75.75 0 0 0 0-1.5h-.5a.75.75 0 0 0 0 1.5h.5Z'/%3E%3C/svg%3E")"#;

/// The `--kz-ansi-*` variables an ANSI block's standard colours resolve through,
/// in SGR order (30 to 37, then 90 to 97). The defaults are the Tango palette.
/// The 16 standard colours as VS Code's terminal draws them, the values an `ansi`
/// block's tokens carry; index-aligned with `ANSI_PALETTE`.
pub(crate) const ANSI_STANDARD_COLORS: [&str; 16] = [
    "#000000", "#cd3131", "#0dbc79", "#e5e510", "#2472c8", "#bc3fbc", "#11a8cd", "#e5e5e5",
    "#666666", "#f14c4c", "#23d18b", "#f5f543", "#3b8eea", "#d670d6", "#29b8db", "#e5e5e5",
];

pub(crate) const ANSI_PALETTE: [(&str, &str); 16] = [
    ("black", "#000000"),
    ("red", "#cc0000"),
    ("green", "#4e9a06"),
    ("yellow", "#c4a000"),
    ("blue", "#3465a4"),
    ("magenta", "#75507b"),
    ("cyan", "#06989a"),
    ("white", "#d3d7cf"),
    ("bright-black", "#555753"),
    ("bright-red", "#ef2929"),
    ("bright-green", "#8ae234"),
    ("bright-yellow", "#fce94f"),
    ("bright-blue", "#729fcf"),
    ("bright-magenta", "#ad7fa8"),
    ("bright-cyan", "#34e2e2"),
    ("bright-white", "#eeeeec"),
];

struct Var {
    name: String,
    value: String,
}

fn nv(name: impl Into<String>, value: impl Into<String>) -> Var {
    Var {
        name: name.into(),
        value: value.into(),
    }
}

pub fn generate_vars(cfg: &Config, light: &ThemeInfo, dark: Option<&ThemeInfo>) -> String {
    let mut sb = String::with_capacity(4096);

    let static_vars = build_static_vars(cfg);
    let mut light_vars = build_theme_vars(light, cfg);

    let mut dark_diff = if let Some(dark) = dark {
        let dark_vars = build_theme_vars(dark, cfg);
        let light_map: std::collections::HashMap<&str, &str> = light_vars
            .iter()
            .map(|v| (v.name.as_str(), v.value.as_str()))
            .collect();
        dark_vars
            .into_iter()
            .filter(|v| light_map.get(v.name.as_str()) != Some(&v.value.as_str()))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    for (name, sv) in &cfg.style_overrides {
        if !sv.light_value().is_empty() {
            light_vars.push(nv(name.clone(), sv.light_value()));
        }
        if dark.is_some() && sv.is_themed() && !sv.dark_value().is_empty() {
            dark_diff.push(nv(name.clone(), sv.dark_value()));
        }
    }

    let root = cfg.theme_css_root.as_str();

    match &cfg.dark_mode {
        DarkMode::Selector(sel) => {
            write_block(&mut sb, root, &static_vars, &light_vars);
            if !dark_diff.is_empty() {
                write_block(&mut sb, &format!("{root}{sel}"), &[], &dark_diff);
            }
        }
        DarkMode::MediaQuery => {
            write_block(&mut sb, root, &static_vars, &light_vars);
            if !dark_diff.is_empty() {
                sb.push_str("@media (prefers-color-scheme: dark) {\n");
                write_block(&mut sb, root, &[], &dark_diff);
                sb.push_str("}\n");
            }
        }
        DarkMode::Both(sel) => {
            write_block(&mut sb, root, &static_vars, &light_vars);
            if !dark_diff.is_empty() {
                sb.push_str("@media (prefers-color-scheme: dark) {\n");
                write_block(&mut sb, root, &[], &dark_diff);
                sb.push_str("}\n");
                write_block(&mut sb, &format!("{root}{sel}"), &[], &dark_diff);
            }
        }
    }

    sb
}

pub fn token_switching_css(cfg: &Config) -> String {
    let mut sb = String::with_capacity(512);

    sb.push_str(&token_switch_rule(".kazari-block", "--sl", "--slbg"));
    sb.push_str(&themed_rule(
        ".kazari-block.kz-themed",
        OverrideSlot::Light,
        true,
        cfg,
    ));

    if cfg.dark_theme.is_none() {
        return sb;
    }

    // The dark themed rule stays on one line: `write_scoped_rules` prefixes lines.
    let dark_rules = token_switch_rule(".kazari-block", "--sd", "--sdbg")
        + &themed_rule(".kazari-block.kz-themed", OverrideSlot::Dark, false, cfg);

    match &cfg.dark_mode {
        DarkMode::Selector(sel) => {
            write_scoped_rules(&mut sb, sel, &dark_rules);
        }
        DarkMode::MediaQuery => {
            sb.push_str("@media (prefers-color-scheme: dark) {\n");
            sb.push_str(&dark_rules);
            sb.push_str("}\n");
        }
        DarkMode::Both(sel) => {
            sb.push_str("@media (prefers-color-scheme: dark) {\n");
            sb.push_str(&dark_rules);
            sb.push_str("}\n");
            write_scoped_rules(&mut sb, sel, &dark_rules);
        }
    }

    sb
}

#[derive(Clone, Copy)]
enum OverrideSlot {
    Light,
    Dark,
}

const COLLAPSE_GRADIENT_DECL: &str = "--kz-collapse-gradient-end: var(--kz-editor-bg); ";

/// Maps the `--kz-ovl-*` / `--kz-ovd-*` variables of a `kz-themed` block back onto
/// the regular `--kz-*` names; the dark slot falls back to the light value.
fn themed_rule(selector: &str, slot: OverrideSlot, gradient: bool, cfg: &Config) -> String {
    let mut sb = String::with_capacity(1024);
    sb.push_str(selector);
    sb.push_str(" { ");
    for name in overridable_var_names(cfg) {
        let suffix = name.trim_start_matches("--kz-");
        match slot {
            OverrideSlot::Light => write!(sb, "{name}: var(--kz-ovl-{suffix}); ").unwrap(),
            OverrideSlot::Dark => write!(
                sb,
                "{name}: var(--kz-ovd-{suffix}, var(--kz-ovl-{suffix})); "
            )
            .unwrap(),
        }
    }
    if gradient && cfg.collapsible.is_some() {
        sb.push_str(COLLAPSE_GRADIENT_DECL);
    }
    sb.push_str("}\n");
    sb
}

/// The variable names a block may override; the set depends on the config only, so
/// a placeholder theme is enough to enumerate it.
fn overridable_var_names(cfg: &Config) -> Vec<String> {
    let placeholder = ThemeInfo {
        bg: "#ffffff".to_owned(),
        fg: "#000000".to_owned(),
        ..Default::default()
    };
    block_overridable_vars(&placeholder, cfg)
        .into_iter()
        .map(|v| v.name)
        .collect()
}

/// Inline declarations for a per-block theme override: light values as
/// `--kz-ovl-*`, dark values as `--kz-ovd-*`. Empty when the light colours are
/// unusable.
pub fn block_override_style(cfg: &Config, light: &ThemeInfo, dark: Option<&ThemeInfo>) -> String {
    if light.bg.is_empty() || light.fg.is_empty() {
        return String::new();
    }
    let mut sb = String::with_capacity(1024);
    write_override_prefixed(&mut sb, "--kz-ovl-", &block_overridable_vars(light, cfg));
    if let Some(dark) = dark
        && cfg.dark_theme.is_some()
        && !dark.bg.is_empty()
        && !dark.fg.is_empty()
    {
        write_override_prefixed(&mut sb, "--kz-ovd-", &block_overridable_vars(dark, cfg));
    }
    sb.trim_end_matches(';').to_owned()
}

fn write_override_prefixed(sb: &mut String, prefix: &str, vars: &[Var]) {
    for v in vars {
        if v.value.is_empty() {
            continue;
        }
        write!(
            sb,
            "{prefix}{}:{};",
            v.name.trim_start_matches("--kz-"),
            v.value
        )
        .unwrap();
    }
}

fn token_switch_rule(selector: &str, color_var: &str, bg_var: &str) -> String {
    format!(
        "{} .kz-line span[style^=\"--\"] {{ color: var({}, inherit); background-color: var({}, transparent); font-style: var(--sfs, inherit); font-weight: var(--sfw, inherit); text-decoration: var(--std, inherit); }}\n",
        selector, color_var, bg_var,
    )
}

fn write_scoped_rules(sb: &mut String, selector: &str, rules: &str) {
    for line in rules.trim_end_matches('\n').split('\n') {
        writeln!(sb, "{} {}", selector, line).unwrap();
    }
}

fn write_block(sb: &mut String, selector: &str, static_vars: &[Var], theme_vars: &[Var]) {
    writeln!(sb, "{} {{", selector).unwrap();
    for v in static_vars {
        writeln!(sb, "  {}: {};", v.name, v.value).unwrap();
    }
    for v in theme_vars {
        writeln!(sb, "  {}: {};", v.name, v.value).unwrap();
    }
    sb.push_str("}\n");
}

/// Rules that let a block override the page theme through `data-kz-theme`.
pub fn theme_toggle_css(cfg: &Config, light: &ThemeInfo, dark: Option<&ThemeInfo>) -> String {
    let Some(dark) = dark else {
        return String::new();
    };
    if !cfg.theme_toggle {
        return String::new();
    }
    let mut sb = String::with_capacity(2048);
    let mut dark_vars = block_overridable_vars(dark, cfg);
    let mut light_vars = block_overridable_vars(light, cfg);
    if cfg.collapsible.is_some() {
        dark_vars.push(nv("--kz-collapse-gradient-end", "var(--kz-editor-bg)"));
        light_vars.push(nv("--kz-collapse-gradient-end", "var(--kz-editor-bg)"));
    }
    write_toggle_vars(&mut sb, ".kazari-block[data-kz-theme=\"dark\"]", &dark_vars);
    write_toggle_vars(
        &mut sb,
        ".kazari-block[data-kz-theme=\"light\"]",
        &light_vars,
    );
    sb.push_str(".kazari-block[data-kz-theme] { --kz-terminal-bg: var(--kz-editor-bg); --kz-terminal-titlebar-bg: var(--kz-toolbar-bg); }\n");
    sb.push_str(&token_switch_rule(
        ".kazari-block[data-kz-theme=\"dark\"]",
        "--sd",
        "--sdbg",
    ));
    sb.push_str(&token_switch_rule(
        ".kazari-block[data-kz-theme=\"light\"]",
        "--sl",
        "--slbg",
    ));
    sb.push_str(&themed_rule(
        ".kazari-block.kz-themed[data-kz-theme=\"dark\"]",
        OverrideSlot::Dark,
        true,
        cfg,
    ));
    sb.push_str(&themed_rule(
        ".kazari-block.kz-themed[data-kz-theme=\"light\"]",
        OverrideSlot::Light,
        true,
        cfg,
    ));
    sb
}

fn write_toggle_vars(sb: &mut String, selector: &str, vars: &[Var]) {
    sb.push_str(selector);
    sb.push_str(" { ");
    for v in vars {
        if !v.value.is_empty() {
            write!(sb, "{}: {}; ", v.name, v.value).unwrap();
        }
    }
    sb.push_str("}\n");
}

fn build_theme_vars(tc: &ThemeInfo, cfg: &Config) -> Vec<Var> {
    let mut vars = block_overridable_vars(tc, cfg);
    if !tc.selection_bg.is_empty() {
        vars.push(nv("--kz-selection-bg", &tc.selection_bg));
    }
    // Section colours follow the theme's fold background; emitted after the static
    // defaults so they win. Not gated on the config, meta ranges render anywhere.
    if !tc.fold_bg.is_empty() {
        vars.push(nv("--kz-collapse-closed-bg", set_alpha(&tc.fold_bg, 0.2)));
        vars.push(nv(
            "--kz-collapse-closed-border",
            set_alpha(&tc.fold_bg, 0.5),
        ));
    }
    vars
}

/// The theme variables a single block can override; the page-only extras
/// (selection) are layered on top by `build_theme_vars`.
fn block_overridable_vars(tc: &ThemeInfo, cfg: &Config) -> Vec<Var> {
    let mut vars = vec![nv("--kz-editor-bg", &tc.bg), nv("--kz-editor-fg", &tc.fg)];

    if !tc.line_number_fg.is_empty() {
        vars.push(nv("--kz-ln-fg", &tc.line_number_fg));
    }

    if is_light(&tc.bg) {
        if tc.line_number_fg.is_empty() {
            vars.push(nv("--kz-ln-fg", "#6e7781"));
        }
        vars.extend([
            nv("--kz-ln-highlight-fg", "#24292f"),
            nv("--kz-gutter-border-color", "rgba(0,0,0,0.1)"),
            nv("--kz-toolbar-bg", "rgba(229, 231, 235, 0.15)"),
            nv("--kz-toolbar-border", "rgba(209, 213, 219, 0.5)"),
            nv("--kz-lang-fg", "#4b5563"),
            nv("--kz-copy-fg", "#4b5563"),
            nv("--kz-copy-fg-hover", "#111827"),
            nv("--kz-copy-bg-hover", "rgba(156, 163, 175, 0.2)"),
        ]);
        if cfg.collapsible.is_some() {
            vars.extend([
                nv("--kz-collapse-btn-fg", "#4b5563"),
                nv("--kz-collapse-btn-bg", "rgba(0, 0, 0, 0.04)"),
                nv("--kz-collapse-btn-hover-bg", "rgba(0, 0, 0, 0.08)"),
                nv("--kz-collapse-btn-border", "rgba(0, 0, 0, 0.15)"),
                nv("--kz-collapse-btn-border-hover", "rgba(0, 0, 0, 0.3)"),
            ]);
        }
        if cfg.themed_scrollbars {
            vars.extend([
                nv("--kz-scrollbar-thumb", "rgba(0, 0, 0, 0.2)"),
                nv("--kz-scrollbar-thumb-hover", "rgba(0, 0, 0, 0.35)"),
            ]);
        }
        if cfg.code_groups {
            vars.extend([
                nv("--kz-group-tab-active-bg", "rgba(255,255,255,0.8)"),
                nv("--kz-group-tab-active-fg", "#24292f"),
                nv("--kz-group-border", "rgba(0,0,0,0.1)"),
            ]);
        }
    } else {
        if tc.line_number_fg.is_empty() {
            vars.push(nv("--kz-ln-fg", "#6e7681"));
        }
        vars.extend([
            nv("--kz-ln-highlight-fg", "#e6edf3"),
            nv("--kz-gutter-border-color", "rgba(255,255,255,0.1)"),
            nv("--kz-toolbar-bg", "rgba(39, 39, 42, 0.6)"),
            nv("--kz-toolbar-border", "rgba(63, 63, 70, 0.4)"),
            nv("--kz-lang-fg", "#a1a1aa"),
            nv("--kz-copy-fg", "#d4d4d8"),
            nv("--kz-copy-fg-hover", "#ffffff"),
            nv("--kz-copy-bg-hover", "rgba(63, 63, 70, 0.8)"),
        ]);
        if cfg.collapsible.is_some() {
            vars.extend([
                nv("--kz-collapse-btn-fg", "#d4d4d8"),
                nv("--kz-collapse-btn-bg", "rgba(255, 255, 255, 0.1)"),
                nv("--kz-collapse-btn-hover-bg", "rgba(255, 255, 255, 0.18)"),
                nv("--kz-collapse-btn-border", "rgba(255, 255, 255, 0.2)"),
                nv(
                    "--kz-collapse-btn-border-hover",
                    "rgba(255, 255, 255, 0.35)",
                ),
            ]);
        }
        if cfg.themed_scrollbars {
            vars.extend([
                nv("--kz-scrollbar-thumb", "rgba(255, 255, 255, 0.15)"),
                nv("--kz-scrollbar-thumb-hover", "rgba(255, 255, 255, 0.3)"),
            ]);
        }
        if cfg.code_groups {
            vars.extend([
                nv("--kz-group-tab-active-bg", "rgba(255,255,255,0.1)"),
                nv("--kz-group-tab-active-fg", "#e6edf3"),
                nv("--kz-group-border", "rgba(255,255,255,0.1)"),
            ]);
        }
    }

    if cfg.terminal_dot_style == crate::types::TerminalDotStyle::Minimal {
        if is_light(&tc.bg) {
            vars.push(nv("--kz-terminal-dots-fg", "#24292f"));
        } else {
            vars.push(nv("--kz-terminal-dots-fg", "#e6edf3"));
        }
    }

    vars
}

fn build_static_vars(cfg: &Config) -> Vec<Var> {
    let mut vars = vec![
        nv("--kz-radius", "0.5rem"),
        nv("--kz-shadow", "0 2px 8px rgba(0,0,0,0.15)"),
        nv("--kz-border", "1px solid transparent"),
        nv("--kz-transition", "150ms ease"),
        nv("--kz-font-family", "'JetBrains Mono Variable', monospace"),
        nv("--kz-font-size", "0.875rem"),
        nv("--kz-font-weight", "500"),
        nv("--kz-line-height", "1.6"),
        nv("--kz-ui-font-family", "system-ui, sans-serif"),
        nv("--kz-ui-font-size", "0.9rem"),
        nv("--kz-ui-font-weight", "400"),
        nv("--kz-ui-line-height", "1.65"),
        nv("--kz-code-padding-block", "1rem"),
        nv("--kz-code-padding-inline", "1.35rem"),
        nv("--kz-title-font-size", "0.8rem"),
        nv("--kz-title-padding", "0.5rem 1rem"),
        nv("--kz-ln-padding-inline", "2ch"),
        nv("--kz-gutter-border-width", "1px"),
        nv("--kz-mark-bg", "rgba(255,200,0,0.12)"),
        nv("--kz-mark-border", "rgba(255,200,0,0.5)"),
        nv("--kz-mark-border-width", "3px"),
        nv("--kz-ins-bg", "rgba(46,160,67,0.12)"),
        nv("--kz-ins-border", "rgba(46,160,67,0.5)"),
        nv("--kz-ins-indicator", "'+'"),
        nv("--kz-del-bg", "rgba(248,81,73,0.12)"),
        nv("--kz-del-border", "rgba(248,81,73,0.5)"),
        nv("--kz-del-indicator", "'-'"),
        nv("--kz-error-bg", "rgba(220,38,38,0.12)"),
        nv("--kz-error-border", "rgba(220,38,38,0.5)"),
        nv("--kz-warning-bg", "rgba(245,158,11,0.12)"),
        nv("--kz-warning-border", "rgba(245,158,11,0.5)"),
        nv("--kz-mark-accent-margin", "0rem"),
        nv("--kz-diff-indicator-margin", "0.3rem"),
        nv("--kz-ins-indicator-color", "rgba(46,160,67,0.8)"),
        nv("--kz-del-indicator-color", "rgba(248,81,73,0.8)"),
        nv("--kz-label-mark-bg", "rgba(255,200,0,0.35)"),
        nv("--kz-label-ins-bg", "rgba(46,160,67,0.35)"),
        nv("--kz-label-del-bg", "rgba(248,81,73,0.35)"),
        nv("--kz-label-error-bg", "rgba(220,38,38,0.35)"),
        nv("--kz-label-warning-bg", "rgba(245,158,11,0.35)"),
        nv("--kz-label-fg", "#ffffff"),
        nv("--kz-label-padding", "0.1rem 0.3rem"),
        nv("--kz-label-font-size", "0.75rem"),
        nv("--kz-label-radius", "0.2rem"),
        nv("--kz-inline-mark-bg", "rgba(255,200,0,0.2)"),
        nv("--kz-inline-mark-border", "rgba(255,200,0,0.5)"),
        nv("--kz-inline-mark-radius", "0.2rem"),
        nv("--kz-inline-mark-padding", "0.15rem"),
        nv("--kz-inline-mark-border-width", "1.5px"),
        nv("--kz-inline-ins-bg", "rgba(46,160,67,0.2)"),
        nv("--kz-inline-ins-border", "rgba(46,160,67,0.5)"),
        nv("--kz-inline-del-bg", "rgba(248,81,73,0.2)"),
        nv("--kz-inline-del-border", "rgba(248,81,73,0.5)"),
        nv("--kz-focus-dimmed-opacity", "0.35"),
        nv("--kz-focus-ring", "rgb(59,130,246)"),
        nv("--kz-toolbar-padding", "0.25rem 1rem"),
        nv("--kz-terminal-header-padding", "0.5rem 1rem"),
        nv("--kz-lang-font-size", "0.8rem"),
        nv("--kz-lang-font-weight", "500"),
        nv("--kz-separator-color", "rgba(161, 161, 170, 0.3)"),
        nv("--kz-copy-radius", "0.375rem"),
        nv("--kz-copy-success-bg", "rgba(34, 197, 94, 0.9)"),
        nv("--kz-copy-success-fg", "#ffffff"),
        nv("--kz-copy-success-border", "rgba(34, 197, 94, 0.8)"),
        nv("--kz-tooltip-bg", "rgba(30,30,30,0.92)"),
        nv("--kz-tooltip-fg", "#ffffff"),
        nv("--kz-tooltip-font-size", "0.75rem"),
        nv("--kz-tooltip-padding", "0.35rem 0.75rem"),
        nv("--kz-tooltip-radius", "6px"),
        nv("--kz-tooltip-offset", "6px"),
        nv("--kz-tooltip-shadow", "0 2px 6px rgba(0,0,0,0.25)"),
        nv("--kz-tooltip-arrow-size", "5px"),
        nv("--kz-terminal-bg", "var(--kz-editor-bg)"),
        nv("--kz-terminal-titlebar-bg", "var(--kz-toolbar-bg)"),
        nv("--kz-terminal-dot-red", "#ff5f57"),
        nv("--kz-terminal-dot-yellow", "#febc2e"),
        nv("--kz-terminal-dot-green", "#28c840"),
        nv("--kz-ln-width", "2ch"),
        nv("--kz-ln-opacity", "1"),
        nv("--kz-ln-highlight-opacity", "0.8"),
        // Never gated on the config: a meta `collapse={ranges}` renders sections on
        // any engine and collapsible.css resolves these without fallbacks.
        nv("--kz-collapse-btn-bg", "rgba(0,0,0,0.04)"),
        nv("--kz-collapse-btn-fg", "#4b5563"),
        nv("--kz-collapse-btn-hover-bg", "rgba(0,0,0,0.08)"),
        nv("--kz-collapse-gradient-start", "transparent"),
        nv("--kz-collapse-gradient-end", "var(--kz-editor-bg)"),
        nv("--kz-collapse-transition", "300ms ease"),
        nv("--kz-collapse-closed-bg", "rgb(84 174 255 / 20%)"),
        nv("--kz-collapse-closed-border", "rgb(84 174 255 / 50%)"),
        nv("--kz-collapse-closed-border-width", "0"),
        nv("--kz-collapse-closed-padding", "4px"),
        nv("--kz-collapse-open-bg", "transparent"),
        nv("--kz-collapse-open-bg-collapsible", "rgb(84 174 255 / 10%)"),
        nv("--kz-collapse-open-border", "transparent"),
        nv("--kz-collapse-open-border-width", "1px"),
        nv("--kz-collapse-closed-fg", "currentColor"),
        nv("--kz-collapse-closed-font-family", "inherit"),
        nv("--kz-collapse-closed-font-size", "inherit"),
        nv("--kz-collapse-closed-line-height", "inherit"),
        nv("--kz-collapse-expand-icon", COLLAPSE_EXPAND_ICON),
        nv("--kz-collapse-collapse-icon", COLLAPSE_COLLAPSE_ICON),
    ];

    if cfg.file_icons {
        vars.extend([
            nv("--kz-file-icon-size", "1rem"),
            nv("--kz-file-icon-margin", "0"),
            nv("--kz-file-icon-opacity", "0.8"),
        ]);
    }

    if cfg.lang_icon_mode != crate::config::LangIconMode::None {
        vars.extend([
            nv("--kz-lang-icon-size", "1.25rem"),
            nv("--kz-lang-icon-margin", "0"),
            nv("--kz-lang-icon-opacity", "0.8"),
        ]);
    }

    if cfg.fullscreen_button {
        vars.push(nv("--kz-fs-font-scale", "1"));
    }

    vars.extend(
        ANSI_PALETTE
            .iter()
            .map(|(name, value)| nv(format!("--kz-ansi-{name}"), *value)),
    );

    if cfg.code_groups {
        vars.extend([
            nv("--kz-group-tab-bg", "transparent"),
            nv("--kz-group-tab-fg", "inherit"),
            nv("--kz-group-tab-active-border", "#007acc"),
            nv("--kz-group-tab-padding", "0.5rem 1rem"),
            nv("--kz-group-border-width", "1px"),
            nv("--kz-group-radius", "var(--kz-radius)"),
        ]);
    }

    vars
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "irosashi")]
    #[test]
    fn ansi_standard_colors_match_irosashi() {
        assert_eq!(super::ANSI_STANDARD_COLORS, irosashi::ANSI_STANDARD_COLORS);
    }

    use super::*;

    #[test]
    fn fold_background_drives_section_colours() {
        let cfg = Config::default();
        let light = ThemeInfo {
            fg: "#333".into(),
            bg: "#fff".into(),
            fold_bg: "#d3d3d3".into(),
            ..Default::default()
        };
        let css = generate_vars(&cfg, &light, None);
        let default_pos = css
            .find("--kz-collapse-closed-bg: rgb(84 174 255 / 20%);")
            .unwrap();
        let themed_pos = css.find("--kz-collapse-closed-bg: #d3d3d333;").unwrap();
        assert!(themed_pos > default_pos, "theme value must win the cascade");
        assert!(css.contains("--kz-collapse-closed-border: #d3d3d380;"));
    }

    #[test]
    fn generate_vars_selector_mode() {
        let cfg = Config {
            dark_mode: DarkMode::Selector(".dark".into()),
            ..Default::default()
        };
        let light = ThemeInfo {
            fg: "#24292e".into(),
            bg: "#ffffff".into(),
            selection_bg: "#0366d6".into(),
            line_number_fg: "#1b1f234d".into(),
            fold_bg: String::new(),
        };
        let dark = ThemeInfo {
            fg: "#e1e4e8".into(),
            bg: "#24292e".into(),
            selection_bg: "#3392ff44".into(),
            line_number_fg: "#e1e4e84d".into(),
            fold_bg: String::new(),
        };
        let css = generate_vars(&cfg, &light, Some(&dark));
        assert!(css.contains(":root {"));
        assert!(css.contains(":root.dark {"));
        assert!(css.contains("--kz-editor-bg: #ffffff"));
        assert!(css.contains("--kz-editor-bg: #24292e"));
    }

    #[test]
    fn generate_vars_media_query_mode() {
        let cfg = Config {
            dark_mode: DarkMode::MediaQuery,
            ..Default::default()
        };
        let light = ThemeInfo {
            fg: "#333".into(),
            bg: "#fff".into(),
            ..Default::default()
        };
        let dark = ThemeInfo {
            fg: "#eee".into(),
            bg: "#222".into(),
            ..Default::default()
        };
        let css = generate_vars(&cfg, &light, Some(&dark));
        assert!(css.contains("@media (prefers-color-scheme: dark)"));
    }

    #[test]
    fn generate_vars_both_mode() {
        let cfg = Config {
            dark_mode: DarkMode::Both(".dark".into()),
            ..Default::default()
        };
        let light = ThemeInfo {
            fg: "#333".into(),
            bg: "#fff".into(),
            ..Default::default()
        };
        let dark = ThemeInfo {
            fg: "#eee".into(),
            bg: "#222".into(),
            ..Default::default()
        };
        let css = generate_vars(&cfg, &light, Some(&dark));
        assert!(css.contains("@media (prefers-color-scheme: dark)"));
        assert!(css.contains(":root.dark {"));
    }

    #[test]
    fn generate_vars_single_theme() {
        let cfg = Config {
            dark_theme: None,
            ..Default::default()
        };
        let light = ThemeInfo {
            fg: "#333".into(),
            bg: "#fff".into(),
            ..Default::default()
        };
        let css = generate_vars(&cfg, &light, None);
        assert!(css.contains(":root {"));
        assert!(!css.contains(".dark"));
        assert!(!css.contains("@media"));
    }

    #[test]
    fn token_switching_css_has_base_rule() {
        let cfg = Config::default();
        let css = token_switching_css(&cfg);
        assert!(css.contains(".kazari-block .kz-line span[style^=\"--\"]"));
        assert!(css.contains("color: var(--sl, inherit)"));
    }

    #[test]
    fn token_switching_css_selector_dark() {
        let cfg = Config {
            dark_mode: DarkMode::Selector(".dark".into()),
            ..Default::default()
        };
        let css = token_switching_css(&cfg);
        assert!(css.contains(".dark .kazari-block .kz-line span[style^=\"--\"]"));
        assert!(css.contains("color: var(--sd, inherit)"));
    }

    #[test]
    fn token_switching_css_media_query_dark() {
        let cfg = Config {
            dark_mode: DarkMode::MediaQuery,
            ..Default::default()
        };
        let css = token_switching_css(&cfg);
        assert!(css.contains("@media (prefers-color-scheme: dark)"));
    }

    #[test]
    fn static_vars_include_core_properties() {
        let cfg = Config::default();
        let vars = build_static_vars(&cfg);
        let names: Vec<&str> = vars.iter().map(|v| v.name.as_str()).collect();
        assert!(names.contains(&"--kz-radius"));
        assert!(names.contains(&"--kz-font-family"));
        assert!(names.contains(&"--kz-mark-bg"));
        assert!(names.contains(&"--kz-terminal-dot-red"));
    }

    #[test]
    fn ansi_palette_is_always_emitted() {
        let cfg = Config {
            dark_theme: None,
            ..Default::default()
        };
        let light = ThemeInfo {
            fg: "#333".into(),
            bg: "#fff".into(),
            ..Default::default()
        };
        let css = generate_vars(&cfg, &light, None);
        assert!(css.contains("  --kz-ansi-red: #cc0000;\n"));
        assert!(css.contains("  --kz-ansi-bright-white: #eeeeec;\n"));
        assert_eq!(css.matches("--kz-ansi-").count(), 16);
    }

    #[test]
    fn theme_toggle_css_scopes_and_switch_rules() {
        let cfg = Config {
            theme_toggle: true,
            ..Default::default()
        };
        let light = ThemeInfo {
            fg: "#333".into(),
            bg: "#fff".into(),
            selection_bg: "#aaa".into(),
            ..Default::default()
        };
        let dark = ThemeInfo {
            fg: "#eee".into(),
            bg: "#222".into(),
            ..Default::default()
        };
        let css = theme_toggle_css(&cfg, &light, Some(&dark));
        let lines: Vec<&str> = css.lines().collect();
        assert_eq!(lines.len(), 7, "{css}");
        assert!(lines[5].starts_with(".kazari-block.kz-themed[data-kz-theme=\"dark\"] { --kz-editor-bg: var(--kz-ovd-editor-bg, var(--kz-ovl-editor-bg)); "));
        assert!(lines[6].starts_with(".kazari-block.kz-themed[data-kz-theme=\"light\"] { --kz-editor-bg: var(--kz-ovl-editor-bg); "));
        assert!(lines[0].starts_with(".kazari-block[data-kz-theme=\"dark\"] { --kz-editor-bg: #222; --kz-editor-fg: #eee; --kz-ln-fg: #6e7681; "), "{}", lines[0]);
        assert!(lines[0].ends_with("; }"));
        assert!(
            lines[1].starts_with(".kazari-block[data-kz-theme=\"light\"] { --kz-editor-bg: #fff; "),
            "{}",
            lines[1]
        );
        assert!(
            !lines[1].contains("--kz-selection-bg"),
            "page-only var stays out"
        );
        assert_eq!(
            lines[2],
            ".kazari-block[data-kz-theme] { --kz-terminal-bg: var(--kz-editor-bg); --kz-terminal-titlebar-bg: var(--kz-toolbar-bg); }"
        );
        assert!(lines[3].starts_with(".kazari-block[data-kz-theme=\"dark\"] .kz-line span[style^=\"--\"] { color: var(--sd, inherit); background-color: var(--sdbg, transparent);"));
        assert!(lines[4].starts_with(".kazari-block[data-kz-theme=\"light\"] .kz-line span[style^=\"--\"] { color: var(--sl, inherit);"));

        assert!(theme_toggle_css(&cfg, &light, None).is_empty());
        let off = Config::default();
        assert!(theme_toggle_css(&off, &light, Some(&dark)).is_empty());
    }

    #[test]
    fn theme_css_root_replaces_root_selector() {
        let cfg = Config {
            theme_css_root: ".docs".into(),
            dark_mode: DarkMode::Selector(".dark".into()),
            ..Default::default()
        };
        let light = ThemeInfo {
            fg: "#333".into(),
            bg: "#fff".into(),
            ..Default::default()
        };
        let dark = ThemeInfo {
            fg: "#eee".into(),
            bg: "#222".into(),
            ..Default::default()
        };
        let css = generate_vars(&cfg, &light, Some(&dark));
        assert!(css.starts_with(".docs {\n"), "{css}");
        assert!(css.contains("\n.docs.dark {\n"), "{css}");
        assert!(!css.contains(":root"), "{css}");
    }

    #[test]
    fn style_overrides_land_in_theme_scopes_sorted() {
        use crate::config::StyleValue;
        let mut cfg = Config {
            dark_mode: DarkMode::Selector(".dark".into()),
            ..Default::default()
        };
        cfg.style_overrides
            .insert("--kz-radius".into(), StyleValue::plain("0"));
        cfg.style_overrides.insert(
            "--kz-editor-bg".into(),
            StyleValue::themed("#fafafa", "#101010"),
        );
        cfg.style_overrides
            .insert("--kz-font-size".into(), StyleValue::themed("", "1rem"));
        let light = ThemeInfo {
            fg: "#333".into(),
            bg: "#fff".into(),
            ..Default::default()
        };
        let dark = ThemeInfo {
            fg: "#eee".into(),
            bg: "#222".into(),
            ..Default::default()
        };
        let css = generate_vars(&cfg, &light, Some(&dark));
        let (light_block, dark_block) = css.split_once(":root.dark {").unwrap();
        assert!(
            light_block.contains("  --kz-editor-bg: #ffffff;\n")
                || light_block.contains("  --kz-editor-bg: #fff;\n")
        );
        let bg_override = light_block.find("  --kz-editor-bg: #fafafa;\n").unwrap();
        let radius = light_block.find("  --kz-radius: 0;\n").unwrap();
        assert!(bg_override < radius, "sorted by name");
        assert!(
            light_block.rfind("--kz-editor-bg: #fafafa").unwrap()
                > light_block.find("--kz-editor-bg: #fff").unwrap(),
            "override comes after the theme value"
        );
        assert!(!light_block.contains("--kz-font-size: ;"));
        assert!(
            dark_block.contains("  --kz-editor-bg: #101010;\n"),
            "{dark_block}"
        );
        assert!(
            dark_block.contains("  --kz-font-size: 1rem;\n"),
            "{dark_block}"
        );
        assert!(!dark_block.contains("--kz-radius"), "{dark_block}");
    }
}
