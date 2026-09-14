const FALLBACK: &str = "#000000";

/// Applies the default-color fallback: a missing or invalid color becomes `#000000`.
pub fn default_color(value: Option<&str>) -> String {
    match value {
        Some(color) if is_valid_color(color) => color.to_owned(),
        _ => FALLBACK.to_owned(),
    }
}

pub fn is_valid_color(value: &str) -> bool {
    let Some(hex) = value.trim().strip_prefix('#') else {
        return false;
    };
    matches!(hex.len(), 3 | 4 | 6 | 8) && hex.bytes().all(|b| b.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validity() {
        assert!(is_valid_color("#abc"));
        assert!(is_valid_color("#ABCD"));
        assert!(is_valid_color(" #aabbcc "));
        assert!(is_valid_color("#aabbccdd"));
        assert!(!is_valid_color("aabbcc"));
        assert!(!is_valid_color("#aabbc"));
        assert!(!is_valid_color("#gggggg"));
        assert!(!is_valid_color(""));
    }

    #[test]
    fn fallback() {
        assert_eq!(default_color(None), "#000000");
        assert_eq!(default_color(Some("nope")), "#000000");
        assert_eq!(default_color(Some("#123456")), "#123456");
    }
}
