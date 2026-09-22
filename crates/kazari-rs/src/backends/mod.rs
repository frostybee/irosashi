//! Backends implementing [`Highlighter`](crate::Highlighter), one per Cargo
//! feature.

#[cfg(feature = "irosashi")]
pub mod irosashi;
#[cfg(feature = "syntect")]
pub mod syntect;
