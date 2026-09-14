use std::collections::HashMap;
use std::sync::Arc;

use crate::Error;
use crate::grammar::{
    Grammar, GrammarResolver, Priority, Rule, resolve_backrefs, resolve_scope_backrefs,
};
use crate::regex::{AnchorActive, Match};
use crate::scope::{ScopeInterner, ScopeListId};
use crate::token::Token;
use crate::tokenizer::captures::handle_captures;
use crate::tokenizer::injections::{InjectionEntry, match_injections, pick_best_match};
use crate::tokenizer::memo::{CompiledSet, EntryRule, Memo, MemoKey};
use crate::tokenizer::state::{RuleRef, StackFrame, WorkFrame, WorkStack};

/// Disjoint borrows of the session's mutable state for one line, so the line
/// functions can recurse (capture retokenization) without `RefCell`.
pub(crate) struct LineCtx<'s> {
    pub memo: &'s mut Memo,
    pub interner: &'s mut ScopeInterner,
    pub injections: &'s mut Vec<InjectionEntry>,
    pub injection_hits: &'s mut HashMap<ScopeListId, Vec<(usize, Priority)>>,
    pub scan_bufs: &'s mut Vec<String>,
    pub resolver: &'s dyn GrammarResolver,
    pub base: &'s Arc<Grammar>,
}

/// Emits tokens back to back: each `produce` closes the token that started where the
/// previous one ended.
pub(crate) struct TokenBuilder {
    tokens: Vec<Token>,
    pub last_end: usize,
}

impl TokenBuilder {
    pub fn new(start: usize) -> Self {
        Self {
            tokens: Vec::new(),
            last_end: start,
        }
    }

    pub fn produce(&mut self, end: usize, scopes: ScopeListId) {
        if end <= self.last_end {
            return;
        }
        self.tokens.push(Token {
            start: self.last_end,
            end,
            scopes,
        });
        self.last_end = end;
    }

    /// Trims tokens back to the bare line: the tokenizer scans across the sentinel
    /// newline exactly like vscode-textmate, whose emitted tokens may cover it.
    pub fn finish_clamped(self, token_end: usize) -> Vec<Token> {
        let mut out = self.tokens;
        out.retain_mut(|tok| {
            if tok.start >= token_end {
                return false;
            }
            tok.end = tok.end.min(token_end);
            tok.start < tok.end
        });
        out
    }
}

pub(crate) fn extract_capture_texts<'t>(
    captures: &[Option<(usize, usize)>],
    text: &'t str,
) -> Vec<&'t str> {
    captures
        .iter()
        .map(|c| match c {
            Some((s, e)) if s <= e && *e <= text.len() => &text[*s..*e],
            _ => "",
        })
        .collect()
}

fn resolve_scope_name<'n>(
    name: &'n str,
    captures: &[Option<(usize, usize)>],
    text: &str,
) -> std::borrow::Cow<'n, str> {
    if !name.contains('$') {
        return std::borrow::Cow::Borrowed(name);
    }
    let texts = extract_capture_texts(captures, text);
    std::borrow::Cow::Owned(resolve_scope_backrefs(name, &texts).into_owned())
}

