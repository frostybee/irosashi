use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use iro::grammar::{Grammar, ROOT_RULE_ID, Rule, RuleId, compile_patterns};

fn grammars_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/grammars")
}

fn load_all() -> Vec<(String, Arc<Grammar>)> {
    let mut entries: Vec<PathBuf> = fs::read_dir(grammars_dir())
        .expect("assets/grammars (run sync-assets)")
        .map(|e| e.unwrap().path())
        .filter(|p| {
            p.extension().is_some_and(|e| e == "json") && p.file_name().unwrap() != "index.json"
        })
        .collect();
    entries.sort();
    entries
        .iter()
        .map(|path| {
            let name = path.file_stem().unwrap().to_str().unwrap().to_owned();
            let data = fs::read(path).unwrap();
            let grammar = Grammar::parse(&data).unwrap_or_else(|err| panic!("{name}: {err}"));
            (name, Arc::new(grammar))
        })
        .collect()
}

#[test]
fn all_grammars_parse_and_match_the_index() {
    let index: serde_json::Value =
        serde_json::from_slice(&fs::read(grammars_dir().join("index.json")).unwrap()).unwrap();
    let grammars = load_all();
    assert_eq!(grammars.len(), 257);
    for (name, grammar) in &grammars {
        let expected = index["grammars"][name]["scopeName"].as_str();
        assert_eq!(expected, Some(grammar.scope_name.as_str()), "{name}");
        assert!(
            matches!(grammar.rule(ROOT_RULE_ID), Rule::Collection { .. }),
            "{name}"
        );
        assert!(grammar.rules.len() > 1, "{name} has no rules");
    }
}

#[test]
fn all_regexes_compile() {
    let mut compiled = 0;
    let mut skipped_backrefs = 0;
    let mut failures = Vec::new();
    for (name, grammar) in load_all() {
        for (index, rule) in grammar.rules.iter().enumerate() {
            let mut check = |label: &str, regex: &iro::regex::LazyRegex| {
                if regex.compiled().is_some() {
                    compiled += 1;
                } else {
                    failures.push(format!("{name} rule {index} {label}: {:?}", regex.source()));
                }
            };
            match rule {
                Rule::Match { regex, .. } => check("match", regex),
                Rule::BeginEnd {
                    begin,
                    end,
                    end_has_backrefs,
                    ..
                } => {
                    check("begin", begin);
                    if *end_has_backrefs {
                        skipped_backrefs += 1;
                    } else {
                        check("end", end);
                    }
                }
                Rule::BeginWhile {
                    begin,
                    while_,
                    while_has_backrefs,
                    ..
                } => {
                    check("begin", begin);
                    if *while_has_backrefs {
                        skipped_backrefs += 1;
                    } else {
                        check("while", while_);
                    }
                }
                _ => {}
            }
        }
    }
    eprintln!("regexes compiled: {compiled}, end/while with backrefs skipped: {skipped_backrefs}");
    assert!(
        failures.is_empty(),
        "{} patterns failed to compile:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn all_rule_contexts_compile() {
    let grammars = load_all();
    let resolver: HashMap<String, Arc<Grammar>> = grammars
        .iter()
        .map(|(_, g)| (g.scope_name.clone(), Arc::clone(g)))
        .collect();
    let mut roots = 0;
    let mut contexts = 0;
    let mut empty_roots = Vec::new();
    for (name, grammar) in &grammars {
        let root = compile_patterns(grammar, ROOT_RULE_ID, grammar, &resolver)
            .unwrap_or_else(|err| panic!("{name} root: {err}"));
        roots += 1;
        if root.is_empty() {
            empty_roots.push(name.clone());
        }
        for id in 0..grammar.rules.len() {
            let id = RuleId(id as u32);
            if !grammar.rule(id).patterns().is_empty() {
                compile_patterns(grammar, id, grammar, &resolver)
                    .unwrap_or_else(|err| panic!("{name} rule {}: {err}", id.0));
                contexts += 1;
            }
        }
    }
    eprintln!(
        "roots compiled: {roots}, rule contexts compiled: {contexts}, empty roots: {empty_roots:?}"
    );
    assert!(
        empty_roots.is_empty(),
        "grammars with an empty root scanner: {empty_roots:?}"
    );
}

#[test]
fn all_injection_selectors_parse() {
    let mut count = 0;
    for (name, grammar) in load_all() {
        for injection in &grammar.injections {
            let composite = &injection.selector.composites[0];
            assert!(
                !composite.expressions.is_empty(),
                "{name}: {:?}",
                injection.raw_selector
            );
            count += 1;
        }
    }
    eprintln!("injection selectors parsed: {count}");
    assert!(count > 0, "no injections found");
}
