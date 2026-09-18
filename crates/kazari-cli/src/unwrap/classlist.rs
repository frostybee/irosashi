use crate::html::Attr;

pub fn attr_value<'a>(attrs: &'a [Attr], key: &str) -> Option<&'a str> {
    attrs
        .iter()
        .find(|a| a.key == key)
        .map(|a| a.value.as_str())
}

pub fn class_list(attrs: &[Attr]) -> impl Iterator<Item = &str> {
    attr_value(attrs, "class")
        .into_iter()
        .flat_map(|v| v.split_ascii_whitespace())
}

/// Exact token match only: `language-css` must not satisfy a check for `language-c`.
pub fn has_class(attrs: &[Attr], name: &str) -> bool {
    class_list(attrs).any(|c| c == name)
}

/// The first class token carrying the prefix, prefix included.
pub fn class_with_prefix<'a>(attrs: &'a [Attr], prefix: &str) -> Option<&'a str> {
    class_list(attrs).find(|c| c.starts_with(prefix) && c.len() > prefix.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attrs(class: &str) -> Vec<Attr> {
        vec![Attr {
            key: "class".into(),
            value: class.into(),
        }]
    }

    #[test]
    fn exact_tokens() {
        let a = attrs("highlight language-css  chroma");
        assert!(has_class(&a, "language-css"));
        assert!(!has_class(&a, "language-c"));
        assert!(!has_class(&a, "high"));
        assert_eq!(class_with_prefix(&a, "language-"), Some("language-css"));
        assert_eq!(class_with_prefix(&attrs("language-"), "language-"), None);
        assert_eq!(class_with_prefix(&[], "x"), None);
        assert!(!has_class(&[], "x"));
    }

    #[test]
    fn attr_lookup() {
        let a = vec![
            Attr {
                key: "data-lang".into(),
                value: "go".into(),
            },
            Attr {
                key: "class".into(),
                value: String::new(),
            },
        ];
        assert_eq!(attr_value(&a, "data-lang"), Some("go"));
        assert_eq!(attr_value(&a, "class"), Some(""));
        assert_eq!(attr_value(&a, "id"), None);
    }
}
