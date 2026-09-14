use std::sync::Arc;

use crate::grammar::{Grammar, RuleId};
use crate::scope::ScopeListId;

/// A rule together with the grammar that owns it.
#[derive(Debug, Clone)]
pub struct RuleRef {
    pub grammar: Arc<Grammar>,
    pub rule: RuleId,
}

impl RuleRef {
    pub fn new(grammar: Arc<Grammar>, rule: RuleId) -> Self {
        Self { grammar, rule }
    }

    pub fn same_rule(&self, other: &RuleRef) -> bool {
        self.rule == other.rule && Arc::ptr_eq(&self.grammar, &other.grammar)
    }
}

impl PartialEq for RuleRef {
    fn eq(&self, other: &Self) -> bool {
        self.same_rule(other)
    }
}

impl Eq for RuleRef {}

/// One open rule context that survives across lines.
///
/// `scopes_after_name` is the stack with the rule's `name` applied;
/// `scopes_after_content` additionally has `contentName`, and is what tokens inside
/// the rule carry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StackFrame {
    pub rule: RuleRef,
    pub scopes_after_name: ScopeListId,
    pub scopes_after_content: ScopeListId,
    /// The end pattern with begin captures substituted; `None` when the rule's static
    /// end pattern applies.
    pub end_override: Option<Arc<str>>,
    /// The resolved while pattern; `Some` exactly for begin/while frames.
    pub while_pattern: Option<Arc<str>>,
    /// The begin match consumed the line's trailing newline, so `\G` may match at
    /// column 0 in the next line's while check.
    pub begin_captured_eol: bool,
}

/// The tokenizer state between two lines: an immutable snapshot of the open rule
/// frames, root first. Cheap to clone and to compare, so callers can cache one per
/// line and stop re-tokenizing when the outgoing state equals the cached one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateStack(Arc<[StackFrame]>);

impl StateStack {
    pub fn frames(&self) -> &[StackFrame] {
        &self.0
    }

    pub fn depth(&self) -> usize {
        self.0.len()
    }

    pub fn top(&self) -> &StackFrame {
        self.0
            .last()
            .expect("a state stack always has a root frame")
    }

    /// Scopes a token at the top of this state carries.
    pub fn scopes(&self) -> ScopeListId {
        self.top().scopes_after_content
    }
}

/// A frame plus the per-line positions that vscode-textmate resets on every line.
#[derive(Debug, Clone)]
pub(crate) struct WorkFrame {
    pub frame: StackFrame,
    pub anchor_position: Option<usize>,
    pub enter_position: Option<usize>,
}

impl WorkFrame {
    pub fn new(frame: StackFrame) -> Self {
        Self {
            frame,
            anchor_position: None,
            enter_position: None,
        }
    }
}

/// The mutable stack used while tokenizing one line. The root frame is never removed.
#[derive(Debug)]
pub(crate) struct WorkStack {
    frames: Vec<WorkFrame>,
}

impl WorkStack {
    pub fn root(frame: StackFrame) -> Self {
        Self {
            frames: vec![WorkFrame::new(frame)],
        }
    }

    pub fn from_snapshot(state: &StateStack) -> Self {
        Self {
            frames: state.frames().iter().cloned().map(WorkFrame::new).collect(),
        }
    }

    pub fn snapshot(&self) -> StateStack {
        StateStack(self.frames.iter().map(|w| w.frame.clone()).collect())
    }

    #[cfg(test)]
    pub fn depth(&self) -> usize {
        self.frames.len()
    }

    pub fn frame(&self, index: usize) -> &WorkFrame {
        &self.frames[index]
    }

    pub fn top(&self) -> &WorkFrame {
        self.frames
            .last()
            .expect("a work stack always has a root frame")
    }

    pub fn scopes(&self) -> ScopeListId {
        self.top().frame.scopes_after_content
    }

