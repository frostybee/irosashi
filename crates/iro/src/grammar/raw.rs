use serde::{Deserialize, Deserializer};
use serde_json::Value;

/// A grammar file as written, with the shape leniency vscode-textmate tolerates.
/// Unknown keys are ignored; malformed captures, repository entries and injections
/// are skipped rather than failing the whole grammar.
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct RawGrammar {
    pub scope_name: Option<String>,
    pub name: Option<String>,
    pub patterns: Vec<RawRule>,
    #[serde(deserialize_with = "de_repository")]
    pub repository: RawRepository,
    #[serde(deserialize_with = "de_injections")]
    pub injections: RawInjections,
    pub injection_selector: Option<String>,
    pub inject_to: Vec<String>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct RawRule {
    pub name: Option<String>,
    pub content_name: Option<String>,
    #[serde(rename = "match")]
    pub match_: Option<String>,
    pub begin: Option<String>,
    pub end: Option<String>,
    #[serde(rename = "while")]
    pub while_: Option<String>,
    pub include: Option<String>,
    #[serde(deserialize_with = "de_captures")]
    pub captures: RawCaptures,
    #[serde(deserialize_with = "de_captures")]
    pub begin_captures: RawCaptures,
    #[serde(deserialize_with = "de_captures")]
    pub end_captures: RawCaptures,
    #[serde(deserialize_with = "de_captures")]
    pub while_captures: RawCaptures,
    pub patterns: Vec<RawRule>,
    #[serde(deserialize_with = "de_repository")]
    pub repository: RawRepository,
    #[serde(deserialize_with = "de_bool_or_int")]
    pub apply_end_pattern_last: bool,
}

/// Capture group number to rule, in source order.
#[derive(Default)]
pub(crate) struct RawCaptures(pub Vec<(u32, RawRule)>);

/// Repository key to rule, in source order.
#[derive(Default)]
pub(crate) struct RawRepository(pub Vec<(String, RawRule)>);

/// Injection selector to rule, in source order.
#[derive(Default)]
pub(crate) struct RawInjections(pub Vec<(String, RawRule)>);

fn de_captures<'de, D: Deserializer<'de>>(deserializer: D) -> Result<RawCaptures, D::Error> {
    let mut out = Vec::new();
    match Value::deserialize(deserializer)? {
        Value::Object(map) => {
            for (key, value) in map {
                let Ok(index) = key.parse::<u32>() else {
                    continue;
                };
                if let Some(rule) = capture_rule(value) {
                    out.push((index, rule));
                }
            }
        }
        Value::Array(items) => {
            for (index, value) in items.into_iter().enumerate() {
                if let Some(rule) = capture_rule(value) {
                    out.push((index as u32, rule));
                }
            }
        }
        _ => {}
    }
    Ok(RawCaptures(out))
}

fn capture_rule(value: Value) -> Option<RawRule> {
    match value {
        Value::String(name) => Some(RawRule {
            name: Some(name),
            ..RawRule::default()
        }),
        Value::Object(_) => serde_json::from_value(value).ok(),
        _ => None,
    }
}

fn de_repository<'de, D: Deserializer<'de>>(deserializer: D) -> Result<RawRepository, D::Error> {
    let mut out = Vec::new();
    if let Value::Object(map) = Value::deserialize(deserializer)? {
        for (key, value) in map {
            let rule = match value {
                Value::Object(_) => serde_json::from_value(value).ok(),
                Value::Array(items) => {
                    let patterns = items
                        .into_iter()
                        .filter_map(|item| serde_json::from_value(item).ok())
                        .collect();
                    Some(RawRule {
                        patterns,
                        ..RawRule::default()
                    })
                }
                _ => None,
            };
            if let Some(rule) = rule {
                out.push((key, rule));
            }
        }
    }
    Ok(RawRepository(out))
}

fn de_injections<'de, D: Deserializer<'de>>(deserializer: D) -> Result<RawInjections, D::Error> {
    let mut out = Vec::new();
    if let Value::Object(map) = Value::deserialize(deserializer)? {
        for (selector, value) in map {
            if let Ok(rule) = serde_json::from_value(value) {
                out.push((selector, rule));
            }
        }
    }
    Ok(RawInjections(out))
}

fn de_bool_or_int<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    Ok(match Value::deserialize(deserializer)? {
        Value::Bool(value) => value,
        Value::Number(number) => number.as_f64().is_some_and(|n| n != 0.0),
        _ => false,
    })
}
