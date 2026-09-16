use std::mem;
use std::sync::Arc;

use crate::grammar::{Captures, Grammar, Rule, RuleId, resolve_scope_backrefs};
use crate::scope::ScopeListId;
use crate::tokenizer::line::{LineCtx, TokenBuilder, extract_capture_texts, run_line};
use crate::tokenizer::state::{RuleRef, StackFrame, WorkStack};

/// Emits the capture groups of a match as a flat, non-overlapping token run. Wider
/// captures are split around narrower ones; a capture with nested patterns is
/// retokenized with them.
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_captures(
    cx: &mut LineCtx<'_>,
    scan: &str,
    captures: &[Option<(usize, usize)>],
    rules: &Captures,
    scopes: ScopeListId,
    out: &mut TokenBuilder,
    grammar: &Arc<Grammar>,
    depth: usize,
) {
    if rules.is_empty() || captures.is_empty() {
        return;
    }
    let Some((capture_start, max_end)) = captures[0] else {
        return;
    };
    out.produce(capture_start, scopes);

    let mut local_stack: Vec<(ScopeListId, usize)> = Vec::new();
    let mut texts: Option<Vec<&str>> = None;

    for (i, capture) in captures.iter().enumerate() {
        let Some(rule) = rules.get(i) else {
            continue;
        };
        let Some((start, end)) = *capture else {
            continue;
        };
        if start >= end {
            continue;
        }
        if start > max_end {
            break;
        }

        while let Some(&(frame_scopes, frame_end)) = local_stack.last() {
            if frame_end > start {
                break;
            }
            out.produce(frame_end, frame_scopes);
            local_stack.pop();
        }

        let parent_scopes = local_stack.last().map(|f| f.0).unwrap_or(scopes);
        if start > out.last_end {
            out.produce(start, parent_scopes);
        }

        let resolved_name: Option<std::borrow::Cow<'_, str>> = match rule.name.as_deref() {
            Some(name) if name.contains('$') => {
                let texts = texts.get_or_insert_with(|| extract_capture_texts(captures, scan));
                Some(resolve_scope_backrefs(name, texts))
            }
            Some(name) => Some(std::borrow::Cow::Borrowed(name)),
            None => None,
        };
        let mut capture_scopes = parent_scopes;
        if let Some(name) = &resolved_name {
            capture_scopes = cx.interner.push_names(capture_scopes, name);
        }

        if let Some(patterns) = rule.patterns
            && has_patterns(grammar, patterns)
        {
            let mut retok_scopes = scopes;
            if let Some(name) = &resolved_name {
                retok_scopes = cx.interner.push_names(retok_scopes, name);
            }
            retokenize_capture(
                cx,
                scan,
                grammar,
                patterns,
                retok_scopes,
                start,
                end,
                out,
                depth,
            );
            continue;
        }

        if rule.name.as_deref().is_some_and(|n| !n.is_empty()) {
            local_stack.push((capture_scopes, end));
        }
    }

    while let Some((frame_scopes, frame_end)) = local_stack.pop() {
        out.produce(frame_end, frame_scopes);
    }
    if out.last_end < max_end {
        out.produce(max_end, scopes);
    }
}

fn has_patterns(grammar: &Grammar, rule: RuleId) -> bool {
    match grammar.rule(rule) {
        Rule::Collection { patterns, .. } => !patterns.is_empty(),
        _ => true,
    }
}

/// Tokenizes `scan[start..end]` with a capture's nested patterns. The line is
/// truncated, not sliced, so offsets stay absolute and lookbehind sees the prefix.
#[allow(clippy::too_many_arguments)]
fn retokenize_capture(
    cx: &mut LineCtx<'_>,
    scan: &str,
    grammar: &Arc<Grammar>,
    patterns: RuleId,
    scopes: ScopeListId,
    start: usize,
    end: usize,
    out: &mut TokenBuilder,
    depth: usize,
) {
    if start >= end {
        return;
    }
    let next = depth + 1;
    if cx.scan_bufs.len() <= next {
        cx.scan_bufs.resize_with(next + 1, String::new);
    }
    let mut buf = mem::take(&mut cx.scan_bufs[next]);
    buf.clear();
    buf.push_str(&scan[..end]);
    buf.push('\n');

    let mut stack = WorkStack::root(StackFrame {
        rule: RuleRef::new(Arc::clone(grammar), patterns),
        scopes_after_name: scopes,
        scopes_after_content: scopes,
        end_override: None,
        while_pattern: None,
        begin_captured_eol: false,
    });
    let mut sub = TokenBuilder::new(start);
    let outer_generation = cx.generation;
    let result = run_line(
        cx, &buf, end, &mut stack, &mut sub, start, false, next, false,
    );
    cx.generation = outer_generation;
    cx.scan_bufs[next] = buf;

    let tokens = sub.finish_clamped(end);
    if result.is_err() || tokens.is_empty() {
        out.produce(end, scopes);
        return;
    }
    for token in tokens {
        out.produce(token.end, token.scopes);
    }
}
