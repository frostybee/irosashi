//! Per-grammar warm tokenization time over every golden-all fixture, for A/B runs.
//! `IRO_TIMING_OUT=path cargo test -p irosashi-fidelity --release --test all_grammars_timing -- --ignored`

use std::io::Write;
use std::time::Instant;

use irosashi::{CodeToTokensOptions, HighlighterBuilder};
use irosashi_fidelity::{assets_dir, golden_all_dir, load_fixtures};

#[test]
#[ignore]
fn all_grammars_timing() {
    let out = std::env::var("IRO_TIMING_OUT").expect("IRO_TIMING_OUT");
    let h = HighlighterBuilder::from_dir(assets_dir()).build().unwrap();
    let fixtures = load_fixtures(&golden_all_dir()).unwrap();
    let mut file = std::fs::File::create(out).unwrap();
    for f in &fixtures {
        let lang = f.grammar.as_str();
        let opts = CodeToTokensOptions::new(lang, "github-dark");
        if h.code_to_tokens(&f.source, &opts).is_err() {
            continue;
        }
        for _ in 0..3 {
            let _ = h.code_to_tokens(&f.source, &opts);
        }
        let reps = 20;
        let mut times = Vec::with_capacity(reps);
        for _ in 0..reps {
            let t = Instant::now();
            let _ = h.code_to_tokens(&f.source, &opts);
            times.push(t.elapsed().as_secs_f64() * 1e3);
        }
        times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        writeln!(file, "{lang},{},{:.4}", f.source.len(), times[reps / 2]).unwrap();
    }
}
