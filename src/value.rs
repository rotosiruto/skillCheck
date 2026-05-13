//! Value type for parsed configuration entries.

use std::collections::BTreeMap;

/// A configuration value: a leaf string or a nested namespace.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// The right-hand side of `key = value`.
    String(String),
    /// A nested namespace produced by dotted keys.
    Map(BTreeMap<String, Value>),
}

impl Value {
    /// Returns the inner string when this is a leaf, otherwise `None`.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s.as_str()),
            Self::Map(_) => None,
        }
    }

    /// True for [`Value::String`].
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    /// True for [`Value::Map`].
    pub fn is_map(&self) -> bool {
        matches!(self, Self::Map(_))
    }
}
