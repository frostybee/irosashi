/// Checks the resolved theme pair against the bundled set before any engine work happens,
/// so a typo produces a clear message instead of a silently degraded page.
pub fn validate_theme_names(names: &[String], light: &str, dark: &str) -> Result<(), String> {
    for name in [light, dark] {
        if names.iter().any(|n| n == name) {
            continue;
        }
        let mut msg = format!("unknown theme {name:?}");
        if let Some(s) = nearest_name(name, names) {
            msg.push_str(&format!(", did you mean {s:?}?"));
        }
        return Err(format!(
            "{msg} Run \"kazari themes\" to list all bundled themes."
        ));
    }
    Ok(())
}

/// The closest bundled name within an edit distance of two: the typo cases worth
/// suggesting without ever proposing something wild.
pub fn nearest_name<'a>(name: &str, names: &'a [String]) -> Option<&'a str> {
    let mut best = None;
    let mut best_dist = 3;
    for n in names {
        let d = edit_distance(name, n, best_dist);
        if d < best_dist {
            best = Some(n.as_str());
            best_dist = d;
        }
    }
    best
}

fn edit_distance(a: &str, b: &str, bound: usize) -> usize {
    let a = a.as_bytes();
    let b = b.as_bytes();
    if a.len().abs_diff(b.len()) >= bound {
        return bound;
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0; b.len() + 1];
    for i in 1..=a.len() {
        cur[0] = i;
        let mut row_min = cur[0];
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
            row_min = row_min.min(cur[j]);
        }
        if row_min >= bound {
            return bound;
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names() -> Vec<String> {
        ["github-dark", "github-light", "dracula", "nord"]
            .into_iter()
            .map(String::from)
            .collect()
    }

    #[test]
    fn suggests_close_names_only() {
        assert_eq!(nearest_name("github-drak", &names()), Some("github-dark"));
        assert_eq!(nearest_name("nrod", &names()), Some("nord"));
        assert_eq!(nearest_name("solarized", &names()), None);
        assert_eq!(nearest_name("github-dark", &names()), Some("github-dark"));
    }

    #[test]
    fn validate_reports_first_bad_name() {
        assert!(validate_theme_names(&names(), "github-light", "github-dark").is_ok());
        let err = validate_theme_names(&names(), "github-light", "github-drak").unwrap_err();
        assert!(err.contains("did you mean \"github-dark\"?"), "{err}");
        assert!(err.ends_with("Run \"kazari themes\" to list all bundled themes."));
        let err = validate_theme_names(&names(), "zzz", "github-dark").unwrap_err();
        assert_eq!(
            err,
            "unknown theme \"zzz\" Run \"kazari themes\" to list all bundled themes."
        );
    }

    #[test]
    fn edit_distance_respects_bound() {
        assert_eq!(edit_distance("kitten", "sitting", 10), 3);
        assert_eq!(edit_distance("kitten", "sitting", 3), 3);
        assert_eq!(edit_distance("a", "a", 3), 0);
        assert_eq!(edit_distance("abc", "abcdef", 3), 3);
    }
}
