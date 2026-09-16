#![no_main]

use std::sync::Arc;

use irosashi::fuzz_internals::{Grammar, ROOT_RULE_ID, compile_patterns};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(grammar) = Grammar::parse(data) else {
        return;
    };
    let grammar = Arc::new(grammar);
    let _ = compile_patterns(&grammar, ROOT_RULE_ID, &grammar, &());
    for &id in grammar.repository.0.values() {
        let _ = compile_patterns(&grammar, id, &grammar, &());
    }
});
