use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use super::fs::mem::MemFs;
use super::*;

fn engine() -> &'static Kazari {
    static ENGINE: OnceLock<Kazari> = OnceLock::new();
    ENGINE.get_or_init(|| {
        Kazari::builder(irosashi::Highlighter::new().unwrap())
            .themes("github-light", Some("github-dark"))
            .build()
            .unwrap()
    })
}

fn config<'a>(fs: &'a MemFs) -> Config<'a> {
    Config {
        engine: engine(),
        check: false,
        skip_unlabeled: false,
        assets_base: String::new(),
        hashed_assets: false,
        concurrency: 2,
        max_file_bytes: 0,
        logger: None,
        fs,
    }
}

const PAGE: &str = "<!DOCTYPE html><html><head><title>x</title></head><body>\n<pre><code class=\"language-go\">package main</code></pre>\n</body></html>";

#[test]
fn rewrites_blocks_writes_assets_and_injects_tags() {
    let fs = MemFs::with(&[
        ("site/index.html", PAGE),
        ("site/sub/dir/page.html", PAGE),
        ("site/notes.txt", "<pre>x</pre>"),
    ]);
    let mut p = Processor::new(config(&fs));
    let r = p.run(Path::new("site")).unwrap();
    assert_eq!(r.assets.len(), 2);
    assert!(r.assets.iter().all(|a| a.action == AssetAction::Created));
    assert_eq!(r.files.len(), 2);
    assert_eq!(r.changed_count, 4);
    for f in &r.files {
        assert_eq!(f.blocks_found, 1);
        assert_eq!(f.blocks_rewritten, 1);
        assert!(f.changed);
        assert!(f.error.is_none());
    }
    let css = fs.get("site/kazari.css").unwrap();
    assert_eq!(css, engine().css());
    let a = engine().assets();
    let index = fs.get("site/index.html").unwrap();
    assert!(
        index.contains(&format!(
            "<link rel=\"stylesheet\" href=\"kazari.css?v={}\" data-kazari=\"assets\"></head>",
            a.css.hash
        )),
        "{index}"
    );
    assert!(index.contains(&format!(
        "<script src=\"kazari.js?v={}\" data-kazari=\"assets\"></script></body>",
        a.js.hash
    )));
    assert!(index.contains("kazari-block"));
    assert!(!index.contains("<pre><code class=\"language-go\">"));
    let deep = fs.get("site/sub/dir/page.html").unwrap();
    assert!(deep.contains("href=\"../../kazari.css?v="), "{deep}");
    assert!(deep.contains("src=\"../../kazari.js?v="));

    // A second run is a complete no-op.
    let writes_before = fs.writes.lock().unwrap().len();
    let mut p = Processor::new(config(&fs));
    let r = p.run(Path::new("site")).unwrap();
    assert_eq!(r.changed_count, 0);
    assert!(
        r.files
            .iter()
            .all(|f| !f.changed && f.blocks_rewritten == 0 && f.blocks_found == 0)
    );
    assert_eq!(r.suppressed, 2);
    assert_eq!(fs.writes.lock().unwrap().len(), writes_before);
}

#[test]
fn check_mode_reports_without_writing() {
    let fs = MemFs::with(&[("site/index.html", PAGE)]);
    let mut cfg = config(&fs);
    cfg.check = true;
    let r = Processor::new(cfg).run(Path::new("site")).unwrap();
    assert_eq!(r.changed_count, 3);
    assert!(r.files[0].changed);
    assert!(fs.writes.lock().unwrap().is_empty());
    assert_eq!(fs.get("site/index.html").unwrap(), PAGE);
}

#[test]
fn stale_tags_are_reconciled_on_pages_without_blocks() {
    let page = "<html><head><link rel=\"stylesheet\" href=\"kazari.css?v=stale000\" data-kazari=\"assets\"></head><body><p>none</p><script src=\"kazari.js?v=stale000\" data-kazari=\"assets\"></script></body></html>";
    let fs = MemFs::with(&[("site/index.html", page)]);
    let r = Processor::new(config(&fs)).run(Path::new("site")).unwrap();
    assert!(r.files[0].changed);
    assert_eq!(r.files[0].blocks_found, 0);
    let out = fs.get("site/index.html").unwrap();
    let a = engine().assets();
    assert!(out.contains(&format!("kazari.css?v={}", a.css.hash)));
    assert!(out.contains(&format!("kazari.js?v={}", a.js.hash)));
    assert!(!out.contains("stale000"));
    assert_eq!(out.matches("data-kazari=\"assets\"").count(), 2);
}

#[test]
fn hashed_assets_and_assets_base() {
    let fs = MemFs::with(&[("site/a/index.html", PAGE)]);
    let mut cfg = config(&fs);
    cfg.hashed_assets = true;
    cfg.assets_base = "https://cdn.example/static/".into();
    let r = Processor::new(cfg).run(Path::new("site")).unwrap();
    let a = engine().assets();
    assert_eq!(
        r.assets[0].path,
        PathBuf::from("site").join(&a.css.filename)
    );
    let out = fs.get("site/a/index.html").unwrap();
    assert!(
        out.contains(&format!(
            "href=\"https://cdn.example/static/{}\"",
            a.css.filename
        )),
        "{out}"
    );
    assert!(out.contains(&format!(
        "src=\"https://cdn.example/static/{}\"",
        a.js.filename
    )));
    assert!(!out.contains("?v="));
}

