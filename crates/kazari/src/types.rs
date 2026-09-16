use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Frame {
    #[default]
    Auto,
    Code,
    Terminal,
    None,
}

/// Line and inline marker kinds, ordered by the priority that wins when markers
/// overlap: a plain mark loses to warning and error, which lose to diff markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum MarkerType {
    Mark = 0,
    Warning = 1,
    Error = 2,
    Del = 3,
    Ins = 4,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DarkMode {
    Selector(String),
    MediaQuery,
    Both(String),
}

impl Default for DarkMode {
    fn default() -> Self {
        Self::Selector(".dark".to_owned())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TerminalDotStyle {
    #[default]
    Colored,
    Minimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LineRange {
    pub start: usize,
    pub end: usize,
}

impl LineRange {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn single(line: usize) -> Self {
        Self::new(line, line)
    }

    pub fn contains(&self, line: usize) -> bool {
        line >= self.start && line <= self.end
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineMarker {
    pub marker_type: MarkerType,
    pub lines: Vec<LineRange>,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineMarker {
    pub marker_type: MarkerType,
    pub text: String,
    pub is_regex: bool,
}

/// A link extracted from `@[text](url)` syntax: byte offsets of the visible text
/// within the cleaned line, and the target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkAnnotation {
    pub start: usize,
    pub end: usize,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ThemeInfo {
    pub fg: String,
    pub bg: String,
    pub selection_bg: String,
    pub line_number_fg: String,
    pub fold_bg: String,
}

impl ThemeInfo {
    pub fn from_iro(tc: &iro::ThemeColors) -> Self {
        Self {
            fg: tc.foreground.clone(),
            bg: tc.background.clone(),
            selection_bg: tc.selection_background.clone().unwrap_or_default(),
            line_number_fg: tc
                .colors
                .get("editorLineNumber.foreground")
                .cloned()
                .unwrap_or_default(),
            fold_bg: tc
                .colors
                .get("editor.foldBackground")
                .cloned()
                .unwrap_or_default(),
        }
    }
}

bitflags::bitflags! {
    /// Which extracted theme colours a [`ThemeAdjustments`] tint applies to.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct AdjustTargets: u8 {
        /// Editor, selection and fold backgrounds.
        const BACKGROUNDS = 1 << 0;
        /// Editor foreground and line number colour.
        const FOREGROUNDS = 1 << 1;
    }
}

/// Tints the extracted theme colours in OKLCH space: hue and chroma are replaced
/// when set, lightness and alpha are kept. Empty targets mean backgrounds.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ThemeAdjustments {
    pub hue: Option<f64>,
    pub chroma: Option<f64>,
    pub targets: AdjustTargets,
}

/// Generated asset content with a content hash and a `kazari-<hash>.<ext>` name
/// for cache-busting deployments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetFile {
    pub content: String,
    /// FNV-1a 32 of the content, eight hex digits.
    pub hash: String,
    pub filename: String,
}

impl AssetFile {
    pub(crate) fn new(content: String, ext: &str) -> Self {
        let hash = crate::hash::fnv1a32(&content);
        let filename = format!("kazari-{hash}.{ext}");
        Self {
            content,
            hash,
            filename,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assets {
    pub css: AssetFile,
    pub js: AssetFile,
}

/// What a post-render callback learns about the block it is given.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockInfo {
    /// The highlighted language, after any `diff` language swap.
    pub lang: String,
    /// From the meta or the extracted file name comment.
    pub title: String,
    /// After frame detection.
    pub frame: Frame,
    /// The copy-button text: preprocessed source without output panel text.
    pub raw_code: String,
    /// Number of rendered lines.
    pub line_count: usize,
    /// The raw `theme=` override string, empty when absent.
    pub theme: String,
    /// The raw fence meta string; empty for [`crate::Kazari::render`].
    pub meta: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Themes {
    pub light: String,
    pub dark: Option<String>,
}

impl Themes {
    pub fn single(theme: &str) -> Self {
        Self {
            light: theme.to_owned(),
            dark: None,
        }
    }

    pub fn dual(light: &str, dark: &str) -> Self {
        Self {
            light: light.to_owned(),
            dark: Some(dark.to_owned()),
        }
    }

    pub fn is_dual(&self) -> bool {
        self.dark.is_some()
    }

    #[allow(dead_code)]
    pub(crate) fn to_slot_keys(&self) -> BTreeMap<String, String> {
        let mut map = BTreeMap::new();
        map.insert("light".to_owned(), self.light.clone());
        if let Some(dark) = &self.dark {
            map.insert("dark".to_owned(), dark.clone());
        }
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_type_priority_order() {
        assert!(MarkerType::Mark < MarkerType::Warning);
        assert!(MarkerType::Warning < MarkerType::Error);
        assert!(MarkerType::Error < MarkerType::Del);
        assert!(MarkerType::Del < MarkerType::Ins);
    }

    #[test]
    fn line_range_contains() {
        let r = LineRange::new(3, 7);
        assert!(!r.contains(2));
        assert!(r.contains(3));
        assert!(r.contains(5));
        assert!(r.contains(7));
        assert!(!r.contains(8));
    }

    #[test]
    fn adjust_targets_default_is_empty() {
        assert!(AdjustTargets::default().is_empty());
        let both = AdjustTargets::BACKGROUNDS | AdjustTargets::FOREGROUNDS;
        assert!(both.contains(AdjustTargets::FOREGROUNDS));
    }

    #[test]
    fn theme_info_from_iro() {
        let tc = iro::ThemeColors {
            kind: "dark".into(),
            foreground: "#d4d4d4".into(),
            background: "#1e1e1e".into(),
            selection_background: Some("#264f78".into()),
            line_highlight_background: None,
            colors: {
                let mut m = std::collections::BTreeMap::new();
                m.insert("editorLineNumber.foreground".into(), "#858585".into());
                m
            },
        };
        let info = ThemeInfo::from_iro(&tc);
        assert_eq!(info.fg, "#d4d4d4");
        assert_eq!(info.bg, "#1e1e1e");
        assert_eq!(info.selection_bg, "#264f78");
        assert_eq!(info.line_number_fg, "#858585");
        assert_eq!(info.fold_bg, "");
    }
}
