//! What sharing compiled patterns across sessions saves. Informational: run with
//! `cargo test -p irosashi-fidelity --release --test shared_store -- --ignored --nocapture`.

use std::time::Instant;

use irosashi::{HighlighterBuilder, TokenizeOptions};
use irosashi_fidelity::assets_dir;
use irosashi_fidelity::bench_inputs::{SMALL, medium};

fn run(h: &irosashi::Highlighter, lang: &str, code: &str) -> (f64, u64, u64) {
    let t = Instant::now();
    let mut session = h.session(lang).unwrap();
    session.tokenize(code, TokenizeOptions::default());
    let ms = t.elapsed().as_secs_f64() * 1e3;
    let stats = session.stats();
    (ms, stats.pattern_compiles, stats.pattern_shared_hits)
}

#[test]
#[ignore]
fn shared_store_breakdown() {
    println!("| lang | first ms | compiles | second session ms | compiles | shared |");
    println!("|---|---:|---:|---:|---:|---:|");
    for (lang, grammar, _) in SMALL {
        let code = medium(grammar);
        let h = HighlighterBuilder::from_dir(assets_dir()).build().unwrap();
        h.registry().grammar(lang).unwrap();
        let (first, compiles, _) = run(&h, lang, &code);
        let (second, again, shared) = run(&h, lang, &code);
        println!("| {lang} | {first:.2} | {compiles} | {second:.2} | {again} | {shared} |");
    }

    println!();
    println!(
        "| lang after javascript and css | alone ms | compiles | after ms | compiles | shared |"
    );
    println!("|---|---:|---:|---:|---:|---:|");
    for (lang, grammar, _) in SMALL {
        if !matches!(lang, "html" | "markdown" | "php" | "typescript") {
            continue;
        }
        let code = medium(grammar);
        let alone = HighlighterBuilder::from_dir(assets_dir()).build().unwrap();
        alone.registry().grammar(lang).unwrap();
        let (alone_ms, alone_compiles, _) = run(&alone, lang, &code);

        let warm = HighlighterBuilder::from_dir(assets_dir()).build().unwrap();
        for (other, other_grammar, _) in SMALL {
            if matches!(other, "javascript" | "css") {
                run(&warm, other, &medium(other_grammar));
            }
        }
        warm.registry().grammar(lang).unwrap();
        let (after_ms, compiles, shared) = run(&warm, lang, &code);
        println!(
            "| {lang} | {alone_ms:.2} | {alone_compiles} | {after_ms:.2} | {compiles} | {shared} |"
        );
    }
}
