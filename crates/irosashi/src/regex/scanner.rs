use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::Error;
use crate::regex::SearchOptions;
use crate::regex::raw::{Regex, Region, Search};
use crate::regex::store::RegexStore;

/// The winning pattern of a scanner search and its capture groups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    /// Index of the winning pattern within the scanner.
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

pub(crate) type CaptureBuf = Vec<Option<(usize, usize)>>;

/// Index of a compiled pattern within one `PatternTable`.
pub(crate) type PatternId = usize;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct ScanStats {
    pub compiles: u64,
    /// Patterns taken from the shared store already compiled.
    pub shared_hits: u64,
    pub searches: u64,
    pub cache_hits: u64,
    pub engine_errors: u64,
}

/// The last search of one pattern: valid for the same line generation and options
/// from any later start, as long as the remembered match is not behind that start.
#[derive(Debug, Default)]
struct SlotCache {
    generation: u64,
    pos: usize,
    options: SearchOptions,
    hit: bool,
    captures: CaptureBuf,
}

struct Slot {
    regex: Arc<Regex>,
    /// The search option bits that can change this pattern's result: `\A` reacts to
    /// `NOT_BEGIN_STRING`, `\G` to `NOT_BEGIN_POSITION`; other patterns to neither.
    anchor_bits: SearchOptions,
    has_g_anchor: bool,
    cache: SlotCache,
}

/// An ordered list of patterns searched together: leftmost match wins, lowest index
/// on ties. Patterns are ids into the table that built it.
#[derive(Debug, Clone, Default)]
pub(crate) struct Scanner {
    slots: Vec<PatternId>,
}

impl Scanner {
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }
}

/// Every pattern a session has compiled, each once, with its last-match cache.
///
/// Searching writes the scratch region and the per-pattern caches, so a table is
/// owned by one session and never shared. The compiled objects themselves are
/// immutable and may come from a store shared with other sessions.
pub(crate) struct PatternTable {
    store: Option<Arc<RegexStore>>,
    by_source: HashMap<Arc<str>, Result<PatternId, Arc<str>>>,
    slots: Vec<Slot>,
    region: Region,
    stats: ScanStats,
}

impl Default for PatternTable {
    fn default() -> Self {
        Self {
            store: None,
            by_source: HashMap::new(),
            slots: Vec::new(),
            region: Region::new(),
            stats: ScanStats::default(),
        }
    }
}

impl PatternTable {
    pub fn with_store(store: Arc<RegexStore>) -> Self {
        Self {
            store: Some(store),
            ..Self::default()
        }
    }

    /// Compiles `source` on first sight; later calls return the same id.
    pub fn intern(&mut self, source: &str) -> Result<PatternId, Arc<str>> {
        if let Some(known) = self.by_source.get(source) {
            return known.clone();
        }
        let (compiled, fresh) = match &self.store {
            Some(store) => store.get_or_compile(source),
            None => (Regex::new(source).map(Arc::new).map_err(Arc::from), true),
        };
        let result = match compiled {
            Ok(regex) => {
                if fresh {
                    self.stats.compiles += 1;
                } else {
                    self.stats.shared_hits += 1;
                }
                let has_g_anchor = has_anchor(source, b'G');
                let mut anchor_bits = SearchOptions::NONE;
                if has_anchor(source, b'A') {
                    anchor_bits = anchor_bits.union(SearchOptions::NOT_BEGIN_STRING);
                }
                if has_g_anchor {
                    anchor_bits = anchor_bits.union(SearchOptions::NOT_BEGIN_POSITION);
                }
                self.slots.push(Slot {
                    regex,
                    anchor_bits,
                    has_g_anchor,
                    cache: SlotCache::default(),
                });
                Ok(self.slots.len() - 1)
            }
            Err(message) => Err(message),
        };
        self.by_source.insert(Arc::from(source), result.clone());
        result
    }

