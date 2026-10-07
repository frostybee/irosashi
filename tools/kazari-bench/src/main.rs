//! Times the `kazari` binary on generated inputs with each backend and prints a
//! markdown table of medians. The inputs come from the fidelity fixtures so the
//! numbers can be reproduced on any checkout:
//!
//! - `small.rs`: the Rust fixture repeated to at least 2.5 KB
//! - `large.rs`: the Rust fixture repeated to at least 45 KB
//! - `site/`: 60 pages with four `pre > code.language-*` blocks each (Rust, Python,
//!   JavaScript, Go), the generic shape `kazari process` recognizes
//!
//! Run: `cargo run -p kazari-bench --release -- --kazari target/release/kazari.exe`

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

const SMALL_BYTES: usize = 2_500;
const LARGE_BYTES: usize = 45 * 1024;
const PAGES: usize = 60;
const PAGE_LANGS: [&str; 4] = ["rust", "python", "javascript", "go"];
const ENGINES: [&str; 2] = ["irosashi", "syntect"];
const WARMUP: usize = 1;
const RUNS: usize = 7;

fn main() {
    let mut args = std::env::args().skip(1);
    let mut kazari = PathBuf::from("target/release/kazari.exe");
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--kazari" => kazari = PathBuf::from(args.next().expect("--kazari <path>")),
            other => {
                eprintln!("unknown argument {other}");
                std::process::exit(2);
            }
        }
    }
    if !kazari.is_file() {
        eprintln!("kazari binary not found at {}", kazari.display());
        std::process::exit(2);
    }

    let work = std::env::temp_dir().join("kazari-bench");
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(&work).expect("create work dir");

    let rust = fixture_source("rust");
    let small = work.join("small.rs");
    let large = work.join("large.rs");
    fs::write(&small, repeat_to(&rust, SMALL_BYTES)).unwrap();
    fs::write(&large, repeat_to(&rust, LARGE_BYTES)).unwrap();
    let site = work.join("site");
    write_site(&site);

    let version = run_capture(&kazari, &["version"]);
    println!("kazari-bench: {}", version.trim());
    println!(
        "inputs: small.rs {} B, large.rs {} B, site: {PAGES} pages x {} blocks",
        fs::metadata(&small).unwrap().len(),
        fs::metadata(&large).unwrap().len(),
        PAGE_LANGS.len()
    );
    println!(
        "{WARMUP} warm-up run and {RUNS} timed runs per cell, wall clock of the child process, median\n"
    );
    println!("| Command | Input | Irosashi (ms) | syntect (ms) |");
    println!("|---|---|---:|---:|");

    let cases: [(&str, Vec<String>, &str); 3] = [
        (
            "`kazari render`",
            vec!["render".into(), small.to_string_lossy().into_owned()],
            "2.5 KB Rust file",
        ),
        (
            "`kazari render`",
            vec!["render".into(), large.to_string_lossy().into_owned()],
            "45 KB Rust file",
        ),
        (
            "`kazari process --check`",
            vec![
                "process".into(),
                site.to_string_lossy().into_owned(),
                "--check".into(),
            ],
            "60 pages, 4 blocks each",
        ),
    ];
    for (command, base, input) in &cases {
        let mut cells = Vec::new();
        for engine in ENGINES {
            let mut argv: Vec<&str> = base.iter().map(String::as_str).collect();
            argv.push("--engine");
            argv.push(engine);
            cells.push(median_ms(&kazari, &argv));
        }
        println!(
            "| {command} | {input} | {:.0} | {:.0} |",
            cells[0], cells[1]
        );
    }
}

fn fixture_source(grammar: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/irosashi-fidelity/testdata/golden")
        .join(format!("{grammar}__{grammar}.json"));
    let json: serde_json::Value = serde_json::from_slice(
        &fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display())),
    )
    .expect("fixture json");
    json["source"].as_str().expect("fixture source").to_owned()
}

fn repeat_to(unit: &str, bytes: usize) -> String {
    let mut out = String::with_capacity(bytes + unit.len());
    while out.len() < bytes {
        out.push_str(unit);
    }
    out
}

fn write_site(site: &Path) {
    fs::create_dir_all(site).unwrap();
    let sources: Vec<(&str, String)> = PAGE_LANGS
        .iter()
        .map(|lang| (*lang, escape(&fixture_source(lang))))
        .collect();
    for page in 0..PAGES {
        let mut html = String::from(
            "<!DOCTYPE html>\n<html lang=\"en\">\n<head><meta charset=\"utf-8\"><title>Page</title></head>\n<body>\n",
        );
        for (lang, source) in &sources {
            html.push_str(&format!(
                "<p>Block</p>\n<pre><code class=\"language-{lang}\">{source}</code></pre>\n"
            ));
        }
        html.push_str("</body>\n</html>\n");
        fs::write(site.join(format!("page-{page:02}.html")), html).unwrap();
    }
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn run_capture(kazari: &Path, args: &[&str]) -> String {
    let out = Command::new(kazari)
        .args(args)
        .output()
        .expect("run kazari");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn median_ms(kazari: &Path, args: &[&str]) -> f64 {
    let mut times = Vec::with_capacity(RUNS);
    for i in 0..WARMUP + RUNS {
        let start = Instant::now();
        let status = Command::new(kazari)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .status()
            .expect("run kazari");
        let elapsed = start.elapsed().as_secs_f64() * 1e3;
        // `process --check` exits 1 when pages would change, which is the expected result.
        assert!(
            status.code().is_some_and(|c| c <= 1),
            "kazari {} failed with {status}",
            args.join(" ")
        );
        if i >= WARMUP {
            times.push(elapsed);
        }
    }
    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    times[times.len() / 2]
}
