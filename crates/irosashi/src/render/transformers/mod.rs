//! Built-in transformers: `[!code ...]` comment notation, fence meta line ranges and
//! visible whitespace.

pub mod meta;
pub mod notation;
pub mod whitespace;

pub use meta::{Meta, parse_meta_ranges};
pub use notation::Notation;
pub use whitespace::Whitespace;
