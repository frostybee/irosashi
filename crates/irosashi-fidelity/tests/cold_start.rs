//! Cold-start breakdown per language. Informational: run with
//! `cargo test -p irosashi-fidelity --release --test cold_start -- --ignored --nocapture`.

use std::time::Instant;

use irosashi::{HighlighterBuilder, TokenizeOptions};
use irosashi_fidelity::assets_dir;
use irosashi_fidelity::bench_inputs::{SMALL, medium};

fn ms(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1e3
}

#[test]
#[ignore]
fn cold_start_breakdown() {
    println!(
        "| lang | build ms | grammar parse ms | session ms | first line ms | rest of medium ms | compiled sets | scope lists | scan steps | warm rerun ms | us per step |"
    );
    println!("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|");
    for (lang, _) in SMALL {
        let med = medium(lang);
        let lines: Vec<&str> = irosashi::split_lines(&med)
            .into_iter()
            .map(|r| &med[r])
            .collect();

        let t = Instant::now();
        let h = HighlighterBuilder::from_dir(assets_dir()).build().unwrap();
        let build = ms(t);

        let t = Instant::now();
        h.registry().grammar(lang).unwrap();
        let parse = ms(t);

        let t = Instant::now();
        let mut session = h.session(lang).unwrap();
        let session_ms = ms(t);

        let mut state = session.initial_state();
        let t = Instant::now();
        let first = session.tokenize_line(lines[0], &state, true, TokenizeOptions::default());
        state = first.state;
        let first_line = ms(t);

        let t = Instant::now();
        for line in &lines[1..] {
            let result = session.tokenize_line(line, &state, false, TokenizeOptions::default());
            state = result.state;
        }
        let rest = ms(t);

        let stats = session.stats();
        let footprint = session.footprint();

        session.reset_stats();
        let t = Instant::now();
        let mut state = session.initial_state();
        for (i, line) in lines.iter().enumerate() {
            let result = session.tokenize_line(line, &state, i == 0, TokenizeOptions::default());
            state = result.state;
        }
        let warm = ms(t);
        let steps = session.stats().scan_steps;
        println!(
            "| {lang} | {build:.2} | {parse:.2} | {session_ms:.2} | {first_line:.2} | {rest:.2} | {} | {} | {steps} | {warm:.2} | {:.2} |",
            stats.memo_misses,
            footprint.scope_lists,
            warm * 1e3 / steps as f64
        );
    }
}