/// Tokenizes `scan` (a line plus its sentinel newline) from `start`, mutating `stack`
/// and emitting into `out`. `token_end` is the bare line length. `top_level` is
/// false during capture retokenization, which skips while checks and injections.
#[allow(clippy::too_many_arguments)]
pub(crate) fn run_line(
    cx: &mut LineCtx<'_>,
    scan: &str,
    token_end: usize,
    stack: &mut WorkStack,
    out: &mut TokenBuilder,
    start: usize,
    is_first_line: bool,
    depth: usize,
    top_level: bool,
) -> Result<(), Arc<Error>> {
    let mut is_first_line = is_first_line;
    let mut anchor_position: Option<usize> = None;
    let mut pos = start;

    if top_level {
        let (line_pos, anchor) =
            check_while_conditions(cx, scan, stack, out, &mut is_first_line, depth)?;
        pos = line_pos;
        anchor_position = anchor;
    }

    let scan_len = scan.len();
    while pos <= scan_len {
        debug_assert!(scan.is_char_boundary(pos));
        let top = stack.top();
        let rule = top.frame.rule.clone();
        let end_override = top.frame.end_override.clone();
        let options = AnchorActive::new(is_first_line, anchor_position, pos).to_search_options();

        let key = MemoKey::new(&rule.grammar, rule.rule, cx.base, end_override.clone());
        let base = cx.base;
        let resolver = cx.resolver;
        let grammar_match = cx.memo.search(
            key,
            || {
                CompiledSet::compile(
                    &rule.grammar,
                    rule.rule,
                    base,
                    resolver,
                    end_override.as_deref(),
                )
            },
            scan,
            pos,
            options,
        )?;

        let injection = if top_level {
            match_injections(cx, stack.scopes(), scan, pos, options)
        } else {
            None
        };

        let Some((m, entry)) = pick_best_match(grammar_match, injection) else {
            if pos < token_end {
                out.produce(token_end, stack.scopes());
            }
            break;
        };

        let (match_start, match_end) = m.range();
        let captured_eol = match_end == scan_len;
        if match_start > pos {
            out.produce(match_start, stack.scopes());
        }
        let has_advanced = match_end > pos;

        match entry {
            EntryRule::End => {
                let popped = stack.top().clone();
                handle_end_rule(cx, scan, &m, stack, out, depth);
                anchor_position = popped.anchor_position;
                if !has_advanced && popped.enter_position == Some(pos) {
                    stack.push(popped);
                    out.produce(token_end, stack.scopes());
                    break;
                }
            }
            EntryRule::Rule(compiled) => {
                let rule_ref = RuleRef::new(compiled.grammar, compiled.rule);
                match rule_ref.grammar.rule(rule_ref.rule) {
                    Rule::Match { .. } => {
                        handle_match_rule(cx, scan, &m, &rule_ref, stack, out, depth);
                        if !has_advanced {
                            stack.safe_pop();
                            out.produce(token_end, stack.scopes());
                            break;
                        }
                    }
                    Rule::BeginEnd { .. } | Rule::BeginWhile { .. } => {
                        handle_begin_rule(
                            cx,
                            scan,
                            &m,
                            &rule_ref,
                            stack,
                            out,
                            pos,
                            captured_eol,
                            depth,
                        );
                        anchor_position = Some(match_end);
                        if !has_advanced && stack.top_has_same_rule_below() {
                            stack.pop();
                            out.produce(token_end, stack.scopes());
                            break;
                        }
                    }
                    Rule::Include(_) | Rule::Collection { .. } | Rule::Noop => {
                        debug_assert!(false, "scanner entries are match or begin rules");
                        if !has_advanced {
                            out.produce(token_end, stack.scopes());
                            break;
                        }
                    }
                }
            }
        }

        if match_end > pos {
            pos = match_end;
            is_first_line = false;
        }
    }
    Ok(())
}

/// Re-checks every open while rule at the start of a line, outermost first. A failing
/// while pops its frame and everything above it. Returns the position to resume at
/// and the anchor position for `\G`.
fn check_while_conditions(
    cx: &mut LineCtx<'_>,
    scan: &str,
    stack: &mut WorkStack,
    out: &mut TokenBuilder,
    is_first_line: &mut bool,
    depth: usize,
) -> Result<(usize, Option<usize>), Arc<Error>> {
    let mut line_pos = 0;
    let mut anchor_position = if stack.top().frame.begin_captured_eol {
        Some(0)
    } else {
        None
    };

    for idx in stack.while_frame_indices() {
        let frame = &stack.frame(idx).frame;
        let pattern = frame
            .while_pattern
            .clone()
            .expect("while frames carry a pattern");
        let rule = frame.rule.clone();
        let scopes = frame.scopes_after_content;
        let options =
            AnchorActive::new(*is_first_line, anchor_position, line_pos).to_search_options();
        let m = match cx.memo.search_while(&pattern, scan, line_pos, options) {
            Ok(Some(m)) => m,
            Ok(None) | Err(_) => {
                stack.truncate(idx);
                break;
            }
        };

        let (capture_start, capture_end) = m.range();
        out.produce(capture_start, scopes);
        if let Rule::BeginWhile { while_captures, .. } = rule.grammar.rule(rule.rule)
            && !while_captures.is_empty()
        {
            handle_captures(
                cx,
                scan,
                &m.captures,
                while_captures,
                scopes,
                out,
                &rule.grammar,
                depth,
            );
        }
        out.produce(capture_end, scopes);
        anchor_position = Some(capture_end);
        if capture_end > line_pos {
            line_pos = capture_end;
            *is_first_line = false;
        }
    }
    Ok((line_pos, anchor_position))
}

