//! C Oniguruma through the `onig` crate, the engine vscode-textmate and Shiki run.

use super::{OPTION_NONE, Search};

/// A compiled pattern. Searching is thread safe; the results live in the caller's
/// `Region`.
pub(crate) struct Regex {
    raw: onig::Regex,
}

impl Regex {
    /// Compiles with capture groups on, UTF-8, and Oniguruma's default syntax.
    pub fn new(source: &str) -> Result<Self, String> {
        onig::Regex::with_options(
            source,
            onig::RegexOptions::REGEX_OPTION_CAPTURE_GROUP,
            onig::Syntax::oniguruma(),
        )
        .map(|raw| Self { raw })
        .map_err(|err| err.description().to_owned())
    }

    /// Searches `text` for a match starting in `start..=text.len()`, with the whole
    /// text visible to lookbehind and anchors.
    pub fn search(&self, text: &str, start: usize, options: u32, region: &mut Region) -> Search {
        debug_assert!(start <= text.len());
        match self.raw.search_with_param(
            text,
            start,
            text.len(),
            onig::SearchOptions::from_bits_retain(options),
            Some(&mut region.raw),
            onig::MatchParam::default(),
        ) {
            Ok(Some(_)) => Search::Found,
            Ok(None) => Search::NotFound,
            Err(_) => Search::Failed,
        }
    }

    /// Whether the pattern matches anywhere in `text`.
    pub fn is_match_anywhere(&self, text: &str) -> bool {
        matches!(
            self.search(text, 0, OPTION_NONE, &mut Region::new()),
            Search::Found
        )
    }
}

/// Capture positions of the last successful search.
pub(crate) struct Region {
    raw: onig::Region,
}

// SAFETY: an `OnigRegion` is heap memory owned by this value alone; Oniguruma keeps
// no reference to it between searches and has no per-thread state tied to it, so
// it may move to another thread. `onig::Region` only lacks the impl because it
// holds raw pointers.
unsafe impl Send for Region {}

impl Region {
    pub fn new() -> Self {
        Self {
            raw: onig::Region::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.raw.len()
    }

    /// Byte range of group `index`, `None` when it did not participate.
    pub fn pos(&self, index: usize) -> Option<(usize, usize)> {
        self.raw.pos(index)
    }
}
