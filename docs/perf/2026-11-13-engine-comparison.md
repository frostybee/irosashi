# Irosashi, giallo and ferriki on one corpus (2026-11-13)

A one-off measurement of three Rust TextMate highlighters against Shiki, taken in one session
on one machine. It is a dated record, not a maintained benchmark: all three projects release
often, and the numbers apply only to the versions below.

| | Version | Licence | Regex engine |
|---|---|---|---|
| Irosashi | 0.3.2 | MIT | Ferroni 2.7.0 |
| [giallo](https://github.com/getzola/giallo) | 0.6.0 | EUPL-1.2 | fancy-regex |
| [ferriki](https://github.com/sebastian-software/ferriki) | 1.3.0 | MIT OR Apache-2.0 | Ferroni |
| Shiki (reference) | 3.19.0 | MIT | Oniguruma (WASM) |

Machine: Intel Core i9-10850K, Windows 10, rustc 1.99.0, Node 24.12.0. Release builds with
`lto = true` and `codegen-units = 1`.

## Method

The corpus and the reference follow a public evaluation,
[yuann3/yuanme#5](https://github.com/yuann3/yuanme/issues/5), so the result can be compared
with it:

- **Corpus:** the 52 code fences of the eyuan.me site at commit `1fcd0c5` (26 C, 15 Bash, 3 sh,
  1 Rust, 7 with no language) plus Shiki's samples for C, Rust, shell, Haskell, Lean, Coq,
  Julia, Python, LaTeX and OCaml from `shikijs/textmate-grammars-themes` at `37edd1b`. That
  is 62 blocks and 18,834 characters. The 55 blocks with a language are scored and timed.
- **Reference:** Shiki 3.19.0 with `catppuccin-latte` and `catppuccin-mocha` as a light and
  dark pair.
- **Same assets for every engine:** the grammar and theme JSON that Shiki 3.19 bundles (240
  grammars, taken from the yuanme repository and checked equal to Shiki's own for every
  grammar the corpus loads). Irosashi and giallo load the JSON directly. ferriki loads a
  catalog generated from the same JSON with `ferriki-asset-gen` 1.3.0.
- **Fidelity:** the share of non-whitespace characters whose light colour, dark colour and
  font style all equal Shiki's, and the number of lines whose token list equals Shiki's.
- **Timing:** one process per engine. Cold is the first pass over the 55 blocks. Warm is the
  median of 20 passes after 3 warm-up passes. Engines were run in turn, several rounds; the
  table gives the range across rounds.

## Fidelity

All three engines match Shiki 3.19 on every character.

| | Characters matching | Lines identical, raw | Lines identical, equal-style neighbours merged |
|---|---:|---:|---:|
| Irosashi | 13,518 / 13,518 | 381 / 687 | 687 / 687 |
| giallo | 13,518 / 13,518 | 381 / 687 | 687 / 687 |
| ferriki | 13,518 / 13,518 | 687 / 687 | 687 / 687 |

Shiki merges adjacent tokens that have the same style before it returns them, and ferriki's
dual-theme call does the same. Irosashi and giallo return the tokens unmerged, so their raw
lists differ from Shiki's until a renderer merges them.

## Speed

Milliseconds for one pass over the 55 blocks.

| | Irosashi | giallo | ferriki |
|---|---:|---:|---:|
| Warm, one theme, tokens | 9.0 to 9.2 | 7.8 to 8.7 | 6.1 to 6.6 |
| Warm, two themes, tokens | 9.2 to 9.5 | 8.2 to 9.0 | 72 to 77 |
| Warm, two themes, HTML | 14.7 to 15.0 | 10.5 to 11.3 | not measured |
| Cold first pass, two themes | 69 to 70 | 80 to 85 | 134 to 151 |
| Cold first pass, one theme | 65 to 69 | 80 to 92 | 71 to 74 |
| Loading the 240 grammars and 2 themes | 21 | 1,080 to 1,110 | 187 to 202 |

Shiki 3.19 in Node, in the same session, takes 114 to 117 ms for a warm two-theme HTML pass
over all 62 blocks.

What the table shows:

- With one theme the three engines are within a factor of 1.5. ferriki is the fastest and
  Irosashi the slowest.
- ferriki's two-theme call (`highlight_with_themes`) is about ten times slower than its
  one-theme call in this version. Its regex mode (`Off`, `Auto`, `Eager`, `Prepared`) does not
  change that.
- Irosashi spends about 5.5 ms of its 15 ms HTML pass in the renderer; giallo's renderer
  takes about 2 ms.
- Loading times are not like for like. Irosashi reads only metadata and parses a grammar when
  it is first used. giallo compiles every grammar when it is added; its `dump` feature exists
  to avoid that and was not used for timing. ferriki restores prepared packages.
- ferriki exposes its two-theme HTML renderer only through a hidden module, so it was not
  timed.

## Binary size

One stripped binary per engine that highlights one string, with the engine's own bundled
assets. An empty program is 0.12 MB.

| | Binary | Assets included |
|---|---:|---|
| Irosashi, `embedded-assets` | 3.88 MB | 257 grammars, 65 themes, deflated |
| Irosashi 0.3.1, for reference | 10.92 MB | the same, as raw JSON |
| giallo, `dump` feature | 4.02 MB | its built-in dump (220+ grammars, about 60 themes), zstd |

ferriki was not built with embedded assets. Its generated catalog for the 240 grammars is
6.9 MB of grammar files plus 92 MB of optional prepared regex packages; the yuanme project
reports that embedding the grammars adds about 6.7 MB.

giallo's `dump` feature links the `zstd` C library, so that binary needs a C compiler to
build. Irosashi and ferriki build without one.

## Limits of this record

- One machine, one corpus of 13,518 scored characters, dominated by C and shell.
- The corpus exercises 12 languages. It says nothing about the other grammars; Irosashi's own
  gate (`FIDELITY.md`) covers 234 grammars against `vscode-textmate`.
- The harness is not in this repository. It is a small Rust program that calls each engine's
  public API as described above.
