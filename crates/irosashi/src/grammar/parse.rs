use crate::Error;
use crate::grammar::backref::has_backref_marker;
use crate::grammar::raw::{RawCaptures, RawGrammar, RawRepository, RawRule};
use crate::grammar::rules::{
    Capture, Captures, Grammar, Include, Injection, ROOT_RULE_ID, Repository, Rule, RuleId,
};
#[cfg(test)]
use crate::grammar::selector::Priority;
use crate::grammar::selector::Selector;
use crate::regex::LazyRegex;

/// vscode-textmate's end pattern for a `begin` rule without `end`: never matches.
const DEFAULT_END: &str = "\u{FFFF}";

impl Grammar {
    pub fn parse(json: &[u8]) -> Result<Grammar, Error> {
        let raw: RawGrammar =
            serde_json::from_slice(json).map_err(|err| Error::GrammarParse(err.to_string()))?;
        let scope_name = raw
            .scope_name
            .filter(|name| !name.is_empty())
            .ok_or_else(|| Error::GrammarParse("missing scopeName".to_owned()))?;

        let mut parser = Parser {
            rules: Vec::new(),
            enclosing: Vec::new(),
            repo_stack: Vec::new(),
            too_large: false,
        };
        let root = parser.reserve();
        let patterns = parser.parse_list(&raw.patterns);
        parser.rules[root.index()] = Rule::Collection {
            patterns,
            repository: None,
        };
        debug_assert_eq!(root, ROOT_RULE_ID);
        let repository = parser.parse_repository(&raw.repository);
        let injections = raw
            .injections
            .0
            .iter()
            .filter_map(|(selector, rule)| {
                parser.parse_rule(rule, false).map(|rule| Injection {
                    raw_selector: selector.clone(),
                    selector: Selector::parse(selector),
                    rule,
                })
            })
            .collect();
        if parser.too_large {
            return Err(Error::GrammarTooLarge);
        }

        Ok(Grammar {
            scope_name,
            name: raw.name,
            rules: parser.rules,
            enclosing: parser.enclosing,
            repository,
            injections,
            injection_selector: raw.injection_selector.as_deref().map(Selector::parse),
            inject_to: raw.inject_to,
        })
    }
}

struct Parser {
    rules: Vec<Rule>,
    enclosing: Vec<Option<RuleId>>,
    repo_stack: Vec<RuleId>,
    too_large: bool,
}

impl Parser {
    fn reserve(&mut self) -> RuleId {
        let Ok(index) = u32::try_from(self.rules.len()) else {
            // Past the arena limit: keep handing out the last slot so parsing can
            // finish, and let `parse` report the error.
            self.too_large = true;
            return RuleId(u32::MAX);
        };
        self.rules.push(Rule::Noop);
        self.enclosing.push(self.repo_stack.last().copied());
        RuleId(index)
    }

    fn parse_list(&mut self, raws: &[RawRule]) -> Vec<RuleId> {
        raws.iter()
            .filter_map(|raw| self.parse_rule(raw, true))
            .collect()
    }

    fn parse_repository(&mut self, raw: &RawRepository) -> Repository {
        let mut repository = Repository::default();
        for (key, rule) in &raw.0 {
            if let Some(id) = self.parse_rule(rule, false) {
                repository.0.insert(key.clone(), id);
            }
        }
        repository
    }

