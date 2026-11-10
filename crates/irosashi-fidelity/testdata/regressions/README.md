# Regression fixtures

`vscode-textmate` fixtures for inputs the synced golden sets do not cover. `tools/sync-assets`
never touches this directory. Generate with Nuri's `tools/genfixtures`, assuming the `nuri`
and `iro` checkouts are siblings, from `nuri/tools/genfixtures`:

```bash
node generate.mjs --config ../../../iro/crates/irosashi-fidelity/testdata/regressions/samples/genfixtures.config.json
```

Every fixture here must be byte-identical (`cargo test -p irosashi-fidelity regressions`).
