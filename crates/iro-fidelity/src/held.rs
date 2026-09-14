use std::fs;
use std::io;
use std::path::Path;

/// Reads `held.toml`, a file of the form `held = ["grammar", ...]` listing grammars
/// allowed to be non-identical. Comments (`#`) and blank lines are ignored.
pub fn load_held(path: &Path) -> io::Result<Vec<String>> {
    let text = fs::read_to_string(path)?;
    parse_held(&text).map_err(|msg| io::Error::new(io::ErrorKind::InvalidData, msg))
}

pub fn parse_held(text: &str) -> Result<Vec<String>, String> {
    let mut held = Vec::new();
    let mut found = false;
    for raw in text.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let Some(rest) = line.strip_prefix("held") else {
            return Err(format!("unexpected line: {raw}"));
        };
        let rest = rest.trim_start();
        let Some(rest) = rest.strip_prefix('=') else {
            return Err(format!("expected `held = [...]`, got: {raw}"));
        };
        let rest = rest.trim();
        let inner = rest
            .strip_prefix('[')
            .and_then(|r| r.strip_suffix(']'))
            .ok_or_else(|| format!("expected a list: {raw}"))?;
        for item in inner.split(',') {
            let item = item.trim();
            if item.is_empty() {
                continue;
            }
            let name = item
                .strip_prefix('"')
                .and_then(|i| i.strip_suffix('"'))
                .ok_or_else(|| format!("expected a quoted name: {item}"))?;
            held.push(name.to_owned());
        }
        found = true;
    }
    if !found {
        return Err("missing `held = [...]`".to_owned());
    }
    held.sort();
    held.dedup();
    Ok(held)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_lists_and_comments() {
        assert_eq!(parse_held("held = []\n").unwrap(), Vec::<String>::new());
        assert_eq!(
            parse_held("# note\nheld = [\"po\", \"vue\", \"po\"] # trailing\n").unwrap(),
            ["po", "vue"]
        );
        assert!(parse_held("").is_err());
        assert!(parse_held("held = [po]").is_err());
        assert!(parse_held("other = []").is_err());
    }
}
