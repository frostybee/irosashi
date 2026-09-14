mod captures;
mod injections;
mod line;
mod memo;
mod state;

use std::collections::HashMap;
use std::mem;
use std::ops::Range;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use crate::grammar::{Grammar, GrammarResolver, Priority, ROOT_RULE_ID};
use crate::scope::{ScopeInterner, ScopeListId};
use crate::theme::{Theme, ThemeId};
use crate::token::{
    Diagnostic, DiagnosticKind, ThemedLine, ThemedToken, Token, TokenStyle, TokensResult,
};
use crate::tokenizer::injections::InjectionEntry;
use crate::tokenizer::line::{LineCtx, TokenBuilder, run_line};
use crate::tokenizer::memo::Memo;
use crate::tokenizer::state::{RuleRef, StackFrame, WorkStack};

pub use injections::InjectionProvider;
pub use state::StateStack;

/// Everything a session needs from its registry.
pub trait Resolver: GrammarResolver + InjectionProvider {}

impl<T: GrammarResolver + InjectionProvider> Resolver for T {}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TokenizeOptions {
    /// Lines longer than this many bytes are emitted as one unstyled token with a
    /// `TooLong` diagnostic. `None` disables the guard.
    pub max_line_length: Option<usize>,
}

/// Result of tokenizing one line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineResult {
    pub tokens: Vec<Token>,
    pub state: StateStack,
    pub diagnostic: Option<DiagnosticKind>,
}

/// Result of tokenizing a whole buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenizeResult {
    pub lines: Vec<Vec<Token>>,
    pub diagnostics: Vec<Diagnostic>,
}

struct Styler {
    theme: Arc<Theme>,
    by_list: Vec<Option<TokenStyle>>,
}

/// A tokenizer for one grammar. Owns the scope interner, the compiled scanner cache
/// and the style caches, so it is used from one thread at a time.
pub struct Session {
    grammar: Arc<Grammar>,
    resolver: Arc<dyn Resolver>,
    options: TokenizeOptions,
    interner: ScopeInterner,
    memo: Memo,
    injections: Vec<InjectionEntry>,
    injection_hits: HashMap<ScopeListId, Vec<(usize, Priority)>>,
    scan_bufs: Vec<String>,
    styles: HashMap<ThemeId, Styler>,
    initial: StateStack,
}

impl Session {
    pub fn new(
        grammar: Arc<Grammar>,
        resolver: Arc<dyn Resolver>,
        options: TokenizeOptions,
    ) -> Self {
        let mut interner = ScopeInterner::new();
        let root_scopes = interner.push_names(ScopeListId::EMPTY, &grammar.scope_name);
        let root = WorkStack::root(StackFrame {
            rule: RuleRef::new(Arc::clone(&grammar), ROOT_RULE_ID),
            scopes_after_name: root_scopes,
            scopes_after_content: root_scopes,
            end_override: None,
            while_pattern: None,
            begin_captured_eol: false,
        });
        let injections = injections::collect_injections(&grammar, resolver.as_ref());
        Self {
            grammar,
            resolver,
            options,
            interner,
            memo: Memo::default(),
            injections,
            injection_hits: HashMap::new(),
            scan_bufs: vec![String::new()],
            styles: HashMap::new(),
            initial: root.snapshot(),
        }
    }

    pub fn grammar(&self) -> &Arc<Grammar> {
        &self.grammar
    }

    /// The state before the first line.
    pub fn initial_state(&self) -> StateStack {
        self.initial.clone()
    }

    pub fn scope_names(&self, scopes: ScopeListId) -> Vec<&str> {
        self.interner.names(scopes)
    }

    pub fn scopes_vec(&self, scopes: ScopeListId) -> Vec<String> {
        self.interner.names_owned(scopes)
    }

