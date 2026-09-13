use std::fmt;

use onig::{RegSet, RegSetLead, RegexOptions};

use crate::Error;
use crate::regex::SearchOptions;

/// The winning pattern of a `PatternSet` search and its capture groups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    /// Index of the winning pattern within the set.
    pub index: usize,
    /// Byte ranges per capture group, group 0 being the whole match.
    /// Groups that did not participate in the match are `None`.
    pub captures: Vec<Option<(usize, usize)>>,
}

impl Match {
    pub fn range(&self) -> (usize, usize) {
        self.captures[0].expect("group 0 is always set on a match")
    }

    pub fn start(&self) -> usize {
        self.range().0
    }

    pub fn end(&self) -> usize {
        self.range().1
    }
}

/// An ordered set of patterns searched together in one native Oniguruma call.
///
/// A search reports the leftmost match; on ties the lowest pattern index wins.
/// Searching writes into the set's internal region storage, so `find_next_match`
/// takes `&mut self`: one set is used by one searcher at a time and is never shared
/// behind a lock.
pub struct PatternSet {
    regset: Option<RegSet>,
    len: usize,
}

impl PatternSet {
    pub fn new(patterns: &[&str]) -> Result<Self, Error> {
        if patterns.is_empty() {
            return Ok(Self {
                regset: None,
                len: 0,
            });
        }
        let regset = RegSet::with_options(patterns, RegexOptions::REGEX_OPTION_CAPTURE_GROUP)
            .map_err(|err| compilation_error(patterns, err))?;
        Ok(Self {
            regset: Some(regset),
            len: patterns.len(),
        })
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Searches `text` from byte offset `start` to the end.
    ///
    /// Always pass the full line, never a slice: lookbehind and `^` must see the
    /// real line content before `start`.
    pub fn find_next_match(
        &mut self,
        text: &str,
        start: usize,
        options: SearchOptions,
    ) -> Option<Match> {
        debug_assert!(text.is_char_boundary(start));
        let regset = self.regset.as_ref()?;
        if text.is_empty() || start > text.len() {
            return None;
        }
        let (index, captures) = regset.captures_with_options(
            text,
            start,
            text.len(),
            RegSetLead::Position,
            options.into_onig(),
        )?;
        let captures = (0..captures.len()).map(|i| captures.pos(i)).collect();
        Some(Match { index, captures })
    }
}

impl fmt::Debug for PatternSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PatternSet({} patterns)", self.len)
    }
}

fn compilation_error(patterns: &[&str], regset_error: onig::Error) -> Error {
    let first_failure = patterns.iter().enumerate().find_map(|(index, pattern)| {
        onig::Regex::with_options(
            pattern,
            RegexOptions::REGEX_OPTION_CAPTURE_GROUP,
            onig::Syntax::default(),
        )
        .err()
        .map(|err| (index, err.to_string()))
    });
    let (index, message) = first_failure.unwrap_or_else(|| (usize::MAX, regset_error.to_string()));
    Error::RegexCompilation { index, message }
}
