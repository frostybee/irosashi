// Renders the golden cases with Shiki's codeToHtml, using irosashi's own grammar and theme
// JSON so the inputs are byte-for-byte the ones the Rust side loads.
import { readFileSync, readdirSync, writeFileSync, mkdirSync, rmSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHighlighterCore } from '@shikijs/core';
import { createOnigurumaEngine } from '@shikijs/engine-oniguruma';

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, '..', '..');
const grammarsDir = join(root, 'crates', 'irosashi', 'assets', 'grammars');
const themesDir = join(root, 'crates', 'irosashi', 'assets', 'themes');
const fixturesDir = join(root, 'crates', 'irosashi-fidelity', 'testdata', 'golden');
const outDir = join(root, 'crates', 'irosashi', 'testdata', 'html');
const scratchDir = join(here, 'scratch');

const SHIKI_VERSION = JSON.parse(
  readFileSync(join(here, 'node_modules', '@shikijs', 'core', 'package.json'), 'utf8'),
).version;

const GRAMMARS = ['json', 'javascript', 'rust', 'html', 'markdown'];
const DARK = 'github-dark';
const LIGHT = 'github-light';
const DUAL = { dark: DARK, light: LIGHT };
const ESCAPING_SOURCE =
  'const s = "</script>" + \'&amp;\' + `tick` + "a > b" + \'\\0\';\n' +
  '// <!-- "quotes" & \'apostrophes\' -->';

function readJson(path) {
  return JSON.parse(readFileSync(path, 'utf8'));
}

function fixtureSource(grammar) {
  const source = readJson(join(fixturesDir, `${grammar}__${grammar}.json`)).source;
  return source.endsWith('\n') ? source.slice(0, -1) : source;
}

function cases() {
  const list = [];
  for (const g of GRAMMARS) {
    const source = fixtureSource(g);
    list.push({ name: `${g}__${DARK}`, lang: g, source, theme: DARK });
    list.push({ name: `${g}__${LIGHT}`, lang: g, source, theme: LIGHT });
    list.push({ name: `${g}__themes`, lang: g, source, themes: DUAL, default_color: 'light' });
  }
  const json = fixtureSource('json');
  list.push({ name: 'json__themes-nodefault', lang: 'json', source: json, themes: DUAL, default_color: false });
  list.push({ name: 'json__themes-lightdark', lang: 'json', source: json, themes: DUAL, default_color: 'light-dark()' });
  list.push({ name: `json__${DARK}-nomerge`, lang: 'json', source: json, theme: DARK, merge_whitespace: false });
  list.push({ name: `escaping__${DARK}`, lang: 'javascript', source: ESCAPING_SOURCE, theme: DARK });
  list.push({ name: 'escaping__themes', lang: 'javascript', source: ESCAPING_SOURCE, themes: DUAL, default_color: 'light' });
  return list;
}

// Every bundled grammar is registered so embedded languages and injections resolve
// exactly as they do in irosashi's registry.
function allGrammars() {
  return readdirSync(grammarsDir)
    .filter((f) => f.endsWith('.json') && f !== 'index.json')
    .sort()
    .map((f) => {
      const grammar = readJson(join(grammarsDir, f));
      if (!grammar.name) grammar.name = f.slice(0, -'.json'.length);
      return grammar;
    });
}

async function main() {
  const langs = allGrammars();
  const themes = [DARK, LIGHT].map((t) => readJson(join(themesDir, `${t}.json`)));
  const highlighter = await createHighlighterCore({
    langs,
    themes,
    engine: createOnigurumaEngine(import('@shikijs/engine-oniguruma/wasm-inlined')),
  });

  rmSync(outDir, { recursive: true, force: true });
  mkdirSync(outDir, { recursive: true });
  mkdirSync(scratchDir, { recursive: true });

  const manifest = { shiki: SHIKI_VERSION, cases: [] };
  for (const c of cases()) {
    const options = { lang: c.lang };
    if (c.theme) options.theme = c.theme;
    if (c.themes) options.themes = c.themes;
    if (c.default_color !== undefined) options.defaultColor = c.default_color;
    if (c.merge_whitespace !== undefined) options.mergeWhitespaces = c.merge_whitespace;

    const html = highlighter.codeToHtml(c.source, options);
    writeFileSync(join(outDir, `${c.name}.html`), html, 'utf8');
    const tokens = highlighter.codeToTokens(c.source, options);
    writeFileSync(join(scratchDir, `${c.name}.tokens.json`), JSON.stringify(tokens, null, 1), 'utf8');
    manifest.cases.push(c);
    console.log(`${c.name}: ${html.length} bytes`);
  }
  writeFileSync(join(outDir, 'manifest.json'), JSON.stringify(manifest, null, 2) + '\n', 'utf8');
}

await main();