    /// Tokenizes one bare line (no terminator) starting from `state`.
    pub fn tokenize_line(
        &mut self,
        line: &str,
        state: &StateStack,
        is_first_line: bool,
    ) -> LineResult {
        if let Some(max) = self.options.max_line_length
            && line.len() > max
        {
            return LineResult {
                tokens: unstyled(line, state),
                state: state.clone(),
                diagnostic: Some(DiagnosticKind::TooLong),
            };
        }

        let mut buf = mem::take(&mut self.scan_bufs[0]);
        buf.clear();
        buf.push_str(line);
        buf.push('\n');

        let mut stack = WorkStack::from_snapshot(state);
        let mut out = TokenBuilder::new(0);
        let Session {
            grammar,
            resolver,
            interner,
            memo,
            injections,
            injection_hits,
            scan_bufs,
            ..
        } = self;
        let mut cx = LineCtx {
            memo,
            interner,
            injections,
            injection_hits,
            scan_bufs,
            resolver: resolver.as_ref(),
            base: grammar,
        };
        let result = catch_unwind(AssertUnwindSafe(|| {
            run_line(
                &mut cx,
                &buf,
                line.len(),
                &mut stack,
                &mut out,
                0,
                is_first_line,
                0,
                true,
            )
        }));
        self.scan_bufs[0] = buf;

        match result {
            Ok(Ok(())) => LineResult {
                tokens: out.finish_clamped(line.len()),
                state: stack.snapshot(),
                diagnostic: None,
            },
            Ok(Err(_)) => LineResult {
                tokens: unstyled(line, state),
                state: state.clone(),
                diagnostic: Some(DiagnosticKind::Regex),
            },
            Err(_) => LineResult {
                tokens: unstyled(line, state),
                state: state.clone(),
                diagnostic: Some(DiagnosticKind::Panic),
            },
        }
    }

    /// Tokenizes a whole buffer line by line.
    pub fn tokenize(&mut self, code: &str) -> TokenizeResult {
        let mut result = TokenizeResult {
            lines: Vec::new(),
            diagnostics: Vec::new(),
        };
        let mut state = self.initial_state();
        for (index, range) in split_lines(code).into_iter().enumerate() {
            let line = self.tokenize_line(&code[range], &state, index == 0);
            if let Some(kind) = line.diagnostic {
                result.diagnostics.push(Diagnostic { line: index, kind });
            }
            result.lines.push(line.tokens);
            state = line.state;
        }
        result
    }

    /// Tokenizes and resolves every token's style against `theme`.
    pub fn themed(&mut self, code: &str, theme: &Arc<Theme>) -> TokensResult {
        let raw = self.tokenize(code);
        let ranges = split_lines(code);
        let lines = raw
            .lines
            .into_iter()
            .zip(ranges)
            .map(|(tokens, range)| ThemedLine {
                tokens: tokens
                    .into_iter()
                    .filter(|t| t.start < t.end)
                    .map(|t| ThemedToken {
                        start: t.start,
                        end: t.end,
                        style: self.style(theme, t.scopes),
                        scopes: t.scopes,
                    })
                    .collect(),
                range,
            })
            .collect();
        TokensResult {
            source: code.to_owned(),
            lines,
            theme: Arc::clone(theme),
            diagnostics: raw.diagnostics,
        }
    }

    /// Resolves the style of a scope stack against `theme`, cached per stack.
    pub fn style(&mut self, theme: &Arc<Theme>, scopes: ScopeListId) -> TokenStyle {
        let interner = &self.interner;
        let styler = self.styles.entry(theme.id()).or_insert_with(|| Styler {
            theme: Arc::clone(theme),
            by_list: Vec::new(),
        });
        let index = scopes.index();
        if styler.by_list.len() <= index {
            styler
                .by_list
                .resize(interner.list_count().max(index + 1), None);
        }
        if let Some(style) = styler.by_list[index] {
            return style;
        }
        let names = interner.names(scopes);
        let settings = styler.theme.resolve(&names);
        let style = TokenStyle {
            color: Some(
                settings
                    .foreground
                    .unwrap_or(styler.theme.default_foreground_id()),
            ),
            bg: settings.background,
            font_style: settings.font_style.unwrap_or_default(),
        };
        styler.by_list[index] = Some(style);
        style
    }
}

fn unstyled(line: &str, state: &StateStack) -> Vec<Token> {
    if line.is_empty() {
        return Vec::new();
    }
    vec![Token {
        start: 0,
        end: line.len(),
        scopes: state.scopes(),
    }]
}

