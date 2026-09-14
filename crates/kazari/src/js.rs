use crate::config::Config;

static COPY_JS: &str = include_str!("../assets/js/copy.js");
static WRAP_JS: &str = include_str!("../assets/js/wrap.js");

pub fn generate(cfg: &Config) -> String {
    let mut sb = String::with_capacity(4096);

    if cfg.copy_button {
        sb.push_str(COPY_JS);
    }

    if cfg.wrap_button {
        sb.push_str(WRAP_JS);
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
            ..Default::default()
        };
        let js = generate(&cfg);
        assert!(js.is_empty());
    }

    #[test]
    fn static_js_files_non_empty() {
        assert!(!COPY_JS.is_empty());
        assert!(!WRAP_JS.is_empty());
    }
}
