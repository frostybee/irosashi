mod languages;

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use serde::Deserialize;

use crate::Error;
use crate::grammar::{Grammar, GrammarResolver};
use crate::regex::LazyRegex;
use crate::theme::Theme;
use crate::tokenizer::InjectionProvider;

pub use languages::{DEFAULT_ALIASES, DEFAULT_EXTENSIONS, DEFAULT_FILENAMES, PLAINTEXT_NAMES};

#[cfg(feature = "embedded-assets")]
static EMBEDDED_GRAMMARS: include_dir::Dir<'static> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/assets/grammars");
#[cfg(feature = "embedded-assets")]
static EMBEDDED_THEMES: include_dir::Dir<'static> =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/assets/themes");

const INDEX_FILE: &str = "index.json";
const INDEX_VERSION: u32 = 1;
const FIRST_LINE_PROBE_LIMIT: usize = 1024;

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GrammarMeta {
    #[serde(default)]
    pub scope_name: String,
    #[serde(default)]
    pub file_types: Vec<String>,
    #[serde(default)]
    pub inject_to: Vec<String>,
    #[serde(default)]
    pub first_line_match: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Index {
    version: u32,
    grammars: BTreeMap<String, GrammarMeta>,
}

/// The `colors` a renderer needs from a theme, plus the whole map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeColors {
    pub kind: String,
    pub foreground: String,
    pub background: String,
    pub selection_background: Option<String>,
    pub line_highlight_background: Option<String>,
    pub colors: BTreeMap<String, String>,
}

/// Where bundled grammar and theme JSON comes from.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum AssetSource {
    /// `grammars/{name}.json` (plus `grammars/index.json`) and `themes/{name}.json`
    /// under a root directory.
    Dir(PathBuf),
    /// The assets compiled into the crate.
    #[cfg(feature = "embedded-assets")]
    Embedded,
    /// No bundled assets; only registered grammars and themes exist.
    None,
}

impl AssetSource {
    fn read(&self, kind: &str, file: &str) -> Option<Cow<'static, [u8]>> {
        match self {
            Self::Dir(root) => fs::read(root.join(kind).join(file)).ok().map(Cow::Owned),
            #[cfg(feature = "embedded-assets")]
            Self::Embedded => {
                let dir = if kind == "grammars" {
                    &EMBEDDED_GRAMMARS
                } else {
                    &EMBEDDED_THEMES
                };
                dir.get_file(file).map(|f| Cow::Borrowed(f.contents()))
            }
            Self::None => None,
        }
    }

    fn list(&self, kind: &str) -> Vec<String> {
        let mut names: Vec<String> = match self {
            Self::Dir(root) => fs::read_dir(root.join(kind))
                .map(|entries| {
                    entries
                        .filter_map(Result::ok)
                        .map(|e| e.path())
                        .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "json"))
                        .filter_map(|p| p.file_stem()?.to_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default(),
            #[cfg(feature = "embedded-assets")]
            Self::Embedded => {
                let dir = if kind == "grammars" {
                    &EMBEDDED_GRAMMARS
                } else {
                    &EMBEDDED_THEMES
                };
                dir.files()
                    .filter_map(|f| f.path().file_stem()?.to_str().map(str::to_owned))
                    .collect()
            }
            Self::None => Vec::new(),
        };
        names.sort();
        names
    }
}

struct GrammarEntry {
    meta: GrammarMeta,
    bytes: Option<Arc<[u8]>>,
    parsed: OnceLock<Result<Arc<Grammar>, Error>>,
}

struct ThemeEntry {
    bytes: Option<Arc<[u8]>>,
    parsed: OnceLock<Result<Arc<Theme>, Error>>,
}

/// Configures a `Registry`. Built-in alias, extension and file name tables are applied
/// first; entries added here override them.
#[derive(Debug, Clone)]
pub struct RegistryBuilder {
    source: AssetSource,
    grammars: BTreeMap<String, Arc<[u8]>>,
    themes: BTreeMap<String, Arc<[u8]>>,
    aliases: Vec<(String, String)>,
    extensions: Vec<(String, String)>,
    filenames: Vec<(String, String)>,
    default_tables: bool,
}

