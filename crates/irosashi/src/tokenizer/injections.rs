use std::collections::HashMap;
use std::mem;
use std::sync::Arc;

use crate::Error;
use crate::grammar::{Grammar, Priority, RuleId, Selector, compile_rule_list};
use crate::regex::{Match, SearchOptions};
use crate::scope::{ScopeInterner, ScopeListId};
use crate::tokenizer::SessionStats;
use crate::tokenizer::line::LineCtx;
use crate::tokenizer::memo::{CompiledSet, EntryRule};

/// Grammars that declare `injectTo` a scope.
pub trait InjectionProvider {
    fn injectors_for(&self, scope: &str) -> Vec<Arc<Grammar>>;
}

impl InjectionProvider for () {
    fn injectors_for(&self, _scope: &str) -> Vec<Arc<Grammar>> {
        Vec::new()
    }
}

impl InjectionProvider for HashMap<String, Arc<Grammar>> {
    fn injectors_for(&self, _scope: &str) -> Vec<Arc<Grammar>> {
        Vec::new()
    }
}

/// One injection: a selector and the rules it contributes, compiled lazily against
/// the grammar they belong to.
pub(crate) struct InjectionEntry {
    pub grammar: Arc<Grammar>,
    pub rules: Vec<RuleId>,
    pub selector: Selector,
    pub set: Option<Result<CompiledSet, Arc<Error>>>,
}

/// Per scope list, the injections whose selector matches it, indexed by
/// `ScopeListId`; `None` until first seen.
pub(crate) type InjectionHits = Vec<Option<Box<[(usize, Priority)]>>>;

/// Grammar-local injections in source order, then external injectors.
pub(crate) fn collect_injections(
    grammar: &Arc<Grammar>,
    provider: &dyn InjectionProvider,
) -> Vec<InjectionEntry> {
    let mut all: Vec<InjectionEntry> = grammar
        .injections
        .iter()
        .map(|inj| InjectionEntry {
            grammar: Arc::clone(grammar),
            rules: vec![inj.rule],
            selector: inj.selector.clone(),
            set: None,
        })
        .collect();
    for injector in provider.injectors_for(&grammar.scope_name) {
        let Some(selector) = injector.injection_selector.clone() else {
            continue;
        };
        if injector.root_patterns().is_empty() {
            continue;
        }
        let rules = injector.root_patterns().to_vec();
        all.push(InjectionEntry {
            grammar: injector,
            rules,
            selector,
            set: None,
        });
    }
    all
}

pub(crate) struct InjectionMatch {
    pub m: Match,
    pub rule: EntryRule,
    pub priority: Priority,
}

/// Fills the hit slot for `scopes` on first sight.
fn ensure_hits(
    hits: &mut InjectionHits,
    injections: &[InjectionEntry],
    interner: &ScopeInterner,
    scopes: ScopeListId,
    stats: &mut SessionStats,
) {
    let idx = scopes.index();
    if hits.len() <= idx {
        hits.resize_with(interner.list_count().max(idx + 1), || None);
    }
    if hits[idx].is_some() {
        stats.injection_list_hits += 1;
        return;
    }
    stats.injection_list_misses += 1;
    let names = interner.names(scopes);
    hits[idx] = Some(
        injections
            .iter()
            .enumerate()
            .filter_map(|(i, inj)| inj.selector.matches(&names).map(|p| (i, p)))
            .collect(),
    );
}