    /// A scanner over `sources` in order. Fails on the first pattern that does not
    /// compile, reporting its index.
    pub fn scanner(&mut self, sources: &[&str]) -> Result<Scanner, Error> {
        let mut slots = Vec::with_capacity(sources.len());
        for (index, source) in sources.iter().enumerate() {
            match self.intern(source) {
                Ok(id) => slots.push(id),
                Err(message) => {
                    return Err(Error::RegexCompilation {
                        index,
                        message: message.to_string(),
                    });
                }
            }
        }
        Ok(Scanner { slots })
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn stats(&self) -> ScanStats {
        self.stats
    }

    pub fn reset_stats(&mut self) {
        self.stats = ScanStats::default();
    }

    /// Searches `text` from byte offset `start` to the end with every pattern of
    /// `scanner`, returning the winning slot index and writing its capture groups
    /// into `captures`.
    ///
    /// Always pass the full line, never a slice: lookbehind and `^` must see the
    /// real line content before `start`. `generation` identifies the text; it must
    /// change whenever the text does, since cached results are keyed on it.
    pub fn find_next_match_into(
        &mut self,
        scanner: &Scanner,
        text: &str,
        generation: u64,
        start: usize,
        options: SearchOptions,
        captures: &mut CaptureBuf,
    ) -> Option<usize> {
        debug_assert!(text.is_char_boundary(start));
        if text.is_empty() || start > text.len() {
            return None;
        }
        let mut best: Option<(usize, usize)> = None;
        for (index, &id) in scanner.slots.iter().enumerate() {
            let Some(match_start) = self.search_slot(id, text, generation, start, options) else {
                continue;
            };
            if best.is_none_or(|(_, best_start)| match_start < best_start) {
                best = Some((index, match_start));
                if match_start == start {
                    break;
                }
            }
        }
        let (index, _) = best?;
        captures.clear();
        captures.extend_from_slice(&self.slots[scanner.slots[index]].cache.captures);
        Some(index)
    }

    #[allow(dead_code)]
    pub fn find_next_match(
        &mut self,
        scanner: &Scanner,
        text: &str,
        generation: u64,
        start: usize,
        options: SearchOptions,
    ) -> Option<Match> {
        let mut captures = Vec::new();
        let index =
            self.find_next_match_into(scanner, text, generation, start, options, &mut captures)?;
        Some(Match { index, captures })
    }

    /// `find_next_match` for one pattern (a while pattern); the match index is 0.
    pub fn find_next_match_single(
        &mut self,
        id: PatternId,
        text: &str,
        generation: u64,
        start: usize,
        options: SearchOptions,
    ) -> Option<Match> {
        debug_assert!(text.is_char_boundary(start));
        if text.is_empty() || start > text.len() {
            return None;
        }
        self.search_slot(id, text, generation, start, options)?;
        Some(Match {
            index: 0,
            captures: self.slots[id].cache.captures.clone(),
        })
    }

    fn search_slot(
        &mut self,
        id: PatternId,
        text: &str,
        generation: u64,
        start: usize,
        options: SearchOptions,
    ) -> Option<usize> {
        let Self {
            slots,
            region,
            stats,
            ..
        } = self;
        let slot = &mut slots[id];
        let cache = &mut slot.cache;
        let key_options = options.intersection(slot.anchor_bits);
        let cacheable = !slot.has_g_anchor || options.contains(SearchOptions::NOT_BEGIN_POSITION);
        if cacheable
            && cache.generation == generation
            && cache.options == key_options
            && cache.pos <= start
        {
            if !cache.hit {
                stats.cache_hits += 1;
                return None;
            }
            let cached_start = cache.captures[0].map_or(start, |(s, _)| s);
            if cached_start >= start {
                stats.cache_hits += 1;
                return Some(cached_start);
            }
        }

        stats.searches += 1;
        let found = slot.regex.search(text, start, options.bits(), region);
        cache.generation = generation;
        cache.pos = start;
        cache.options = key_options;
        cache.captures.clear();
        match found {
            Search::Found => {
                cache.hit = true;
                cache
                    .captures
                    .extend((0..region.len()).map(|i| region.pos(i)));
                cache.captures.first().copied().flatten().map(|(s, _)| s)
            }
            Search::NotFound => {
                cache.hit = false;
                None
            }
            Search::Failed => {
                stats.engine_errors += 1;
                cache.hit = false;
                None
            }
        }
    }
}

impl fmt::Debug for PatternTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PatternTable({} patterns)", self.slots.len())
    }
}

/// vscode-oniguruma never caches a pattern containing `\G` while `\G` can fire, since
/// its meaning depends on the search start; with `NOT_BEGIN_POSITION` the anchor is
/// dead and the pattern behaves like vscode-textmate's rewritten `G0` variant. Like
/// upstream, an escaped backslash before the letter also counts.
fn has_anchor(source: &str, letter: u8) -> bool {
    source
        .as_bytes()
        .windows(2)
        .any(|pair| pair == [b'\\', letter])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_on_one_store_share_regexes_but_not_caches() {
        let store = Arc::new(RegexStore::default());
        let mut a = PatternTable::with_store(Arc::clone(&store));
        let mut b = PatternTable::with_store(store);
        let scanner_a = a.scanner(&["b+", "c"]).unwrap();
        let scanner_b = b.scanner(&["b+", "c"]).unwrap();
        assert_eq!((a.stats().compiles, a.stats().shared_hits), (2, 0));
        assert_eq!((b.stats().compiles, b.stats().shared_hits), (0, 2));
        assert!(Arc::ptr_eq(&a.slots[0].regex, &b.slots[0].regex));

        let found = a.find_next_match(&scanner_a, "abbc", 1, 0, SearchOptions::NONE);
        assert_eq!(found.unwrap().range(), (1, 3));
        a.find_next_match(&scanner_a, "abbc", 1, 0, SearchOptions::NONE);
        assert!(a.stats().cache_hits > 0);

        let found = b.find_next_match(&scanner_b, "xxc", 1, 0, SearchOptions::NONE);
        assert_eq!(found.unwrap().range(), (2, 3));
        assert_eq!(b.stats().cache_hits, 0);
    }
}
