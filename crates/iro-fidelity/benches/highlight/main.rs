use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::{Duration, Instant};

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use iro::{CodeToHtmlOptions, CodeToTokensOptions, Highlighter, TokenizeOptions};
use iro_fidelity::alloc::{CountingAlloc, measure};
use iro_fidelity::bench_inputs::{SMALL, large, medium};
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

fn tokens(h: &Highlighter, lang: &str, code: &str) {
    black_box(
        h.code_to_tokens(code, &CodeToTokensOptions::new(lang, THEME))
            .unwrap(),
    );
}

/// Allocation counts for the small snippets, printed once so every bench run has
/// them next to the timings.
fn print_allocations() {
    let h = highlighter();
    println!("| allocs (warm, small) | bytes in | allocs | KB |");
    println!("|---|---:|---:|---:|");
    for (lang, code) in SMALL {
        tokens(&h, lang, code);
        let (_, a) = measure(|| tokens(&h, lang, code));
        println!(
            "| tokens/{lang} | {} | {} | {:.1} |",
            code.len(),
            a.count,
            a.bytes as f64 / 1024.0
        );
    }
}

fn cold(c: &mut Criterion) {
    let mut group = c.benchmark_group("cold");
    group
        .sample_size(10)
        .measurement_time(Duration::from_secs(15));
    group.bench_function("highlighter_new", |b| {
        b.iter_batched(
            || (),
            |()| black_box(highlighter()),
            BatchSize::PerIteration,
        )
    });
    for (lang, code) in SMALL {
        group.throughput(Throughput::Bytes(code.len() as u64));
        group.bench_with_input(BenchmarkId::new("first_tokens", lang), &code, |b, code| {
            b.iter_custom(|iters| {
                let mut total = Duration::ZERO;
                for _ in 0..iters {
                    let h = highlighter();
                    let start = Instant::now();
                    tokens(&h, lang, code);
                    total += start.elapsed();
                    drop(h);
                }
                total
            })
        });
    }
    group.finish();
}

fn warm_tokens(c: &mut Criterion) {
    let h = highlighter();
    let mut group = c.benchmark_group("warm/tokens");
    for (lang, small) in SMALL {
        let med = medium(lang);
        let big = large(lang);
        for (size, code) in [
            ("small", small),
            ("medium", med.as_str()),
            ("large", big.as_str()),
        ] {
            tokens(&h, lang, code);
            group.throughput(Throughput::Bytes(code.len() as u64));
            group.bench_with_input(BenchmarkId::new(lang, size), &code, |b, code| {
                b.iter(|| tokens(&h, lang, code))
            });
        }
    }
    group.finish();
}

fn warm_multi(c: &mut Criterion) {
    let h = highlighter();
    let themes = dual();
    let mut group = c.benchmark_group("warm/tokens_multi");
    for (lang, code) in SMALL {
        let options = CodeToTokensOptions::new(lang, "");
        black_box(h.code_to_tokens_multi(code, &themes, &options).unwrap());
        group.throughput(Throughput::Bytes(code.len() as u64));
        group.bench_with_input(BenchmarkId::new(lang, "small"), &code, |b, code| {
            b.iter(|| black_box(h.code_to_tokens_multi(code, &themes, &options).unwrap()))
        });
    }
    group.finish();
}

fn warm_html(c: &mut Criterion) {
    let h = highlighter();
    let mut group = c.benchmark_group("warm/html");
    for (lang, code) in SMALL {
        let iro = CodeToHtmlOptions::new(lang, THEME);
        let shiki = CodeToHtmlOptions::new(lang, THEME).shiki();
        black_box(h.code_to_html(code, &iro).unwrap());
        black_box(h.code_to_html(code, &shiki).unwrap());
        group.throughput(Throughput::Bytes(code.len() as u64));
        group.bench_with_input(BenchmarkId::new("iro", lang), &code, |b, code| {
            b.iter(|| black_box(h.code_to_html(code, &iro).unwrap()))
        });
        group.bench_with_input(BenchmarkId::new("shiki", lang), &code, |b, code| {
            b.iter(|| black_box(h.code_to_html(code, &shiki).unwrap()))
        });
    }
    let (lang, code) = SMALL[3];
    let shiki_dual = CodeToHtmlOptions::multi(lang, dual()).shiki();
    black_box(h.code_to_html(code, &shiki_dual).unwrap());
    group.throughput(Throughput::Bytes(code.len() as u64));
    group.bench_with_input(BenchmarkId::new("shiki_dual", lang), &code, |b, code| {
        b.iter(|| black_box(h.code_to_html(code, &shiki_dual).unwrap()))
    });
    group.finish();
}

fn warm_tokenize_line(c: &mut Criterion) {
    let h = highlighter();
    let mut group = c.benchmark_group("warm/tokenize_line");
    for (lang, _) in SMALL {
        let med = medium(lang);
        let lines: Vec<&str> = iro::tokenizer::split_lines(&med)
            .into_iter()
            .map(|r| &med[r])
            .collect();
        let mut session = h.session(lang).unwrap();
        let run = |session: &mut iro::Session| {
            let mut state = session.initial_state();
            for (i, line) in lines.iter().enumerate() {
                let result =
                    session.tokenize_line(line, &state, i == 0, TokenizeOptions::default());
                black_box(&result.tokens);
                state = result.state;
            }
        };
        run(&mut session);
        group.throughput(Throughput::Bytes(med.len() as u64));
        group.bench_with_input(BenchmarkId::new(lang, "medium"), &(), |b, ()| {
            b.iter(|| run(&mut session))
        });
    }
    group.finish();
}

fn benches(c: &mut Criterion) {
    print_allocations();
    cold(c);
    warm_tokens(c);
    warm_multi(c);
    warm_html(c);
    warm_tokenize_line(c);
}

criterion_group!(highlight, benches);
criterion_main!(highlight);
