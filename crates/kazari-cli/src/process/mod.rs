//! Post-processes a static site generator's built HTML output: finds code blocks in each
//! page, recovers their source through the unwrap layer, re-renders them with a shared
//! engine, splices the results back while preserving every other byte, and emits the
//! engine's CSS and JS once with injected link and script tags.

mod assets;
pub mod fs;
mod splice;

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

use kazari_rs::Kazari;

use crate::html;
use crate::unwrap::scan::{self, AssetKind, Candidate, CandidateKind};
use crate::unwrap::{self, BARE_PRE_CHAIN, DIV_WRAPPER_CHAIN};
use assets::{AssetInfo, link_tag, script_tag};
use fs::FileSystem;
use splice::{Edit, apply_edits};

const DEFAULT_MAX_FILE_BYTES: u64 = 32 << 20;

pub type Logger = Box<dyn Fn(&str) + Sync>;

pub struct Config<'a> {
    pub engine: &'a Kazari,
    /// Reports what would change without writing anything.
    pub check: bool,
    /// Leaves blocks with no detectable language untouched instead of rendering them as
    /// plain text frames.
    pub skip_unlabeled: bool,
    /// Prefixes asset URLs verbatim instead of the default per file relative path.
    pub assets_base: String,
    /// Content hashed filenames instead of kazari.css and kazari.js. Old hashed files from
    /// previous runs are not cleaned up.
    pub hashed_assets: bool,
    /// Worker count; zero means the available parallelism.
    pub concurrency: usize,
    /// Files larger than this are skipped; zero means 32 MiB. Token dense pages inflate
    /// far beyond their byte size in memory.
    pub max_file_bytes: u64,
    /// Progress and warnings; `None` is silent.
    pub logger: Option<Logger>,
    pub fs: &'a dyn FileSystem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetAction {
    Created,
    Updated,
    Unchanged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetResult {
    pub path: PathBuf,
    pub action: AssetAction,
}

#[derive(Debug, Default)]
pub struct FileResult {
    pub path: PathBuf,
    pub blocks_found: usize,
    pub blocks_rewritten: usize,
    pub blocks_skipped: Vec<&'static str>,
    pub suppressed: usize,
    pub changed: bool,
    pub error: Option<String>,
}

#[derive(Debug, Default)]
pub struct RunResult {
    pub files: Vec<FileResult>,
    pub assets: Vec<AssetResult>,
    pub suppressed: usize,
    pub changed_count: usize,
}

pub struct Processor<'a> {
    cfg: Config<'a>,
    fs: &'a dyn FileSystem,
    assets: AssetInfo,
}

impl<'a> Processor<'a> {
    pub fn new(mut cfg: Config<'a>) -> Self {
        if cfg.concurrency == 0 {
            cfg.concurrency = std::thread::available_parallelism().map_or(1, |n| n.get());
        }
        if cfg.max_file_bytes == 0 {
            cfg.max_file_bytes = DEFAULT_MAX_FILE_BYTES;
        }
        let fs = cfg.fs;
        Processor {
            cfg,
            fs,
            assets: AssetInfo::default(),
        }
    }

    fn log(&self, msg: String) {
        if let Some(l) = &self.cfg.logger {
            l(&msg);
        }
    }

    /// Processes every HTML file under `root`. Assets are written first so no page ever
    /// references content that is not yet on disk. Files are processed concurrently, each
    /// worker writing only its own result slot; paths are sorted so output ordering is
    /// deterministic.
    pub fn run(&mut self, root: &Path) -> Result<RunResult, String> {
        self.assets = self.build_assets();

        let mut result = RunResult {
            assets: self.write_assets(root),
            ..Default::default()
        };
        result.changed_count += result
            .assets
            .iter()
            .filter(|a| a.action != AssetAction::Unchanged)
            .count();

        let mut paths = self
            .fs
            .walk_files(root)
            .map_err(|e| format!("walking {}: {e}", root.display()))?;
        paths.retain(|p| self.should_process(p));
        paths.sort();

        let slots: Vec<Mutex<Option<FileResult>>> =
            paths.iter().map(|_| Mutex::new(None)).collect();
        let next = AtomicUsize::new(0);
        let workers = self.cfg.concurrency.min(paths.len().max(1));
        let this = &*self;
        std::thread::scope(|s| {
            for _ in 0..workers {
                s.spawn(|| {
                    loop {
                        let i = next.fetch_add(1, Ordering::Relaxed);
                        if i >= paths.len() {
                            break;
                        }
                        let fr = this.process_file(root, &paths[i]);
                        *slots[i].lock().unwrap() = Some(fr);
                    }
                });
            }
        });

        result.files = slots
            .into_iter()
            .map(|m| m.into_inner().unwrap().expect("every slot is filled"))
            .collect();
        for fr in &result.files {
            if fr.changed {
                result.changed_count += 1;
            }
            result.suppressed += fr.suppressed;
        }
        Ok(result)
    }