impl RegistryBuilder {
    pub fn new(source: AssetSource) -> Self {
        Self {
            source,
            grammars: BTreeMap::new(),
            themes: BTreeMap::new(),
            aliases: Vec::new(),
            extensions: Vec::new(),
            filenames: Vec::new(),
            default_tables: true,
        }
    }

    pub fn from_dir(root: impl Into<PathBuf>) -> Self {
        Self::new(AssetSource::Dir(root.into()))
    }

    #[cfg(feature = "embedded-assets")]
    pub fn embedded() -> Self {
        Self::new(AssetSource::Embedded)
    }

    pub fn without_bundled_assets() -> Self {
        Self::new(AssetSource::None)
    }

    /// Registers grammar JSON under `name`, replacing a bundled grammar of the same
    /// name. Registering the same name twice is an error.
    pub fn grammar(mut self, name: &str, json: impl Into<Arc<[u8]>>) -> Result<Self, Error> {
        if self.grammars.contains_key(name) {
            return Err(Error::GrammarParse(format!(
                "grammar {name:?} registered twice"
            )));
        }
        self.grammars.insert(name.to_owned(), json.into());
        Ok(self)
    }

    pub fn theme(mut self, name: &str, json: impl Into<Arc<[u8]>>) -> Result<Self, Error> {
        if self.themes.contains_key(name) {
            return Err(Error::ThemeParse(format!(
                "theme {name:?} registered twice"
            )));
        }
        self.themes.insert(name.to_owned(), json.into());
        Ok(self)
    }

    pub fn alias(mut self, alias: &str, target: &str) -> Self {
        self.aliases.push((alias.to_owned(), target.to_owned()));
        self
    }

    /// Maps a file extension (without the dot) to a grammar name.
    pub fn extension(mut self, ext: &str, lang: &str) -> Self {
        self.extensions
            .push((ext.to_ascii_lowercase(), lang.to_owned()));
        self
    }

    /// Maps an exact file name to a grammar name.
    pub fn filename(mut self, filename: &str, lang: &str) -> Self {
        self.filenames.push((filename.to_owned(), lang.to_owned()));
        self
    }

    /// Skips the built-in alias, extension and file name tables.
    pub fn without_default_tables(mut self) -> Self {
        self.default_tables = false;
        self
    }

