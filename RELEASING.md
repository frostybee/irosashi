# Releasing

Steps for publishing the crates and the `kazari` binaries. Run them in order from the
repository root.

## 1. Check the workspace

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

CI runs the same three commands on Windows, Linux and macOS for every push to `main`.

## 2. Run the full fidelity gate

CI checks the 32-grammar core gate only. The full matrix (234 grammars, two themes) needs
30 MB of fixtures that are not in the repository, so it runs locally before every release.

If `crates/irosashi-fidelity/testdata/golden-all` is missing, copy it from a Nuri checkout:

```bash
cargo run -p sync-assets -- --nuri-root <path-to-nuri>
```

Then run the gate:

```bash
IRO_REQUIRE_GOLDEN_ALL=1 cargo test -p irosashi-fidelity --release -- --ignored golden_all --nocapture
```

The output must contain `468/468 triples identical, 234/234 grammars` and `failing: []`.
With `IRO_REQUIRE_GOLDEN_ALL` set, the test fails when the fixtures are missing. Without
it, the test skips and still reports `ok`.

If grammars, themes or fixtures changed since the last release, regenerate the report and
commit it:

```bash
IRO_WRITE_REPORT=1 cargo test -p irosashi-fidelity
```

## 3. Set the versions

- Bump `version` in the `Cargo.toml` of every crate that changed: `crates/irosashi`,
  `crates/kazari-rs`, `crates/kazari-cli`. A breaking change in a `0.x` crate raises the
  minor version.
- Update the dependency versions that point at a bumped crate: `irosashi` in
  `crates/kazari-rs/Cargo.toml`, and `irosashi` and `kazari-rs` in
  `crates/kazari-cli/Cargo.toml`.
- In `CHANGELOG.md`, move the entries under `Unreleased` to a new heading with the version
  and the date.
- Commit.

## 4. Publish to crates.io

Dry run first, then publish in dependency order. Each crate must be live on crates.io
before the next one is published.

```bash
cargo publish --dry-run -p irosashi
cargo publish -p irosashi
cargo publish -p kazari-rs
cargo publish -p kazari-cli
```

Known issue: for 0.1.0, the 1.5 MiB `irosashi` upload failed three times from Windows
cargo (HTTP 503, then `STREAM_CLOSED`). Publishing from WSL with
`CARGO_TARGET_DIR=/tmp/iro-target` worked.

## 5. Tag

Tags follow the `irosashi` version (`v0.1.2` so far).

```bash
git tag v<version>
git push origin v<version>
```

The tag starts `.github/workflows/release.yml`, which builds `kazari` and creates the
GitHub release.

## 6. Check the GitHub release

When the workflow finishes, the release must list five archives, each with a `.sha256`
file:

- `x86_64-pc-windows-msvc` (`.zip`)
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`

The build matrix uses `fail-fast: false`, so one failed target does not stop the others.
A missing archive means its job failed: open the workflow run to see why.
