---
title: Installation
description: "Add irosashi to a Rust project and verify the build."
sidebar:
  order: 1
---

## Prerequisites

- **Rust 1.93** or later (stable toolchain).
- **A C compiler** for the vendored Oniguruma build (`onig-sys`). On Windows, install Visual Studio with the "Desktop development with C++" workload. On Linux and macOS, the system `cc` is enough.

## Add the crate

Add `irosashi` to your `Cargo.toml`:

```toml title="Cargo.toml"
[dependencies]
irosashi = "0.1"
```

To render decorated code blocks with frames, line numbers, markers, and Typst output, add `kazari-rs` as well:

```toml title="Cargo.toml"
[dependencies]
irosashi = "0.1"
kazari-rs = "0.1"
```

The `markdown` feature on `kazari-rs` pulls in `pulldown-cmark` for rendering fenced code blocks inside Markdown documents:

```toml title="Cargo.toml"
kazari-rs = { version = "0.1", features = ["markdown"] }
```

## Embedded assets

The `embedded-assets` feature is on by default. It compiles 257 grammars and 65 VS Code themes into the binary so `Highlighter::new()` works with no external files. The compressed size is roughly 1.5 MiB.

To load grammars and themes from a directory instead, disable the default feature and use `HighlighterBuilder::from_dir`:

```toml title="Cargo.toml"
[dependencies]
irosashi = { version = "0.1", default-features = false }
```

```rust
let highlighter = irosashi::HighlighterBuilder::from_dir("path/to/assets").build()?;
```

The directory must contain `grammars/{name}.json`, `grammars/index.json`, and `themes/{name}.json`. The `tools/sync-assets` workspace tool copies these from a Nuri checkout in the expected layout.

## Verify the build

```bash
cargo build
```

The first build compiles the vendored Oniguruma C library. Subsequent builds reuse the cached object files.

## Without Rust

If you want to use Irosashi and Kazari from the terminal without writing Rust, install the [`kazari` binary](/docs/getting-started/cli) instead:

```bash
cargo install kazari-cli
```

Prebuilt binaries are also available from [GitHub releases](https://github.com/frostybee/irosashi/releases).