/// Byte ranges of the bare lines of `code`: `\n` and a directly preceding `\r` are
/// stripped, a trailing newline does not yield a final empty line, and empty input
/// has no lines.
pub fn split_lines(code: &str) -> Vec<Range<usize>> {
    let bytes = code.as_bytes();
    let mut lines = Vec::new();
    let mut start = 0;
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'\n' {
            let mut end = i;
            if end > start && bytes[end - 1] == b'\r' {
                end -= 1;
            }
            lines.push(start..end);
            start = i + 1;
        }
    }
    if start < bytes.len() {
        lines.push(start..bytes.len());
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::single_range_in_vec_init)]
    fn split_lines_strips_terminators_and_trailing_newline() {
        assert!(split_lines("").is_empty());
        assert_eq!(split_lines("a"), [0..1]);
        assert_eq!(split_lines("a\n"), [0..1]);
        assert_eq!(split_lines("a\r\nb\n\nc"), [0..1, 3..4, 5..5, 6..7]);
        assert_eq!(split_lines("\n"), [0..0]);
    }

    fn session(json: &str) -> Session {
        let grammar = Arc::new(Grammar::parse(json.as_bytes()).unwrap());
        Session::new(grammar, Arc::new(()), TokenizeOptions::default())
    }

    #[test]
    fn crlf_and_lf_give_identical_tokens() {
        let mut s =
            session(r#"{"scopeName": "source.t", "patterns": [{"match": "\\w+", "name": "w"}]}"#);
        let lf = s.tokenize("ab cd\nef");
        let crlf = s.tokenize("ab cd\r\nef");
        assert_eq!(lf, crlf);
        assert_eq!(lf.lines.len(), 2);
    }

    #[test]
    fn empty_input_has_no_lines_and_empty_line_has_no_tokens() {
        let mut s =
            session(r#"{"scopeName": "source.t", "patterns": [{"match": "\\w+", "name": "w"}]}"#);
        assert!(s.tokenize("").lines.is_empty());
        let r = s.tokenize("a\n\nb");
        assert_eq!(r.lines.len(), 3);
        assert!(r.lines[1].is_empty());
    }

    #[test]
    fn max_line_length_emits_one_unstyled_token_and_keeps_state() {
        let grammar = Arc::new(
            Grammar::parse(
                br#"{"scopeName": "source.t", "patterns": [{"begin": "\\(", "end": "\\)", "name": "paren", "patterns": [{"match": "x", "name": "x"}]}]}"#,
            )
            .unwrap(),
        );
        let mut s = Session::new(
            grammar,
            Arc::new(()),
            TokenizeOptions {
                max_line_length: Some(4),
            },
        );
        let r = s.tokenize("(x\nxxxxxxxx\nx)");
        assert_eq!(
            r.diagnostics,
            [Diagnostic {
                line: 1,
                kind: DiagnosticKind::TooLong
            }]
        );
        assert_eq!(r.lines[1].len(), 1);
        assert_eq!(s.scope_names(r.lines[1][0].scopes), ["source.t", "paren"]);
        assert_eq!(
            s.scope_names(r.lines[2][0].scopes),
            ["source.t", "paren", "x"]
        );
    }

    #[test]
    fn regex_failure_in_a_context_is_a_line_diagnostic() {
        let mut s = session(
            r#"{"scopeName": "source.t", "patterns": [{"begin": "\\(", "end": "\\)", "name": "q", "patterns": [{"match": "[", "name": "bad"}]}]}"#,
        );
        let r = s.tokenize("(x)\ny");
        assert_eq!(r.lines[0].len(), 1);
        assert_eq!(s.scope_names(r.lines[0][0].scopes), ["source.t"]);
        assert_eq!(
            r.diagnostics,
            [Diagnostic {
                line: 0,
                kind: DiagnosticKind::Regex
            }]
        );
        assert_eq!(r.lines[1].len(), 1);
    }

    #[test]
    fn state_snapshots_compare_equal_across_lines() {
        let mut s = session(
            r#"{"scopeName": "source.t", "patterns": [{"begin": "\\(", "end": "\\)", "name": "paren", "patterns": [{"match": "x", "name": "x"}]}]}"#,
        );
        let initial = s.initial_state();
        let a = s.tokenize_line("(x", &initial, true);
        let b = s.tokenize_line("x x", &a.state, false);
        assert_eq!(a.state, b.state);
        assert_ne!(a.state, initial);
        let c = s.tokenize_line(")", &b.state, false);
        assert_eq!(c.state, initial);
    }
}
