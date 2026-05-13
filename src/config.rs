//! High-level [`Config`] wrapper with dotted-path lookup.

use std::collections::BTreeMap;
use std::path::Path;
use std::str::FromStr;

use crate::error::ParseError;
use crate::parser;
use crate::value::Value;

/// A parsed sysctl.conf-style configuration.
///
/// ```
/// use skillcheck::Config;
///
/// let cfg: Config = "log.file = /var/log/console.log".parse().unwrap();
/// assert_eq!(cfg.get_str("log.file"), Some("/var/log/console.log"));
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Config {
    root: BTreeMap<String, Value>,
}

impl Config {
    /// Load from a file on disk.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ParseError> {
        Ok(Self {
            root: parser::parse_file(path)?,
        })
    }

    /// Borrow the root map.
    pub fn root(&self) -> &BTreeMap<String, Value> {
        &self.root
    }

    /// Look up a value by a dotted path (`"log.file"`).
    ///
    /// Returns `None` when any segment is missing or when a non-final
    /// segment is a leaf string instead of a namespace.
    pub fn get(&self, path: &str) -> Option<&Value> {
        let mut segments = path.split('.');
        let head = segments.next()?;
        let mut cursor = self.root.get(head)?;
        for segment in segments {
            cursor = match cursor {
                Value::Map(m) => m.get(segment)?,
                Value::String(_) => return None,
            };
        }
        Some(cursor)
    }

    /// Look up a string value by a dotted path.
    pub fn get_str(&self, path: &str) -> Option<&str> {
        self.get(path)?.as_str()
    }

    /// Number of top-level entries.
    pub fn len(&self) -> usize {
        self.root.len()
    }

    /// True when there are no entries.
    pub fn is_empty(&self) -> bool {
        self.root.is_empty()
    }
}

impl FromStr for Config {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            root: parser::parse_str(s)?,
        })
    }
}
