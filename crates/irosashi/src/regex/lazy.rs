use std::fmt;
use std::sync::{Arc, OnceLock};

use onig::{RegexOptions, Syntax};
use serde::{Deserialize, Serialize};

use crate::regex::rewrite_z_anchor;

/// A pattern string compiled on first use.
///
/// Construction applies the vscode-textmate `\z` rewrite; this is the only place it
/// happens, so every grammar pattern (match, begin, end, while, and backref-resolved
/// end patterns derived from `source()`) goes through it exactly once.
#[derive(Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub struct LazyRegex {
    source: String,
    compiled: OnceLock<Option<Arc<onig::Regex>>>,
}

impl LazyRegex {
    pub fn new(pattern: &str) -> Self {
        Self {
            source: rewrite_z_anchor(pattern),
            compiled: OnceLock::new(),
        }
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    /// The compiled regex, or `None` if Oniguruma rejects the pattern.
    pub fn compiled(&self) -> Option<&Arc<onig::Regex>> {
        self.compiled
            .get_or_init(|| {
                onig::Regex::with_options(
                    &self.source,
                    RegexOptions::REGEX_OPTION_CAPTURE_GROUP,
                    Syntax::default(),
                )
                .ok()
                .map(Arc::new)
            })
            .as_ref()
    }
}

impl From<String> for LazyRegex {
    fn from(pattern: String) -> Self {
        Self::new(&pattern)
    }
}

impl From<LazyRegex> for String {
    fn from(regex: LazyRegex) -> Self {
        regex.source
    }
}

impl Clone for LazyRegex {
    fn clone(&self) -> Self {
        Self {
            source: self.source.clone(),
            compiled: OnceLock::new(),
        }
    }
}

impl PartialEq for LazyRegex {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source
    }
}

impl Eq for LazyRegex {}

impl fmt::Debug for LazyRegex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "LazyRegex({:?})", self.source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_z_anchor_in_source() {
        assert_eq!(LazyRegex::new(r"foo\z").source(), r"foo$(?!\n)(?<!\n)");
        assert_eq!(LazyRegex::new(r"foo\\z").source(), r"foo\\z");
    }

    #[test]
    fn compiles_once_and_caches() {
        let regex = LazyRegex::new(r"\w+");
        let first = regex.compiled().expect("valid pattern compiles");
        let second = regex.compiled().unwrap();
        assert!(Arc::ptr_eq(first, second));
    }

    #[test]
    fn invalid_pattern_yields_none() {
        assert!(LazyRegex::new("(?P<").compiled().is_none());
    }

    #[test]
    fn serializes_as_source_string() {
        let regex = LazyRegex::new(r"a\z");
        let json = serde_json::to_string(&regex).unwrap();
        assert_eq!(json, r#""a$(?!\\n)(?<!\\n)""#);
        let back: LazyRegex = serde_json::from_str(&json).unwrap();
        assert_eq!(back, regex);
    }
}
