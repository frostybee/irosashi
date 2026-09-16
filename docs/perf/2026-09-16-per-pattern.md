# Per-pattern scanner with a last-match cache, 2026-09-16

Machine: Intel Core i9-10850K (10 cores, 20 threads), Windows 10 Education 10.0.19045,
rustc 1.93.0, `onig_sys` 69.9.3 (Oniguruma 6.9.9), `profile.bench` (release, debug = 1).
Node.js 24.12.0, Shiki 4.4.3. Theme `github-dark` throughout. Quiet machine for every run.

"Regset" is the committed tree at `94a37ba` (one Oniguruma regset per rule context, one
native call per scan step), numbers from `2026-09-16-shiki-sizes.md`. "Per-pattern" is
this phase: one compiled `Regex` per distinct pattern source per session, searched
individually with vscode-oniguruma's per-pattern last-match cache, direct `onig_search`
calls (`crates/irosashi/src/regex/scanner.rs`, `raw.rs`). Shiki numbers are from
`tools/shiki-bench` on identical bytes. Inputs as in `bench_inputs.rs`.

Commands:

```bash
cargo bench -p irosashi-fidelity --bench highlight -- "warm/tokens/"
cargo bench -p irosashi-fidelity --bench highlight -- "cold/"
cargo run -p bench-report
cargo test -p irosashi-fidelity --release --test cold_start -- --ignored --nocapture
cargo test -p irosashi-fidelity --release --test alloc_audit -- --ignored --nocapture
cd tools/shiki-bench && node bench.mjs
```

## Warm, small (inline snippets)

| Language | Bytes | Lines | Regset (ms) | Per-pattern (ms) | Shiki (ms) | Per-pattern / Shiki |
|---|---:|---:|---:|---:|---:|---:|
| Go | 117 | 11 | 0.27 | 0.14 | 0.38 | 0.37 |
| JavaScript | 309 | 13 | 2.36 | 0.92 | 1.09 | 0.84 |
| HTML | 304 | 16 | 0.30 | 0.34 | 0.51 | 0.67 |
| TypeScript | 203 | 11 | 0.86 | 0.37 | 0.55 | 0.67 |
| Markdown | 135 | 12 | 0.27 | 0.22 | 0.21 | 1.05 |
| Python | 415 | 16 | 0.70 | 0.36 | 0.83 | 0.43 |
| Bash | 339 | 16 | 0.48 | 0.29 | 0.56 | 0.52 |
| PHP | 385 | 20 | 0.92 | 0.78 | 1.12 | 0.70 |
| CSS | 424 | 25 | 0.64 | 0.79 | 0.74 | 1.07 |
| Rust | 607 | 25 | 1.17 | 0.40 | 1.19 | 0.34 |

## Warm, medium (fidelity fixture sources)

| Language | Bytes | Lines | Regset (ms) | Per-pattern (ms) | Shiki (ms) | Per-pattern / Shiki |
|---|---:|---:|---:|---:|---:|---:|
| Go | 329 | 18 | 0.52 | 0.25 | 0.64 | 0.39 |
| JavaScript | 3946 | 151 | 15.14 | 7.55 | 8.16 | 0.93 |
| HTML | 1596 | 52 | 0.73 | 0.61 | 1.13 | 0.54 |
| TypeScript | 2205 | 77 | 7.81 | 3.73 | 3.95 | 0.94 |
| Markdown | 3725 | 170 | 2.49 | 2.33 | 1.39 | 1.68 |
| Python | 315 | 12 | 0.32 | 0.20 | 0.37 | 0.54 |
| Bash | 661 | 28 | 0.60 | 0.32 | 0.67 | 0.48 |
| PHP | 650 | 29 | 0.75 | 0.84 | 0.88 | 0.95 |
| CSS | 578 | 46 | 0.72 | 0.83 | 0.81 | 1.02 |
| Rust | 1011 | 39 | 0.92 | 0.44 | 0.93 | 0.47 |

## Warm, large (medium repeated to 50 KiB)