    pub fn push(&mut self, frame: WorkFrame) {
        self.frames.push(frame);
    }

    /// Removes and returns the top frame; the root is returned by clone but kept.
    pub fn pop(&mut self) -> WorkFrame {
        if self.frames.len() <= 1 {
            return self.frames[0].clone();
        }
        self.frames.pop().expect("checked non-empty")
    }

    pub fn safe_pop(&mut self) {
        if self.frames.len() > 1 {
            self.frames.pop();
        }
    }

    /// Keeps the first `len` frames, and always at least the root.
    pub fn truncate(&mut self, len: usize) {
        self.frames.truncate(len.max(1));
    }

    /// Whether the top frame's rule is already open in a frame entered at the same
    /// position: the infinite-push guard of vscode-textmate.
    pub fn top_has_same_rule_below(&self) -> bool {
        let Some((top, below)) = self.frames.split_last() else {
            return false;
        };
        for frame in below.iter().rev() {
            if frame.enter_position != top.enter_position {
                break;
            }
            if frame.frame.rule.same_rule(&top.frame.rule) {
                return true;
            }
        }
        false
    }

    pub fn while_frame_indices(&self) -> Vec<usize> {
        self.frames
            .iter()
            .enumerate()
            .filter(|(_, w)| w.frame.while_pattern.is_some())
            .map(|(i, _)| i)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grammar::ROOT_RULE_ID;

    fn grammar() -> Arc<Grammar> {
        Arc::new(
            Grammar::parse(
                br#"{"scopeName": "source.t", "patterns": [{"match": "a"}, {"match": "b"}]}"#,
            )
            .unwrap(),
        )
    }

    fn frame(grammar: &Arc<Grammar>, rule: RuleId) -> StackFrame {
        StackFrame {
            rule: RuleRef::new(Arc::clone(grammar), rule),
            scopes_after_name: ScopeListId::EMPTY,
            scopes_after_content: ScopeListId::EMPTY,
            end_override: None,
            while_pattern: None,
            begin_captured_eol: false,
        }
    }

    fn work(grammar: &Arc<Grammar>, rule: RuleId, enter: Option<usize>) -> WorkFrame {
        WorkFrame {
            frame: frame(grammar, rule),
            anchor_position: None,
            enter_position: enter,
        }
    }

    #[test]
    fn root_is_never_popped() {
        let g = grammar();
        let mut stack = WorkStack::root(frame(&g, ROOT_RULE_ID));
        stack.safe_pop();
        assert_eq!(stack.depth(), 1);
        let popped = stack.pop();
        assert_eq!(stack.depth(), 1);
        assert_eq!(popped.frame.rule.rule, ROOT_RULE_ID);
        stack.truncate(0);
        assert_eq!(stack.depth(), 1);
    }

    #[test]
    fn same_rule_below_only_within_equal_enter_positions() {
        let g = grammar();
        let mut stack = WorkStack::root(frame(&g, ROOT_RULE_ID));
        stack.push(work(&g, RuleId(1), Some(3)));
        stack.push(work(&g, RuleId(2), Some(3)));
        stack.push(work(&g, RuleId(1), Some(3)));
        assert!(stack.top_has_same_rule_below());

        stack.pop();
        stack.push(work(&g, RuleId(1), Some(5)));
        assert!(!stack.top_has_same_rule_below());

        let other = grammar();
        stack.pop();
        stack.push(work(&other, RuleId(1), Some(3)));
        assert!(!stack.top_has_same_rule_below());
    }

    #[test]
    fn snapshots_ignore_transient_positions() {
        let g = grammar();
        let mut a = WorkStack::root(frame(&g, ROOT_RULE_ID));
        a.push(work(&g, RuleId(1), Some(3)));
        let snap = a.snapshot();
        let b = WorkStack::from_snapshot(&snap);
        assert_eq!(b.top().enter_position, None);
        assert_eq!(b.snapshot(), snap);
    }
}
