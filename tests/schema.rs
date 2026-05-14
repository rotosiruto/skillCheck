//! 課題 2 のスキーマバリデーション。

use skillcheck::{Config, Schema, SchemaParseError, Type, ValidationError};

const SCHEMA: &str = "\
endpoint = string
debug = bool
log.file = string
retry = integer
ratio = float
";

fn schema() -> Schema {
    SCHEMA.parse().expect("valid schema")
}

#[test]
fn validate_passes_for_well_formed_config() {
    let cfg: Config = "\
endpoint = localhost:3000
debug = true
log.file = /var/log/console.log
retry = 3
ratio = 0.5
"
    .parse()
    .unwrap();

    schema().validate(&cfg).expect("should pass");
}

#[test]
fn validate_collects_missing_keys() {
    let cfg: Config = "\
endpoint = localhost:3000
debug = true
"
    .parse()
    .unwrap();

    let errors = schema().validate(&cfg).unwrap_err();
    assert!(errors.contains(&ValidationError::MissingKey {
        key: "log.file".into()
    }));
    assert!(errors.contains(&ValidationError::MissingKey {
        key: "retry".into()
    }));
    assert!(errors.contains(&ValidationError::MissingKey {
        key: "ratio".into()
    }));
}

#[test]
fn validate_rejects_unknown_keys() {
    let cfg: Config = "\
endpoint = localhost:3000
debug = true
log.file = /var/log/x
retry = 1
ratio = 1.0
extra = nope
"
    .parse()
    .unwrap();

    let errors = schema().validate(&cfg).unwrap_err();
    assert!(errors.contains(&ValidationError::UnknownKey {
        key: "extra".into()
    }));
}

#[test]
fn validate_detects_type_mismatch_for_integer() {
    let cfg: Config = "\
endpoint = localhost
debug = true
log.file = /x
retry = abc
ratio = 0.0
"
    .parse()
    .unwrap();

    let errors = schema().validate(&cfg).unwrap_err();
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::TypeMismatch {
            key,
            expected: Type::Integer,
            actual_value
        } if key == "retry" && actual_value == "abc"
    )));
}

#[test]
fn validate_detects_type_mismatch_for_bool() {
    let cfg: Config = "\
endpoint = localhost
debug = maybe
log.file = /x
retry = 1
ratio = 0.0
"
    .parse()
    .unwrap();

    let errors = schema().validate(&cfg).unwrap_err();
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::TypeMismatch {
            expected: Type::Bool,
            ..
        }
    )));
}

#[test]
fn validate_detects_type_mismatch_for_float() {
    let cfg: Config = "\
endpoint = localhost
debug = true
log.file = /x
retry = 1
ratio = nope
"
    .parse()
    .unwrap();

    let errors = schema().validate(&cfg).unwrap_err();
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::TypeMismatch {
            expected: Type::Float,
            ..
        }
    )));
}

#[test]
fn validate_detects_shape_mismatch_when_data_is_scalar_for_namespace() {
    let cfg: Config = "\
endpoint = localhost
debug = true
log = oops
retry = 1
ratio = 0.0
"
    .parse()
    .unwrap();

    let errors = schema().validate(&cfg).unwrap_err();
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::ShapeMismatch {
            key,
            expected_kind: "map",
            actual_kind: "scalar"
        } if key == "log"
    )));
}

#[test]
fn validate_detects_shape_mismatch_when_data_is_map_for_scalar() {
    let s: Schema = "endpoint = string".parse().unwrap();
    let cfg: Config = "endpoint.host = localhost".parse().unwrap();

    let errors = s.validate(&cfg).unwrap_err();
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::ShapeMismatch {
            key,
            expected_kind: "scalar",
            actual_kind: "map"
        } if key == "endpoint"
    )));
}

#[test]
fn schema_parse_rejects_unknown_type_name() {
    let err = "key = somethingweird".parse::<Schema>().unwrap_err();
    match err {
        SchemaParseError::UnknownType { type_name } => assert_eq!(type_name, "somethingweird"),
        other => panic!("expected UnknownType, got {other:?}"),
    }
}

#[test]
fn schema_parse_propagates_syntax_error() {
    let err = "broken-line".parse::<Schema>().unwrap_err();
    assert!(matches!(err, SchemaParseError::Parse(_)));
}

#[test]
fn validate_collects_all_errors_at_once() {
    let cfg: Config = "\
debug = maybe
log.file = /x
retry = abc
ratio = bad
extra = ignored
"
    .parse()
    .unwrap();

    let errors = schema().validate(&cfg).unwrap_err();

    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::MissingKey { key } if key == "endpoint"
    )));
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::TypeMismatch {
            expected: Type::Bool,
            ..
        }
    )));
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::TypeMismatch {
            expected: Type::Integer,
            ..
        }
    )));
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::TypeMismatch {
            expected: Type::Float,
            ..
        }
    )));
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::UnknownKey { key } if key == "extra"
    )));
}
