use std::collections::HashMap;
use std::sync::Arc;

use crate::Error;
use crate::grammar::{Grammar, Priority, RuleId, Selector, compile_rule_list};
use crate::regex::{Match, SearchOptions};
use crate::scope::{ScopeInterner, ScopeListId};
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

/// Injections whose selector matches `scopes`, with the matching priority.
fn matching_injections<'a>(
    hits: &'a mut HashMap<ScopeListId, Vec<(usize, Priority)>>,
    injections: &[InjectionEntry],
    interner: &ScopeInterner,
    scopes: ScopeListId,
) -> &'a [(usize, Priority)] {
    hits.entry(scopes).or_insert_with(|| {
        let names = interner.names(scopes);
        injections
            .iter()
            .enumerate()
            .filter_map(|(i, inj)| inj.selector.matches(&names).map(|p| (i, p)))
            .collect()
    })
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
    let candidates =
        matching_injections(cx.injection_hits, cx.injections, cx.interner, scopes).to_vec();
    let mut best: Option<InjectionMatch> = None;
    for (index, priority) in candidates {
        let entry = &mut cx.injections[index];
        let base = cx.base;
        let resolver = cx.resolver;
        let set = entry.set.get_or_insert_with(|| {
            compile_rule_list(&entry.grammar, &entry.rules, base, resolver)
                .and_then(CompiledSet::from_rules)
                .map_err(Arc::new)
        });
        let Ok(compiled) = set else {
            continue;
        };
        if compiled.rules.is_empty() {
            continue;
        }
        let Some(m) = compiled.set.find_next_match(text, pos, options) else {
            continue;
        };
        let start = m.start();
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
            let rule = compiled.rules[m.index].clone();
            best = Some(InjectionMatch { m, rule, priority });
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
