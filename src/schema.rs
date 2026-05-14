//! 型スキーマと、それによる Config の検証。

use std::collections::BTreeMap;
use std::path::Path;
use std::str::FromStr;

use thiserror::Error;

use crate::config::Config;
use crate::error::ParseError;
use crate::parser;
use crate::value::Value;

/// スキーマで宣言できる値の型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    String,
    Bool,
    Integer,
    Float,
}

impl FromStr for Type {
    type Err = SchemaParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "string" => Ok(Self::String),
            "bool" => Ok(Self::Bool),
            "integer" => Ok(Self::Integer),
            "float" => Ok(Self::Float),
            other => Err(SchemaParseError::UnknownType {
                type_name: other.to_owned(),
            }),
        }
    }
}

/// スキーマファイル自体のロード/解析時に起きるエラー。
#[derive(Debug, Error)]
pub enum SchemaParseError {
    /// 下層のパーサーからのエラー（I/O や文法）。
    #[error("schema parse error: {0}")]
    Parse(#[from] ParseError),

    /// 知らない型名が宣言されていた。
    #[error("unknown type '{type_name}'")]
    UnknownType { type_name: String },
}

/// バリデーション時の違反。`validate` は失敗するとこれを `Vec` で返す。
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {
    /// schema にあるが data にない。
    #[error("missing key '{key}'")]
    MissingKey { key: String },

    /// data にあるが schema にない。
    #[error("unknown key '{key}' (not declared in schema)")]
    UnknownKey { key: String },

    /// 値が宣言された型に変換できない。
    #[error("type mismatch at '{key}': expected {expected:?}, got '{actual_value}'")]
    TypeMismatch {
        key: String,
        expected: Type,
        actual_value: String,
    },

    /// 構造のミスマッチ（schema は map なのに data はスカラー、など）。
    #[error("shape mismatch at '{key}': expected {expected_kind}, got {actual_kind}")]
    ShapeMismatch {
        key: String,
        expected_kind: &'static str,
        actual_kind: &'static str,
    },
}

/// パースされたスキーマ。`validate(&Config)` で検証する。
#[derive(Debug, Clone)]
pub struct Schema {
    nodes: BTreeMap<String, Node>,
}

#[derive(Debug, Clone)]
enum Node {
    Leaf(Type),
    Nested(BTreeMap<String, Node>),
}

impl Schema {
    /// ファイルから読み込んで解析する。
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, SchemaParseError> {
        let raw = parser::parse_file(path)?;
        Self::from_value_map(&raw)
    }

    /// Config を検証する。違反があれば全件 `Vec` で返す。
    pub fn validate(&self, config: &Config) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        validate_map(&self.nodes, config.root(), "", &mut errors);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn from_value_map(map: &BTreeMap<String, Value>) -> Result<Self, SchemaParseError> {
        let mut nodes = BTreeMap::new();
        for (key, value) in map {
            nodes.insert(key.clone(), value_to_node(value)?);
        }
        Ok(Self { nodes })
    }
}

impl FromStr for Schema {
    type Err = SchemaParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let raw = parser::parse_str(s)?;
        Self::from_value_map(&raw)
    }
}

fn value_to_node(value: &Value) -> Result<Node, SchemaParseError> {
    match value {
        Value::String(type_name) => Ok(Node::Leaf(type_name.parse::<Type>()?)),
        Value::Map(m) => {
            let mut inner = BTreeMap::new();
            for (k, v) in m {
                inner.insert(k.clone(), value_to_node(v)?);
            }
            Ok(Node::Nested(inner))
        }
    }
}

fn validate_map(
    schema: &BTreeMap<String, Node>,
    data: &BTreeMap<String, Value>,
    prefix: &str,
    errors: &mut Vec<ValidationError>,
) {
    let join = |key: &str| -> String {
        if prefix.is_empty() {
            key.to_owned()
        } else {
            format!("{prefix}.{key}")
        }
    };

    for (key, node) in schema {
        let path = join(key);
        match data.get(key) {
            None => report_missing_leaves(node, &path, errors),
            Some(value) => validate_node(node, value, &path, errors),
        }
    }

    for key in data.keys() {
        if !schema.contains_key(key) {
            errors.push(ValidationError::UnknownKey { key: join(key) });
        }
    }
}

fn validate_node(node: &Node, value: &Value, path: &str, errors: &mut Vec<ValidationError>) {
    match (node, value) {
        (Node::Leaf(ty), Value::String(s)) => {
            if !type_matches(*ty, s) {
                errors.push(ValidationError::TypeMismatch {
                    key: path.to_owned(),
                    expected: *ty,
                    actual_value: s.clone(),
                });
            }
        }
        (Node::Leaf(_), Value::Map(_)) => errors.push(ValidationError::ShapeMismatch {
            key: path.to_owned(),
            expected_kind: "scalar",
            actual_kind: "map",
        }),
        (Node::Nested(inner), Value::Map(m)) => validate_map(inner, m, path, errors),
        (Node::Nested(_), Value::String(_)) => errors.push(ValidationError::ShapeMismatch {
            key: path.to_owned(),
            expected_kind: "map",
            actual_kind: "scalar",
        }),
    }
}

/// schema の枝が data に丸ごと存在しない時、配下の leaf すべてを欠損として報告する。
fn report_missing_leaves(node: &Node, path: &str, errors: &mut Vec<ValidationError>) {
    match node {
        Node::Leaf(_) => errors.push(ValidationError::MissingKey {
            key: path.to_owned(),
        }),
        Node::Nested(inner) => {
            for (k, child) in inner {
                let sub_path = format!("{path}.{k}");
                report_missing_leaves(child, &sub_path, errors);
            }
        }
    }
}

fn type_matches(ty: Type, s: &str) -> bool {
    match ty {
        Type::String => true,
        Type::Bool => matches!(s, "true" | "false"),
        Type::Integer => s.parse::<i64>().is_ok(),
        Type::Float => s.parse::<f64>().is_ok(),
    }
}
