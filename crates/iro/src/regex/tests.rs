use super::{AnchorActive, Match, PatternSet, SearchOptions, rewrite_z_anchor};
use crate::Error;

fn set(patterns: &[&str]) -> PatternSet {
    PatternSet::new(patterns).expect("patterns compile")
}

fn find(patterns: &[&str], text: &str, start: usize) -> Option<Match> {
    set(patterns).find_next_match(text, start, SearchOptions::NONE)
}

fn find_some(patterns: &[&str], text: &str, start: usize) -> Match {
    find(patterns, text, start).expect("expected a match")
}

#[test]
fn test_round_trip() {
    let m = find_some(&["hello"], "say hello world", 0);
    assert_eq!(m.index, 0);
    assert_eq!(m.captures[0], Some((4, 9)));
}

#[test]
fn test_multi_pattern_leftmost() {
    let m = find_some(&[r"\d+", "world", "hello", "hel"], "hello world 42", 0);
    assert_eq!(
        m.index, 2,
        "hello (index 2) wins over hel (index 3) at the same position"
    );
    assert_eq!(m.captures[0], Some((0, 5)));
}

#[test]
fn test_multi_pattern_tie_break() {
    let m = find_some(&["hel", "hello"], "hello", 0);
    assert_eq!(m.index, 0, "lowest index wins a tie");
}

#[test]
fn test_capture_groups() {
    let m = find_some(&[r"(\w+)\s+(\w+)"], "hello world", 0);
    assert_eq!(m.captures, vec![Some((0, 11)), Some((0, 5)), Some((6, 11))]);
}

