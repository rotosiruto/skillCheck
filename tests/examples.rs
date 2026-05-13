//! Tests for the two reference inputs in the task.

use std::collections::BTreeMap;

use skillcheck::{Config, Value, parse_str};

#[test]
fn example_1_matches_expected_shape() {
    let input = "\
endpoint = localhost:3000
debug = true
log.file = /var/log/console.log
";

    let actual = parse_str(input).expect("parse should succeed");

    let mut log = BTreeMap::new();
    log.insert(
        "file".to_owned(),
        Value::String("/var/log/console.log".to_owned()),
    );
    let mut expected = BTreeMap::new();
    expected.insert(
        "endpoint".to_owned(),
        Value::String("localhost:3000".to_owned()),
    );
    expected.insert("debug".to_owned(), Value::String("true".to_owned()));
    expected.insert("log".to_owned(), Value::Map(log));

    assert_eq!(actual, expected);
}

#[test]
fn example_2_skips_commented_line() {
    let input = "\
endpoint = localhost:3000
# debug = true
log.file = /var/log/console.log
log.name = default.log
";

    let actual = parse_str(input).expect("parse should succeed");

    let mut log = BTreeMap::new();
    log.insert(
        "file".to_owned(),
        Value::String("/var/log/console.log".to_owned()),
    );
    log.insert("name".to_owned(), Value::String("default.log".to_owned()));
    let mut expected = BTreeMap::new();
    expected.insert(
        "endpoint".to_owned(),
        Value::String("localhost:3000".to_owned()),
    );
    expected.insert("log".to_owned(), Value::Map(log));

    assert_eq!(actual, expected);
    assert!(!actual.contains_key("debug"));
}

#[test]
fn config_wrapper_provides_dotted_lookup() {
    let cfg: Config = "\
endpoint = localhost:3000
log.file = /var/log/console.log
log.name = default.log
"
    .parse()
    .expect("parse should succeed");

    assert_eq!(cfg.get_str("endpoint"), Some("localhost:3000"));
    assert_eq!(cfg.get_str("log.file"), Some("/var/log/console.log"));
    assert_eq!(cfg.get_str("log.name"), Some("default.log"));
    assert!(cfg.get("log").unwrap().is_map());
    assert!(cfg.get("log.file").unwrap().is_string());
    assert_eq!(cfg.get_str("log.missing"), None);
    assert_eq!(cfg.get_str("does.not.exist"), None);
    assert_eq!(cfg.len(), 2);
}
