use std::fmt;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::regex::raw::Regex;
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
    compiled: OnceLock<Option<Regex>>,
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

    /// `false` if Oniguruma rejects the pattern.
    pub fn compiles(&self) -> bool {
        self.compiled().is_some()
    }

    /// Whether the pattern matches anywhere in `text`; a pattern that does not
    /// compile matches nothing.
    pub fn is_match(&self, text: &str) -> bool {
        self.compiled().is_some_and(|re| re.is_match_anywhere(text))
    }

    fn compiled(&self) -> Option<&Regex> {
        self.compiled
            .get_or_init(|| Regex::new(&self.source).ok())
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
        assert!(std::ptr::eq(first, second));
    }

    #[test]
    fn invalid_pattern_never_matches() {
        let regex = LazyRegex::new("(?P<");
        assert!(!regex.compiles());
        assert!(!regex.is_match("(?P<"));
    }

    #[test]
    fn is_match_searches_the_whole_text() {
        let regex = LazyRegex::new(r"^#!/.*\bswift");
        assert!(regex.is_match("#!/usr/bin/env swift -O"));
        assert!(!regex.is_match(" #!/usr/bin/env swift"));
        assert!(LazyRegex::new("b+").is_match("abbc"));
        assert!(!LazyRegex::new("b+").is_match(""));
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
