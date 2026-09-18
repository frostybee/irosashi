/// Replaces `src[start..end]` with `replacement`. An insertion has `start == end`. All
/// offsets are positions in the original file bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    pub start: usize,
    pub end: usize,
    pub replacement: Vec<u8>,
}

/// Builds the output in a single forward pass over one ascending edit list, copying
/// verbatim between edits. Insertions sort before a replacement starting at the same
/// offset, so a tag injected exactly at a region boundary lands outside the replaced span.
/// Overlapping edits are an internal invariant violation and abort the file rather than
/// corrupt it.
pub fn apply_edits(src: &[u8], edits: &[Edit]) -> Result<Vec<u8>, String> {
    let mut sorted: Vec<&Edit> = edits.iter().collect();
    sorted.sort_by(|a, b| a.start.cmp(&b.start).then(a.end.cmp(&b.end)));

    let mut out = Vec::with_capacity(src.len());
    let mut pos = 0;
    for e in sorted {
        if e.start < pos || e.end < e.start || e.end > src.len() {
            return Err(format!(
                "overlapping or out of range edit [{}:{}] at position {pos}",
                e.start, e.end
            ));
        }
        out.extend_from_slice(&src[pos..e.start]);
        out.extend_from_slice(&e.replacement);
        pos = e.end;
    }
    out.extend_from_slice(&src[pos..]);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edit(start: usize, end: usize, r: &str) -> Edit {
        Edit {
            start,
            end,
            replacement: r.as_bytes().to_vec(),
        }
    }

    #[test]
    fn replaces_inserts_and_keeps_the_rest() {
        let src = b"0123456789";
        let out = apply_edits(src, &[edit(7, 9, "X"), edit(2, 4, "ab"), edit(5, 5, "+")]).unwrap();
        assert_eq!(out, b"01ab4+56X9");
    }

    #[test]
    fn insertion_at_region_start_lands_outside() {
        let out = apply_edits(b"abcd", &[edit(1, 3, "R"), edit(1, 1, "I")]).unwrap();
        assert_eq!(out, b"aIRd");
    }

    #[test]
    fn overlap_and_range_errors() {
        assert!(apply_edits(b"abcd", &[edit(0, 3, ""), edit(2, 4, "")]).is_err());
        assert!(apply_edits(b"abcd", &[edit(3, 2, "")]).is_err());
        assert!(apply_edits(b"abcd", &[edit(2, 9, "")]).is_err());
        assert_eq!(apply_edits(b"abcd", &[]).unwrap(), b"abcd");
    }
}
