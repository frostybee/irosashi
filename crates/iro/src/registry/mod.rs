use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use serde::Deserialize;

use crate::Error;
use crate::grammar::{Grammar, GrammarResolver};
use crate::theme::Theme;
use crate::tokenizer::InjectionProvider;

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

const INDEX_VERSION: u32 = 1;

/// Grammars and themes loaded lazily from an assets directory laid out as
/// `grammars/{name}.json` (plus `grammars/index.json`) and `themes/{name}.json`.
#[derive(Debug)]
pub struct Registry {
    grammars_dir: PathBuf,
    themes_dir: PathBuf,
    meta: BTreeMap<String, GrammarMeta>,
    scope_index: HashMap<String, String>,
    injection_index: HashMap<String, Vec<String>>,
    grammars: Mutex<HashMap<String, Arc<Grammar>>>,
    themes: Mutex<HashMap<String, Arc<Theme>>>,
}

impl Registry {
    pub fn from_dir(root: &Path) -> Result<Self, Error> {
        let grammars_dir = root.join("grammars");
        let themes_dir = root.join("themes");
        let index_path = grammars_dir.join("index.json");
        let bytes = fs::read(&index_path)
            .map_err(|err| Error::Io(format!("{}: {err}", index_path.display())))?;
        let index: Index = serde_json::from_slice(&bytes)
            .map_err(|err| Error::GrammarParse(format!("index.json: {err}")))?;
        if index.version != INDEX_VERSION {
            return Err(Error::GrammarParse(format!(
                "index.json: unsupported version {}",
                index.version
            )));
        }

        let mut scope_index = HashMap::new();
        let mut injection_index: HashMap<String, Vec<String>> = HashMap::new();
        for (name, meta) in &index.grammars {
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
        }

        Ok(Self {
            grammars_dir,
            themes_dir,
            meta: index.grammars,
            scope_index,
            injection_index,
            grammars: Mutex::new(HashMap::new()),
            themes: Mutex::new(HashMap::new()),
        })
    }

    pub fn grammar_names(&self) -> impl Iterator<Item = &str> {
        self.meta.keys().map(String::as_str)
    }

    pub fn grammar_meta(&self, name: &str) -> Option<&GrammarMeta> {
        self.meta.get(name)
    }

    pub fn grammar(&self, name: &str) -> Result<Arc<Grammar>, Error> {
        if !self.meta.contains_key(name) {
            return Err(Error::LanguageNotFound(name.to_owned()));
        }
        let mut cache = self.grammars.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(grammar) = cache.get(name) {
            return Ok(Arc::clone(grammar));
        }
        let path = self.grammars_dir.join(format!("{name}.json"));
        let bytes =
            fs::read(&path).map_err(|err| Error::Io(format!("{}: {err}", path.display())))?;
        let grammar = Arc::new(Grammar::parse(&bytes)?);
        cache.insert(name.to_owned(), Arc::clone(&grammar));
        Ok(grammar)
    }

    pub fn grammar_by_scope(&self, scope: &str) -> Result<Arc<Grammar>, Error> {
        match self.scope_index.get(scope) {
            Some(name) => self.grammar(name),
            None => Err(Error::LanguageNotFound(scope.to_owned())),
        }
    }

    pub fn theme(&self, name: &str) -> Result<Arc<Theme>, Error> {
        let mut cache = self.themes.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(theme) = cache.get(name) {
            return Ok(Arc::clone(theme));
        }
        let path = self.themes_dir.join(format!("{name}.json"));
        let bytes = fs::read(&path).map_err(|_| Error::ThemeNotFound(name.to_owned()))?;
        let theme = Arc::new(Theme::parse(&bytes)?);
        cache.insert(name.to_owned(), Arc::clone(&theme));
        Ok(theme)
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
