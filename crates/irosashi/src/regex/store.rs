use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::regex::raw::Regex;

type Compiled = Result<Arc<Regex>, Arc<str>>;

/// Compiled patterns shared by every session of one highlighter, keyed by source.
///
/// Only the immutable compiled objects live here. Match caches and the scratch region
/// stay in each session's `PatternTable`.
#[derive(Default)]
pub(crate) struct RegexStore {
    by_source: Mutex<HashMap<Arc<str>, Compiled>>,
}

impl RegexStore {
    /// The compiled pattern for `source` and whether this call compiled it. A pattern
    /// Oniguruma rejects is remembered with its message.
    pub fn get_or_compile(&self, source: &str) -> (Compiled, bool) {
        if let Some(known) = self.lock().get(source) {
            return (known.clone(), false);
        }
        // Compiled outside the lock so lookups of other patterns are not held up; when
        // two threads race, the first insert wins and both use that object.
        let compiled = Regex::new(source).map(Arc::new).map_err(Arc::from);
        let mut by_source = self.lock();
        let stored = by_source.entry(Arc::from(source)).or_insert(compiled);
        (stored.clone(), true)
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.lock().len()
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<Arc<str>, Compiled>> {
        self.by_source
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

impl std::fmt::Debug for RegexStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RegexStore")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_source_is_compiled_once() {
        let store = RegexStore::default();
        let (first, compiled) = store.get_or_compile(r"\w+");
        assert!(compiled);
        let (second, compiled) = store.get_or_compile(r"\w+");
        assert!(!compiled);
        assert!(Arc::ptr_eq(&first.unwrap(), &second.unwrap()));
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn rejected_patterns_are_remembered() {
        let store = RegexStore::default();
        let (first, compiled) = store.get_or_compile("(");
        assert!(compiled);
        let (second, compiled) = store.get_or_compile("(");
        assert!(!compiled);
        let (Err(first), Err(second)) = (first, second) else {
            panic!("unbalanced paren compiled");
        };
        assert!(Arc::ptr_eq(&first, &second));
    }

    #[test]
    fn racing_threads_end_with_one_object_per_source() {
        let store = Arc::new(RegexStore::default());
        let sources: Vec<String> = (0..50).map(|i| format!("a{{{i}}}b")).collect();
        let results: Vec<Vec<Arc<Regex>>> = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..8)
                .map(|_| {
                    scope.spawn(|| {
                        sources
                            .iter()
                            .map(|s| store.get_or_compile(s).0.unwrap())
                            .collect::<Vec<_>>()
                    })
                })
                .collect();
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });
        assert_eq!(store.len(), sources.len());
        for other in &results[1..] {
            for (a, b) in results[0].iter().zip(other) {
                assert!(Arc::ptr_eq(a, b));
            }
        }
    }
}
