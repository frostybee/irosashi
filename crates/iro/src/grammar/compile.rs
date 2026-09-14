use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::Error;
use crate::grammar::rules::{Grammar, Include, ROOT_RULE_ID, Rule, RuleId};

const MAX_INCLUDE_DEPTH: usize = 256;

/// Looks up other grammars for `source.x` and `source.x#key` includes.
pub trait GrammarResolver {
    fn grammar_by_scope(&self, scope: &str) -> Option<Arc<Grammar>>;
}

impl GrammarResolver for () {
    fn grammar_by_scope(&self, _scope: &str) -> Option<Arc<Grammar>> {
        None
    }
}

impl GrammarResolver for HashMap<String, Arc<Grammar>> {
    fn grammar_by_scope(&self, scope: &str) -> Option<Arc<Grammar>> {
        self.get(scope).cloned()
    }
}

/// One scanner entry: the rule whose `match` or `begin` pattern is searched, together
/// with the grammar that owns it (includes may pull rules from other grammars).
#[derive(Debug, Clone)]
pub struct CompiledRule {
    pub grammar: Arc<Grammar>,
    pub rule: RuleId,
}

impl CompiledRule {
    pub fn pattern(&self) -> &str {
        self.grammar.rule(self.rule).scanner_pattern().unwrap_or("")
    }
}

/// Flattens the child patterns of `rule` into scanner order, resolving includes.
///
/// `base` is the grammar tokenization started with (`$base`); pass `grammar` itself
/// when there is no embedding. Missing repository keys and unknown grammars resolve to
/// nothing, as in vscode-textmate. Include cycles and nesting beyond 256 levels are
/// reported as errors.
pub fn compile_patterns(
    grammar: &Arc<Grammar>,
    rule: RuleId,
    base: &Arc<Grammar>,
    resolver: &dyn GrammarResolver,
) -> Result<Vec<CompiledRule>, Error> {
    let mut compiler = Compiler {
        base,
        resolver,
        visited: HashSet::new(),
        out: Vec::new(),
    };
    let context = Context::root(grammar);
    match grammar.rule(rule) {
        Rule::Collection {
            patterns,
            repository,
        } => {
            let context = if repository.is_some() {
                context.with_local(rule)
            } else {
                context
            };
            compiler.compile_list(&context, patterns)?;
        }
        Rule::Include(include) => compiler.compile_include(&context, include)?,
        other => compiler.compile_list(&context, other.patterns())?,
    }
    Ok(compiler.out)
}

/// Flattens an explicit list of rules of `grammar` into scanner order. Unlike
/// `compile_patterns`, a `Match` or `BeginEnd` rule in `ids` contributes itself rather
/// than its children, which is what injections need.
pub fn compile_rule_list(
    grammar: &Arc<Grammar>,
    ids: &[RuleId],
    base: &Arc<Grammar>,
    resolver: &dyn GrammarResolver,
) -> Result<Vec<CompiledRule>, Error> {
    let mut compiler = Compiler {
        base,
        resolver,
        visited: HashSet::new(),
        out: Vec::new(),
    };
    compiler.compile_list(&Context::root(grammar), ids)?;
    Ok(compiler.out)
}

#[derive(Clone)]
enum RepoRef {
    Root(Arc<Grammar>),
    Local(Arc<Grammar>, RuleId),
}

impl RepoRef {
    fn get(&self, key: &str) -> Option<RuleId> {
        match self {
            Self::Root(grammar) => grammar.repository.get(key),
            Self::Local(grammar, rule) => match grammar.rule(*rule) {
                Rule::Collection {
                    repository: Some(repository),
                    ..
                } => repository.get(key),
                _ => None,
            },
        }
    }
}

/// The grammar being walked and its repository chain, innermost last.
#[derive(Clone)]
struct Context {
    grammar: Arc<Grammar>,
    repos: Vec<RepoRef>,
}

impl Context {
    fn root(grammar: &Arc<Grammar>) -> Self {
        Self {
            grammar: Arc::clone(grammar),
            repos: vec![RepoRef::Root(Arc::clone(grammar))],
        }
    }