    /// Keeps the walk to HTML pages and the emitted asset files out of the pipeline.
    fn should_process(&self, path: &Path) -> bool {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase());
        if !matches!(ext.as_deref(), Some("html") | Some("htm")) {
            return false;
        }
        let base = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        base != self.assets.css_name && base != self.assets.js_name
    }

    fn process_file(&self, root: &Path, path: &Path) -> FileResult {
        let mut fr = FileResult {
            path: path.to_path_buf(),
            ..Default::default()
        };

        let src = match self.fs.read_file(path) {
            Ok(s) => s,
            Err(e) => {
                fr.error = Some(e.to_string());
                return fr;
            }
        };
        if src.len() as u64 > self.cfg.max_file_bytes {
            fr.error = Some(format!(
                "{} is {} bytes, over the {} byte limit",
                path.display(),
                src.len(),
                self.cfg.max_file_bytes
            ));
            self.log(format!(
                "kazari process: skipping oversized file {}",
                path.display()
            ));
            return fr;
        }
        let tokens = html::tokenize(&src);
        let page = scan::discover(&src, &tokens);
        fr.suppressed = page.suppressed;
        if page.suppressed_unclosed {
            self.log(format!(
                "kazari process: {}: an unclosed kazari block scope suppressed the rest of the file",
                path.display()
            ));
        }

        let mut edits = Vec::new();
        for c in &page.candidates {
            self.handle_candidate(path, c, true, &mut fr, &mut edits);
        }

        let rel = path
            .strip_prefix(root)
            .map(Path::to_path_buf)
            .unwrap_or_else(|_| PathBuf::from(path.file_name().unwrap_or_default()));
        let css_href = self.asset_href(&rel, &self.assets.css_name, &self.assets.css_hash);
        let js_href = self.asset_href(&rel, &self.assets.js_name, &self.assets.js_hash);
        let (link, script) = (link_tag(&css_href), script_tag(&js_href));

        // Existing tags are always reconciled to canonical form, whether or not any block
        // in this file was rewritten; a config only change must still refresh stale cache
        // busting hashes. Only injecting a brand new tag is gated on a rewritten block.
        let (mut have_link, mut have_script) = (false, false);
        for tag in &page.asset_tags {
            let canonical = match tag.kind {
                AssetKind::Link => {
                    have_link = true;
                    &link
                }
                AssetKind::Script => {
                    have_script = true;
                    &script
                }
            };
            if src[tag.byte_start..tag.byte_end] != *canonical.as_bytes() {
                edits.push(Edit {
                    start: tag.byte_start,
                    end: tag.byte_end,
                    replacement: canonical.as_bytes().to_vec(),
                });
            }
        }
        if fr.blocks_rewritten > 0 {
            if !have_link {
                edits.push(Edit {
                    start: page.link_insert,
                    end: page.link_insert,
                    replacement: link.into_bytes(),
                });
            }
            if !have_script {
                edits.push(Edit {
                    start: page.script_insert,
                    end: page.script_insert,
                    replacement: script.into_bytes(),
                });
            }
        }

        if edits.is_empty() {
            return fr;
        }
        let out = match apply_edits(&src, &edits) {
            Ok(o) => o,
            Err(e) => {
                fr.error = Some(e.clone());
                self.log(format!(
                    "kazari process: {} left untouched: {e}",
                    path.display()
                ));
                return fr;
            }
        };
        if out == src {
            return fr;
        }
        fr.changed = true;
        if self.cfg.check {
            return fr;
        }
        if let Err(e) = self.fs.write_file(path, &out) {
            fr.error = Some(e.to_string());
        }
        fr
    }

    fn handle_candidate(
        &self,
        path: &Path,
        c: &Candidate<'_>,
        allow_reoffer: bool,
        fr: &mut FileResult,
        edits: &mut Vec<Edit>,
    ) {
        fr.blocks_found += 1;
        if c.malformed {
            fr.blocks_skipped.push("malformed");
            self.log(format!(
                "kazari process: {}: unclosed element at byte {} left untouched",
                path.display(),
                c.byte_start
            ));
            return;
        }
        if let Some(reason) = unwrap::skip_reason(c.tokens) {
            fr.blocks_skipped.push(reason);
            return;
        }
        let chain = match c.kind {
            CandidateKind::Wrapper => DIV_WRAPPER_CHAIN,
            CandidateKind::BarePre => BARE_PRE_CHAIN,
        };
        let Some((region, _)) = unwrap::run_chain(chain, c.tokens) else {
            // A site component using a trigger class must not swallow a genuine code
            // block nested inside it: re-offer the contents once, one level deep only.
            if c.kind == CandidateKind::Wrapper && allow_reoffer && c.tokens.len() > 2 {
                let inner = scan::discover_within(&c.tokens[1..c.tokens.len() - 1]);
                fr.suppressed += inner.suppressed;
                if !inner.candidates.is_empty() {
                    for ic in &inner.candidates {
                        self.handle_candidate(path, ic, false, fr, edits);
                    }
                    return;
                }
            }
            fr.blocks_skipped.push("unrecognized-shape");
            return;
        };
        if region.lang.is_empty() && self.cfg.skip_unlabeled {
            fr.blocks_skipped.push("unlabeled");
            return;
        }
        match self.cfg.engine.render_with_meta(&region.code, &region.meta) {
            Ok(rendered) => {
                edits.push(Edit {
                    start: c.byte_start,
                    end: c.byte_end,
                    replacement: rendered.into_bytes(),
                });
                fr.blocks_rewritten += 1;
            }
            Err(e) => {
                fr.blocks_skipped.push("render-error");
                self.log(format!(
                    "kazari process: {}: render error, block left untouched: {e}",
                    path.display()
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests;
