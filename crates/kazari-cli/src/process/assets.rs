use std::path::Path;

use super::{AssetAction, AssetResult, Processor};

/// The engine's stylesheet and script, cached once per run so every page references byte
/// identical content under one hash.
#[derive(Debug, Default, Clone)]
pub struct AssetInfo {
    pub css_name: String,
    pub js_name: String,
    pub css_content: Vec<u8>,
    pub js_content: Vec<u8>,
    pub css_hash: String,
    pub js_hash: String,
}

impl Processor<'_> {
    pub(super) fn build_assets(&self) -> AssetInfo {
        let a = self.cfg.engine.assets();
        let mut info = AssetInfo {
            css_content: a.css.content.into_bytes(),
            js_content: a.js.content.into_bytes(),
            css_hash: a.css.hash,
            js_hash: a.js.hash,
            css_name: "kazari.css".to_owned(),
            js_name: "kazari.js".to_owned(),
        };
        if self.cfg.hashed_assets {
            info.css_name = a.css.filename;
            info.js_name = a.js.filename;
        }
        info
    }

    /// Emits both asset files at the output root, write if different. Runs before the
    /// per file pass so no page ever references content that is not yet on disk.
    pub(super) fn write_assets(&self, root: &Path) -> Vec<AssetResult> {
        let entries = [
            (&self.assets.css_name, &self.assets.css_content),
            (&self.assets.js_name, &self.assets.js_content),
        ];
        let mut out = Vec::new();
        for (name, content) in entries {
            let path = root.join(name);
            let action = match self.fs.read_file(&path) {
                Ok(existing) if existing == *content => AssetAction::Unchanged,
                Ok(_) => AssetAction::Updated,
                Err(_) => AssetAction::Created,
            };
            if action != AssetAction::Unchanged
                && !self.cfg.check
                && let Err(e) = self.fs.write_file(&path, content)
            {
                self.log(format!("kazari process: writing {}: {e}", path.display()));
            }
            out.push(AssetResult { path, action });
        }
        out
    }

    /// The URL a page uses to reference an asset: by default a relative path computed from
    /// the page's depth below the output root, built with forward slashes so Windows paths
    /// never leak into an href. Relative paths stay correct under subpath deployments.
    /// `assets_base` overrides with a verbatim prefix. Plain names carry a content hash
    /// query for cache busting; hashed filenames already embed it.
    pub(super) fn asset_href(&self, rel_path: &Path, name: &str, hash: &str) -> String {
        let href = if !self.cfg.assets_base.is_empty() {
            format!("{}/{name}", self.cfg.assets_base.trim_end_matches('/'))
        } else {
            let depth = rel_path.components().count().saturating_sub(1);
            format!("{}{name}", "../".repeat(depth))
        };
        if self.cfg.hashed_assets {
            href
        } else {
            format!("{href}?v={hash}")
        }
    }
}

pub fn link_tag(href: &str) -> String {
    format!("<link rel=\"stylesheet\" href=\"{href}\" data-kazari=\"assets\">")
}

pub fn script_tag(href: &str) -> String {
    format!("<script src=\"{href}\" data-kazari=\"assets\"></script>")
}
