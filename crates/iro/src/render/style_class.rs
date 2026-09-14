use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;

/// Replaces inline `style` attributes with hashed class names and collects the
/// stylesheet. Share one map across every block of a page so equal styles share a
/// class.
#[derive(Debug, Default, Clone)]
pub struct StyleClassMap {
    by_canon: HashMap<String, String>,
    rules: BTreeMap<String, String>,
}

impl StyleClassMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// The class name for a set of declarations, registering it on first use.
    pub fn get(&mut self, props: &[(String, String)]) -> &str {
        let canon = canonical(props);
        if !self.by_canon.contains_key(&canon) {
            let class = format!("_s_{:x}", fnv1a64(canon.as_bytes()));
            self.rules.insert(class.clone(), rule_body(props));
            self.by_canon.insert(canon.clone(), class);
        }
        &self.by_canon[&canon]
    }

    /// One rule per class, sorted by class name, each on its own line.
    pub fn css(&self) -> String {
        let mut out = String::new();
        for (class, body) in &self.rules {
            let _ = writeln!(out, ".{class} {{ {body} }}");
        }
        out
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

fn sorted(props: &[(String, String)]) -> Vec<&(String, String)> {
    let mut v: Vec<_> = props.iter().collect();
    v.sort_by(|a, b| a.0.cmp(&b.0));
    v
}

fn canonical(props: &[(String, String)]) -> String {
    sorted(props)
        .iter()
        .map(|(k, v)| format!("{k}:{v}"))
        .collect::<Vec<_>>()
        .join(";")
}

fn rule_body(props: &[(String, String)]) -> String {
    sorted(props)
        .iter()
        .map(|(k, v)| format!("{k}: {v}"))
        .collect::<Vec<_>>()
        .join("; ")
}

pub(crate) fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    fn props(list: &[(&str, &str)]) -> Vec<(String, String)> {
        list.iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect()
    }

    #[test]
    fn fnv_vectors() {
        assert_eq!(fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a64(b"a"), 0xaf63_dc4c_8601_ec8c);
    }

    #[test]
    fn class_is_stable_prefixed_and_key_order_independent() {
        let mut m = StyleClassMap::new();
        let a = m
            .get(&props(&[("color", "#ff0000"), ("font-style", "italic")]))
            .to_owned();
        let b = m
            .get(&props(&[("font-style", "italic"), ("color", "#ff0000")]))
            .to_owned();
        let c = m.get(&props(&[("color", "#00ff00")])).to_owned();
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert!(a.starts_with("_s_"));
        assert_eq!(m.len(), 2);
        let mut other = StyleClassMap::new();
        assert_eq!(
            other.get(&props(&[("color", "#ff0000"), ("font-style", "italic")])),
            a
        );
    }

    #[test]
    fn css_lists_sorted_rules_with_spaced_declarations() {
        let mut m = StyleClassMap::new();
        let red = m.get(&props(&[("color", "#ff0000")])).to_owned();
        let green = m
            .get(&props(&[("font-weight", "bold"), ("color", "#00ff00")]))
            .to_owned();
        let mut expected: Vec<(String, String)> = vec![
            (red.clone(), format!(".{red} {{ color: #ff0000 }}\n")),
            (
                green.clone(),
                format!(".{green} {{ color: #00ff00; font-weight: bold }}\n"),
            ),
        ];
        expected.sort();
        let joined: String = expected.into_iter().map(|(_, line)| line).collect();
        assert_eq!(m.css(), joined);
        assert!(StyleClassMap::new().is_empty());
    }
}
