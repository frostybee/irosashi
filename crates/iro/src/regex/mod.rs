mod options;
mod pattern_set;
mod rewrite;

pub use options::{AnchorActive, SearchOptions};
pub use pattern_set::{Match, PatternSet};
pub use rewrite::rewrite_z_anchor;

#[cfg(test)]
mod tests;