    fn with_local(&self, collection: RuleId) -> Self {
        let mut repos = self.repos.clone();
        repos.push(RepoRef::Local(Arc::clone(&self.grammar), collection));
        Self {
            grammar: Arc::clone(&self.grammar),
            repos,
        }
    }

    fn lookup(&self, key: &str) -> Option<RuleId> {
        self.repos.iter().rev().find_map(|repo| repo.get(key))
    }
}

struct Compiler<'a> {
    base: &'a Arc<Grammar>,
    resolver: &'a dyn GrammarResolver,
    visited: HashSet<(usize, RuleId)>,
    out: Vec<CompiledRule>,
}

impl Compiler<'_> {
    fn compile_list(&mut self, context: &Context, ids: &[RuleId]) -> Result<(), Error> {
        for &id in ids {
            match context.grammar.rule(id) {
                Rule::Match { regex, .. } => {
                    if !regex.source().is_empty() {
                        self.emit(context, id);
                    }
                }
                Rule::BeginEnd {
                    begin, patterns, ..
                }
                | Rule::BeginWhile {
                    begin, patterns, ..
                } => {
                    if !begin.source().is_empty() && !self.all_unresolvable(context, patterns) {
                        self.emit(context, id);
                    }
                }
                Rule::Collection {
                    patterns,
                    repository,
                } => {
                    if repository.is_some() {
                        self.compile_list(&context.with_local(id), patterns)?;
                    } else {
                        self.compile_list(context, patterns)?;
                    }
                }
                Rule::Include(include) => self.compile_include(context, include)?,
                Rule::Noop => {}
            }
        }
        Ok(())
    }

    fn emit(&mut self, context: &Context, rule: RuleId) {
        self.out.push(CompiledRule {
            grammar: Arc::clone(&context.grammar),
            rule,
        });
    }

    fn compile_include(&mut self, context: &Context, include: &Include) -> Result<(), Error> {
        match include {
            Include::SelfRef => {
                let root = context.grammar.root_patterns();
                self.compile_guarded(context, root_key(context), |c, ctx| {
                    c.compile_list(ctx, root)
                })
            }
            Include::Base => {
                let base = Arc::clone(self.base);
                let base_context = Context::root(&base);
                self.compile_guarded(&base_context, root_key(&base_context), |c, ctx| {
                    c.compile_list(ctx, base.root_patterns())
                })
            }
            Include::Local(key) => match context.lookup(key) {
                Some(id) => self.compile_resolved(context, id),
                None => Ok(()),
            },
            Include::Scope(scope) => match self.resolver.grammar_by_scope(scope) {
                Some(foreign) => {
                    let foreign_context = Context::root(&foreign);
                    self.compile_guarded(&foreign_context, root_key(&foreign_context), |c, ctx| {
                        c.compile_list(ctx, foreign.root_patterns())
                    })
                }
                None => Ok(()),
            },
            Include::ScopeKey(scope, key) => match self.resolver.grammar_by_scope(scope) {
                Some(foreign) => match foreign.repository.get(key) {
                    Some(id) => self.compile_resolved(&Context::root(&foreign), id),
                    None => Ok(()),
                },
                None => Ok(()),
            },
        }
    }

    /// Compiles a rule reached through an include: a collection contributes its
    /// children (and shadows with its repository), anything else contributes itself.
    fn compile_resolved(&mut self, context: &Context, id: RuleId) -> Result<(), Error> {
        let key = (Arc::as_ptr(&context.grammar) as usize, id);
        self.compile_guarded(context, key, |c, ctx| match ctx.grammar.rule(id) {
            Rule::Collection {
                patterns,
                repository,
            } => {
                if repository.is_some() {
                    c.compile_list(&ctx.with_local(id), patterns)
                } else {
                    c.compile_list(ctx, patterns)
                }
            }
            _ => c.compile_list(ctx, &[id]),
        })
    }

    fn compile_guarded(
        &mut self,
        context: &Context,
        key: (usize, RuleId),
        body: impl FnOnce(&mut Self, &Context) -> Result<(), Error>,
    ) -> Result<(), Error> {
        if !self.visited.insert(key) {
            return Err(Error::GrammarCycle);
        }
        let result = if self.visited.len() > MAX_INCLUDE_DEPTH {
            Err(Error::GrammarDepth)
        } else {
            body(self, context)
        };
        self.visited.remove(&key);
        result
    }

    /// vscode-textmate drops a begin rule from the scanner when every child is an
    /// include whose target does not exist. Existence only: a self-include counts as
    /// resolvable.
    fn all_unresolvable(&self, context: &Context, patterns: &[RuleId]) -> bool {
        !patterns.is_empty()
            && patterns.iter().all(|&id| match context.grammar.rule(id) {
                Rule::Include(include) => !self.include_exists(context, include),
                _ => false,
            })
    }

    fn include_exists(&self, context: &Context, include: &Include) -> bool {
        match include {
            Include::SelfRef | Include::Base => true,
            Include::Local(key) => context.lookup(key).is_some(),
            Include::Scope(scope) => self.resolver.grammar_by_scope(scope).is_some(),
            Include::ScopeKey(scope, key) => self
                .resolver
                .grammar_by_scope(scope)
                .is_some_and(|foreign| foreign.repository.get(key).is_some()),
        }
    }
}