| Language | Bytes | Lines | Regset (ms) | Per-pattern (ms) | Shiki (ms) | Per-pattern / Shiki |
|---|---:|---:|---:|---:|---:|---:|
| Go | 51324 | 2653 | 79.4 | 39.0 | 88.8 | 0.44 |
| JavaScript | 51298 | 1963 | 198.0 | 98.4 | 105.9 | 0.93 |
| HTML | 52668 | 1716 | 24.6 | 20.2 | 35.6 | 0.57 |
| TypeScript | 52920 | 1848 | 190.2 | 89.1 | 96.3 | 0.93 |
| Markdown | 52150 | 2380 | 34.7 | 32.9 | 20.0 | 1.65 |
| Python | 51345 | 1794 | 46.3 | 28.1 | 54.1 | 0.52 |
| Bash | 51558 | 2184 | 46.6 | 24.6 | 52.3 | 0.47 |
| PHP | 51350 | 2291 | 61.4 | 68.2 | 69.4 | 0.98 |
| CSS | 51442 | 4094 | 64.8 | 72.8 | 71.0 | 1.03 |
| Rust | 51561 | 1989 | 47.4 | 22.2 | 46.8 | 0.47 |

## Cold start (fresh highlighter, first `code_to_tokens` on the small snippet)

`Highlighter::new()` is 1.47 ms in both trees. The regset Markdown number is the
committed tree re-measured today (56.5 ms); the 24.4 ms in the older docs predates the
injection work. The other regset numbers are from `2026-09-14-bench.md`.

| Language | Regset (ms) | Per-pattern (ms) | Shiki (ms) |
|---|---:|---:|---:|
| Go | 8.8 | 3.2 | 99.8 |
| JavaScript | 57.9 | 20.6 | 80.4 |
| HTML | 29.5 | 26.6 | 48.4 |
| TypeScript | 67.3 | 22.5 | 81.2 |
| Markdown | 56.5 | 44.6 | 9.5 |
| Python |  | 3.8 | 17.7 |
| Bash |  | 2.3 | 8.5 |
| PHP |  | 30.3 | 111.2 |
| CSS |  | 24.2 | 59.8 |
| Rust |  | 1.5 | 6.1 |

## Cache counters on the medium fixtures (`cold_start` test, warm rerun)

| Language | Scan steps | Compiled patterns | Pattern searches | Cache hits | Searches per step | us per step |
|---|---:|---:|---:|---:|---:|---:|
| Go | 110 | 102 | 1756 | 1998 | 16.0 | 2.6 |
| JavaScript | 1170 | 246 | 21621 | 51886 | 18.5 | 6.9 |
| HTML | 304 | 143 | 2364 | 1071 | 7.8 | 2.3 |
| TypeScript | 599 | 277 | 10619 | 23185 | 17.7 | 6.6 |
| Markdown | 357 | 240 | 16679 | 1829 | 46.7 | 5.8 |
| Python | 70 | 111 | 1011 | 2162 | 14.4 | 3.1 |
| Bash | 194 | 84 | 1739 | 1590 | 9.0 | 1.8 |
| PHP | 113 | 249 | 2925 | 1865 | 25.9 | 7.8 |
| CSS | 223 | 72 | 1393 | 641 | 6.2 | 3.6 |
| Rust | 168 | 84 | 3289 | 7840 | 19.6 | 3.1 |

Shiki per step on the same fixtures: JavaScript 7.0 us, TypeScript 6.6, Markdown 3.9.

## Reading

- Gate: 468/468 fidelity pairs byte-identical throughout. JavaScript and TypeScript
  large improved 2.0x and 2.1x (gate was 1.5x) and are now faster than Shiki at every
  size. Markdown improved 5 percent (gate missed) and stays 1.6x slower than Shiki. PHP
  and CSS regressed 11 and 12 percent against the regset and sit at parity with Shiki.
  Go, Python, Bash, Rust and HTML improved 18 to 53 percent.
- Three things were needed to get there, each measured on the way. (1) The plain port of
  the cache rule (never cache a pattern containing `\G`, key on the full options word)
  gave only 5 percent on JavaScript, because the anchor options flip at every begin
  match and evicted every cached entry twice around it, and because Markdown has 266
  patterns with `\G` out of 311. Keying each pattern only on the option bits it uses
  (`\A` on `NOT_BEGIN_STRING`, `\G` on `NOT_BEGIN_POSITION`) and caching `\G`
  patterns while `\G` is disabled, which is exactly vscode-textmate's four rewritten
  scanner variants, took JavaScript from 13.3 to 8.1 us per step. (2) The `onig` crate's
  search entry point costs 72 ns per call (a match-param allocation and an encoding
  check per call) against 19 ns for `onig_search` itself; with 18 to 47 searches per
  step that is 1 to 2.5 us, so the scan loop calls `onig_sys` directly. (3) Bounding
  each search to the best start so far, which is what makes the regset cheap on
  contexts with many patterns and few steps per line (Markdown, PHP, CSS), is not
  possible: `onig_search`'s range argument also caps where a match may end and broke
  fidelity to 190/468; the internal `search_in_range` with a separate data range is not
  exported. That is the residue on Markdown, PHP and CSS.
