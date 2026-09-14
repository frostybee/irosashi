use std::fmt::Write;

use crate::config::Config;
use crate::types::{DarkMode, ThemeInfo};

struct Var {
    name: &'static str,
    value: String,
}

fn nv(name: &'static str, value: impl Into<String>) -> Var {
    Var {
        name,
        value: value.into(),
    }
}

pub fn generate_vars(cfg: &Config, light: &ThemeInfo, dark: Option<&ThemeInfo>) -> String {
    let mut sb = String::with_capacity(4096);

    let static_vars = build_static_vars(cfg);
    let light_vars = build_theme_vars(light, cfg);

    let dark_diff = if let Some(dark) = dark {
        let dark_vars = build_theme_vars(dark, cfg);
        let light_map: std::collections::HashMap<&str, &str> = light_vars
            .iter()
            .map(|v| (v.name, v.value.as_str()))
            .collect();
        dark_vars
            .into_iter()
            .filter(|v| light_map.get(v.name) != Some(&v.value.as_str()))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let root = ":root";

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

    if cfg.dark_theme.is_none() {
        return sb;
    }

    let dark_rules = token_switch_rule(".kazari-block", "--sd", "--sdbg");

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

fn is_light(bg: &str) -> bool {
    let hex = bg.trim_start_matches('#');
    if hex.len() < 6 {
        return true;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(128);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(128);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(128);
    let luminance = 0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64;
    luminance > 128.0
}

fn build_theme_vars(tc: &ThemeInfo, cfg: &Config) -> Vec<Var> {
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
        if cfg.themed_scrollbars {
            vars.extend([
                nv("--kz-scrollbar-thumb", "rgba(0, 0, 0, 0.2)"),
                nv("--kz-scrollbar-thumb-hover", "rgba(0, 0, 0, 0.35)"),
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
        if cfg.themed_scrollbars {
            vars.extend([
                nv("--kz-scrollbar-thumb", "rgba(255, 255, 255, 0.15)"),
                nv("--kz-scrollbar-thumb-hover", "rgba(255, 255, 255, 0.3)"),
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

    if !tc.selection_bg.is_empty() {
        vars.push(nv("--kz-selection-bg", &tc.selection_bg));
    }

    vars
}

fn build_static_vars(cfg: &Config) -> Vec<Var> {
    let vars = vec![
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
        nv("--kz-mark-accent-margin", "0rem"),
        nv("--kz-diff-indicator-margin", "0.3rem"),
        nv("--kz-ins-indicator-color", "rgba(46,160,67,0.8)"),
        nv("--kz-del-indicator-color", "rgba(248,81,73,0.8)"),
        nv("--kz-label-mark-bg", "rgba(255,200,0,0.35)"),
        nv("--kz-label-ins-bg", "rgba(46,160,67,0.35)"),
        nv("--kz-label-del-bg", "rgba(248,81,73,0.35)"),
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
    ];

    let _ = cfg;
    vars
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_light_detects_correctly() {
        assert!(is_light("#ffffff"));
        assert!(is_light("#f0f0f0"));
        assert!(!is_light("#1e1e1e"));
        assert!(!is_light("#000000"));
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
        let names: Vec<&str> = vars.iter().map(|v| v.name).collect();
        assert!(names.contains(&"--kz-radius"));
        assert!(names.contains(&"--kz-font-family"));
        assert!(names.contains(&"--kz-mark-bg"));
        assert!(names.contains(&"--kz-terminal-dot-red"));
    }
}