fn root_key(context: &Context) -> (usize, RuleId) {
    (Arc::as_ptr(&context.grammar) as usize, ROOT_RULE_ID)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grammar(json: &str) -> Arc<Grammar> {
        Arc::new(Grammar::parse(json.as_bytes()).expect("test grammar parses"))
    }

    fn patterns(
        grammar: &Arc<Grammar>,
        rule: RuleId,
        base: &Arc<Grammar>,
        resolver: &dyn GrammarResolver,
    ) -> Vec<String> {
        compile_patterns(grammar, rule, base, resolver)
            .expect("compiles")
            .iter()
            .map(|c| c.pattern().to_owned())
            .collect()
    }

    fn root(grammar: &Arc<Grammar>) -> Vec<String> {
        patterns(grammar, ROOT_RULE_ID, grammar, &())
    }

    #[test]
    fn flattens_in_source_order_through_self_and_local_includes() {
        let g = grammar(
            r##"{"scopeName": "source.t", "patterns": [
                {"match": "a"}, {"include": "#b"}, {"match": "c"}
            ], "repository": {
                "b": {"patterns": [{"match": "b1"}, {"include": "#d"}, {"match": "b2"}]},
                "d": {"match": "d"}
            }}"##,
        );
        assert_eq!(root(&g), ["a", "b1", "d", "b2", "c"]);
    }

    #[test]
    fn self_include_inside_repository_expands_root() {
        let g = grammar(
            r##"{"scopeName": "source.t", "patterns": [{"match": "a"}, {"include": "#nested"}],
            "repository": {"nested": {"begin": "\\(", "end": "\\)", "patterns": [{"include": "$self"}]}}}"##,
        );
        assert_eq!(root(&g), ["a", "\\("]);
        let nested = g.repository.get("nested").unwrap();
        assert_eq!(patterns(&g, nested, &g, &()), ["a", "\\("]);
    }

    #[test]
    fn local_repository_shadows_outer_entries() {
        let g = grammar(
            r##"{"scopeName": "source.t", "patterns": [{"include": "#outer"}, {"include": "#x"}],
            "repository": {
                "x": {"match": "outer-x"},
                "outer": {"patterns": [{"include": "#x"}], "repository": {"x": {"match": "inner-x"}}}
            }}"##,
        );
        assert_eq!(root(&g), ["inner-x", "outer-x"]);
    }

    #[test]
    fn missing_local_key_and_unknown_scope_are_silent() {
        let g = grammar(
            r##"{"scopeName": "source.t", "patterns": [
                {"include": "#missing"}, {"include": "source.other"}, {"include": "source.other#k"}, {"match": "a"}
            ]}"##,
        );
        assert_eq!(root(&g), ["a"]);
    }

    #[test]
    fn cross_grammar_includes_switch_context() {
        let host = grammar(
            r##"{"scopeName": "source.host", "patterns": [
                {"include": "source.guest"}, {"include": "source.guest#part"}, {"match": "h"}
            ]}"##,
        );
        let guest = grammar(
            r##"{"scopeName": "source.guest", "patterns": [{"match": "g"}, {"include": "#part"}],
            "repository": {"part": {"patterns": [{"match": "p"}, {"include": "#local"}]}, "local": {"match": "l"}}}"##,
        );
        let resolver: HashMap<String, Arc<Grammar>> =
            HashMap::from([("source.guest".to_owned(), Arc::clone(&guest))]);
        let compiled = compile_patterns(&host, ROOT_RULE_ID, &host, &resolver).unwrap();
        let got: Vec<(&str, &str)> = compiled
            .iter()
            .map(|c| (c.grammar.scope_name.as_str(), c.pattern()))
            .collect();
        assert_eq!(
            got,
            [
                ("source.guest", "g"),
                ("source.guest", "p"),
                ("source.guest", "l"),
                ("source.guest", "p"),
                ("source.guest", "l"),
                ("source.host", "h"),
            ]
        );
    }

    #[test]
    fn base_resolves_to_the_base_grammar_not_self() {
        let base = grammar(r#"{"scopeName": "source.base", "patterns": [{"match": "base"}]}"#);
        let embedded = grammar(
            r#"{"scopeName": "source.embedded", "patterns": [{"include": "$base"}, {"match": "self"}]}"#,
        );
        assert_eq!(
            patterns(&embedded, ROOT_RULE_ID, &base, &()),
            ["base", "self"]
        );
        assert!(matches!(
            compile_patterns(&embedded, ROOT_RULE_ID, &embedded, &()),
            Err(Error::GrammarCycle)
        ));
    }

    #[test]
    fn collection_cycle_is_an_error() {
        let g = grammar(
            r##"{"scopeName": "source.t", "patterns": [{"include": "#a"}],
            "repository": {"a": {"patterns": [{"include": "#b"}]}, "b": {"patterns": [{"include": "#a"}]}}}"##,
        );
        assert!(matches!(
            compile_patterns(&g, ROOT_RULE_ID, &g, &()),
            Err(Error::GrammarCycle)
        ));
        let g = grammar(r#"{"scopeName": "source.t", "patterns": [{"include": "$self"}]}"#);
        assert!(matches!(
            compile_patterns(&g, ROOT_RULE_ID, &g, &()),
            Err(Error::GrammarCycle)
        ));
    }

    #[test]
    fn begin_rules_with_only_missing_includes_are_dropped() {
        let g = grammar(
            r##"{"scopeName": "source.t", "patterns": [
                {"begin": "dropped", "end": "x", "patterns": [{"include": "#missing"}, {"include": "source.nope"}]},
                {"begin": "kept-self", "end": "x", "patterns": [{"include": "#kept"}]},
                {"begin": "kept-empty", "end": "x"},
                {"begin": "kept-mixed", "end": "x", "patterns": [{"include": "#missing"}, {"match": "m"}]}
            ], "repository": {"kept": {"begin": "kept-self", "end": "x", "patterns": [{"include": "#kept"}]}}}"##,
        );
        assert_eq!(root(&g), ["kept-self", "kept-empty", "kept-mixed"]);
    }

    #[test]
    fn rule_context_compiles_children_only() {
        let g = grammar(
            r##"{"scopeName": "source.t", "patterns": [{"include": "#block"}],
            "repository": {"block": {"begin": "b", "end": "e", "patterns": [{"match": "inner"}]}}}"##,
        );
        let block = g.repository.get("block").unwrap();
        assert_eq!(patterns(&g, block, &g, &()), ["inner"]);
        let inner = g.rule(block).patterns()[0];
        assert!(patterns(&g, inner, &g, &()).is_empty());
    }
}
