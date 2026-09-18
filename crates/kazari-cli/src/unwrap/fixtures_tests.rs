use std::path::Path;

use serde::Deserialize;

use super::scan::{CandidateKind, discover};
use super::*;
use crate::html::tokenize;

/// Mirrors expected.json. `matched` distinguishes a region no unwrapper claims from zero
/// valued fields, and `skip` records which pre chain rule fired.
#[derive(Deserialize, Default)]
struct Expectation {
    #[serde(rename = "match")]
    matched: bool,
    #[serde(default)]
    lang: String,
    #[serde(default)]
    meta: String,
    #[serde(default)]
    code: String,
    #[serde(default)]
    skip: String,
}

fn fixture_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/unwrap")
}

fn fixture_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs: Vec<_> = std::fs::read_dir(fixture_root())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    dirs
}

/// The bytes of input.html are read exactly; nothing normalizes line endings, because the
/// CRLF fixture pins that raw carriage returns reach the tokenizer.
fn load(dir: &Path) -> (Vec<u8>, Expectation) {
    let src = std::fs::read(dir.join("input.html")).unwrap();
    let raw = std::fs::read_to_string(dir.join("expected.json")).unwrap();
    (src, serde_json::from_str(&raw).unwrap())
}

/// Runs the production sequence (discover, skip_reason, chain by kind, run_chain) on
/// every fixture and checks every field byte exactly.
#[test]
fn every_fixture_recovers_the_expected_region() {
    let dirs = fixture_dirs();
    assert_eq!(dirs.len(), 31);
    for dir in dirs {
        let name = dir.file_name().unwrap().to_string_lossy().into_owned();
        let (src, want) = load(&dir);
        let tokens = tokenize(&src);
        // Skip fixtures are checked the way the processor sees them: the whole fixture is
        // the candidate region, and a reserved class on the root is a skip, not a
        // suppression.
        if !want.skip.is_empty() {
            assert!(!want.matched, "{name}: skipped fixture must not match");
            assert_eq!(
                skip_reason(&tokens),
                Some(want.skip.as_str()),
                "{name}: skip reason"
            );
            continue;
        }
        let page = discover(&src, &tokens);
        let Some(c) = page.candidates.first() else {
            assert!(!want.matched, "{name}: no candidate found");
            continue;
        };
        assert!(!c.malformed, "{name}: malformed");
        assert_eq!(skip_reason(c.tokens), None, "{name}: unexpected skip");
        let chain = match c.kind {
            CandidateKind::Wrapper => DIV_WRAPPER_CHAIN,
            CandidateKind::BarePre => BARE_PRE_CHAIN,
        };
        match run_chain(chain, c.tokens) {
            None => assert!(!want.matched, "{name}: expected a match"),
            Some((region, unwrapper)) => {
                assert!(want.matched, "{name}: unexpected match by {unwrapper}");
                assert_eq!(region.lang, want.lang, "{name}: lang");
                assert_eq!(region.meta, want.meta, "{name}: meta");
                assert_eq!(region.code, want.code, "{name}: code");
            }
        }
    }
}

/// Every matched fixture rendered through both front doors gives the same bytes: the
/// hand authored code and meta straight through `render_with_meta`, and the fixture's
/// input.html through the processor. This is also the only place the recovered pairs
/// (synthesized hl ranges, meta overrides, NBSP indentation, empty code, empty language)
/// are fed through the real engine.
#[test]
fn parity_between_direct_render_and_processing() {
    use crate::process::fs::mem::MemFs;
    use crate::process::{Config, Processor};

    let engine = kazari_rs::Kazari::builder(irosashi::Highlighter::new().unwrap())
        .themes("github-light", Some("github-dark"))
        .build()
        .unwrap();
    for dir in fixture_dirs() {
        let name = dir.file_name().unwrap().to_string_lossy().into_owned();
        let (src, want) = load(&dir);
        if !want.matched {
            continue;
        }
        let direct = engine.render_with_meta(&want.code, &want.meta).unwrap();
        let fs = MemFs::default();
        fs.files
            .lock()
            .unwrap()
            .insert("site/index.html".into(), src.clone());
        let cfg = Config {
            engine: &engine,
            check: false,
            skip_unlabeled: false,
            assets_base: String::new(),
            hashed_assets: false,
            concurrency: 1,
            max_file_bytes: 0,
            logger: None,
            fs: &fs,
        };
        let r = Processor::new(cfg).run(Path::new("site")).unwrap();
        let f = &r.files[0];
        assert!(f.error.is_none(), "{name}: {:?}", f.error);
        assert_eq!(
            f.blocks_rewritten, 1,
            "{name}: blocks {:?}",
            f.blocks_skipped
        );
        let out = fs.get("site/index.html").unwrap();
        assert!(
            out.contains(&direct),
            "{name}: processed page does not contain the direct render"
        );
        assert!(
            !out.contains(String::from_utf8_lossy(&src).trim_end()),
            "{name}: original block left in place"
        );
    }
}
