use std::collections::HashMap;
use std::mem;
use std::sync::Arc;

use crate::Error;
use crate::grammar::{CompiledRule, Grammar, GrammarResolver, Rule, RuleId, compile_patterns};
use crate::regex::{Match, PatternId, PatternTable, ScanStats, Scanner, SearchOptions};

pub(crate) use crate::regex::CaptureBuf;

/// Identity of a scanner context: the open rule, the base grammar that `$base`
/// resolves to, and the begin-capture-resolved end pattern when the rule has one.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct MemoKey {
    pub grammar: usize,
    pub rule: RuleId,
    pub base: usize,
    pub end: Option<Arc<str>>,
}

impl MemoKey {
    pub fn new(
        grammar: &Arc<Grammar>,
        rule: RuleId,
        base: &Arc<Grammar>,
        end: Option<Arc<str>>,
    ) -> Self {
        Self {
            grammar: Arc::as_ptr(grammar) as usize,
            rule,
            base: Arc::as_ptr(base) as usize,
            end,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum EntryRule {
    End,
    Rule(CompiledRule),
}

#[derive(Debug)]
pub(crate) struct CompiledSet {
    pub rules: Vec<EntryRule>,
    pub scanner: Scanner,
}

impl CompiledSet {
    /// Builds the scanner for `rule`'s children, with the end pattern first (or last
    /// for `applyEndPatternLast`) when `rule` is a begin/end rule.
    pub fn compile(
        grammar: &Arc<Grammar>,
        rule: RuleId,
        base: &Arc<Grammar>,
        resolver: &dyn GrammarResolver,
        end_override: Option<&str>,
        table: &mut PatternTable,
    ) -> Result<Self, Error> {
        let compiled = compile_patterns(grammar, rule, base, resolver)?;
        let end = match grammar.rule(rule) {
            Rule::BeginEnd {
                end,
                apply_end_pattern_last,
                ..
            } => Some((
                end_override.unwrap_or(end.source()),
                *apply_end_pattern_last,
            )),
            _ => None,
        };
        Self::build(compiled, end, table)
    }

    pub fn from_rules(
        compiled: Vec<CompiledRule>,
        table: &mut PatternTable,
    ) -> Result<Self, Error> {
        Self::build(compiled, None, table)
    }

    fn build(
        compiled: Vec<CompiledRule>,
        end: Option<(&str, bool)>,
        table: &mut PatternTable,
    ) -> Result<Self, Error> {
        let mut rules = Vec::with_capacity(compiled.len() + 1);
        let mut patterns: Vec<&str> = Vec::with_capacity(compiled.len() + 1);
        if let Some((end_pattern, false)) = end {
            rules.push(EntryRule::End);
            patterns.push(end_pattern);
        }
        for rule in &compiled {
            patterns.push(rule.pattern());
        }
        rules.extend(compiled.iter().cloned().map(EntryRule::Rule));
        if let Some((end_pattern, true)) = end {
            rules.push(EntryRule::End);
            patterns.push(end_pattern);
        }
        let scanner = table.scanner(&patterns)?;
        Ok(Self { rules, scanner })
    }
}

/// Index of a compiled context within one `Memo`; valid only for that memo.
pub(crate) type SetId = usize;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct MemoStats {
    pub hits: u64,
    pub misses: u64,
    pub errors: u64,
    pub while_hits: u64,
    pub while_misses: u64,
}

/// Per-session cache of compiled scanner contexts and while patterns, over the
/// session's pattern table.
#[derive(Debug, Default)]
pub(crate) struct Memo {
    sets: Vec<Result<CompiledSet, Arc<Error>>>,
    index: HashMap<MemoKey, SetId>,
    whiles: HashMap<Arc<str>, Result<PatternId, Arc<Error>>>,
    table: PatternTable,
    stats: MemoStats,
}

impl Memo {
    /// The id of the context identified by `key`, compiling it on first use.
    pub fn resolve(
        &mut self,
        key: MemoKey,
        compile: impl FnOnce(&mut PatternTable) -> Result<CompiledSet, Error>,
    ) -> SetId {
        if let Some(&id) = self.index.get(&key) {
            self.stats.hits += 1;
            return id;
        }
        self.stats.misses += 1;
        let compiled = compile(&mut self.table).map_err(Arc::new);
        if compiled.is_err() {
            self.stats.errors += 1;
        }
        self.sets.push(compiled);
        let id = self.sets.len() - 1;
        self.index.insert(key, id);
        id
    }

    /// Searches a resolved context. The capture groups are written into `captures`,
    /// whose allocation the returned `Match` takes over; the caller hands it back
    /// after use so a line reuses one buffer.
    pub fn search(
        &mut self,
        id: SetId,
        text: &str,
        generation: u64,
        pos: usize,
        options: SearchOptions,
        captures: &mut CaptureBuf,
    ) -> Result<Option<(Match, EntryRule)>, Arc<Error>> {
        match &self.sets[id] {
            Err(err) => Err(Arc::clone(err)),
            Ok(compiled) => Ok(self
                .table
                .find_next_match_into(&compiled.scanner, text, generation, pos, options, captures)
                .map(|index| {
                    let rule = compiled.rules[index].clone();
                    let m = Match {
                        index,
                        captures: mem::take(captures),
                    };
                    (m, rule)
                })),
        }
    }

    pub fn search_while(
        &mut self,
        pattern: &Arc<str>,
        text: &str,
        generation: u64,
        pos: usize,
        options: SearchOptions,
    ) -> Result<Option<Match>, Arc<Error>> {
        let id = match self.whiles.get(pattern) {
            Some(known) => {
                self.stats.while_hits += 1;
                known.clone()
            }
            None => {
                self.stats.while_misses += 1;
                let compiled = self.table.intern(pattern).map_err(|message| {
                    Arc::new(Error::RegexCompilation {
                        index: 0,
                        message: message.to_string(),
                    })
                });
                self.whiles.insert(Arc::clone(pattern), compiled.clone());
                compiled
            }
        }?;
        Ok(self
            .table
            .find_next_match_single(id, text, generation, pos, options))
    }

    /// Searches an injection scanner against the same table and caches.
    pub fn search_scanner(
        &mut self,
        scanner: &Scanner,
        text: &str,
        generation: u64,
        pos: usize,
        options: SearchOptions,
        captures: &mut CaptureBuf,
    ) -> Option<usize> {
        self.table
            .find_next_match_into(scanner, text, generation, pos, options, captures)
    }

    pub fn table_mut(&mut self) -> &mut PatternTable {
        &mut self.table
    }

    /// Compiled contexts (including while patterns) and compiled patterns.
    pub fn len(&self) -> (usize, usize) {
        (self.sets.len() + self.whiles.len(), self.table.len())
    }

    pub fn scan_stats(&self) -> ScanStats {
        self.table.stats()
    }

    pub fn stats(&self) -> MemoStats {
        self.stats
    }

    pub fn reset_stats(&mut self) {
        self.stats = MemoStats::default();
        self.table.reset_stats();
    }

    #[cfg(test)]
    pub fn entry_len(&self, key: &MemoKey) -> Option<usize> {
        match &self.sets[*self.index.get(key)?] {
            Ok(compiled) => Some(compiled.rules.len()),
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::ROOT_RULE_ID;

    fn grammar() -> Arc<Grammar> {
        Arc::new(
            Grammar::parse(
                br#"{"scopeName": "source.t", "patterns": [
                    {"begin": "\\(", "end": "\\)", "patterns": [{"match": "x"}]},
                    {"begin": "<", "end": ">", "applyEndPatternLast": 1, "patterns": [{"match": "y"}]}
                ]}"#,
            )
            .unwrap(),
        )
    }

    fn search_key(
        memo: &mut Memo,
        g: &Arc<Grammar>,
        key: &MemoKey,
    ) -> Result<Option<usize>, Arc<Error>> {
        let override_end = key.end.clone();
        let rule = key.rule;
        let id = memo.resolve(key.clone(), |table| {
            CompiledSet::compile(g, rule, g, &(), override_end.as_deref(), table)
        });
        let mut buf = CaptureBuf::new();
        memo.search(id, "(x)\n", 1, 0, SearchOptions::NONE, &mut buf)
            .map(|m| m.map(|(m, _)| m.start()))
    }

    #[test]
    fn end_pattern_position_follows_apply_end_pattern_last() {
        let g = grammar();
        let [paren, angle] = g.root_patterns() else {
            panic!("two root rules");
        };
        let mut table = PatternTable::default();
        let first = CompiledSet::compile(&g, *paren, &g, &(), None, &mut table).unwrap();
        assert!(matches!(first.rules[0], EntryRule::End));
        assert_eq!(first.rules.len(), 2);
        let last = CompiledSet::compile(&g, *angle, &g, &(), None, &mut table).unwrap();
        assert!(matches!(last.rules[1], EntryRule::End));
        assert_eq!(last.rules.len(), 2);
    }

    #[test]
    fn empty_end_override_is_distinct_from_no_end() {
        let g = grammar();
        let paren = g.root_patterns()[0];
        let mut memo = Memo::default();
        let root = MemoKey::new(&g, ROOT_RULE_ID, &g, None);
        let empty_end = MemoKey::new(&g, paren, &g, Some(Arc::from("")));
        let static_end = MemoKey::new(&g, paren, &g, None);
        assert_ne!(root, static_end);
        assert_ne!(empty_end, static_end);
        for key in [&root, &empty_end, &static_end] {
            search_key(&mut memo, &g, key).unwrap();
        }
        assert_eq!(memo.entry_len(&root), Some(2));
        assert_eq!(memo.entry_len(&empty_end), Some(2));
        assert_eq!(memo.entry_len(&static_end), Some(2));
        search_key(&mut memo, &g, &root).unwrap();
        assert_eq!(memo.stats().misses, 3);
        assert_eq!(memo.stats().hits, 1);
    }

    #[test]
    fn bad_pattern_errors_are_memoized() {
        let g = grammar();
        let paren = g.root_patterns()[0];
        let mut memo = Memo::default();
        let key = MemoKey::new(&g, paren, &g, Some(Arc::from("(")));
        assert!(search_key(&mut memo, &g, &key).is_err());
        let id = memo.resolve(key, |_| unreachable!());
        let mut buf = CaptureBuf::new();
        assert!(
            memo.search(id, "x\n", 1, 0, SearchOptions::NONE, &mut buf)
                .is_err()
        );
        assert_eq!(memo.stats().errors, 1);
        assert!(
            memo.search_while(&Arc::from("["), "x\n", 1, 0, SearchOptions::NONE)
                .is_err()
        );
    }
}
