//! `ferroni`, a pure Rust port of Oniguruma. The C-shaped entry points are used
//! because they take the raw search option bits and a reusable region, which the
//! safe `ferroni::api::Regex` does not expose.

use ferroni::encodings::utf8::ONIG_ENCODING_UTF8;
use ferroni::error::RegexError;
use ferroni::oniguruma::RegexType;
use ferroni::oniguruma::{
    ONIG_MISMATCH, ONIG_OPTION_CAPTURE_GROUP, ONIG_REGION_NOTPOS, OnigOptionType, OnigRegion,
};
use ferroni::regsyntax::OnigSyntaxOniguruma;

use super::{OPTION_NONE, OPTION_NOT_BEGIN_POSITION, OPTION_NOT_BEGIN_STRING, Search};

const _: () = assert!(OPTION_NONE == OnigOptionType::NONE.bits());
const _: () = assert!(OPTION_NOT_BEGIN_STRING == OnigOptionType::NOT_BEGIN_STRING.bits());
const _: () = assert!(OPTION_NOT_BEGIN_POSITION == OnigOptionType::NOT_BEGIN_POSITION.bits());

/// A compiled pattern. Searching is thread safe; the results live in the caller's
/// `Region`.
pub(crate) struct Regex {
    raw: RegexType,
}

impl Regex {
    /// Compiles with capture groups on, UTF-8, and Oniguruma's default syntax.
    pub fn new(source: &str) -> Result<Self, String> {
        ferroni::regcomp::onig_new(
            source.as_bytes(),
            ONIG_OPTION_CAPTURE_GROUP,
            &ONIG_ENCODING_UTF8,
            &OnigSyntaxOniguruma,
        )
        .map(|raw| Self { raw })
        .map_err(|err| match err {
            RegexError::Syntax { message, .. } => message,
            other => other.to_string(),
        })
    }

    /// Searches `text` for a match starting in `start..=text.len()`, with the whole
    /// text visible to lookbehind and anchors.
    pub fn search(&self, text: &str, start: usize, options: u32, region: &mut Region) -> Search {
        debug_assert!(start <= text.len());
        let bytes = text.as_bytes();
        let (code, raw) = ferroni::regexec::onig_search(
            &self.raw,
            bytes,
            bytes.len(),
            start,
            bytes.len(),
            region.raw.take(),
            OnigOptionType::from_bits_retain(options),
        );
        region.raw = raw;
        if code >= 0 {
            Search::Found
        } else if code == ONIG_MISMATCH {
            Search::NotFound
        } else {
            Search::Failed
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
    raw: Option<OnigRegion>,
}

impl Region {
    pub fn new() -> Self {
        Self {
            raw: Some(OnigRegion::new()),
        }
    }

    pub fn len(&self) -> usize {
        self.raw
            .as_ref()
            .map_or(0, |raw| raw.num_regs.max(0) as usize)
    }

    /// Byte range of group `index`, `None` when it did not participate.
    pub fn pos(&self, index: usize) -> Option<(usize, usize)> {
        if index >= self.len() {
            return None;
        }
        let raw = self.raw.as_ref()?;
        let (beg, end) = (*raw.beg.get(index)?, *raw.end.get(index)?);
        if beg == ONIG_REGION_NOTPOS || beg < 0 || end < beg {
            None
        } else {
            Some((beg as usize, end as usize))
        }
    }
}
