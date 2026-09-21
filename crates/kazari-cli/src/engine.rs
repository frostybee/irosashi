use std::path::{Path, PathBuf};

use clap::Args;
use kazari_rs::{Config, FileConfig, Kazari, ProcessFile};

use crate::Fail;

pub const DEFAULT_LIGHT_THEME: &str = "github-light";
pub const DEFAULT_DARK_THEME: &str = "github-dark";

const CONFIG_NAMES: [&str; 3] = [
    "kazari.config.yaml",
    "kazari.config.yml",
    "kazari.config.json",
];

#[derive(Args, Debug, Default, Clone)]
pub struct EngineArgs {
    /// Config file path (default: auto-discover kazari.config.yaml|.yml|.json in the target
    /// directory, then the working directory)
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Light syntax theme name (overrides config)
    #[arg(long, value_name = "NAME")]
    pub theme_light: Option<String>,

    /// Dark syntax theme name (overrides config)
    #[arg(long, value_name = "NAME")]
    pub theme_dark: Option<String>,

    /// Minimum WCAG contrast ratio of token colours, 0 to 21 (overrides config; 0 turns it off)
    #[arg(long, value_name = "RATIO")]
    pub min_contrast: Option<f64>,
}

/// A config file resolved from the flags, the target directory or the working directory.
pub struct LoadedConfig {
    pub path: PathBuf,
    pub file: FileConfig,
}

/// An explicit path must exist and parse; auto discovery probes the target dir then the
/// working directory and finding nothing is fine. Parse errors are always hard errors.
pub fn load_file_config(explicit: Option<&Path>, dir: &Path) -> Result<Option<LoadedConfig>, Fail> {
    if let Some(path) = explicit {
        return parse_config_file(path).map(Some);
    }
    for d in [dir, Path::new(".")] {
        for name in CONFIG_NAMES {
            let path = d.join(name);
            if path.is_file() {
                return parse_config_file(&path).map(Some);
            }
        }
    }
    Ok(None)
}

fn parse_config_file(path: &Path) -> Result<LoadedConfig, Fail> {
    let data = std::fs::read_to_string(path)
        .map_err(|e| Fail::new(format!("reading config {}: {e}", path.display())))?;
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    let parsed = match ext.as_str() {
        "yaml" | "yml" => FileConfig::from_yaml(&data),
        "json" => FileConfig::from_json(&data),
        _ => {
            return Err(Fail::new(format!(
                "config {}: unsupported extension, use .yaml, .yml, or .json",
                path.display()
            )));
        }
    };
    let file = parsed.map_err(|e| Fail::new(format!("config {}: {e}", path.display())))?;
    Ok(LoadedConfig {
        path: path.to_path_buf(),
        file,
    })
}

/// Everything a subcommand needs after the engine is built.
pub struct Engine {
    pub kazari: Kazari,
    pub process: Option<ProcessFile>,
    pub config_path: Option<PathBuf>,
}

impl EngineArgs {
    /// Resolves the config file and themes, validates the theme names against the bundled
    /// set and builds the engine. `dir` is where config discovery starts.
    pub fn build(&self, dir: &Path, highlighter: irosashi::Highlighter) -> Result<Engine, Fail> {
        self.build_with(dir, highlighter, |_| {})
    }

    /// Like `build`, with a hook that adjusts the config after the file config is applied
    /// and before the engine is built.
    pub fn build_with(
        &self,
        dir: &Path,
        highlighter: irosashi::Highlighter,
        configure: impl FnOnce(&mut Config),
    ) -> Result<Engine, Fail> {
        self.build_with_defaults(dir, highlighter, |_| {}, configure)
    }