    pub fn build(self) -> Result<Registry, Error> {
        let mut grammars: BTreeMap<String, GrammarEntry> = BTreeMap::new();
        if !matches!(self.source, AssetSource::None) {
            let bytes = self
                .source
                .read("grammars", INDEX_FILE)
                .ok_or_else(|| Error::Io(format!("grammars/{INDEX_FILE} not found")))?;
            let index: Index = serde_json::from_slice(&bytes)
                .map_err(|err| Error::GrammarParse(format!("{INDEX_FILE}: {err}")))?;
            if index.version != INDEX_VERSION {
                return Err(Error::GrammarParse(format!(
                    "{INDEX_FILE}: unsupported version {}",
                    index.version
                )));
            }
            for (name, meta) in index.grammars {
                grammars.insert(
                    name,
                    GrammarEntry {
                        meta,
                        bytes: None,
                        parsed: OnceLock::new(),
                    },
                );
            }
        }
        for (name, bytes) in self.grammars {
            let meta: GrammarMeta = serde_json::from_slice(&bytes)
                .map_err(|err| Error::GrammarParse(format!("grammar {name:?}: {err}")))?;
            grammars.insert(
                name,
                GrammarEntry {
                    meta,
                    bytes: Some(bytes),
                    parsed: OnceLock::new(),
                },
            );
        }

        let mut themes: BTreeMap<String, ThemeEntry> = BTreeMap::new();
        for name in self.source.list("themes") {
            themes.insert(
                name,
                ThemeEntry {
                    bytes: None,
                    parsed: OnceLock::new(),
                },
            );
        }
        for (name, bytes) in self.themes {
            themes.insert(
                name,
                ThemeEntry {
                    bytes: Some(bytes),
                    parsed: OnceLock::new(),
                },
            );
        }

        let mut scope_index = HashMap::new();
        let mut injection_index: HashMap<String, Vec<String>> = HashMap::new();
        let mut ext_index: HashMap<String, String> = HashMap::new();
        let mut first_line = Vec::new();
        for (name, entry) in &grammars {
            let meta = &entry.meta;
            if !meta.scope_name.is_empty() {
                scope_index
                    .entry(meta.scope_name.clone())
                    .or_insert_with(|| name.clone());
            }
            for target in &meta.inject_to {
                injection_index
                    .entry(target.clone())
                    .or_default()
                    .push(name.clone());
            }
            for ext in &meta.file_types {
                let ext = ext.trim_start_matches('.').to_ascii_lowercase();
                if ext.is_empty() {
                    continue;
                }
                if entry.bytes.is_some() {
                    ext_index.insert(ext, name.clone());
                } else {
                    ext_index.entry(ext).or_insert_with(|| name.clone());
                }
            }
            if let Some(pattern) = &meta.first_line_match {
                first_line.push((name.clone(), LazyRegex::new(pattern)));
            }
        }

        let mut filename_index = HashMap::new();
        let mut aliases = HashMap::new();
        if self.default_tables {
            for (ext, lang) in DEFAULT_EXTENSIONS {
                ext_index.insert((*ext).to_owned(), (*lang).to_owned());
            }
            for (file, lang) in DEFAULT_FILENAMES {
                filename_index.insert((*file).to_owned(), (*lang).to_owned());
            }
            for (alias, lang) in DEFAULT_ALIASES {
                aliases.insert((*alias).to_owned(), (*lang).to_owned());
            }
        }
        for (ext, lang) in self.extensions {
            ext_index.insert(ext, lang);
        }
        for (file, lang) in self.filenames {
            filename_index.insert(file, lang);
        }
        for (alias, lang) in self.aliases {
            aliases.insert(alias, lang);
        }

        Ok(Registry {
            source: self.source,
            grammars,
            themes,
            aliases,
            ext_index,
            filename_index,
            scope_index,
            injection_index,
            first_line,
        })
    }
}

/// Grammars and themes, parsed lazily and at most once each. Immutable after build,
/// so it is shared freely across threads.
pub struct Registry {
    source: AssetSource,
    grammars: BTreeMap<String, GrammarEntry>,
    themes: BTreeMap<String, ThemeEntry>,
    aliases: HashMap<String, String>,
    ext_index: HashMap<String, String>,
    filename_index: HashMap<String, String>,
    scope_index: HashMap<String, String>,
    injection_index: HashMap<String, Vec<String>>,
    first_line: Vec<(String, LazyRegex)>,
}

impl std::fmt::Debug for Registry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Registry")
            .field("grammars", &self.grammars.len())
            .field("themes", &self.themes.len())
            .finish()
    }
}

impl Registry {
    pub fn from_dir(root: &Path) -> Result<Self, Error> {
        RegistryBuilder::from_dir(root).build()
    }

    #[cfg(feature = "embedded-assets")]
    pub fn embedded() -> Result<Self, Error> {
        RegistryBuilder::embedded().build()
    }

    /// Grammar names, sorted.
    pub fn languages(&self) -> impl Iterator<Item = &str> {
        self.grammars.keys().map(String::as_str)
    }

    /// Theme names, sorted.
    pub fn themes(&self) -> impl Iterator<Item = &str> {
        self.themes.keys().map(String::as_str)
    }

    pub fn grammar_meta(&self, name: &str) -> Option<&GrammarMeta> {
        self.grammars.get(name).map(|e| &e.meta)
    }

