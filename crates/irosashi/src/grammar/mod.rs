mod backref;
mod compile;
mod parse;
mod raw;
mod rules;
mod selector;

#[cfg(test)]
mod tests_corpus;

pub(crate) use backref::{resolve_backrefs, resolve_scope_backrefs};
pub(crate) use compile::{CompiledRule, GrammarResolver, compile_patterns, compile_rule_list};
pub(crate) use rules::{Captures, Grammar, ROOT_RULE_ID, Rule, RuleId};
pub(crate) use selector::{Priority, Selector};
