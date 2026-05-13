//! Low-level parsing functions. Most callers should use [`crate::Config`].

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::error::ParseError;
use crate::value::Value;

/// Parse a sysctl.conf-style string into a nested [`BTreeMap`].
pub fn parse_str(input: &str) -> Result<BTreeMap<String, Value>, ParseError> {
    let mut root: BTreeMap<String, Value> = BTreeMap::new();

    for (idx, raw) in input.lines().enumerate() {
        let line_no = idx + 1;
        let trimmed = raw.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
            continue;
        }

        // Strip the sysctl "ignore-errors" marker if present.
        let body = trimmed.strip_prefix('-').unwrap_or(trimmed);

        let Some(eq_pos) = body.find('=') else {
            return Err(ParseError::Syntax {
                line: line_no,
                message: "expected '=' separator".into(),
            });
        };

        let raw_key = body[..eq_pos].trim();
        let raw_value = body[eq_pos + 1..].trim();

        if raw_key.is_empty() {
            return Err(ParseError::Syntax {
                line: line_no,
                message: "empty key".into(),
            });
        }

        let segments: Vec<&str> = raw_key.split('.').collect();
        if segments.iter().any(|s| s.is_empty()) {
            return Err(ParseError::Syntax {
                line: line_no,
                message: format!("invalid key '{raw_key}': empty segment"),
            });
        }

        insert_nested(&mut root, &segments, raw_value, line_no)?;
    }

    Ok(root)
}

/// Read a file and parse its contents.
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<BTreeMap<String, Value>, ParseError> {
    let contents = fs::read_to_string(path.as_ref())?;
    parse_str(&contents)
}

fn insert_nested(
    map: &mut BTreeMap<String, Value>,
    segments: &[&str],
    value: &str,
    line_no: usize,
) -> Result<(), ParseError> {
    let (head, rest) = segments.split_first().expect("segments is non-empty");

    if rest.is_empty() {
        if let Some(Value::Map(_)) = map.get(*head) {
            return Err(ParseError::KeyConflict {
                line: line_no,
                key: (*head).to_owned(),
            });
        }
        map.insert((*head).to_owned(), Value::String(value.to_owned()));
        Ok(())
    } else {
        let entry = map
            .entry((*head).to_owned())
            .or_insert_with(|| Value::Map(BTreeMap::new()));

        match entry {
            Value::Map(inner) => insert_nested(inner, rest, value, line_no),
            Value::String(_) => Err(ParseError::KeyConflict {
                line: line_no,
                key: (*head).to_owned(),
            }),
        }
    }
}
