# gen-html-goldens

Generates the HTML goldens in `crates/irosashi/testdata/html/` from Shiki's `codeToHtml`. The
goldens are what `crates/irosashi/tests/html_goldens.rs` holds the Shiki preset of
`HtmlRenderer` byte-identical to.

Grammars and themes are read from `crates/irosashi/assets/`, so Shiki highlights the same
bytes irosashi loads. Sources come from the fidelity fixtures in
`crates/irosashi-fidelity/testdata/golden/` with the trailing newline removed, because Shiki
emits a final empty line for it and irosashi, like Nuri, does not.

Regenerating is a deliberate, reviewed step, like regenerating the fixtures:

```bash
cd tools/gen-html-goldens
npm ci
npm run generate
```

`manifest.json` records the Shiki version and, per case, the language, source and
options. Token dumps for each case land in `scratch/` (gitignored) to tell a tokenizer
difference from a renderer one.
