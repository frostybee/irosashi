//! Mechanism tests over the hand-built mini grammars. These assert properties of the
//! token stream (a scope appears on a given text, offsets reconstruct the line), not
//! exact vectors; exactness is checked by the fidelity fixtures.

use std::path::Path;
use std::sync::Arc;

use iro::grammar::Grammar;
use iro::tokenizer::split_lines;
use iro::{Session, Token, TokenizeOptions};

fn session(name: &str) -> Session {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("testdata/mini")
        .join(name);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let grammar = Arc::new(Grammar::parse(&bytes).expect("mini grammar parses"));
    Session::new(grammar, Arc::new(()))
}

struct Line<'a> {
    text: &'a str,
    tokens: Vec<Token>,
}

fn tokenize<'a>(session: &mut Session, code: &'a str) -> Vec<Line<'a>> {
    let result = session.tokenize(code, TokenizeOptions::default());
    assert!(
        result.diagnostics.is_empty(),
        "diagnostics: {:?}",
        result.diagnostics
    );
    let lines: Vec<Line<'a>> = split_lines(code)
        .into_iter()
        .zip(result.lines)
        .map(|(range, tokens)| Line {
            text: &code[range],
            tokens,
        })
        .collect();
    for line in &lines {
        assert_reconstructs(line);
    }
    lines
}

fn assert_reconstructs(line: &Line<'_>) {
    let mut pos = 0;
    for token in &line.tokens {
        assert_eq!(
            token.start, pos,
            "gap or overlap before {token:?} in {:?}",
            line.text
        );
        assert!(
            token.end > token.start,
            "empty token {token:?} in {:?}",
            line.text
        );
        assert!(line.text.is_char_boundary(token.start) && line.text.is_char_boundary(token.end));
        pos = token.end;
    }
    assert_eq!(pos, line.text.len(), "tokens do not cover {:?}", line.text);
}

fn scopes_of<'s>(session: &'s Session, line: &Line<'_>, text: &str) -> Vec<&'s str> {
    let token = line
        .tokens
        .iter()
        .find(|t| t.text(line.text) == text)
        .unwrap_or_else(|| panic!("no token with text {text:?} in {:?}", line.text));
    session.scope_names(token.scopes)
}

fn has_scope(session: &Session, line: &Line<'_>, scope: &str) -> bool {
    line.tokens
        .iter()
        .any(|t| session.scope_names(t.scopes).contains(&scope))
}

#[test]
fn match_only() {
    let mut s = session("match_only.json");
    let lines = tokenize(&mut s, "if x return 42");
    let l = &lines[0];
    assert_eq!(
        scopes_of(&s, l, "if"),
        ["source.test-match", "keyword.control.test"]
    );
    assert_eq!(
        scopes_of(&s, l, "x"),
        ["source.test-match", "variable.other.test"]
    );
    assert_eq!(
        scopes_of(&s, l, "42"),
        ["source.test-match", "constant.numeric.test"]
    );
    assert_eq!(scopes_of(&s, l, " "), ["source.test-match"]);
}

