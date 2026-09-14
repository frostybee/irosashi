use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Frame {
    #[default]
    Auto,
    Code,
    Terminal,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum MarkerType {
    Mark = 0,
    Del = 1,
    Ins = 2,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

    pub fn parse_override(s: &str) -> Self {
        if let Some((light, dark)) = s.split_once(',') {
            let light = light.trim();
            let dark = dark.trim();
            if !dark.is_empty() {
                return Self::dual(light, dark);
            }
            return Self::single(light);
        }
        Self::single(s.trim())
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
        assert!(MarkerType::Mark < MarkerType::Del);
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
    fn themes_parse_override() {
        let single = Themes::parse_override("monokai");
        assert_eq!(single.light, "monokai");
        assert!(single.dark.is_none());

        let dual = Themes::parse_override("github-light, github-dark");
        assert_eq!(dual.light, "github-light");
        assert_eq!(dual.dark.as_deref(), Some("github-dark"));

        let trailing = Themes::parse_override("foo,");
        assert_eq!(trailing.light, "foo");
        assert!(trailing.dark.is_none());
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
