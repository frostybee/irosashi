mod captures;
mod injections;
mod line;
mod memo;
mod state;

use std::collections::{HashMap, HashSet};
use std::mem;
use std::ops::Range;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use crate::grammar::{Grammar, GrammarResolver, ROOT_RULE_ID};
use crate::scope::{ScopeInterner, ScopeListId};
use crate::theme::{Theme, ThemeId};
use crate::token::{
    Diagnostic, DiagnosticKind, ScopeTable, ThemeSlot, ThemedLine, ThemedToken, Token, TokenStyle,
    TokensResult,
};
use crate::tokenizer::injections::{InjectionEntry, InjectionHits};
use crate::tokenizer::line::{LineCtx, TokenBuilder, run_line};
use crate::tokenizer::memo::{CaptureBuf, Memo};
use crate::tokenizer::state::{RuleRef, StackFrame, WorkStack};

pub use injections::InjectionProvider;
pub use state::StateStack;

/// Everything a session needs from its registry.
pub trait Resolver: GrammarResolver + InjectionProvider + Send + Sync {}

impl<T: GrammarResolver + InjectionProvider + Send + Sync> Resolver for T {}

/// Per-call tokenizer guards.
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

/// How much a session has accumulated; used to retire pooled sessions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionFootprint {
    pub scope_lists: usize,
    pub compiled_sets: usize,
}

/// Counters accumulated over a session's life, for cache tuning and benchmarks.
/// A memo lookup happens once per open frame per line; a scan step is one regset
/// search of the grammar's context.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SessionStats {
    pub lines: u64,
    pub scan_steps: u64,
    pub memo_hits: u64,
    pub memo_misses: u64,
    pub memo_errors: u64,
    pub while_hits: u64,
    pub while_misses: u64,
    pub injection_list_hits: u64,
    pub injection_list_misses: u64,
    pub injection_searches: u64,
    pub injection_set_compiles: u64,
    pub capture_retokenizations: u64,
    pub style_hits: u64,
    pub style_misses: u64,
}

struct Styler {
    theme: Arc<Theme>,
    by_list: Vec<Option<TokenStyle>>,
}

/// A tokenizer for one grammar. Owns the scope interner, the compiled scanner cache
/// and the style caches, so it is used from one thread at a time. Building one
/// resolves the grammar's injectors through the resolver.
pub struct Session {
    grammar: Arc<Grammar>,
    resolver: Arc<dyn Resolver>,
    interner: ScopeInterner,
    memo: Memo,
    injections: Vec<InjectionEntry>,
    injection_hits: InjectionHits,
    scan_bufs: Vec<String>,
    capture_buf: CaptureBuf,
    stats: SessionStats,
    styles: HashMap<ThemeId, Styler>,
    initial: StateStack,
}