    /// Like `build_with`, with a second hook that sets command defaults before the file
    /// config is applied, so an explicit key in the file still wins over them.
    pub fn build_with_defaults(
        &self,
        dir: &Path,
        highlighter: irosashi::Highlighter,
        defaults: impl FnOnce(&mut Config),
        configure: impl FnOnce(&mut Config),
    ) -> Result<Engine, Fail> {
        if let Some(ratio) = self.min_contrast
            && !(0.0..=21.0).contains(&ratio)
        {
            return Err(Fail::new(format!(
                "min-contrast must be between 0 and 21, got {ratio}"
            )));
        }
        let loaded = load_file_config(self.config.as_deref(), dir)?;
        let (config_path, file) = match loaded {
            Some(l) => (Some(l.path), Some(l.file)),
            None => (None, None),
        };

        let mut light = DEFAULT_LIGHT_THEME.to_owned();
        let mut dark = DEFAULT_DARK_THEME.to_owned();
        if let Some(themes) = file.as_ref().and_then(|f| f.themes.as_ref()) {
            if !themes.light.is_empty() {
                light = themes.light.clone();
            }
            if let Some(d) = themes.dark.as_ref().filter(|d| !d.is_empty()) {
                dark = d.clone();
            }
        }
        if let Some(t) = &self.theme_light {
            light = t.clone();
        }
        if let Some(t) = &self.theme_dark {
            dark = t.clone();
        }
        crate::themes::validate_theme_names(&highlighter.themes(), &light, &dark)
            .map_err(Fail::new)?;

        let mut config = Config::default();
        defaults(&mut config);
        let process = match file {
            Some(f) => {
                let process = f.process.clone();
                f.apply(&mut config).map_err(|e| Fail::new(e.to_string()))?;
                process
            }
            None => None,
        };
        if let Some(ratio) = self.min_contrast {
            config.min_contrast = ratio;
        }
        configure(&mut config);
        let kazari = Kazari::builder(highlighter)
            .config(config)
            .themes(&light, Some(&dark))
            .warning_handler(|msg| eprintln!("{msg}"))
            .build()
            .map_err(|e| Fail::new(e.to_string()))?;
        Ok(Engine {
            kazari,
            process,
            config_path,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovery_probes_dir_then_cwd_and_tolerates_absence() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(load_file_config(None, tmp.path()).unwrap().is_none());
        std::fs::write(
            tmp.path().join("kazari.config.yml"),
            "themes:\n  light: nord\n",
        )
        .unwrap();
        let l = load_file_config(None, tmp.path()).ok().flatten().unwrap();
        assert_eq!(l.path, tmp.path().join("kazari.config.yml"));
        assert_eq!(l.file.themes.unwrap().light, "nord");
    }

    #[test]
    fn explicit_path_errors_are_hard() {
        let tmp = tempfile::tempdir().unwrap();
        let missing = tmp.path().join("nope.yaml");
        let err = load_file_config(Some(&missing), tmp.path()).err().unwrap();
        assert!(err.message.starts_with("reading config"), "{}", err.message);
        let bad = tmp.path().join("kazari.config.toml");
        std::fs::write(&bad, "x").unwrap();
        let err = load_file_config(Some(&bad), tmp.path()).err().unwrap();
        assert!(
            err.message.contains("unsupported extension"),
            "{}",
            err.message
        );
        let json = tmp.path().join("c.json");
        std::fs::write(
            &json,
            "{\"themes\":{\"light\":\"nord\"},\"process\":{\"concurrency\":0}}",
        )
        .unwrap();
        let err = load_file_config(Some(&json), tmp.path()).err().unwrap();
        assert!(
            err.message.contains("process.concurrency"),
            "{}",
            err.message
        );
    }

    #[test]
    fn command_defaults_lose_to_the_file_and_the_file_loses_to_configure() {
        let build = |yaml: &str, off: bool| {
            let tmp = tempfile::tempdir().unwrap();
            std::fs::write(tmp.path().join("kazari.config.yaml"), yaml).unwrap();
            let hl = irosashi::Highlighter::new().unwrap();
            let e = EngineArgs::default()
                .build_with_defaults(
                    tmp.path(),
                    hl,
                    |c| c.code_groups = true,
                    |c| {
                        if off {
                            c.code_groups = false;
                        }
                    },
                )
                .ok()
                .unwrap();
            e.kazari.config().code_groups
        };
        assert!(build("copyButton: true\n", false));
        assert!(!build("codeGroups: false\n", false));
        assert!(!build("codeGroups: true\n", true));
    }

    #[test]
    fn min_contrast_flag_overrides_the_config_file() {
        let build = |flag: Option<f64>| {
            let tmp = tempfile::tempdir().unwrap();
            std::fs::write(tmp.path().join("kazari.config.yaml"), "minContrast: 3\n").unwrap();
            let args = EngineArgs {
                min_contrast: flag,
                ..Default::default()
            };
            args.build(tmp.path(), irosashi::Highlighter::new().unwrap())
                .map(|e| e.kazari.config().min_contrast)
        };
        assert_eq!(build(None).ok(), Some(3.0));
        assert_eq!(build(Some(7.0)).ok(), Some(7.0));
        assert_eq!(build(Some(0.0)).ok(), Some(0.0));
        for bad in [22.0, -1.0, f64::NAN] {
            let err = build(Some(bad)).err().unwrap();
            assert!(
                err.message
                    .starts_with("min-contrast must be between 0 and 21"),
                "{}",
                err.message
            );
        }
    }

    #[test]
    fn theme_resolution_order() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("kazari.config.yaml"),
            "themes:\n  light: nord\n  dark: dracula\nprocess:\n  hashedAssets: true\n",
        )
        .unwrap();
        let hl = irosashi::Highlighter::new().unwrap();
        let args = EngineArgs {
            theme_dark: Some("github-dark".into()),
            ..Default::default()
        };
        let e = args.build(tmp.path(), hl).ok().unwrap();
        assert_eq!(e.kazari.config().light_theme, "nord");
        assert_eq!(e.kazari.config().dark_theme.as_deref(), Some("github-dark"));
        assert_eq!(e.process.unwrap().hashed_assets, Some(true));
        assert!(e.config_path.is_some());

        let hl = irosashi::Highlighter::new().unwrap();
        let args = EngineArgs {
            theme_light: Some("github-ligth".into()),
            ..Default::default()
        };
        let err = args.build(tmp.path(), hl).err().unwrap();
        assert!(
            err.message.contains("did you mean \"github-light\"?"),
            "{}",
            err.message
        );
    }
}