    /// The grammar name `name` refers to: itself when it is a grammar, otherwise its
    /// alias target (one hop), otherwise nothing.
    pub fn resolve_language(&self, name: &str) -> Option<&str> {
        if let Some((key, _)) = self.grammars.get_key_value(name) {
            return Some(key);
        }
        let target = self.aliases.get(name)?;
        self.grammars
            .get_key_value(target.as_str())
            .map(|(k, _)| k.as_str())
    }

    pub fn grammar(&self, name: &str) -> Result<Arc<Grammar>, Error> {
        let resolved = self
            .resolve_language(name)
            .ok_or_else(|| Error::LanguageNotFound(name.to_owned()))?;
        let entry = &self.grammars[resolved];
        entry
            .parsed
            .get_or_init(|| {
                let bytes = match &entry.bytes {
                    Some(bytes) => Cow::Borrowed(&**bytes),
                    None => self
                        .source
                        .read("grammars", &format!("{resolved}.json"))
                        .ok_or_else(|| Error::Io(format!("grammar {resolved:?} missing")))?,
                };
                Grammar::parse(&bytes).map(Arc::new)
            })
            .clone()
    }

    pub fn grammar_by_scope(&self, scope: &str) -> Result<Arc<Grammar>, Error> {
        match self.scope_index.get(scope) {
            Some(name) => self.grammar(name),
            None => Err(Error::LanguageNotFound(scope.to_owned())),
        }
    }

    pub fn theme(&self, name: &str) -> Result<Arc<Theme>, Error> {
        let entry = self
            .themes
            .get(name)
            .ok_or_else(|| Error::ThemeNotFound(name.to_owned()))?;
        entry
            .parsed
            .get_or_init(|| {
                let bytes = match &entry.bytes {
                    Some(bytes) => Cow::Borrowed(&**bytes),
                    None => self
                        .source
                        .read("themes", &format!("{name}.json"))
                        .ok_or_else(|| Error::ThemeNotFound(name.to_owned()))?,
                };
                Theme::parse(&bytes).map(Arc::new)
            })
            .clone()
    }

    pub fn theme_colors(&self, name: &str) -> Result<ThemeColors, Error> {
        let theme = self.theme(name)?;
        Ok(ThemeColors {
            kind: theme.kind.clone(),
            foreground: theme.default_foreground().to_owned(),
            background: theme.default_background().to_owned(),
            selection_background: theme.colors.get("editor.selectionBackground").cloned(),
            line_highlight_background: theme.colors.get("editor.lineHighlightBackground").cloned(),
            colors: theme.colors.clone(),
        })
    }

    /// Exact file name first, then the lowercased extension.
    pub fn detect_by_filename(&self, filename: &str) -> Option<&str> {
        let path = Path::new(filename);
        let base = path.file_name()?.to_str()?;
        if let Some(lang) = self.filename_index.get(base) {
            return Some(lang);
        }
        let ext = path.extension()?.to_str()?.to_ascii_lowercase();
        self.ext_index.get(&ext).map(String::as_str)
    }

    /// The first grammar (in name order) whose `firstLineMatch` matches the line.
    pub fn detect_by_first_line(&self, line: &str) -> Option<&str> {
        let line = line.lines().next().unwrap_or("");
        let mut end = line.len().min(FIRST_LINE_PROBE_LIMIT);
        while !line.is_char_boundary(end) {
            end -= 1;
        }
        let probe = &line[..end];
        self.first_line
            .iter()
            .find(|(_, regex)| regex.compiled().is_some_and(|re| re.is_match(probe)))
            .map(|(name, _)| name.as_str())
    }
}

impl GrammarResolver for Registry {
    fn grammar_by_scope(&self, scope: &str) -> Option<Arc<Grammar>> {
        Registry::grammar_by_scope(self, scope).ok()
    }
}

