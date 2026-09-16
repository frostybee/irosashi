pub mod contrast;
mod matcher;
mod normalize;
mod parse;

use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};

use bitflags::bitflags;

bitflags! {
    /// Font style bits, matching vscode-textmate's numeric encoding.
    /// `empty()` is "no styling"; "not set" is `Option<FontStyle>::None`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct FontStyle: u8 {
        const ITALIC = 1;
        const BOLD = 2;
        const UNDERLINE = 4;
        const STRIKETHROUGH = 8;
    }
}

impl FontStyle {
    pub fn is_italic(self) -> bool {
        self.contains(Self::ITALIC)
    }

    pub fn is_bold(self) -> bool {
        self.contains(Self::BOLD)
    }

    pub fn is_underline(self) -> bool {
        self.contains(Self::UNDERLINE)
    }

    pub fn is_strikethrough(self) -> bool {
        self.contains(Self::STRIKETHROUGH)
    }
}

/// Identity of a theme instance, unique for the life of the process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ThemeId(u64);

/// Index into a theme's color table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColorId(pub(crate) u32);

impl ColorId {
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

/// The settings one theme rule contributes. `None` means the rule does not set that
/// property, so a lower-scoring rule may still provide it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TokenSettings {
    pub foreground: Option<ColorId>,
    pub background: Option<ColorId>,
    pub font_style: Option<FontStyle>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenColor {
    pub scopes: Vec<String>,
    pub settings: TokenSettings,
    selectors: Vec<matcher::CompiledSelector>,
}

#[derive(Debug)]
pub struct Theme {
    id: ThemeId,
    pub name: String,
    pub display_name: String,
    pub kind: String,
    pub(crate) colors: BTreeMap<String, String>,
    pub(crate) token_colors: Vec<TokenColor>,
    color_table: Vec<String>,
    color_ids: HashMap<String, ColorId>,
    default_foreground: ColorId,
    default_background: ColorId,
}

static NEXT_THEME_ID: AtomicU64 = AtomicU64::new(1);

impl Theme {
    pub(crate) fn next_id() -> ThemeId {
        ThemeId(NEXT_THEME_ID.fetch_add(1, Ordering::Relaxed))
    }

    pub fn id(&self) -> ThemeId {
        self.id
    }

    pub fn color(&self, id: ColorId) -> &str {
        &self.color_table[id.0 as usize]
    }

    pub fn default_foreground_id(&self) -> ColorId {
        self.default_foreground
    }

    pub fn default_background_id(&self) -> ColorId {
        self.default_background
    }

    pub fn default_foreground(&self) -> &str {
        self.color(self.default_foreground)
    }

    pub fn default_background(&self) -> &str {
        self.color(self.default_background)
    }

    pub fn colors(&self) -> &BTreeMap<String, String> {
        &self.colors
    }

    pub fn token_color_count(&self) -> usize {
        self.token_colors.len()
    }

    fn intern_color(&mut self, color: &str) -> ColorId {
        if let Some(&id) = self.color_ids.get(color) {
            return id;
        }
        let id = ColorId(self.color_table.len() as u32);
        self.color_table.push(color.to_owned());
        self.color_ids.insert(color.to_owned(), id);
        id
    }
}
