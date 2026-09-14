# iro

Shiki's output, in-process Rust, MIT licensed: VS Code grammars and themes, byte-identical
tokens and HTML, incremental per-line tokenization, and Typst output.

Iro is a Rust port of [Shiki](https://shiki.style): TextMate grammar-based syntax highlighting
with VS Code themes, verified against `vscode-textmate` fixtures. The workspace has three crates:

- `iro`: tokenizer, grammar compiler, theme resolution, token APIs, minimal HTML renderer.
- `kazari`: presentation layer on top of `iro` (fence meta, line numbers, markers, decorated
  HTML, Typst output, `kazari.config.yaml`).
- `iro-fidelity`: fixture comparison and scoring against `vscode-textmate` output.

## Status

Early development. The regex layer (native Oniguruma via `onig-regset`), the grammar model
and compiler, the tokenizer, and theme resolution are in place. All 32 core grammars are
byte-identical to `vscode-textmate` on `github-dark` and `github-light`. The public
`Highlighter` API, embedded assets, the fidelity gate, and the HTML renderer are next.