impl InjectionProvider for Registry {
    fn injectors_for(&self, scope: &str) -> Vec<Arc<Grammar>> {
        self.injection_index
            .get(scope)
            .map(|names| names.iter().filter_map(|n| self.grammar(n).ok()).collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assets() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets")
    }

    fn registry() -> Registry {
        Registry::from_dir(&assets()).unwrap()
    }

    #[test]
    fn lists_all_bundled_assets() {
        let r = registry();
        assert_eq!(r.languages().count(), 257);
        assert_eq!(r.themes().count(), 65);
        assert!(r.languages().any(|l| l == "javascript"));
        assert!(r.themes().any(|t| t == "github-dark"));
    }

    #[test]
    fn grammar_name_beats_alias_and_aliases_are_single_hop() {
        let r = RegistryBuilder::from_dir(assets())
            .alias("json", "javascript")
            .alias("chain", "js")
            .build()
            .unwrap();
        assert_eq!(r.resolve_language("json"), Some("json"));
        assert_eq!(r.resolve_language("js"), Some("javascript"));
        assert_eq!(r.resolve_language("chain"), None);
        assert_eq!(r.resolve_language("nope"), None);
        assert!(matches!(r.grammar("nope"), Err(Error::LanguageNotFound(_))));
    }

    #[test]
    fn detection_by_filename_and_extension() {
        let r = registry();
        assert_eq!(r.detect_by_filename("Makefile"), Some("make"));
        assert_eq!(r.detect_by_filename("src/main.RS"), Some("rust"));
        assert_eq!(r.detect_by_filename("a/b/Dockerfile"), Some("docker"));
        assert_eq!(r.detect_by_filename("noext"), None);
        assert_eq!(r.detect_by_filename("x.unknownext"), None);
        let custom = RegistryBuilder::from_dir(assets())
            .extension("Foo", "go")
            .filename("BUILD", "python")
            .build()
            .unwrap();
        assert_eq!(custom.detect_by_filename("x.foo"), Some("go"));
        assert_eq!(custom.detect_by_filename("BUILD"), Some("python"));
    }

    #[test]
    fn detection_by_first_line_is_deterministic_and_capped() {
        let r = registry();
        assert_eq!(
            r.detect_by_first_line("#!/usr/bin/env swift"),
            Some("swift")
        );
        assert_eq!(
            r.detect_by_first_line("{% extends \"base.html\" %}"),
            Some("jinja-html")
        );
        assert_eq!(r.detect_by_first_line("plain text"), None);
        let long = format!("{}#!/usr/bin/env swift", " ".repeat(2000));
        assert_eq!(r.detect_by_first_line(&long), None);
    }

    #[test]
    fn registered_grammar_overrides_bundled_and_duplicates_are_errors() {
        let json = br#"{"scopeName": "source.custom", "fileTypes": ["json"], "patterns": [{"match": "x", "name": "x"}]}"#;
        let r = RegistryBuilder::from_dir(assets())
            .grammar("json", &json[..])
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(r.grammar("json").unwrap().scope_name, "source.custom");
        assert_eq!(r.detect_by_filename("a.json"), Some("json"));
        assert!(
            RegistryBuilder::without_bundled_assets()
                .grammar("g", &json[..])
                .unwrap()
                .grammar("g", &json[..])
                .is_err()
        );
    }

    #[test]
    fn theme_lookup_and_colors() {
        let r = registry();
        let colors = r.theme_colors("github-dark").unwrap();
        assert_eq!(colors.kind, "dark");
        assert!(colors.background.starts_with('#'));
        assert!(matches!(r.theme("nope"), Err(Error::ThemeNotFound(_))));
        assert!(Arc::ptr_eq(
            &r.theme("github-dark").unwrap(),
            &r.theme("github-dark").unwrap()
        ));
    }

    #[cfg(feature = "embedded-assets")]
    #[test]
    fn embedded_and_directory_sources_agree() {
        let dir = registry();
        let embedded = Registry::embedded().unwrap();
        assert_eq!(
            dir.languages().collect::<Vec<_>>(),
            embedded.languages().collect::<Vec<_>>()
        );
        assert_eq!(
            dir.themes().collect::<Vec<_>>(),
            embedded.themes().collect::<Vec<_>>()
        );
        assert_eq!(
            *dir.grammar("go").unwrap(),
            *embedded.grammar("go").unwrap()
        );
    }
}
