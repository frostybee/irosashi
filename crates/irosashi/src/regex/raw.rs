//! The regex engine behind every pattern. `ferroni`, a pure Rust port of
//! Oniguruma, is the default; the `onig` feature uses C Oniguruma instead, and wins
//! when both are enabled.

#[cfg(all(feature = "ferroni", not(feature = "onig")))]
#[path = "raw/ferroni.rs"]
mod engine;
#[cfg(feature = "onig")]
#[path = "raw/onig.rs"]
mod engine;

#[cfg(not(any(feature = "ferroni", feature = "onig")))]
compile_error!("irosashi needs a regex engine: enable the `ferroni` or `onig` feature");

pub(crate) use engine::{Regex, Region};

/// Oniguruma's search option bits, which both engines share.
pub(crate) const OPTION_NONE: u32 = 0;
pub(crate) const OPTION_NOT_BEGIN_STRING: u32 = 1 << 22;
pub(crate) const OPTION_NOT_BEGIN_POSITION: u32 = 1 << 24;

pub(crate) enum Search {
    Found,
    NotFound,
    Failed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiles_searches_and_reports_errors() {
        let re = Regex::new(r"(\w+)(?:\s+(\d+))?").unwrap();
        let mut region = Region::new();
        assert!(matches!(
            re.search("hi 42", 0, 0, &mut region),
            Search::Found
        ));
        assert_eq!(region.len(), 3);
        assert_eq!(region.pos(0), Some((0, 5)));
        assert_eq!(region.pos(2), Some((3, 5)));
        assert!(matches!(re.search("hi", 1, 0, &mut region), Search::Found));
        assert_eq!(region.pos(2), None);
        assert!(matches!(
            re.search("  ", 0, 0, &mut region),
            Search::NotFound
        ));
        let err = match Regex::new("(") {
            Ok(_) => panic!("unbalanced paren compiled"),
            Err(err) => err,
        };
        assert!(err.contains("end pattern"), "{err}");
    }

    #[test]
    fn not_begin_options_are_honoured() {
        let re = Regex::new(r"\G\w").unwrap();
        let mut region = Region::new();
        assert!(matches!(re.search("abc", 1, 0, &mut region), Search::Found));
        assert!(matches!(
            re.search("abc", 1, OPTION_NOT_BEGIN_POSITION, &mut region),
            Search::NotFound
        ));
    }
}