- Cold start: patterns shared by many rule contexts compile once per session instead of
  once per context. JavaScript and TypeScript first tokenization dropped from 58 and 67 ms
  to 21 and 23 ms; Go from 8.8 to 3.2. Markdown's 45 ms is mostly lazy parsing of the
  grammars it embeds (7.6 ms with every grammar preloaded).
- Allocations per call are unchanged from the regset path (JavaScript medium 1317
  allocations, 270 KB); the per-pattern caches reuse their capture vectors.

## All 234 grammars (fixture sources, warm, best of three runs per tree)

`IRO_TIMING_OUT=out.csv cargo test -p irosashi-fidelity --release --test all_grammars_timing -- --ignored`
on each tree (the committed tree via `git stash`). Medians of 20 calls after 3 warmups,
`code_to_tokens` on each grammar's `golden-all` fixture (0.2 to 4 KB).

| | Regset | Per-pattern |
|---|---:|---:|
| Sum over 234 fixtures (ms) | 194.4 | 142.6 |
| Median per-grammar ratio (per-pattern / regset) | | 0.76 |
| Faster by more than 5 percent | | 187 |
| Within 5 percent | | 13 |
| Slower by more than 5 percent | | 34 |

The 34 slower grammars, all under 1.7 ms absolute on these inputs:

| Grammar | Bytes | Regset (ms) | Per-pattern (ms) | Ratio |
|---|---:|---:|---:|---:|
| llvm | 916 | 0.371 | 0.943 | 2.54 |
| cmake | 1454 | 0.362 | 0.718 | 1.98 |
| kusto | 264 | 0.107 | 0.186 | 1.73 |
| fish | 418 | 0.130 | 0.204 | 1.57 |
| gn | 3980 | 0.590 | 0.890 | 1.51 |
| logo | 312 | 0.078 | 0.114 | 1.46 |
| mermaid | 826 | 0.256 | 0.370 | 1.45 |
| raku | 899 | 0.372 | 0.515 | 1.38 |
| stata | 1064 | 1.051 | 1.446 | 1.38 |
| apex | 721 | 0.616 | 0.844 | 1.37 |
| riscv | 952 | 0.298 | 0.402 | 1.35 |
| dax | 527 | 0.118 | 0.159 | 1.35 |
| docker | 473 | 0.050 | 0.067 | 1.35 |
| mipsasm | 617 | 0.106 | 0.141 | 1.33 |
| yaml | 1208 | 0.588 | 0.754 | 1.28 |
| csharp | 971 | 0.893 | 1.118 | 1.25 |
| sparql | 244 | 0.081 | 0.101 | 1.24 |
| css | 578 | 0.650 | 0.808 | 1.24 |
| dream-maker | 470 | 0.203 | 0.241 | 1.19 |
| cypher | 850 | 0.387 | 0.460 | 1.19 |
| java | 1050 | 0.999 | 1.184 | 1.18 |
| php | 650 | 0.721 | 0.847 | 1.17 |
| system-verilog | 504 | 0.844 | 0.972 | 1.15 |
| bat | 438 | 0.307 | 0.353 | 1.15 |
| emacs-lisp | 1065 | 1.088 | 1.243 | 1.14 |
| sql | 422 | 0.810 | 0.922 | 1.14 |
| pascal | 3638 | 1.470 | 1.669 | 1.14 |
| glimmer-ts | 245 | 0.303 | 0.331 | 1.09 |
| diff | 3107 | 0.222 | 0.242 | 1.09 |
| http | 466 | 0.089 | 0.097 | 1.09 |
| sas | 355 | 0.368 | 0.394 | 1.07 |
| dotenv | 248 | 0.048 | 0.051 | 1.06 |
| tsx | 506 | 0.715 | 0.756 | 1.06 |
| turtle | 851 | 0.096 | 0.101 | 1.05 |

The regressed grammars are not the ones with many patterns: llvm has 24 compiled patterns
and 5.6 searches per step, cmake 21 and 12.6. Their patterns are cheap to filter but
expensive to reject at each candidate position, so a full-line search per stale pattern
does work the regset's position-lead scan never did past the leftmost match. This is the
same missing bounded search as on Markdown, PHP and CSS. Largest improvements: wolfram
0.16x, apl 0.27x, jssm 0.29x, regexp 0.31x, codeql 0.35x, wikitext 0.37x, c 0.44x.