#[test]
fn skip_reasons_unlabeled_and_oversized() {
    let page = "<body><pre><code>no lang</code></pre><pre class=\"kz-x\"></pre><pre data-kazari=\"ignore\"><code>x</code></pre><div class=\"highlight\"><span>not code</span></div><pre>no code element</pre></body>";
    let big = PAGE.repeat(3);
    let fs = MemFs::with(&[("site/index.html", page), ("site/big.html", &big)]);
    let mut cfg = config(&fs);
    cfg.skip_unlabeled = true;
    cfg.max_file_bytes = (page.len() + 1) as u64;
    let r = Processor::new(cfg).run(Path::new("site")).unwrap();
    let big = r
        .files
        .iter()
        .find(|f| f.path.ends_with("big.html"))
        .unwrap();
    assert!(
        big.error.as_deref().unwrap().contains("over the"),
        "{:?}",
        big.error
    );
    let index = r
        .files
        .iter()
        .find(|f| f.path.ends_with("index.html"))
        .unwrap();
    assert_eq!(index.blocks_found, 4);
    assert_eq!(index.blocks_rewritten, 0);
    assert_eq!(index.suppressed, 1);
    assert_eq!(
        index.blocks_skipped,
        vec![
            "unlabeled",
            "data-kazari-ignore",
            "unrecognized-shape",
            "unrecognized-shape"
        ]
    );
    assert!(!index.changed);
    assert!(
        fs.writes
            .lock()
            .unwrap()
            .iter()
            .all(|p| !p.ends_with("index.html"))
    );
}

#[test]
fn wrapper_without_match_reoffers_inner_block_once() {
    let page = "<body><div class=\"highlight-widget\"><p>hi</p><pre><code class=\"language-go\">package main</code></pre></div></body>";
    let fs = MemFs::with(&[("site/index.html", page)]);
    let r = Processor::new(config(&fs)).run(Path::new("site")).unwrap();
    assert_eq!(r.files[0].blocks_found, 2);
    assert_eq!(r.files[0].blocks_rewritten, 1);
    let out = fs.get("site/index.html").unwrap();
    assert!(
        out.contains("<div class=\"highlight-widget\"><p>hi</p><div class=\"kazari-block"),
        "{out}"
    );
    assert!(
        out.contains("</div></div><script src=\"kazari.js?v="),
        "{out}"
    );
}

#[test]
fn asset_files_are_not_processed_as_pages() {
    let fs = MemFs::with(&[
        ("site/kazari.css", "old"),
        ("site/kazari.js.html", "<pre>x</pre>"),
    ]);
    let r = Processor::new(config(&fs)).run(Path::new("site")).unwrap();
    assert_eq!(r.assets[0].action, AssetAction::Updated);
    assert_eq!(r.files.len(), 1);
}

fn copy_tree(from: &Path, to: &Path) {
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let dest = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            std::fs::create_dir_all(&dest).unwrap();
            copy_tree(&entry.path(), &dest);
        } else {
            std::fs::copy(entry.path(), dest).unwrap();
        }
    }
}

fn read_tree(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut out: Vec<_> = super::fs::OsFs
        .walk_files(root)
        .unwrap()
        .into_iter()
        .map(|p| {
            (
                p.strip_prefix(root).unwrap().to_path_buf(),
                std::fs::read(&p).unwrap(),
            )
        })
        .collect();
    out.sort();
    out
}

/// Processes each corpus case end to end on the real filesystem, compares every file
/// against the committed golden tree, then runs a second time asserting a complete no-op.
/// Set KAZARI_UPDATE_GOLDEN=1 to regenerate goldens after an intentional rendering change;
/// review them by eye before committing, since the corpus is the spec.
#[test]
fn corpus_matches_goldens_and_reruns_are_noops() {
    let update = std::env::var("KAZARI_UPDATE_GOLDEN").as_deref() == Ok("1");
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/corpus");
    let mut cases: Vec<_> = std::fs::read_dir(&corpus)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    cases.sort();
    assert_eq!(cases.len(), 5);
    let mem = MemFs::default();
    for case in cases {
        let name = case.file_name().unwrap().to_string_lossy().into_owned();
        let tmp = tempfile::tempdir().unwrap();
        copy_tree(&case.join("input"), tmp.path());
        let cfg = Config {
            fs: &super::fs::OsFs,
            ..config(&mem)
        };
        let r = Processor::new(cfg).run(tmp.path()).unwrap();
        for f in &r.files {
            assert!(
                f.error.is_none(),
                "{name}: {}: {:?}",
                f.path.display(),
                f.error
            );
        }
        assert!(tmp.path().join("kazari.css").is_file(), "{name}: css asset");
        assert!(tmp.path().join("kazari.js").is_file(), "{name}: js asset");

        let golden = case.join("golden");
        let got: Vec<_> = read_tree(tmp.path())
            .into_iter()
            .filter(|(p, _)| p.extension().is_some_and(|e| e == "html"))
            .collect();
        if update {
            let _ = std::fs::remove_dir_all(&golden);
            for (rel, bytes) in &got {
                let dest = golden.join(rel);
                std::fs::create_dir_all(dest.parent().unwrap()).unwrap();
                std::fs::write(dest, bytes).unwrap();
            }
        }
        let want = read_tree(&golden);
        assert_eq!(got.len(), want.len(), "{name}: file count");
        for ((gp, gb), (wp, wb)) in got.iter().zip(&want) {
            assert_eq!(gp, wp, "{name}: path");
            assert!(gb == wb, "{name}: {} differs from golden", gp.display());
        }

        let cfg = Config {
            fs: &super::fs::OsFs,
            ..config(&mem)
        };
        let r = Processor::new(cfg).run(tmp.path()).unwrap();
        assert_eq!(r.changed_count, 0, "{name}: second run changed files");
        assert!(
            r.assets.iter().all(|a| a.action == AssetAction::Unchanged),
            "{name}"
        );
    }
}
