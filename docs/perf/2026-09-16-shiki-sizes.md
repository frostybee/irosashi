# Shiki comparison at three input sizes, 2026-09-16

Machine: Intel Core i9-10850K (10 cores, 20 threads), Windows 10 Education 10.0.19045,
rustc 1.93.0, `onig-regset` 6.7.0, `profile.bench` (release, debug = 1). Node.js 24.12.0,
Shiki 4.4.3. Irosashi at `94a37ba`. Theme `github-dark` throughout. Nothing else running
on the machine during either run.

Inputs are the ones in `crates/irosashi-fidelity/src/bench_inputs.rs`: small is the 10
inline snippets, medium is the fidelity fixture source
(`crates/irosashi-fidelity/testdata/golden/{grammar}__{grammar}.json`), large is medium
repeated until it is at least 50 KiB. `tools/shiki-bench/bench.mjs` reads the same fixture
files and repeats with the same byte rule, so both columns saw identical bytes (the byte
counts below are printed by both harnesses and match).

Commands:

```bash
cd tools/shiki-bench && node bench.mjs
cargo bench -p irosashi-fidelity --bench highlight -- "warm/tokens/"
cargo run -p bench-report
```

Irosashi numbers are Criterion medians (100 samples). Shiki numbers are medians of 50
`codeToTokens` calls (15 for large) after one warmup call on a shared highlighter. Ratio is
Irosashi / Shiki: above 1 means Shiki is faster.

## Small (inline snippets)

| Language | Bytes | Lines | Irosashi (ms) | Shiki (ms) | Ratio |
|---|---:|---:|---:|---:|---:|
| Go | 117 | 11 | 0.27 | 0.38 | 0.72 |
| JavaScript | 309 | 13 | 2.36 | 1.09 | 2.16 |
| HTML | 304 | 16 | 0.30 | 0.51 | 0.59 |
| TypeScript | 203 | 11 | 0.86 | 0.55 | 1.55 |
| Markdown | 135 | 12 | 0.27 | 0.21 | 1.26 |
| Python | 415 | 16 | 0.70 | 0.83 | 0.85 |
| Bash | 339 | 16 | 0.48 | 0.56 | 0.85 |
| PHP | 385 | 20 | 0.92 | 1.12 | 0.82 |
| CSS | 424 | 25 | 0.64 | 0.74 | 0.86 |
| Rust | 607 | 25 | 1.17 | 1.19 | 0.98 |

## Medium (fidelity fixture sources)

| Language | Bytes | Lines | Irosashi (ms) | Shiki (ms) | Ratio |
|---|---:|---:|---:|---:|---:|
| Go | 329 | 18 | 0.52 | 0.64 | 0.80 |
| JavaScript | 3946 | 151 | 15.14 | 8.16 | 1.85 |
| HTML | 1596 | 52 | 0.73 | 1.13 | 0.64 |
| TypeScript | 2205 | 77 | 7.81 | 3.95 | 1.98 |
| Markdown | 3725 | 170 | 2.49 | 1.39 | 1.79 |
| Python | 315 | 12 | 0.32 | 0.37 | 0.86 |
| Bash | 661 | 28 | 0.60 | 0.67 | 0.90 |
| PHP | 650 | 29 | 0.75 | 0.88 | 0.86 |
| CSS | 578 | 46 | 0.72 | 0.81 | 0.88 |
| Rust | 1011 | 39 | 0.92 | 0.93 | 0.99 |

## Large (medium repeated to 50 KiB)