#[test]
fn begin_end_with_captures_and_nested_escape() {
    let mut s = session("begin_end.json");
    let lines = tokenize(&mut s, r#"say "hi\n there" now"#);
    let l = &lines[0];
    assert_eq!(
        scopes_of(&s, l, "say"),
        ["source.test-beginend", "variable.other.test"]
    );
    assert_eq!(
        scopes_of(&s, l, "hi"),
        ["source.test-beginend", "string.quoted.double.test"]
    );
    assert_eq!(
        scopes_of(&s, l, r"\n"),
        [
            "source.test-beginend",
            "string.quoted.double.test",
            "constant.character.escape.test"
        ]
    );
    let quotes: Vec<_> = l.tokens.iter().filter(|t| t.text(l.text) == "\"").collect();
    assert_eq!(quotes.len(), 2);
    assert_eq!(
        s.scope_names(quotes[0].scopes),
        [
            "source.test-beginend",
            "string.quoted.double.test",
            "punctuation.definition.string.begin.test"
        ]
    );
    assert_eq!(
        s.scope_names(quotes[1].scopes),
        [
            "source.test-beginend",
            "string.quoted.double.test",
            "punctuation.definition.string.end.test"
        ]
    );
}

#[test]
fn begin_end_spans_lines_and_state_carries() {
    let mut s = session("begin_end.json");
    let lines = tokenize(&mut s, "\"open\nstill\" done");
    assert!(has_scope(&s, &lines[1], "string.quoted.double.test"));
    assert_eq!(
        scopes_of(&s, &lines[1], "done"),
        ["source.test-beginend", "variable.other.test"]
    );
}

#[test]
fn begin_end_backref_end_uses_the_begin_quote() {
    let mut s = session("begin_end_backref.json");
    let lines = tokenize(&mut s, "'a\"b' x \"c'd\" y");
    let l = &lines[0];
    assert_eq!(
        scopes_of(&s, l, "a\"b"),
        ["source.test-backref", "string.quoted.test"]
    );
    assert_eq!(
        scopes_of(&s, l, "c'd"),
        ["source.test-backref", "string.quoted.test"]
    );
    assert_eq!(
        scopes_of(&s, l, "x"),
        ["source.test-backref", "variable.other.test"]
    );
    assert_eq!(
        scopes_of(&s, l, "y"),
        ["source.test-backref", "variable.other.test"]
    );
}

#[test]
fn g_anchor_matches_only_at_the_anchor_position() {
    let mut s = session("g_anchor.json");
    let lines = tokenize(&mut s, "- hello world");
    let l = &lines[0];
    assert_eq!(
        scopes_of(&s, l, "- "),
        [
            "source.test-ganchor",
            "meta.list.test",
            "punctuation.definition.list.test"
        ]
    );
    assert_eq!(
        scopes_of(&s, l, "hello"),
        [
            "source.test-ganchor",
            "meta.list.test",
            "entity.name.first-word.test"
        ]
    );
    assert_eq!(
        scopes_of(&s, l, "world"),
        [
            "source.test-ganchor",
            "meta.list.test",
            "variable.other.test"
        ]
    );
}

#[test]
fn begin_while_captured_eol_keeps_the_block_open() {
    let mut s = session("begin_while_eol.json");
    let lines = tokenize(&mut s, ">>>\n.one\n.two\nthree");
    assert_eq!(lines.len(), 4);
    assert!(has_scope(&s, &lines[1], "meta.block.test"));
    assert!(has_scope(&s, &lines[1], "inner.word.test"));
    assert!(has_scope(&s, &lines[2], "meta.block.test"));
    assert!(!has_scope(&s, &lines[3], "meta.block.test"));
    assert!(has_scope(&s, &lines[3], "other.word.test"));
}

#[test]
fn begin_while_without_captured_eol_pops_on_next_line() {
    let mut s = session("begin_while_no_eol.json");
    let lines = tokenize(&mut s, "<<< x\n.one");
    assert!(has_scope(&s, &lines[0], "meta.block.test"));
    assert!(!has_scope(&s, &lines[1], "meta.block.test"));
}

#[test]
fn injection_self_adds_patterns_to_root() {
    let mut s = session("injection_self.json");
    let lines = tokenize(&mut s, "if # note else");
    let l = &lines[0];
    assert_eq!(
        scopes_of(&s, l, "if"),
        ["source.test-injection", "keyword.test"]
    );
    assert_eq!(
        scopes_of(&s, l, "# note else"),
        ["source.test-injection", "comment.line.test"]
    );
}

#[test]
fn injection_begin_end_rule() {
    let mut s = session("injection_begin_end.json");
    let lines = tokenize(&mut s, "a {12 b} c");
    let l = &lines[0];
    assert_eq!(
        scopes_of(&s, l, "a"),
        ["source.test-injection-be", "word.test"]
    );
    assert_eq!(
        scopes_of(&s, l, "12"),
        [
            "source.test-injection-be",
            "block.injected.test",
            "number.injected.test"
        ]
    );
    assert_eq!(
        scopes_of(&s, l, "{"),
        ["source.test-injection-be", "block.injected.test"]
    );
    assert_eq!(
        scopes_of(&s, l, "c"),
        ["source.test-injection-be", "word.test"]
    );
}

#[test]
fn injection_negation_excludes_comments() {
    let mut s = session("injection_negation.json");
    let lines = tokenize(&mut s, "if 42 // 43");
    let l = &lines[0];
    assert_eq!(
        scopes_of(&s, l, "42"),
        ["source.test-negation", "injected.number.test"]
    );
    let injected_inside_comment = l.tokens.iter().any(|t| {
        let names = s.scope_names(t.scopes);
        names.contains(&"comment.line.test") && names.contains(&"injected.number.test")
    });
    assert!(!injected_inside_comment);
    assert_eq!(
        scopes_of(&s, l, " 43"),
        ["source.test-negation", "comment.line.test"]
    );
}

#[test]
fn injection_priority_left_wins_ties() {
    let mut s = session("injection_priority_left.json");
    let lines = tokenize(&mut s, "abc");
    assert_eq!(
        scopes_of(&s, &lines[0], "abc"),
        ["source.test-priority-left", "injected.word.test"]
    );
}

#[test]
fn injection_priority_right_loses_ties() {
    let mut s = session("injection_priority_right.json");
    let lines = tokenize(&mut s, "abc");
    assert_eq!(
        scopes_of(&s, &lines[0], "abc"),
        ["source.test-priority-right", "word.test"]
    );
}

#[test]
fn overlapping_captures_split_around_sub_captures() {
    let mut s = session("overlapping_captures.json");
    let lines = tokenize(&mut s, "fn main(a, b)");
    let l = &lines[0];
    let root = "source.test-captures";
    assert_eq!(
        scopes_of(&s, l, "fn"),
        [root, "meta.function.test", "keyword.function.test"]
    );
    assert_eq!(scopes_of(&s, l, " "), [root, "meta.function.test"]);
    assert_eq!(
        scopes_of(&s, l, "main"),
        [root, "meta.function.test", "entity.name.function.test"]
    );
    assert_eq!(
        scopes_of(&s, l, "(a, b)"),
        [root, "meta.function.test", "meta.parameters.test"]
    );

    let lines = tokenize(&mut s, "let x = 5");
    let l = &lines[0];
    assert_eq!(
        scopes_of(&s, l, "let"),
        [root, "meta.assignment.test", "storage.type.test"]
    );
    assert_eq!(
        scopes_of(&s, l, "x"),
        [root, "meta.assignment.test", "variable.name.test"]
    );
    assert_eq!(scopes_of(&s, l, " = "), [root, "meta.assignment.test"]);
    assert_eq!(
        scopes_of(&s, l, "5"),
        [root, "meta.assignment.test", "variable.value.test"]
    );
}

#[test]
fn non_ascii_offsets_are_bytes_on_char_boundaries() {
    let mut s = session("begin_end.json");
    let code = "héllo \"日本語 😀\" wörld";
    let lines = tokenize(&mut s, code);
    let l = &lines[0];
    assert_eq!(
        scopes_of(&s, l, "héllo"),
        ["source.test-beginend", "variable.other.test"]
    );
    assert_eq!(
        scopes_of(&s, l, "日本語 😀"),
        ["source.test-beginend", "string.quoted.double.test"]
    );
    assert_eq!(
        scopes_of(&s, l, "wörld"),
        ["source.test-beginend", "variable.other.test"]
    );
    let string = l
        .tokens
        .iter()
        .find(|t| t.text(l.text) == "日本語 😀")
        .unwrap();
    assert_eq!((string.start, string.end), (8, 22));
}

#[test]
fn tokenize_line_api_matches_whole_buffer() {
    let mut s = session("begin_end.json");
    let code = "a \"b\nc\" d";
    let whole = s.tokenize(code, TokenizeOptions::default());
    let mut state = s.initial_state();
    let mut per_line = Vec::new();
    for (i, range) in split_lines(code).into_iter().enumerate() {
        let r = s.tokenize_line(&code[range], &state, i == 0, TokenizeOptions::default());
        per_line.push(r.tokens);
        state = r.state;
    }
    assert_eq!(whole.lines, per_line);
    assert_eq!(state, s.initial_state());
}
