/// Search-time options passed to Oniguruma for one `find_next_match` call.
///
/// These are the real Oniguruma option bits (`ONIG_OPTION_NOT_BEGIN_STRING`,
/// `ONIG_OPTION_NOT_BEGIN_POSITION`), not `NOTBOL`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchOptions(onig::SearchOptions);

impl SearchOptions {
    pub const NONE: Self = Self(onig::SearchOptions::SEARCH_OPTION_NONE);

    /// Disables `\A`: the search start is not the beginning of the string.
    pub const NOT_BEGIN_STRING: Self = Self(onig::SearchOptions::SEARCH_OPTION_NOT_BEGIN_STRING);

    /// Disables `\G`: the search start is not the anchor position.
    pub const NOT_BEGIN_POSITION: Self =
        Self(onig::SearchOptions::SEARCH_OPTION_NOT_BEGIN_POSITION);

    pub fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub fn contains(self, other: Self) -> bool {
        self.0.contains(other.0)
    }

    pub(crate) fn into_onig(self) -> onig::SearchOptions {
        self.0
    }
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self::NONE
    }
}

/// Which of the `\A` and `\G` anchors may fire for the current search.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnchorActive {
    /// Both `\A` and `\G` fire: first line, searching at the anchor position.
    AG,
    /// Only `\A` fires: first line, not at the anchor position.
    A,
    /// Only `\G` fires: continuation line, at the anchor position.
    G,
    /// Neither fires.
    None,
}

impl AnchorActive {
    pub fn new(is_first_line: bool, anchor_position: Option<usize>, current_pos: usize) -> Self {
        let at_anchor = anchor_position == Some(current_pos);
        match (is_first_line, at_anchor) {
            (true, true) => Self::AG,
            (true, false) => Self::A,
            (false, true) => Self::G,
            (false, false) => Self::None,
        }
    }

    /// Maps to search options by disabling every anchor that must not fire.
    pub fn to_search_options(self) -> SearchOptions {
        match self {
            Self::AG => SearchOptions::NONE,
            Self::A => SearchOptions::NOT_BEGIN_POSITION,
            Self::G => SearchOptions::NOT_BEGIN_STRING,
            Self::None => SearchOptions::NOT_BEGIN_STRING.union(SearchOptions::NOT_BEGIN_POSITION),
        }
    }
}
