use serde::{Deserialize, Serialize};

/// How an injection that matches at the same position as a grammar rule is ranked.
/// Ordered so that `Left < None < Right` mirrors vscode-textmate's `-1, 0, 1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Priority {
    /// `L:` the injection wins ties.
    Left,
    None,
    /// `R:` the grammar wins ties.
    Right,
}

/// A parsed injection selector.
///
/// Grammar: `selector = composite (',' composite)*`,
/// `composite = ('L:' | 'R:')? expression+`, `expression = '-'? (group | scopePath)`,
/// `group = '(' composite ('|' composite)* ')'`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selector {
    pub composites: Vec<Composite>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Composite {
    pub priority: Priority,
    pub expressions: Vec<Expression>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Expression {
    pub negate: bool,
    pub term: Term,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Term {
    Group(Vec<Composite>),
    Path(String),
}

impl Selector {
    /// Lenient and infallible: unterminated groups are tolerated, unknown characters
    /// are skipped.
    pub fn parse(input: &str) -> Self {
        Parser { input, pos: 0 }.parse_selector()
    }

    /// The priority of the first composite matching `scopes` (outermost first).
    pub fn matches(&self, scopes: &[&str]) -> Option<Priority> {
        self.composites
            .iter()
            .find(|composite| composite.matches(scopes))
            .map(|composite| composite.priority)
    }
}

impl Composite {
    fn matches(&self, scopes: &[&str]) -> bool {
        if self.expressions.is_empty() {
            return false;
        }
        let (negative, positive): (Vec<_>, Vec<_>) =
            self.expressions.iter().partition(|e| e.negate);
        if negative.iter().any(|e| e.matches_stack(scopes)) {
            return false;
        }
        if positive.is_empty() {
            return true;
        }
        let mut remaining = positive.len();
        for scope in scopes.iter().rev() {
            if positive[remaining - 1].matches_single(scope) {
                remaining -= 1;
                if remaining == 0 {
                    return true;
                }
            }
        }
        false
    }

    /// All positive paths in order as a subsequence of the stack, outermost first.
    fn matches_as_subsequence(&self, scopes: &[&str]) -> bool {
        let paths: Vec<&str> = self
            .expressions
            .iter()
            .filter_map(|e| match (&e.term, e.negate) {
                (Term::Path(path), false) => Some(path.as_str()),
                _ => None,
            })
            .collect();
        if paths.is_empty() {
            return true;
        }
        let mut next = 0;
        for scope in scopes {
            if scope_matches(paths[next], scope) {
                next += 1;
                if next == paths.len() {
                    return true;
                }
            }
        }
        false
    }
}

impl Expression {
    fn matches_single(&self, scope: &str) -> bool {
        match &self.term {
            Term::Path(path) => scope_matches(path, scope),
            Term::Group(alternatives) => alternatives.iter().any(|alt| {
                alt.expressions
                    .iter()
                    .any(|e| !e.negate && e.matches_single(scope))
            }),
        }
    }

    fn matches_stack(&self, scopes: &[&str]) -> bool {
        match &self.term {
            Term::Path(path) => scopes.iter().any(|scope| scope_matches(path, scope)),
            Term::Group(alternatives) => alternatives
                .iter()
                .any(|alt| alt.matches_as_subsequence(scopes)),
        }
    }
}

fn is_path_start(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'.' | b':' | b'_')
}

/// `selector` is a prefix of `scope` ending at a dot boundary.
fn scope_matches(selector: &str, scope: &str) -> bool {
    match scope.strip_prefix(selector) {
        Some(rest) => rest.is_empty() || rest.starts_with('.'),
        None => false,
    }
}

struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl Parser<'_> {
    fn parse_selector(&mut self) -> Selector {
        let mut composites = vec![self.parse_composite()];
        while self.peek() == Some(b',') {
            self.advance();
            composites.push(self.parse_composite());
        }
        Selector { composites }
    }

    fn parse_composite(&mut self) -> Composite {
        self.skip_spaces();
        let priority = if self.rest().starts_with("L:") {
            self.pos += 2;
            Priority::Left
        } else if self.rest().starts_with("R:") {
            self.pos += 2;
            Priority::Right
        } else {
            Priority::None
        };
        let mut expressions = Vec::new();
        while let Some(c) = self.peek()
            && !matches!(c, b',' | b')' | b'|')
        {
            if c == b'-' || c == b'(' || is_path_start(c) {
                expressions.push(self.parse_expression());
            } else {
                self.advance();
            }
        }
        Composite {
            priority,
            expressions,
        }
    }

    fn parse_expression(&mut self) -> Expression {
        let mut negate = false;
        if self.peek() == Some(b'-') {
            negate = true;
            self.advance();
        }
        let term = if self.peek() == Some(b'(') {
            Term::Group(self.parse_group())
        } else {
            let path = self.parse_scope_path();
            if path.is_empty() {
                self.advance();
            }
            Term::Path(path)
        };
        Expression { negate, term }
    }

    fn parse_group(&mut self) -> Vec<Composite> {
        self.advance();
        let mut alternatives = vec![self.parse_composite()];
        while matches!(self.peek(), Some(b'|' | b',')) {
            self.advance();
            alternatives.push(self.parse_composite());
        }
        if self.peek() == Some(b')') {
            self.advance();
        }
        alternatives
    }

    /// A path token as vscode-textmate's selector tokenizer reads it: `[\w.:]` then
    /// `[\w.:-]*`. Any other character between tokens is skipped, so
    /// `source.ts#meta.decorator.ts` reads as two paths.
    fn parse_scope_path(&mut self) -> String {
        let start = self.pos;
        let mut bytes = self.rest().bytes();
        let len = match bytes.next() {
            Some(first) if is_path_start(first) => {
                1 + bytes.take_while(|&b| is_path_start(b) || b == b'-').count()
            }
            _ => 0,
        };
        self.pos += len;
        self.input[start..self.pos].to_owned()
    }

    fn rest(&self) -> &str {
        &self.input[self.pos..]
    }

    fn peek(&mut self) -> Option<u8> {
        self.skip_spaces();
        self.rest().bytes().next()
    }

    fn advance(&mut self) {
        if let Some(c) = self.rest().chars().next() {
            self.pos += c.len_utf8();
        }
    }

    fn skip_spaces(&mut self) {
        let spaces = self.rest().len() - self.rest().trim_start().len();
        self.pos += spaces;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path(expression: &Expression) -> &str {
        match &expression.term {
            Term::Path(path) => path,
            Term::Group(_) => panic!("expected a path"),
        }
    }

    #[test]
    fn parses_priority_prefixes() {
        let selector = Selector::parse("L:source.js");
        assert_eq!(selector.composites.len(), 1);
        assert_eq!(selector.composites[0].priority, Priority::Left);
        assert_eq!(selector.composites[0].expressions.len(), 1);
        assert_eq!(path(&selector.composites[0].expressions[0]), "source.js");
        assert_eq!(
            Selector::parse("R:source.python").composites[0].priority,
            Priority::Right
        );
        assert_eq!(
            Selector::parse("source.python").composites[0].priority,
            Priority::None
        );
    }

    #[test]
    fn skips_characters_outside_the_token_set() {
        let selector = Selector::parse("L:source.ts#meta.decorator.ts -comment");
        let expressions = &selector.composites[0].expressions;
        assert_eq!(expressions.len(), 3);
        assert_eq!(path(&expressions[0]), "source.ts");
        assert_eq!(path(&expressions[1]), "meta.decorator.ts");
        assert!(expressions[2].negate);
        assert_eq!(
            selector.matches(&["source.ts.ng", "meta.decorator.ts", "meta.objectliteral.ts"]),
            Some(Priority::Left)
        );
        assert!(
            selector
                .matches(&["source.ts.ng", "comment.line"])
                .is_none()
        );
        let dashed = Selector::parse("meta.tag-name -a_b.c");
        assert_eq!(path(&dashed.composites[0].expressions[0]), "meta.tag-name");
        assert_eq!(path(&dashed.composites[0].expressions[1]), "a_b.c");
    }

    #[test]
    fn parses_negation_commas_and_groups() {
        let selector = Selector::parse("L:text.html -comment.block");
        let expressions = &selector.composites[0].expressions;
        assert_eq!(expressions.len(), 2);
        assert!(!expressions[0].negate);
        assert!(expressions[1].negate);
        assert_eq!(path(&expressions[1]), "comment.block");

        assert_eq!(
            Selector::parse("L:source.ts, L:source.js, L:source.coffee")
                .composites
                .len(),
            3
        );

        let selector = Selector::parse(
            "L:(meta.script.svelte | meta.style.svelte) (meta.lang.ts | meta.lang.typescript) - (meta source)",
        );
        let expressions = &selector.composites[0].expressions;
        assert_eq!(expressions.len(), 3);
        assert!(matches!(&expressions[0].term, Term::Group(alts) if alts.len() == 2));
        assert!(expressions[2].negate);
    }

    #[test]
    fn tolerates_garbage_without_hanging() {
        let selector = Selector::parse("!!! ((( L: , | ) source.js");
        assert!(!selector.composites.is_empty());
        assert_eq!(Selector::parse("").composites[0].expressions.len(), 0);
        assert_eq!(Selector::parse("").matches(&["source.js"]), None);
        Selector::parse("é.ü -(x");
    }

    #[test]
    fn matches_basic_prefix_rule() {
        let selector = Selector::parse("L:source.js");
        assert_eq!(selector.matches(&["source.js"]), Some(Priority::Left));
        assert_eq!(
            selector.matches(&["source.js", "meta.function"]),
            Some(Priority::Left)
        );
        assert_eq!(selector.matches(&["source.python"]), None);
        assert_eq!(
            selector.matches(&["text.html", "source.js.embedded"]),
            Some(Priority::Left)
        );
        assert_eq!(selector.matches(&["source.jsx"]), None);
        assert_eq!(
            Selector::parse("R:a").matches(&["a"]),
            Some(Priority::Right)
        );
        assert_eq!(Selector::parse("a").matches(&["a"]), Some(Priority::None));
        assert_eq!(
            Selector::parse("x, R:a").matches(&["a"]),
            Some(Priority::Right)
        );
    }

    #[test]
    fn matches_negation_over_whole_stack() {
        let selector = Selector::parse("L:text.html -comment.block");
        assert!(selector.matches(&["text.html", "meta.tag"]).is_some());
        assert!(selector.matches(&["text.html", "comment.block"]).is_none());
        assert!(
            selector
                .matches(&["text.html.basic", "comment.block.html"])
                .is_none()
        );
        assert!(selector.matches(&["text.html"]).is_some());
    }

    #[test]
    fn matches_groups_and_ordered_subsequences() {
        let selector = Selector::parse("L:(source.ts | source.js)");
        assert!(selector.matches(&["source.ts"]).is_some());
        assert!(selector.matches(&["source.js"]).is_some());
        assert!(selector.matches(&["source.python"]).is_none());

        let selector = Selector::parse("L:meta.script.svelte - meta.lang - (meta source)");
        assert!(
            selector
                .matches(&["source.svelte", "meta.script.svelte"])
                .is_some()
        );
        assert!(
            selector
                .matches(&[
                    "source.svelte",
                    "meta.script.svelte",
                    "meta.embedded.block.svelte",
                    "source.js"
                ])
                .is_none()
        );
        assert!(
            selector
                .matches(&["source.svelte", "meta.script.svelte", "meta.lang.ts"])
                .is_none()
        );

        let selector = Selector::parse("a b");
        assert!(selector.matches(&["a", "x", "b"]).is_some());
        assert!(selector.matches(&["b", "a"]).is_none());
        assert!(selector.matches(&["a"]).is_none());
    }

    #[test]
    fn priority_ordering_mirrors_numeric_values() {
        assert!(Priority::Left < Priority::None);
        assert!(Priority::None < Priority::Right);
    }
}
