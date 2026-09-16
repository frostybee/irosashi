/// FNV-1a 32-bit as eight lowercase hex digits.
pub fn fnv1a32(s: &str) -> String {
    let mut h: u32 = 0x811c_9dc5;
    for b in s.bytes() {
        h ^= u32::from(b);
        h = h.wrapping_mul(0x0100_0193);
    }
    format!("{h:08x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_vectors() {
        assert_eq!(fnv1a32(""), "811c9dc5");
        assert_eq!(fnv1a32("a"), "e40c292c");
        assert_eq!(fnv1a32("foobar"), "bf9cf968");
    }
}
