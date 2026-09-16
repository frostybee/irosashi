mod lazy;
mod options;
mod raw;
mod rewrite;
mod scanner;

pub use lazy::LazyRegex;
pub use options::{AnchorActive, SearchOptions};
pub use rewrite::rewrite_z_anchor;
pub use scanner::Match;
pub(crate) use scanner::{CaptureBuf, PatternId, PatternTable, ScanStats, Scanner};

#[cfg(test)]
mod tests;
