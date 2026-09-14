//! Allocation counts per warm call. Informational: run with
//! `cargo test -p iro-fidelity --release --test alloc_audit -- --ignored --nocapture`.

use std::collections::BTreeMap;

use iro::{CodeToHtmlOptions, CodeToTokensOptions, Highlighter, TokenizeOptions};
use iro_fidelity::alloc::{CountingAlloc, measure};
use iro_fidelity::bench_inputs::{SMALL, medium};
use iro_fidelity::highlighter;

#[global_allocator]
static ALLOC: CountingAlloc = CountingAlloc;

const THEME: &str = "github-dark";

fn dual() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("dark".to_owned(), "github-dark".to_owned()),
        ("light".to_owned(), "github-light".to_owned()),
    ])
}

fn row(name: &str, bytes: usize, f: impl FnOnce()) {
    let (_, a) = measure(f);
    println!(
        "| {name} | {bytes} | {} | {:.1} |",
        a.count,
        a.bytes as f64 / 1024.0
    );
}

fn warm(h: &Highlighter, lang: &str, code: &str) {
    h.code_to_tokens(code, &CodeToTokensOptions::new(lang, THEME))
        .unwrap();
    h.code_to_tokens_multi(code, &dual(), &CodeToTokensOptions::new(lang, ""))
        .unwrap();
    h.code_to_html(code, &CodeToHtmlOptions::new(lang, THEME))
        .unwrap();
    h.code_to_html(code, &CodeToHtmlOptions::new(lang, THEME).shiki())
        .unwrap();
}

#[test]
#[ignore]
fn allocations_per_call() {
    let h = highlighter();
    println!("| call | bytes in | allocs | KB |");
    println!("|---|---:|---:|---:|");
    for (lang, small) in SMALL {
        let med = medium(lang);
        for (size, code) in [("small", small), ("medium", med.as_str())] {
            warm(&h, lang, code);
            row(&format!("tokens/{lang}/{size}"), code.len(), || {
                h.code_to_tokens(code, &CodeToTokensOptions::new(lang, THEME))
                    .unwrap();
            });
            row(&format!("tokens_multi/{lang}/{size}"), code.len(), || {
                h.code_to_tokens_multi(code, &dual(), &CodeToTokensOptions::new(lang, ""))
                    .unwrap();
            });
            row(&format!("html_iro/{lang}/{size}"), code.len(), || {
                h.code_to_html(code, &CodeToHtmlOptions::new(lang, THEME))
                    .unwrap();
            });
            row(&format!("html_shiki/{lang}/{size}"), code.len(), || {
                h.code_to_html(code, &CodeToHtmlOptions::new(lang, THEME).shiki())
                    .unwrap();
            });
        }
        let mut session = h.session(lang).unwrap();
        let lines: Vec<&str> = iro::split_lines(&med)
            .into_iter()
            .map(|r| &med[r])
            .collect();
        let run = |session: &mut iro::Session| {
            let mut state = session.initial_state();
            for (i, line) in lines.iter().enumerate() {
                let result =
                    session.tokenize_line(line, &state, i == 0, TokenizeOptions::default());
                state = result.state;
            }
        };
        run(&mut session);
        let (_, a) = measure(|| run(&mut session));
        println!(
            "| tokenize_line/{lang}/medium (per line) | {} | {:.1} | {:.2} |",
            med.len(),
            a.count as f64 / lines.len() as f64,
            a.bytes as f64 / 1024.0 / lines.len() as f64
        );
    }
}