| Language | Bytes | Lines | Irosashi (ms) | Shiki (ms) | Ratio |
|---|---:|---:|---:|---:|---:|
| Go | 51324 | 2653 | 79.4 | 88.8 | 0.89 |
| JavaScript | 51298 | 1963 | 198.0 | 105.9 | 1.87 |
| HTML | 52668 | 1716 | 24.6 | 35.6 | 0.69 |
| TypeScript | 52920 | 1848 | 190.2 | 96.3 | 1.97 |
| Markdown | 52150 | 2380 | 34.7 | 20.0 | 1.74 |
| Python | 51345 | 1794 | 46.3 | 54.1 | 0.86 |
| Bash | 51558 | 2184 | 46.6 | 52.3 | 0.89 |
| PHP | 51350 | 2291 | 61.4 | 69.4 | 0.88 |
| CSS | 51442 | 4094 | 64.8 | 71.0 | 0.91 |
| Rust | 51561 | 1989 | 47.4 | 46.8 | 1.01 |

## Reading

- The gap does not shrink with input size. JavaScript is 1.9x and TypeScript 2.0x at both
  medium and large, so the per-step overhead is not a fixed cost that long files amortise.
- Markdown joins them once the input has real structure: 1.3x on the 12-line snippet, 1.7
  to 1.8x on the fixture and the 50 KiB input. The three languages where Shiki wins are the
  three with the largest rule contexts (JavaScript and TypeScript) or the most injected
  patterns per line (Markdown's fenced blocks and inline grammars).
- Everywhere else Irosashi is 10 to 35 percent faster at every size, and Rust is even.
- This is consistent with the deferred-item explanation: vscode-oniguruma's per-pattern
  last-match cache re-searches only the patterns whose cached match is behind the cursor,
  and the saving scales with the number of patterns in the context, not with file length.

## Raw `bench-report` rows

| bench | median ms | mean ms | MB/s |
|---|---:|---:|---:|
| warm/tokens/bash/large | 46.588 | 46.854 | 1.06 |
| warm/tokens/bash/medium | 0.600 | 0.601 | 1.05 |
| warm/tokens/bash/small | 0.479 | 0.479 | 0.68 |
| warm/tokens/css/large | 64.767 | 64.426 | 0.76 |
| warm/tokens/css/medium | 0.718 | 0.724 | 0.77 |
| warm/tokens/css/small | 0.637 | 0.638 | 0.63 |
| warm/tokens/go/large | 79.420 | 79.727 | 0.62 |
| warm/tokens/go/medium | 0.518 | 0.520 | 0.61 |
| warm/tokens/go/small | 0.272 | 0.272 | 0.41 |
| warm/tokens/html/large | 24.646 | 24.692 | 2.04 |
| warm/tokens/html/medium | 0.727 | 0.751 | 2.09 |
| warm/tokens/html/small | 0.299 | 0.301 | 0.97 |
| warm/tokens/javascript/large | 197.994 | 198.304 | 0.25 |
| warm/tokens/javascript/medium | 15.135 | 15.146 | 0.25 |
| warm/tokens/javascript/small | 2.358 | 2.472 | 0.12 |
| warm/tokens/markdown/large | 34.736 | 34.708 | 1.43 |
| warm/tokens/markdown/medium | 2.487 | 2.492 | 1.43 |
| warm/tokens/markdown/small | 0.265 | 0.265 | 0.49 |
| warm/tokens/php/large | 61.418 | 61.861 | 0.80 |
| warm/tokens/php/medium | 0.753 | 0.757 | 0.82 |
| warm/tokens/php/small | 0.921 | 1.033 | 0.40 |
| warm/tokens/python/large | 46.344 | 46.317 | 1.06 |
| warm/tokens/python/medium | 0.316 | 0.324 | 0.95 |
| warm/tokens/python/small | 0.704 | 0.704 | 0.56 |
| warm/tokens/rust/large | 47.409 | 47.384 | 1.04 |
| warm/tokens/rust/medium | 0.924 | 0.923 | 1.04 |
| warm/tokens/rust/small | 1.166 | 1.165 | 0.50 |
| warm/tokens/typescript/large | 190.171 | 194.826 | 0.27 |
| warm/tokens/typescript/medium | 7.813 | 7.859 | 0.27 |
| warm/tokens/typescript/small | 0.856 | 0.856 | 0.23 |
