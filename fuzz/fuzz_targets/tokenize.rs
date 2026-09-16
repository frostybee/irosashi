#![no_main]

use std::sync::{Arc, OnceLock};

use irosashi::fuzz_internals::Grammar;
use irosashi::{Session, TokenizeOptions, split_lines};
use libfuzzer_sys::fuzz_target;

const GRAMMARS: &[&[u8]] = &[
    include_bytes!("../../crates/irosashi/testdata/mini/begin_end.json"),
    include_bytes!("../../crates/irosashi/testdata/mini/begin_end_backref.json"),
    include_bytes!("../../crates/irosashi/testdata/mini/begin_while_eol.json"),
    include_bytes!("../../crates/irosashi/testdata/mini/begin_while_no_eol.json"),
    include_bytes!("../../crates/irosashi/testdata/mini/g_anchor.json"),
    include_bytes!("../../crates/irosashi/testdata/mini/injection_begin_end.json"),
    include_bytes!("../../crates/irosashi/testdata/mini/injection_negation.json"),
    include_bytes!("../../crates/irosashi/testdata/mini/injection_priority_left.json"),
    include_bytes!("../../crates/irosashi/testdata/mini/injection_priority_right.json"),
    include_bytes!("../../crates/irosashi/testdata/mini/injection_self.json"),
    include_bytes!("../../crates/irosashi/testdata/mini/match_only.json"),
    include_bytes!("../../crates/irosashi/testdata/mini/overlapping_captures.json"),
    include_bytes!("../../crates/irosashi/assets/grammars/json.json"),
];

fn grammars() -> &'static [Arc<Grammar>] {
    static PARSED: OnceLock<Vec<Arc<Grammar>>> = OnceLock::new();
    PARSED.get_or_init(|| {
        GRAMMARS
            .iter()
            .map(|bytes| Arc::new(Grammar::parse(bytes).expect("bundled grammar parses")))
            .collect()
    })
}

fuzz_target!(|data: &[u8]| {
    let Some((&selector, code)) = data.split_first() else {
        return;
    };
    let Ok(code) = std::str::from_utf8(code) else {
        return;
    };
    let grammar = Arc::clone(&grammars()[selector as usize % GRAMMARS.len()]);
    let mut session = Session::new(grammar, Arc::new(()));
    let result = session.tokenize(code, TokenizeOptions::default());
    let lines = split_lines(code);
    assert_eq!(result.lines.len(), lines.len());
    for (range, tokens) in lines.into_iter().zip(&result.lines) {
        let line = &code[range];
        let mut pos = 0;
        for token in tokens {
            assert_eq!(token.start, pos);
            assert!(token.end > token.start);
            assert!(line.is_char_boundary(token.end));
            pos = token.end;
        }
        assert_eq!(pos, line.len());
    }
});
