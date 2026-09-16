---
title: Per-line tokenization
description: "Incremental tokenization with explicit state handles for editors and previews."
sidebar:
  order: 2
---

The `code_to_*` methods tokenize the entire input in one call. The `Session` API tokenizes one
line at a time, returning an explicit state handle that the caller stores. When the user edits
line 40 of a 200-line file, restart tokenization from the last clean state instead of
re-processing lines 1 through 39.

## Create a session

A session is scoped to one grammar. It compiles pattern sets lazily as new rule contexts are
encountered:

```rust
let hl = irosashi::Highlighter::new()?;
let mut session = hl.session("rust")?;
```

The session owns the scope interner and the compiled pattern cache. It is not `Send` or `Sync`:
each thread that tokenizes needs its own session.

## Tokenize a file line by line

`tokenize_line` takes one bare line (no trailing `\n`), a reference to the previous state, a
flag for whether this is the first line in the file, and a `TokenizeOptions` struct:

```rust
use irosashi::TokenizeOptions;

let mut state = session.initial_state();
for (i, line) in source.lines().enumerate() {
    let result = session.tokenize_line(line, &state, i == 0, TokenizeOptions::default());
    for token in &result.tokens {
        let scopes = session.scope_names(token.scopes);
        println!("  {:?} {:?}", &line[token.start..token.end], scopes);
    }
    if let Some(kind) = result.diagnostic {
        eprintln!("line {}: {kind:?}", i + 1);
    }
    state = result.state;
}
```

`initial_state()` returns the state before the first line. Each call returns a `LineResult`:

| Field | Type | Description |
|-------|------|-------------|
| `tokens` | `Vec<Token>` | Raw tokens with byte offsets and scope list IDs |
| `state` | `StateStack` | The state to pass into the next line |
| `diagnostic` | `Option<DiagnosticKind>` | `Some(TooLong)` when the line exceeded `max_line_length`, or a recovered panic |

`TokenizeOptions` has one field, `max_line_length: Option<usize>`, which overrides the
highlighter's default. Lines that exceed it are emitted as a single unstyled token.

## Manage state across edits

Store the `StateStack` returned by each line alongside the line content. When an edit changes
line N:

1. Find the highest line before N whose stored state is still valid. In most grammars, a
   single-line edit invalidates only the edited line and everything after it.
2. Restart `tokenize_line` from that state and re-tokenize forward until the returned state
   matches the previously stored state for a line. At that point, the rest of the file is
   unchanged.

`StateStack` implements `Clone`, `PartialEq`, and `Eq`, so the equality check is a direct
comparison.

## Resolve scope names

Raw `Token`s carry a `ScopeListId` instead of string scope names. Resolve them through the
session:

```rust
let names: Vec<&str> = session.scope_names(token.scopes);
// ["source.rust", "meta.function.rust", "keyword.other.fn.rust"]
```

`scope_names` returns references into the session's interner. For owned strings, use
`scopes_vec`.

## Apply a theme

The session can resolve theme colours directly. `Session::style` takes an `Arc<Theme>` and a
`ScopeListId` and returns a `TokenStyle` with the colour and font style for that scope stack.
For whole-file theming in one pass, `Session::themed` tokenizes the full buffer and resolves
one or more themes, returning a `TokensResult` identical to `Highlighter::code_to_tokens`:

```rust
use irosashi::{ThemeSlot, TokenizeOptions};

let themes = vec![ThemeSlot::new("dark", "github-dark")];
let result = session.themed(source, TokenizeOptions::default(), themes, false);
```

## Monitor session cost

Two diagnostics help tune cache behaviour:

```rust
let stats = session.stats();
println!("lines: {}, scan steps: {}, memo hits: {}", stats.lines, stats.scan_steps, stats.memo_hits);

let footprint = session.footprint();
println!("scope lists: {}, compiled sets: {}", footprint.scope_lists, footprint.compiled_sets);
```

`SessionStats` tracks lines tokenized, regex scan steps, memo cache hits and misses, and
injection searches. `SessionFootprint` reports the current size of the scope interner and the
compiled pattern set cache. Both are useful for benchmarking and for deciding when to retire
and recreate a long-lived session.

Call `session.reset_stats()` to zero the counters without discarding the compiled patterns.