#[test]
fn test_backreference() {
    let m = find_some(&[r#"(["']).*?\1"#], "she said 'hi' ok", 0);
    assert_eq!(m.captures[0], Some((9, 13)));
    assert_eq!(m.captures[1], Some((9, 10)));
}

#[test]
fn test_lookahead() {
    let m = find_some(&[r"\w+(?=\()"], "foo(bar)", 0);
    assert_eq!(m.captures[0], Some((0, 3)));
}

#[test]
fn test_lookbehind() {
    let m = find_some(&[r"(?<=\.)\w+"], "obj.method", 0);
    assert_eq!(m.captures[0], Some((4, 10)));
}

#[test]
fn test_g_anchor() {
    let mut ps = set(&[r"\G\w"]);
    let m = ps.find_next_match("abc", 0, SearchOptions::NONE).unwrap();
    assert_eq!(m.captures[0], Some((0, 1)));
    let m = ps.find_next_match("abc", 1, SearchOptions::NONE).unwrap();
    assert_eq!(m.captures[0], Some((1, 2)));
}

#[test]
fn test_utf8_multibyte() {
    let text = "変数 = 42";
    let m = find_some(&[r"\d+"], text, 0);
    let (start, end) = m.range();
    assert_eq!(&text[start..end], "42");
    assert!(start >= 6, "offsets must be bytes, not chars: got {start}");
}

#[test]
fn test_utf8_emoji() {
    let text = "hi 👋 ok";
    let m = find_some(&[".+"], text, 0);
    assert_eq!(m.range(), (0, text.len()));
}

#[test]
fn test_no_match() {
    assert_eq!(find(&[r"\d+"], "hello", 0), None);
}

#[test]
fn test_unmatched_optional_group() {
    let m = find_some(&[r"(\w+)(?:\s+(\d+))?"], "hello", 0);
    assert_eq!(m.captures, vec![Some((0, 5)), Some((0, 5)), None]);
}

#[test]
fn test_empty_text() {
    assert_eq!(find(&[r"\w+"], "", 0), None);
}

#[test]
fn test_scanner_iterative() {
    let text = "foo bar baz";
    let mut ps = set(&[r"\w+"]);
    let mut words = Vec::new();
    let mut pos = 0;
    while let Some(m) = ps.find_next_match(text, pos, SearchOptions::NONE) {
        let (start, end) = m.range();
        words.push(&text[start..end]);
        pos = end;
    }
    assert_eq!(words, ["foo", "bar", "baz"]);
}

#[test]
fn test_empty_pattern_zero_width() {
    let m = find_some(&[""], "hello", 0);
    assert_eq!(m.range(), (0, 0));
}

#[test]
fn test_invalid_pattern() {
    let err = PatternSet::new(&[r"\w+", "(?P<"]).expect_err("expected a compilation error");
    match err {
        Error::RegexCompilation { index, .. } => assert_eq!(index, 1),
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn test_empty_set_never_matches() {
    let mut ps = set(&[]);
    assert!(ps.is_empty());
    assert_eq!(ps.find_next_match("hello", 0, SearchOptions::NONE), None);
}

struct GrammarCase {
    name: &'static str,
    pattern: &'static str,
    input: &'static str,
    want: Option<&'static [Option<(usize, usize)>]>,
}

const PYTHON_BUILTINS: &str = "(?x)\n  (?<!\\.) \\b(\n    __import__ | abs | aiter | all | any | anext | ascii | bin\n    | breakpoint | callable | chr | compile | copyright | credits\n    | delattr | dir | divmod | enumerate | eval | exec | exit\n    | filter | format | getattr | globals | hasattr | hash | help\n    | hex | id | input | isinstance | issubclass | iter | len\n    | license | locals | map | max | memoryview | min | next\n    | oct | open | ord | pow | print | quit | range | reload | repr\n    | reversed | round | setattr | sorted | sum | vars | zip\n  )\\b\n";

const GO_KEYWORDS: &str = r"\b(break|case|continue|default|defer|else|fallthrough|for|go|goto|if|range|return|select|switch)\b";

#[test]
fn test_real_grammar_patterns() {
    let cases = [
        GrammarCase {
            name: "Go/language_constants_nil",
            pattern: r"\b(?:(true|false)|(nil)|(iota))\b",
            input: "if ready && count > 0 { return nil }",
            want: Some(&[Some((31, 34)), None, Some((31, 34)), None]),
        },
        GrammarCase {
            name: "Go/language_constants_true",
            pattern: r"\b(?:(true|false)|(nil)|(iota))\b",
            input: "var ok = true",
            want: Some(&[Some((9, 13)), Some((9, 13)), None, None]),
        },
        GrammarCase {
            name: "Go/line_comment",
            pattern: "//",
            input: "x := 42 // comment here",
            want: Some(&[Some((8, 10))]),
        },
        GrammarCase {
            name: "JS/trycatch_keywords",
            pattern: r"(?<![_$[:alnum:]])(?:(?<=\.\.\.)|(?<!\.))(catch|finally|throw|try)(?![_$[:alnum:]])(?:(?=\.\.\.)|(?!\.))",
            input: "try { fetch() } catch (e) { throw e }",
            want: Some(&[Some((0, 3)), Some((0, 3))]),
        },
        GrammarCase {
            name: "JS/jsx_assignment_operator",
            pattern: r#"=(?=\s*(?:'|"|\{|/\*|//|\n))"#,
            input: r#"className="active""#,
            want: Some(&[Some((9, 10))]),
        },
        GrammarCase {
            name: "Python/builtin_print",
            pattern: PYTHON_BUILTINS,
            input: r#"result = print("hello")"#,
            want: Some(&[Some((9, 14)), Some((9, 14))]),
        },
        GrammarCase {
            name: "Python/builtin_no_method_call",
            pattern: PYTHON_BUILTINS,
            input: r#"obj.print("hello")"#,
            want: None,
        },
        GrammarCase {
            name: "Python/decorator",
            pattern: "(?x)\n  ^\\s*\n  ((@)) \\s* (?=[[:alpha:]]\\w*)\n",
            input: "@staticmethod\ndef hello():",
            want: Some(&[Some((0, 1)), Some((0, 1)), Some((0, 1))]),
        },
        GrammarCase {
            name: "Go/no_match_keyword_in_identifier",
            pattern: GO_KEYWORDS,
            input: "forLoop := 1",
            want: None,
        },
        GrammarCase {
            name: "Go/keyword_in_source",
            pattern: GO_KEYWORDS,
            input: "for i := range items {",
            want: Some(&[Some((0, 3)), Some((0, 3))]),
        },
    ];

    for case in cases {
        let got = find(&[case.pattern], case.input, 0);
        match case.want {
            None => assert!(
                got.is_none(),
                "{}: expected no match, got {got:?}",
                case.name
            ),
            Some(want) => {
                let m = got.unwrap_or_else(|| panic!("{}: expected a match", case.name));
                assert_eq!(m.captures, want, "{}", case.name);
            }
        }
    }
}

#[test]
fn test_real_grammar_multi_pattern_scan() {
    let patterns = [
        GO_KEYWORDS,
        r"\bchan\b",
        r"\bconst\b",
        r"\bvar\b",
        r"\bfunc\b",
        r"\binterface\b",
    ];
    let m = find_some(&patterns, r#"func main() { var x = "hello" }"#, 0);
    assert_eq!(m.index, 4, "func is the leftmost keyword");
    assert_eq!(m.range(), (0, 4));
}

#[test]
fn test_z_anchor_rewrite() {
    assert_eq!(rewrite_z_anchor(r"\z"), r"$(?!\n)(?<!\n)");
    assert_eq!(rewrite_z_anchor(r"^start\z"), r"^start$(?!\n)(?<!\n)");
    assert_eq!(rewrite_z_anchor(r"\zmiddle"), r"$(?!\n)(?<!\n)middle");
    assert_eq!(
        rewrite_z_anchor(r"\z.*\z"),
        r"$(?!\n)(?<!\n).*$(?!\n)(?<!\n)"
    );
    assert_eq!(rewrite_z_anchor("^normal$"), "^normal$");
    assert_eq!(rewrite_z_anchor(r"\\z"), r"\\z");
    assert_eq!(rewrite_z_anchor(r"\\\z"), r"\\$(?!\n)(?<!\n)");
    assert_eq!(rewrite_z_anchor(r"\A\G\n\t"), r"\A\G\n\t");
    assert_eq!(rewrite_z_anchor(""), "");
    assert_eq!(rewrite_z_anchor(r"trailing\"), r"trailing\");
    assert_eq!(
        rewrite_z_anchor(r#"^(?:(?=(msg(?:id(_plural)?|ctxt))\s*"[^"])|\s*$).*\z"#),
        r#"^(?:(?=(msg(?:id(_plural)?|ctxt))\s*"[^"])|\s*$).*$(?!\n)(?<!\n)"#
    );
}

#[test]
fn test_z_rewrite_compiles_and_matches() {
    let rewritten = rewrite_z_anchor(r"foo\z");
    let m = find_some(&[&rewritten], "foo", 0);
    assert_eq!(m.range(), (0, 3));
    assert_eq!(find(&[&rewritten], "foo\n", 0), None);
    assert_eq!(find(&[r"foo\z"], "foo\n", 0), None);
}

#[test]
fn test_anchor_active_mapping() {
    assert_eq!(AnchorActive::new(true, Some(0), 0), AnchorActive::AG);
    assert_eq!(AnchorActive::new(true, Some(0), 3), AnchorActive::A);
    assert_eq!(AnchorActive::new(true, None, 0), AnchorActive::A);
    assert_eq!(AnchorActive::new(false, Some(3), 3), AnchorActive::G);
    assert_eq!(AnchorActive::new(false, Some(0), 3), AnchorActive::None);

    let both = SearchOptions::NOT_BEGIN_STRING.union(SearchOptions::NOT_BEGIN_POSITION);
    assert_eq!(AnchorActive::AG.to_search_options(), SearchOptions::NONE);
    assert_eq!(
        AnchorActive::A.to_search_options(),
        SearchOptions::NOT_BEGIN_POSITION
    );
    assert_eq!(
        AnchorActive::G.to_search_options(),
        SearchOptions::NOT_BEGIN_STRING
    );
    assert_eq!(AnchorActive::None.to_search_options(), both);
    assert!(both.contains(SearchOptions::NOT_BEGIN_STRING));
    assert!(both.contains(SearchOptions::NOT_BEGIN_POSITION));
    assert!(!SearchOptions::NONE.contains(SearchOptions::NOT_BEGIN_STRING));
}

fn go_grammar() -> serde_json::Value {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/grammars/go.json");
    let data = std::fs::read(path).expect("assets/grammars/go.json (run sync-assets)");
    serde_json::from_slice(&data).expect("go.json parses")
}

fn collect_patterns<'a>(value: &'a serde_json::Value, out: &mut Vec<&'a str>) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, child) in map {
                match child.as_str() {
                    Some(pattern) if key == "match" || key == "begin" => out.push(pattern),
                    _ => collect_patterns(child, out),
                }
            }
        }
        serde_json::Value::Array(items) => items.iter().for_each(|v| collect_patterns(v, out)),
        _ => {}
    }
}

#[test]
fn test_go_grammar_compiles_as_one_set() {
    let grammar = go_grammar();
    let mut patterns = Vec::new();
    collect_patterns(&grammar, &mut patterns);
    assert!(
        patterns.len() > 100,
        "collected only {} patterns",
        patterns.len()
    );
    let ps = PatternSet::new(&patterns).expect("every match and begin pattern of go.json compiles");
    assert_eq!(ps.len(), patterns.len());
}

#[test]
fn test_go_grammar_package_offsets() {
    let grammar = go_grammar();
    let rule = &grammar["repository"]["package_name"]["patterns"][0];
    let mut patterns = vec![rule["begin"].as_str().expect("package_name begin pattern")];
    for nested in rule["patterns"].as_array().expect("nested patterns") {
        if let Some(pattern) = nested["match"].as_str() {
            patterns.push(pattern);
        }
    }

    let line = "package main\n";
    let m = find_some(&patterns, line, 0);
    assert_eq!(m.index, 0, "the begin pattern wins at position 0");
    assert_eq!(m.captures[0], Some((0, 8)));
    assert_eq!(m.captures[1], Some((0, 7)));
    assert_eq!(&line[0..7], "package");
}

#[test]
fn test_not_begin_options_behavior() {
    let mut a = set(&[r"\A\w+"]);
    assert_eq!(
        a.find_next_match("hello", 0, SearchOptions::NONE)
            .map(|m| m.range()),
        Some((0, 5))
    );
    assert_eq!(
        a.find_next_match("hello", 0, SearchOptions::NOT_BEGIN_STRING),
        None
    );

    let mut g = set(&[r"\G\w"]);
    assert_eq!(
        g.find_next_match("abc", 1, SearchOptions::NONE)
            .map(|m| m.range()),
        Some((1, 2))
    );
    assert_eq!(
        g.find_next_match("abc", 1, SearchOptions::NOT_BEGIN_POSITION),
        None
    );

    let mut plain = set(&[r"\w+"]);
    assert_eq!(
        plain
            .find_next_match("abc", 1, AnchorActive::None.to_search_options())
            .map(|m| m.range()),
        Some((1, 3)),
        "options only disable anchors, unanchored patterns still match"
    );
}