fn handle_match_rule(
    cx: &mut LineCtx<'_>,
    scan: &str,
    m: &Match,
    rule: &RuleRef,
    stack: &WorkStack,
    out: &mut TokenBuilder,
    depth: usize,
) {
    let Rule::Match { name, captures, .. } = rule.grammar.rule(rule.rule) else {
        return;
    };
    let mut scopes = stack.scopes();
    if let Some(name) = name {
        let resolved = resolve_scope_name(name, &m.captures, scan);
        scopes = cx.interner.push_names(scopes, &resolved);
    }
    if captures.is_empty() {
        out.produce(m.end(), scopes);
    } else {
        handle_captures(
            cx,
            scan,
            &m.captures,
            captures,
            scopes,
            out,
            &rule.grammar,
            depth,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_begin_rule(
    cx: &mut LineCtx<'_>,
    scan: &str,
    m: &Match,
    rule: &RuleRef,
    stack: &mut WorkStack,
    out: &mut TokenBuilder,
    pos: usize,
    captured_eol: bool,
    depth: usize,
) {
    let (name, content_name, begin_captures, needs_texts, end, while_) =
        match rule.grammar.rule(rule.rule) {
            Rule::BeginEnd {
                name,
                content_name,
                begin_captures,
                needs_begin_capture_texts,
                end,
                end_has_backrefs,
                ..
            } => (
                name,
                content_name,
                begin_captures,
                *needs_begin_capture_texts,
                Some((end, *end_has_backrefs)),
                None,
            ),
            Rule::BeginWhile {
                name,
                content_name,
                begin_captures,
                needs_begin_capture_texts,
                while_,
                while_has_backrefs,
                ..
            } => (
                name,
                content_name,
                begin_captures,
                *needs_begin_capture_texts,
                None,
                Some((while_, *while_has_backrefs)),
            ),
            _ => return,
        };

    let texts: Vec<&str> = if needs_texts {
        extract_capture_texts(&m.captures, scan)
    } else {
        Vec::new()
    };
    let parent_scopes = stack.scopes();
    let scopes_after_name = match name {
        Some(name) => {
            let resolved = resolve_scope_backrefs(name, &texts);
            cx.interner.push_names(parent_scopes, &resolved)
        }
        None => parent_scopes,
    };
    let scopes_after_content = match content_name {
        Some(content) => {
            let resolved = resolve_scope_backrefs(content, &texts);
            cx.interner.push_names(scopes_after_name, &resolved)
        }
        None => scopes_after_name,
    };

    if begin_captures.is_empty() {
        out.produce(m.end(), scopes_after_name);
    } else {
        handle_captures(
            cx,
            scan,
            &m.captures,
            begin_captures,
            scopes_after_name,
            out,
            &rule.grammar,
            depth,
        );
    }

    let end_override = match end {
        Some((end, true)) => Some(Arc::from(resolve_backrefs(end.source(), &texts))),
        _ => None,
    };
    let while_pattern = match while_ {
        Some((while_, true)) => Some(Arc::from(resolve_backrefs(while_.source(), &texts))),
        Some((while_, false)) => Some(Arc::from(while_.source())),
        None => None,
    };

    stack.push(WorkFrame {
        frame: StackFrame {
            rule: rule.clone(),
            scopes_after_name,
            scopes_after_content,
            end_override,
            while_pattern,
            begin_captured_eol: captured_eol,
        },
        anchor_position: Some(m.end()),
        enter_position: Some(pos),
    });
}

fn handle_end_rule(
    cx: &mut LineCtx<'_>,
    scan: &str,
    m: &Match,
    stack: &mut WorkStack,
    out: &mut TokenBuilder,
    depth: usize,
) {
    let top = &stack.top().frame;
    let scopes = top.scopes_after_name;
    let rule = top.rule.clone();
    match rule.grammar.rule(rule.rule) {
        Rule::BeginEnd { end_captures, .. } if !end_captures.is_empty() => {
            handle_captures(
                cx,
                scan,
                &m.captures,
                end_captures,
                scopes,
                out,
                &rule.grammar,
                depth,
            );
        }
        _ => out.produce(m.end(), scopes),
    }
    stack.pop();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_ignores_non_advancing_produce_and_clamps() {
        let mut b = TokenBuilder::new(0);
        let s = ScopeListId::EMPTY;
        b.produce(0, s);
        b.produce(3, s);
        b.produce(2, s);
        b.produce(6, s);
        b.produce(7, s);
        let tokens = b.finish_clamped(5);
        assert_eq!(tokens.len(), 2);
        assert_eq!((tokens[0].start, tokens[0].end), (0, 3));
        assert_eq!((tokens[1].start, tokens[1].end), (3, 5));
    }

    #[test]
    fn capture_texts_skip_unmatched_groups() {
        let texts = extract_capture_texts(&[Some((0, 3)), None, Some((1, 2))], "abc\n");
        assert_eq!(texts, ["abc", "", "b"]);
    }
}
