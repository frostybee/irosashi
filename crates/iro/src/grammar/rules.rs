use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::grammar::Selector;
use crate::regex::LazyRegex;

/// Index into a grammar's rule arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuleId(pub u32);

impl RuleId {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// The grammar's top-level pattern list.
pub const ROOT_RULE_ID: RuleId = RuleId(0);

/// The target of an `include` directive, resolved when a rule context is compiled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Include {
    /// `$self`: the current grammar's top-level patterns.
    SelfRef,
    /// `$base`: the top-level patterns of the grammar tokenization started with.
    Base,
    /// `#key`: a repository entry of the current grammar.
    Local(String),
    /// `source.x`: another grammar's top-level patterns.
    Scope(String),
    /// `source.x#key`: a repository entry of another grammar.
    ScopeKey(String, String),
}

impl Include {
    pub fn parse(include: &str) -> Self {
        if include == "$self" {
            Self::SelfRef
        } else if include == "$base" {
            Self::Base
        } else if let Some(key) = include.strip_prefix('#') {
            Self::Local(key.to_owned())
        } else if let Some((scope, key)) = include.split_once('#') {
            Self::ScopeKey(scope.to_owned(), key.to_owned())
        } else {
            Self::Scope(include.to_owned())
        }
    }
}

/// What a capture group contributes: a scope name, nested patterns to retokenize the
/// captured text with (a `Collection` rule), or both.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capture {
    pub name: Option<String>,
    pub patterns: Option<RuleId>,
}

/// Captures indexed by group number; `None` where the grammar assigns nothing.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Captures(pub Vec<Option<Capture>>);

impl Captures {
    pub fn get(&self, group: usize) -> Option<&Capture> {
        self.0.get(group).and_then(Option::as_ref)
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().all(Option::is_none)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Repository(pub HashMap<String, RuleId>);

impl Repository {
    pub fn get(&self, key: &str) -> Option<RuleId> {
        self.0.get(key).copied()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rule {
    Match {
        name: Option<String>,
        regex: LazyRegex,
        captures: Captures,
    },
    BeginEnd {
        name: Option<String>,
        content_name: Option<String>,
        begin: LazyRegex,
        end: LazyRegex,
        end_has_backrefs: bool,
        begin_captures: Captures,
        end_captures: Captures,
        patterns: Vec<RuleId>,
        apply_end_pattern_last: bool,
        needs_begin_capture_texts: bool,
    },
    BeginWhile {
        name: Option<String>,
        content_name: Option<String>,
        begin: LazyRegex,
        while_: LazyRegex,
        while_has_backrefs: bool,
        begin_captures: Captures,
        while_captures: Captures,
        patterns: Vec<RuleId>,
        needs_begin_capture_texts: bool,
    },
    Include(Include),
    Collection {
        patterns: Vec<RuleId>,
        repository: Option<Repository>,
    },
    Noop,
}

impl Rule {
    /// Child patterns forming this rule's inner scanner context.
    pub fn patterns(&self) -> &[RuleId] {
        match self {
            Self::BeginEnd { patterns, .. }
            | Self::BeginWhile { patterns, .. }
            | Self::Collection { patterns, .. } => patterns,
            _ => &[],
        }
    }

    /// The pattern this rule contributes to its parent's scanner.
    pub fn scanner_pattern(&self) -> Option<&str> {
        match self {
            Self::Match { regex, .. } => Some(regex.source()),
            Self::BeginEnd { begin, .. } | Self::BeginWhile { begin, .. } => Some(begin.source()),
            _ => None,
        }
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Match { name, .. }
            | Self::BeginEnd { name, .. }
            | Self::BeginWhile { name, .. } => name.as_deref(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Injection {
    pub raw_selector: String,
    pub selector: Selector,
    pub rule: RuleId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grammar {
    pub scope_name: String,
    pub name: Option<String>,
    /// `rules[0]` is the root `Collection` holding the top-level patterns.
    pub rules: Vec<Rule>,
    pub repository: Repository,
    /// In source order; vscode-textmate stable-sorts injections by priority.
    pub injections: Vec<Injection>,
    pub injection_selector: Option<Selector>,
    pub inject_to: Vec<String>,
}

impl Grammar {
    pub fn rule(&self, id: RuleId) -> &Rule {
        &self.rules[id.index()]
    }

    pub fn root_patterns(&self) -> &[RuleId] {
        self.rule(ROOT_RULE_ID).patterns()
    }
}
