use clap::ValueEnum;
use kazari_rs::EngineName;
use kazari_rs::backends::syntect::SyntectHighlighter;

use crate::Fail;

/// The highlighting backend named on the command line or in the config file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum)]
pub enum EngineKind {
    /// VS Code grammars and themes, byte-identical to Shiki
    #[default]
    Irosashi,
    /// Sublime Text grammars and base16 themes, as syntect renders them elsewhere
    Syntect,
}

impl From<EngineName> for EngineKind {
    fn from(name: EngineName) -> Self {
        match name {
            EngineName::Irosashi => EngineKind::Irosashi,
            EngineName::Syntect => EngineKind::Syntect,
        }
    }
}

impl EngineKind {
    pub fn create(self) -> Result<Backend, Fail> {
        match self {
            EngineKind::Irosashi => irosashi::Highlighter::new()
                .map(|hl| Backend::Irosashi(Box::new(hl)))
                .map_err(|e| Fail::new(format!("initializing highlighter: {e}"))),
            EngineKind::Syntect => Ok(Backend::Syntect(Box::new(SyntectHighlighter::new()))),
        }
    }
}

/// A constructed backend, before it is handed to the engine.
pub enum Backend {
    Irosashi(Box<irosashi::Highlighter>),
    Syntect(Box<SyntectHighlighter>),
}

impl Backend {
    pub fn kind(&self) -> EngineKind {
        match self {
            Backend::Irosashi(_) => EngineKind::Irosashi,
            Backend::Syntect(_) => EngineKind::Syntect,
        }
    }

    pub fn themes(&self) -> Vec<String> {
        match self {
            Backend::Irosashi(hl) => hl.themes(),
            Backend::Syntect(hl) => hl.themes(),
        }
    }

    pub fn languages(&self) -> Vec<String> {
        match self {
            Backend::Irosashi(hl) => hl.languages(),
            Backend::Syntect(hl) => hl.languages(),
        }
    }

    pub fn detect_language(&self, file_name: &str) -> Option<String> {
        match self {
            Backend::Irosashi(hl) => hl.detect_language(file_name),
            Backend::Syntect(hl) => hl.detect_language(file_name),
        }
    }

    pub fn into_highlighter(self) -> Box<dyn kazari_rs::Highlighter> {
        match self {
            Backend::Irosashi(hl) => hl,
            Backend::Syntect(hl) => hl,
        }
    }
}
