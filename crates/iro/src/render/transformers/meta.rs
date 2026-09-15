use std::num::ParseIntError;

use crate::render::Node;
use crate::render::transformer::Transformer;
use crate::token::{LineRange, in_ranges};

/// Parses the `{1,3-5,7}` part of a code fence meta string. Text outside the braces
/// is ignored; no braces means no ranges.
pub fn parse_meta_ranges(meta: &str) -> Result<Vec<LineRange>, ParseIntError> {
    let meta = meta.trim();
    let Some(start) = meta.find('{') else {
        return Ok(Vec::new());
    };
    let Some(end) = meta.rfind('}') else {
        return Ok(Vec::new());
    };
    if end <= start {
        return Ok(Vec::new());
    }
    let mut ranges = Vec::new();
    for part in meta[start + 1..end].split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        match part.split_once('-') {
            Some((s, e)) => ranges.push(LineRange::new(s.trim().parse()?, e.trim().parse()?)),
            None => ranges.push(LineRange::single(part.parse()?)),
        }
    }
    Ok(ranges)
}

/// Adds `highlighted` to the lines named by a fence meta string such as `{1,3-5}`.
/// A meta string that does not parse highlights nothing.
#[derive(Debug, Clone, Default)]
pub struct Meta {
    ranges: Vec<LineRange>,
}

impl Meta {
    pub fn new(meta: &str) -> Self {
        Self {
            ranges: parse_meta_ranges(meta).unwrap_or_default(),
        }
    }

    pub fn ranges(&self) -> &[LineRange] {
        &self.ranges
    }
}

impl Transformer for Meta {
    fn name(&self) -> &str {
        "meta"
    }

    fn line(&mut self, el: &mut Node, line: usize) {
        if in_ranges(&self.ranges, line) {
            el.push_class("highlighted");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_singles_ranges_and_ignores_text_outside_braces() {
        assert_eq!(
            parse_meta_ranges("js {1, 3-5 ,7} title=x").unwrap(),
            [
                LineRange::single(1),
                LineRange::new(3, 5),
                LineRange::single(7)
            ]
        );
        assert_eq!(parse_meta_ranges("{}").unwrap(), []);
        assert_eq!(parse_meta_ranges("no braces").unwrap(), []);
        assert_eq!(parse_meta_ranges("}{").unwrap(), []);
        assert_eq!(parse_meta_ranges("").unwrap(), []);
        assert!(parse_meta_ranges("{1,x}").is_err());
        assert!(Meta::new("{1,x}").ranges().is_empty());
        assert_eq!(Meta::new("{2}").ranges(), [LineRange::single(2)]);
    }
}
