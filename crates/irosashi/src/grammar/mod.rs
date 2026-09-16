mod backref;
mod compile;
mod parse;
mod raw;
mod rules;
mod selector;

#[cfg(test)]
mod tests_corpus;

pub(crate) use backref::{resolve_backrefs, resolve_scope_backrefs};
pub use compile::compile_patterns;
pub(crate) use compile::{CompiledRule, GrammarResolver, compile_rule_list};
pub(crate) use rules::{Captures, Rule, RuleId};
pub use rules::{Grammar, ROOT_RULE_ID};
pub(crate) use selector::{Priority, Selector};
