use crate::config::Config;
use crate::types::ThemeInfo;

static BASE: &str = include_str!("../assets/css/base.css");
static LINE_NUMBERS: &str = include_str!("../assets/css/line-numbers.css");
static MARKERS: &str = include_str!("../assets/css/markers.css");
static INLINE_MARKERS: &str = include_str!("../assets/css/inline-markers.css");
static FOCUS: &str = include_str!("../assets/css/focus.css");
static FRAME: &str = include_str!("../assets/css/frame.css");
static TOOLBAR: &str = include_str!("../assets/css/toolbar.css");
static TOOLTIP: &str = include_str!("../assets/css/tooltip.css");
static TERMINAL: &str = include_str!("../assets/css/terminal.css");
static TERMINAL_DOTS_COLORED: &str = include_str!("../assets/css/terminal-dots-colored.css");
static TERMINAL_DOTS_MINIMAL: &str = include_str!("../assets/css/terminal-dots-minimal.css");
static COPY: &str = include_str!("../assets/css/copy.css");
static WRAP: &str = include_str!("../assets/css/wrap.css");
static SCROLLBAR: &str = include_str!("../assets/css/scrollbar.css");
static STYLE_RESET: &str = include_str!("../assets/css/style-reset.css");
static SELECTION: &str = include_str!("../assets/css/selection.css");

pub fn generate(cfg: &Config, light: &ThemeInfo, dark: Option<&ThemeInfo>) -> String {
    let mut sb = String::with_capacity(16384);

    sb.push_str(&crate::theme_css::generate_vars(cfg, light, dark));
    sb.push_str(&crate::theme_css::token_switching_css(cfg));

    sb.push_str(BASE);
    sb.push_str(LINE_NUMBERS);
    sb.push_str(MARKERS);
    sb.push_str(INLINE_MARKERS);
    sb.push_str(FOCUS);

    if cfg.style_reset {
        sb.push_str(STYLE_RESET);
    }
    if cfg.themed_scrollbars {
        sb.push_str(SCROLLBAR);
    }
    if cfg.themed_selection {
        sb.push_str(SELECTION);
    }

    sb.push_str(FRAME);
    sb.push_str(TOOLBAR);
    sb.push_str(TOOLTIP);
    sb.push_str(TERMINAL);
    sb.push_str(TERMINAL_DOTS_COLORED);
    sb.push_str(TERMINAL_DOTS_MINIMAL);

    if cfg.copy_button {
        sb.push_str(COPY);
    }

    sb.push_str(WRAP);

    sb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_includes_base_and_vars() {
        let cfg = Config::default();
        let light = ThemeInfo {
            fg: "#333".into(),
            bg: "#fff".into(),
            ..Default::default()
        };
        let css = generate(&cfg, &light, None);
        assert!(css.contains(":root {"));
        assert!(css.contains("--kz-radius"));
        assert!(css.contains(".kazari-block"));
    }

    #[test]
    fn generate_gates_copy_css() {
        let cfg = Config {
            copy_button: false,
            ..Default::default()
        };
        let light = ThemeInfo {
            fg: "#333".into(),
            bg: "#fff".into(),
            ..Default::default()
        };
        let css = generate(&cfg, &light, None);
        assert!(!css.contains(COPY));
    }

    #[test]
    fn all_static_files_non_empty() {
        assert!(!BASE.is_empty());
        assert!(!LINE_NUMBERS.is_empty());
        assert!(!MARKERS.is_empty());
        assert!(!INLINE_MARKERS.is_empty());
        assert!(!FOCUS.is_empty());
        assert!(!FRAME.is_empty());
        assert!(!TOOLBAR.is_empty());
        assert!(!TOOLTIP.is_empty());
        assert!(!TERMINAL.is_empty());
        assert!(!TERMINAL_DOTS_COLORED.is_empty());
        assert!(!TERMINAL_DOTS_MINIMAL.is_empty());
        assert!(!COPY.is_empty());
        assert!(!WRAP.is_empty());
        assert!(!SCROLLBAR.is_empty());
        assert!(!STYLE_RESET.is_empty());
        assert!(!SELECTION.is_empty());
    }
}