    /// Returns `None` for rules that contribute nothing, so parents drop them.
    ///
    /// Dispatch follows vscode-textmate: inside a `patterns` list an `include` overrides
    /// every sibling key; elsewhere `match`, then `begin`, then `patterns` win over it.
    fn parse_rule(&mut self, raw: &RawRule, in_pattern_list: bool) -> Option<RuleId> {
        let match_ = raw.match_.as_deref();
        let begin = raw.begin.as_deref();
        let has_patterns = !raw.patterns.is_empty();

        if let Some(include) = raw.include.as_deref()
            && (in_pattern_list || (match_.is_none() && begin.is_none() && !has_patterns))
        {
            if include.is_empty() {
                return None;
            }
            let id = self.reserve();
            self.rules[id.index()] = Rule::Include(Include::parse(include));
            return Some(id);
        }

        if let Some(pattern) = match_ {
            if pattern.is_empty() {
                return None;
            }
            let id = self.reserve();
            let captures = self.parse_captures(&raw.captures);
            self.rules[id.index()] = Rule::Match {
                name: raw.name.clone(),
                regex: LazyRegex::new(pattern),
                captures,
            };
            return Some(id);
        }

        if let Some(begin) = begin {
            let id = self.reserve();
            let begin_captures = self.parse_captures(pick(&raw.begin_captures, &raw.captures));
            let patterns = self.parse_list(&raw.patterns);
            let scope_backrefs =
                has_scope_backref(&raw.name) || has_scope_backref(&raw.content_name);
            self.rules[id.index()] = if let Some(while_) = raw.while_.as_deref() {
                let while_captures = self.parse_captures(pick(&raw.while_captures, &raw.captures));
                Rule::BeginWhile {
                    name: raw.name.clone(),
                    content_name: raw.content_name.clone(),
                    begin: LazyRegex::new(begin),
                    while_: LazyRegex::new(while_),
                    while_has_backrefs: has_backref_marker(while_),
                    begin_captures,
                    while_captures,
                    patterns,
                    needs_begin_capture_texts: has_backref_marker(while_) || scope_backrefs,
                }
            } else {
                let end = raw.end.as_deref().unwrap_or(DEFAULT_END);
                let end_captures = self.parse_captures(pick(&raw.end_captures, &raw.captures));
                Rule::BeginEnd {
                    name: raw.name.clone(),
                    content_name: raw.content_name.clone(),
                    begin: LazyRegex::new(begin),
                    end: LazyRegex::new(end),
                    end_has_backrefs: has_backref_marker(end),
                    begin_captures,
                    end_captures,
                    patterns,
                    apply_end_pattern_last: raw.apply_end_pattern_last,
                    needs_begin_capture_texts: has_backref_marker(end) || scope_backrefs,
                }
            };
            return Some(id);
        }

        if has_patterns || !raw.repository.0.is_empty() {
            return Some(self.parse_collection(&raw.patterns, &raw.repository));
        }

        None
    }

    /// A collection with its own repository scopes every rule parsed under it, its
    /// repository entries included, so includes inside resolve through it later.
    fn parse_collection(&mut self, patterns: &[RawRule], repository: &RawRepository) -> RuleId {
        let id = self.reserve();
        let (repository, patterns) = if repository.0.is_empty() {
            (None, self.parse_list(patterns))
        } else {
            self.repo_stack.push(id);
            let repository = self.parse_repository(repository);
            let patterns = self.parse_list(patterns);
            self.repo_stack.pop();
            (Some(repository), patterns)
        };
        self.rules[id.index()] = Rule::Collection {
            patterns,
            repository,
        };
        id
    }

    fn parse_captures(&mut self, raw: &RawCaptures) -> Captures {
        let Some(max_group) = raw.0.iter().map(|(group, _)| *group).max() else {
            return Captures::default();
        };
        let mut captures = vec![None; max_group as usize + 1];
        for (group, rule) in &raw.0 {
            let patterns = (!rule.patterns.is_empty() || !rule.repository.0.is_empty())
                .then(|| self.parse_collection(&rule.patterns, &rule.repository));
            if rule.name.is_none() && patterns.is_none() {
                continue;
            }
            captures[*group as usize] = Some(Capture {
                name: rule.name.clone(),
                patterns,
            });
        }
        Captures(captures)
    }
}

/// `beginCaptures`, `endCaptures` and `whileCaptures` fall back to `captures` when
/// absent, all or nothing per side.
fn pick<'a>(specific: &'a RawCaptures, shared: &'a RawCaptures) -> &'a RawCaptures {
    if specific.0.is_empty() {
        shared
    } else {
        specific
    }
}

