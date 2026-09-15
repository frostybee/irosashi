use crate::config::Config;

static COPY_JS: &str = include_str!("../assets/js/copy.js");
static WRAP_JS: &str = include_str!("../assets/js/wrap.js");
static FULLSCREEN_JS: &str = include_str!("../assets/js/fullscreen.js");
static THEME_TOGGLE_JS: &str = include_str!("../assets/js/theme-toggle.js");
static OUTPUT_JS: &str = include_str!("../assets/js/output.js");
static COLLAPSIBLE_JS: &str = include_str!("../assets/js/collapsible.js");

pub fn generate(cfg: &Config) -> String {
    let mut sb = String::with_capacity(4096);

    if cfg.copy_button {
        sb.push_str(COPY_JS);
    }

    if cfg.wrap_button {
        sb.push_str(WRAP_JS);
    }

    if cfg.fullscreen_button {
        sb.push_str(FULLSCREEN_JS);
    }

    if cfg.theme_toggle && cfg.dark_theme.is_some() {
        sb.push_str(THEME_TOGGLE_JS);
    }

    if cfg.output_panel {
        sb.push_str(OUTPUT_JS);
    }

    if cfg.collapsible.is_some() {
        sb.push_str(COLLAPSIBLE_JS);
    }

    sb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_includes_copy_and_wrap() {
        let cfg = Config::default();
        let js = generate(&cfg);
        assert!(!js.is_empty());
    }

    #[test]
    fn generate_gates_features() {
        let cfg = Config {
            copy_button: false,
            wrap_button: false,
            fullscreen_button: false,
            ..Default::default()
        };
        let js = generate(&cfg);
        assert!(js.is_empty());
    }

    #[test]
    fn static_js_files_non_empty() {
        assert!(!COPY_JS.is_empty());
        assert!(!WRAP_JS.is_empty());
        assert!(!FULLSCREEN_JS.is_empty());
        assert!(!THEME_TOGGLE_JS.is_empty());
        assert!(!OUTPUT_JS.is_empty());
        assert!(!COLLAPSIBLE_JS.is_empty());
    }
}
