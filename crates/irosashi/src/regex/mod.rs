mod lazy;
mod options;
mod raw;
mod rewrite;
mod scanner;
mod store;

pub use lazy::LazyRegex;
pub use options::{AnchorActive, SearchOptions};
pub use rewrite::rewrite_z_anchor;
pub use scanner::Match;
pub(crate) use scanner::{CaptureBuf, PatternId, PatternTable, ScanStats, Scanner};
pub(crate) use store::RegexStore;

#[cfg(test)]
mod tests;