fn has_scope_backref(name: &Option<String>) -> bool {
    name.as_deref().is_some_and(|name| name.contains('$'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grammar(json: &str) -> Grammar {
        Grammar::parse(json.as_bytes()).expect("test grammar parses")
    }

    fn with_patterns(patterns: &str) -> Grammar {
        grammar(&format!(
            r#"{{"scopeName": "source.t", "patterns": {patterns}}}"#
        ))
    }

    fn first_rule(patterns: &str) -> (Grammar, RuleId) {
        let g = with_patterns(patterns);
        let id = g.root_patterns()[0];
        (g, id)
    }

    fn names(captures: &Captures) -> Vec<Option<&str>> {
        captures
            .0
            .iter()
            .map(|c| c.as_ref().and_then(|c| c.name.as_deref()))
            .collect()
    }

    #[test]
    fn errors_on_invalid_json_and_missing_scope_name() {
        assert!(matches!(Grammar::parse(b"{"), Err(Error::GrammarParse(_))));
        assert!(matches!(
            Grammar::parse(b"{\"patterns\": []}"),
            Err(Error::GrammarParse(_))
        ));
        assert!(matches!(
            Grammar::parse(b"{\"scopeName\": \"\"}"),
            Err(Error::GrammarParse(_))
        ));
    }

    #[test]
    fn ignores_unknown_fields_and_keeps_metadata() {
        let g = grammar(
            r#"{"$schema": "x", "version": 1, "displayName": "T", "information_for_contributors": [],
                "scopeName": "source.t", "name": "t", "injectTo": ["source.a"], "patterns": []}"#,
        );
        assert_eq!(g.scope_name, "source.t");
        assert_eq!(g.name.as_deref(), Some("t"));
        assert_eq!(g.inject_to, ["source.a"]);
        assert!(
            matches!(g.rule(ROOT_RULE_ID), Rule::Collection { patterns, repository: None } if patterns.is_empty())
        );
    }

    #[test]
    fn root_is_zero_and_parents_precede_children() {
        let g = with_patterns(
            r#"[{"begin": "b", "end": "e", "patterns": [{"match": "m", "captures": {"1": {"patterns": [{"match": "n"}]}}}]}]"#,
        );
        let begin = g.root_patterns()[0];
        assert!(begin.0 > ROOT_RULE_ID.0);
        let Rule::BeginEnd { patterns, .. } = g.rule(begin) else {
            panic!()
        };
        assert!(patterns[0].0 > begin.0);
        let Rule::Match { captures, .. } = g.rule(patterns[0]) else {
            panic!()
        };
        let nested = captures.get(1).unwrap().patterns.unwrap();
        assert!(nested.0 > patterns[0].0);
        assert!(matches!(g.rule(nested), Rule::Collection { .. }));
    }

    #[test]
    fn captures_accept_arrays_strings_and_skip_bad_keys() {
        let (g, id) = first_rule(
            r#"[{"match": "m", "captures": ["zero", {"name": "one"}, 7, {"patterns": []}]}]"#,
        );
        let Rule::Match { captures, .. } = g.rule(id) else {
            panic!()
        };
        assert_eq!(names(captures), [Some("zero"), Some("one"), None, None]);

        let (g, id) = first_rule(
            r#"[{"match": "m", "captures": {"2": "two", "end": {"name": "bad"}, "0": {"name": "zero"}, "x1": "no"}}]"#,
        );
        let Rule::Match { captures, .. } = g.rule(id) else {
            panic!()
        };
        assert_eq!(names(captures), [Some("zero"), None, Some("two")]);

        let (g, id) = first_rule(r#"[{"match": "m", "captures": "junk"}]"#);
        let Rule::Match { captures, .. } = g.rule(id) else {
            panic!()
        };
        assert!(captures.is_empty());
    }

    #[test]
    fn apply_end_pattern_last_accepts_bool_or_int() {
        for (value, want) in [
            ("true", true),
            ("1", true),
            ("0", false),
            ("false", false),
            ("\"yes\"", false),
        ] {
            let (g, id) = first_rule(&format!(
                r#"[{{"begin": "b", "end": "e", "applyEndPatternLast": {value}}}]"#
            ));
            let Rule::BeginEnd {
                apply_end_pattern_last,
                ..
            } = g.rule(id)
            else {
                panic!()
            };
            assert_eq!(*apply_end_pattern_last, want, "value {value}");
        }
    }

    #[test]
    fn repository_values_may_be_arrays_and_drop_empty_rules() {
        let g = grammar(
            r##"{"scopeName": "source.t", "patterns": [{"include": "#list"}, {}, {"name": "only"}],
                "repository": {"list": [{"match": "a"}, {}, {"match": "b"}], "junk": 3, "empty": [{}]}}"##,
        );
        assert_eq!(g.root_patterns().len(), 1);
        let list = g.repository.get("list").unwrap();
        let Rule::Collection {
            patterns,
            repository: None,
        } = g.rule(list)
        else {
            panic!()
        };
        assert_eq!(patterns.len(), 2);
        assert!(g.repository.get("junk").is_none());
        let empty = g.repository.get("empty").unwrap();
        assert!(matches!(g.rule(empty), Rule::Collection { patterns, .. } if patterns.is_empty()));
    }

    #[test]
    fn empty_match_and_empty_include_are_dropped() {
        let g = with_patterns(r#"[{"match": "", "name": "x"}, {"include": ""}, {"match": "ok"}]"#);
        assert_eq!(g.root_patterns().len(), 1);
    }

    #[test]
    fn include_wins_in_pattern_lists_but_not_on_repository_entries() {
        let (g, id) = first_rule(
            r##"[{"include": "#a", "match": "x", "begin": "y", "patterns": [{"match": "z"}]}]"##,
        );
        assert!(matches!(g.rule(id), Rule::Include(Include::Local(key)) if key == "a"));

        let g = grammar(
            r##"{"scopeName": "source.t", "patterns": [], "repository": {
                "p": {"include": "#a", "patterns": [{"match": "z"}]},
                "m": {"include": "#a", "match": "x"},
                "i": {"include": "#a"}
            }}"##,
        );
        assert!(matches!(
            g.rule(g.repository.get("p").unwrap()),
            Rule::Collection { .. }
        ));
        assert!(matches!(
            g.rule(g.repository.get("m").unwrap()),
            Rule::Match { .. }
        ));
        assert!(matches!(
            g.rule(g.repository.get("i").unwrap()),
            Rule::Include(_)
        ));
    }

    #[test]
    fn parses_include_targets() {
        assert_eq!(Include::parse("$self"), Include::SelfRef);
        assert_eq!(Include::parse("$base"), Include::Base);
        assert_eq!(
            Include::parse("#key.dotted"),
            Include::Local("key.dotted".into())
        );
        assert_eq!(
            Include::parse("source.js"),
            Include::Scope("source.js".into())
        );
        assert_eq!(
            Include::parse("source.js#a#b"),
            Include::ScopeKey("source.js".into(), "a#b".into())
        );
    }

    #[test]
    fn begin_end_defaults_and_capture_fallbacks() {
        let (g, id) = first_rule(
            r#"[{"begin": "b", "captures": {"0": {"name": "shared"}}, "endCaptures": {"1": {"name": "end-only"}}}]"#,
        );
        let Rule::BeginEnd {
            end,
            begin_captures,
            end_captures,
            ..
        } = g.rule(id)
        else {
            panic!()
        };
        assert_eq!(end.source(), "\u{FFFF}");
        assert_eq!(names(begin_captures), [Some("shared")]);
        assert_eq!(names(end_captures), [None, Some("end-only")]);

        let (g, id) = first_rule(r#"[{"begin": "b", "while": "w", "captures": {"0": "shared"}}]"#);
        let Rule::BeginWhile {
            begin_captures,
            while_captures,
            ..
        } = g.rule(id)
        else {
            panic!()
        };
        assert_eq!(names(begin_captures), [Some("shared")]);
        assert_eq!(names(while_captures), [Some("shared")]);
    }

    #[test]
    fn empty_begin_is_a_zero_width_begin_rule() {
        let g = with_patterns(
            r#"[{"begin": "", "end": "e", "patterns": [{"match": "m"}]}, {"begin": "", "end": "e"}]"#,
        );
        assert_eq!(g.root_patterns().len(), 2);
        for &id in g.root_patterns() {
            assert!(matches!(
                g.rule(id),
                Rule::BeginEnd { begin, .. } if begin.source().is_empty()
            ));
        }
    }

    #[test]
    fn flags_backref_needs() {
        let g = with_patterns(
            r#"[{"begin": "a", "end": "\\1"}, {"begin": "a", "end": "z", "name": "x.$1"},
                {"begin": "a", "while": "w", "contentName": "${1:/downcase}"}, {"begin": "a", "end": "z"},
                {"begin": "a", "while": "\\2"}]"#,
        );
        let flags: Vec<bool> = g
            .root_patterns()
            .iter()
            .map(|&id| match g.rule(id) {
                Rule::BeginEnd {
                    needs_begin_capture_texts,
                    ..
                }
                | Rule::BeginWhile {
                    needs_begin_capture_texts,
                    ..
                } => *needs_begin_capture_texts,
                _ => panic!(),
            })
            .collect();
        assert_eq!(flags, [true, true, true, false, true]);
        assert!(matches!(
            g.rule(g.root_patterns()[0]),
            Rule::BeginEnd {
                end_has_backrefs: true,
                ..
            }
        ));
        assert!(matches!(
            g.rule(g.root_patterns()[4]),
            Rule::BeginWhile {
                while_has_backrefs: true,
                ..
            }
        ));
    }

    #[test]
    fn injections_keep_source_order_and_skip_empty_rules() {
        let g = grammar(
            r#"{"scopeName": "source.t", "patterns": [], "injections": {
                "R:b": {"patterns": [{"match": "b"}]}, "L:a": {"patterns": [{"match": "a"}]}, "x": {}
            }, "injectionSelector": "L:text.html"}"#,
        );
        let selectors: Vec<&str> = g
            .injections
            .iter()
            .map(|i| i.raw_selector.as_str())
            .collect();
        assert_eq!(selectors, ["R:b", "L:a"]);
        assert_eq!(
            g.injections[0].selector.composites[0].priority,
            Priority::Right
        );
        assert!(g.injection_selector.is_some());
    }
}