impl Session {
    pub fn new(grammar: Arc<Grammar>, resolver: Arc<dyn Resolver>) -> Self {
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
            interner,
            memo: Memo::default(),
            injections,
            injection_hits: Vec::new(),
            scan_bufs: vec![String::new()],
            capture_buf: Vec::new(),
            stats: SessionStats::default(),
            styles: HashMap::new(),
            initial: root.snapshot(),
        }
    }

    /// Counters since the session was created or `reset_stats` was called.
    pub fn stats(&self) -> SessionStats {
        let memo = self.memo.stats();
        SessionStats {
            memo_hits: memo.hits,
            memo_misses: memo.misses,
            memo_errors: memo.errors,
            while_hits: memo.while_hits,
            while_misses: memo.while_misses,
            ..self.stats
        }
    }

    pub fn reset_stats(&mut self) {
        self.stats = SessionStats::default();
        self.memo.reset_stats();
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

    pub fn footprint(&self) -> SessionFootprint {
        SessionFootprint {
            scope_lists: self.interner.list_count(),
            compiled_sets: self.memo.len(),
        }
    }

    /// Tokenizes one bare line (no terminator) starting from `state`.
    pub fn tokenize_line(
        &mut self,
        line: &str,
        state: &StateStack,
        is_first_line: bool,
        options: TokenizeOptions,
    ) -> LineResult {
        if let Some(max) = options.max_line_length
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
            capture_buf,
            stats,
            ..
        } = self;
        stats.lines += 1;
        let mut cx = LineCtx {
            memo,
            interner,
            injections,
            injection_hits,
            scan_bufs,
            capture_buf,
            stats,
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
    pub fn tokenize(&mut self, code: &str, options: TokenizeOptions) -> TokenizeResult {
        let mut result = TokenizeResult {
            lines: Vec::new(),
            diagnostics: Vec::new(),
        };
        let mut state = self.initial_state();
        for (index, range) in split_lines(code).into_iter().enumerate() {
            let line = self.tokenize_line(&code[range], &state, index == 0, options);
            if let Some(kind) = line.diagnostic {
                result.diagnostics.push(Diagnostic { line: index, kind });
            }
            result.lines.push(line.tokens);
            state = line.state;
        }
        result
    }

    /// Tokenizes and resolves every token's style against each theme slot. Slot 0 is
    /// the default theme carried inline by the tokens; with more than one slot the
    /// result's `styles` table holds every slot's style per scope stack.
    pub fn themed(
        &mut self,
        code: &str,
        options: TokenizeOptions,
        themes: Vec<ThemeSlot>,
        include_scopes: bool,
    ) -> TokensResult {
        assert!(!themes.is_empty(), "themed needs at least one theme");
        let ranges = split_lines(code);
        let multi = themes.len() > 1;
        let mut styles: HashMap<ScopeListId, Box<[TokenStyle]>> = HashMap::new();
        let mut used: HashSet<ScopeListId> = HashSet::new();
        let mut diagnostics = Vec::new();

        let mut lines = Vec::with_capacity(ranges.len());
        let mut state = self.initial_state();
        for (index, range) in ranges.into_iter().enumerate() {
            let line = self.tokenize_line(&code[range.clone()], &state, index == 0, options);
            if let Some(kind) = line.diagnostic {
                diagnostics.push(Diagnostic { line: index, kind });
            }
            state = line.state;
            let tokens = line.tokens;
            let mut themed = Vec::with_capacity(tokens.len());
            for t in tokens {
                if t.start >= t.end {
                    continue;
                }
                let style = self.style(&themes[0].theme, t.scopes);
                if multi && !styles.contains_key(&t.scopes) {
                    let all: Box<[TokenStyle]> = themes
                        .iter()
                        .map(|slot| self.style(&slot.theme, t.scopes))
                        .collect();
                    styles.insert(t.scopes, all);
                }
                if include_scopes {
                    used.insert(t.scopes);
                }
                themed.push(ThemedToken {
                    start: t.start,
                    end: t.end,
                    style,
                    scopes: t.scopes,
                });
            }
            lines.push(ThemedLine::new(range, themed));
        }

        let scopes = include_scopes.then(|| {
            ScopeTable::new(
                used.into_iter()
                    .map(|id| (id, self.interner.names_shared(id)))
                    .collect(),
            )
        });
        TokensResult::new(code.to_owned(), lines, themes, styles, scopes, diagnostics)
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
            self.stats.style_hits += 1;
            return style;
        }
        self.stats.style_misses += 1;
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

    const NO_OPTS: TokenizeOptions = TokenizeOptions {
        max_line_length: None,
    };

    #[test]
    fn session_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Session>();
    }

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
        Session::new(grammar, Arc::new(()))
    }

    #[test]
    fn stats_count_memo_hits_after_the_first_pass() {
        let mut s = session(
            r#"{"scopeName": "source.t", "patterns": [
                {"begin": "\\(", "end": "\\)", "patterns": [{"match": "x", "name": "x"}]}]}"#,
        );
        s.tokenize("(x)\n(x x)", NO_OPTS);
        let first = s.stats();
        assert_eq!(first.lines, 2);
        assert_eq!(first.memo_misses, 2);
        assert!(first.scan_steps >= 6);
        s.tokenize("(x)\n(x x)", NO_OPTS);
        let second = s.stats();
        assert_eq!(second.memo_misses, 2);
        assert!(second.memo_hits > first.memo_hits);
        s.reset_stats();
        assert_eq!(s.stats(), SessionStats::default());
    }

    #[test]
    fn empty_begin_pushes_a_zero_width_scope() {
        let mut s = session(
            r#"{"scopeName": "source.t", "patterns": [
                {"begin": "(if)\\s+", "end": "$", "name": "flow", "captures": {"1": {"name": "kw"}},
                 "patterns": [{"begin": "", "end": "$", "name": "embedded",
                               "patterns": [{"match": "\\w+", "name": "w"}]}]}]}"#,
        );
        let r = s.tokenize("if cond", NO_OPTS);
        let last = r.lines[0].last().unwrap();
        assert_eq!((last.start, last.end), (3, 7));
        assert_eq!(
            s.scope_names(last.scopes),
            ["source.t", "flow", "embedded", "w"]
        );
    }

    #[test]
    fn crlf_and_lf_give_identical_tokens() {
        let mut s =
            session(r#"{"scopeName": "source.t", "patterns": [{"match": "\\w+", "name": "w"}]}"#);
        let lf = s.tokenize("ab cd\nef", NO_OPTS);
        let crlf = s.tokenize("ab cd\r\nef", NO_OPTS);
        assert_eq!(lf, crlf);
        assert_eq!(lf.lines.len(), 2);
    }

    #[test]
    fn empty_input_has_no_lines_and_empty_line_has_no_tokens() {
        let mut s =
            session(r#"{"scopeName": "source.t", "patterns": [{"match": "\\w+", "name": "w"}]}"#);
        assert!(s.tokenize("", NO_OPTS).lines.is_empty());
        let r = s.tokenize("a\n\nb", NO_OPTS);
        assert_eq!(r.lines.len(), 3);
        assert!(r.lines[1].is_empty());
    }

    #[test]
    fn max_line_length_emits_one_unstyled_token_and_keeps_state() {
        let mut s = session(
            r#"{"scopeName": "source.t", "patterns": [{"begin": "\\(", "end": "\\)", "name": "paren", "patterns": [{"match": "x", "name": "x"}]}]}"#,
        );
        let r = s.tokenize(
            "(x\nxxxxxxxx\nx)",
            TokenizeOptions {
                max_line_length: Some(4),
            },
        );
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
        let r = s.tokenize("(x)\ny", NO_OPTS);
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
        let a = s.tokenize_line("(x", &initial, true, NO_OPTS);
        let b = s.tokenize_line("x x", &a.state, false, NO_OPTS);
        assert_eq!(a.state, b.state);
        assert_ne!(a.state, initial);
        let c = s.tokenize_line(")", &b.state, false, NO_OPTS);
        assert_eq!(c.state, initial);
    }

    #[test]
    fn themed_multi_fills_a_style_per_slot_and_scopes_on_request() {
        let mut s = session(
            r#"{"scopeName": "source.t", "patterns": [{"match": "\\d+", "name": "constant.numeric"}]}"#,
        );
        let dark = Arc::new(
            Theme::parse(
                br##"{"name": "d", "colors": {"editor.foreground": "#111111"}, "tokenColors": [{"scope": "constant", "settings": {"foreground": "#aaaaaa"}}]}"##,
            )
            .unwrap(),
        );
        let light = Arc::new(
            Theme::parse(
                br##"{"name": "l", "colors": {"editor.foreground": "#222222"}, "tokenColors": [{"scope": "constant", "settings": {"foreground": "#bbbbbb"}}]}"##,
            )
            .unwrap(),
        );
        let slots = vec![
            ThemeSlot {
                key: "dark".into(),
                theme: dark,
            },
            ThemeSlot {
                key: "light".into(),
                theme: light,
            },
        ];
        let r = s.themed("a 42", NO_OPTS, slots, true);
        assert!(r.is_multi());
        let line = &r.lines[0];
        let num = line.tokens[1];
        assert_eq!(r.color_in(0, r.style_in(&num, 0).color.unwrap()), "#aaaaaa");
        assert_eq!(r.color_in(1, r.style_in(&num, 1).color.unwrap()), "#bbbbbb");
        let plain = line.tokens[0];
        assert_eq!(
            r.color_in(1, r.style_in(&plain, 1).color.unwrap()),
            "#222222"
        );
        assert_eq!(r.styles.len(), 2);
        let names: Vec<&str> = r.scopes_of(&num).unwrap().iter().map(|s| &**s).collect();
        assert_eq!(names, ["source.t", "constant.numeric"]);
    }
}