/// The earliest injection match at or after `pos`. On equal starts an `L:` injection
/// displaces one without left priority; otherwise the earlier-listed injection keeps
/// the slot.
pub(crate) fn match_injections(
    cx: &mut LineCtx<'_>,
    scopes: ScopeListId,
    text: &str,
    pos: usize,
    options: SearchOptions,
) -> Option<InjectionMatch> {
    if cx.injections.is_empty() {
        return None;
    }
    let LineCtx {
        injections,
        injection_hits,
        interner,
        resolver,
        base,
        stats,
        capture_buf,
        ..
    } = cx;
    ensure_hits(injection_hits, injections, interner, scopes, stats);
    let hits: &[(usize, Priority)] = injection_hits[scopes.index()].as_deref().unwrap_or(&[]);
    let mut best: Option<InjectionMatch> = None;
    for &(index, priority) in hits {
        let entry = &mut injections[index];
        if entry.set.is_none() {
            stats.injection_set_compiles += 1;
            entry.set = Some(
                compile_rule_list(&entry.grammar, &entry.rules, base, *resolver)
                    .and_then(CompiledSet::from_rules)
                    .map_err(Arc::new),
            );
        }
        let Some(Ok(compiled)) = &mut entry.set else {
            continue;
        };
        if compiled.rules.is_empty() {
            continue;
        }
        stats.injection_searches += 1;
        let Some(match_index) = compiled
            .set
            .find_next_match_into(text, pos, options, capture_buf)
        else {
            continue;
        };
        let start = capture_buf[0].map_or(pos, |(s, _)| s);
        let replace = match &best {
            None => true,
            Some(current) => {
                start < current.m.start()
                    || (start == current.m.start()
                        && priority == Priority::Left
                        && current.priority != Priority::Left)
            }
        };
        if replace {
            let rule = compiled.rules[match_index].clone();
            let recycled = best.take().map(|b| b.m.captures).unwrap_or_default();
            let captures = mem::replace(*capture_buf, recycled);
            capture_buf.clear();
            best = Some(InjectionMatch {
                m: Match {
                    index: match_index,
                    captures,
                },
                rule,
                priority,
            });
        }
    }
    best
}

/// Chooses between the grammar's match and the best injection match: earlier start
/// wins; on a tie the injection wins only with `L:` priority.
pub(crate) fn pick_best_match(
    grammar_match: Option<(Match, EntryRule)>,
    injection: Option<InjectionMatch>,
) -> Option<(Match, EntryRule)> {
    match (grammar_match, injection) {
        (g, None) => g,
        (None, Some(inj)) => Some((inj.m, inj.rule)),
        (Some(g), Some(inj)) => {
            let (g_start, i_start) = (g.0.start(), inj.m.start());
            if i_start < g_start || (i_start == g_start && inj.priority == Priority::Left) {
                Some((inj.m, inj.rule))
            } else {
                Some(g)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::CompiledRule;

    fn m(start: usize) -> Match {
        Match {
            index: 0,
            captures: vec![Some((start, start + 1))],
        }
    }

    fn grammar() -> Arc<Grammar> {
        Arc::new(
            Grammar::parse(br#"{"scopeName": "source.t", "patterns": [{"match": "a"}]}"#).unwrap(),
        )
    }

    fn entry(g: &Arc<Grammar>) -> EntryRule {
        EntryRule::Rule(CompiledRule {
            grammar: Arc::clone(g),
            rule: RuleId(1),
        })
    }

    fn inj(start: usize, priority: Priority, g: &Arc<Grammar>) -> InjectionMatch {
        InjectionMatch {
            m: m(start),
            rule: entry(g),
            priority,
        }
    }

    fn is_end(result: Option<(Match, EntryRule)>) -> bool {
        matches!(result, Some((_, EntryRule::End)))
    }

    #[test]
    fn pick_best_match_table() {
        let g = grammar();
        let gm = |start| Some((m(start), EntryRule::End));
        assert!(pick_best_match(None, None).is_none());
        assert!(is_end(pick_best_match(gm(2), None)));
        assert!(!is_end(pick_best_match(
            None,
            Some(inj(2, Priority::None, &g))
        )));
        assert!(!is_end(pick_best_match(
            gm(5),
            Some(inj(2, Priority::None, &g))
        )));
        assert!(is_end(pick_best_match(
            gm(1),
            Some(inj(2, Priority::Left, &g))
        )));
        assert!(is_end(pick_best_match(
            gm(2),
            Some(inj(2, Priority::None, &g))
        )));
        assert!(is_end(pick_best_match(
            gm(2),
            Some(inj(2, Priority::Right, &g))
        )));
        assert!(!is_end(pick_best_match(
            gm(2),
            Some(inj(2, Priority::Left, &g))
        )));
    }
}
