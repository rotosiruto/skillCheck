//! Syntax edge cases.

use skillcheck::{Config, ParseError, parse_str};

#[test]
fn supports_semicolon_comments() {
    let input = "\
; this is a comment
key = value
";
    let cfg: Config = input.parse().unwrap();
    assert_eq!(cfg.get_str("key"), Some("value"));
}

#[test]
fn whitespace_around_separator_is_trimmed() {
    let input = "    key   =    spaced value   ";
    let cfg: Config = input.parse().unwrap();
    assert_eq!(cfg.get_str("key"), Some("spaced value"));
}

#[test]
fn equals_inside_value_is_preserved() {
    let input = "url = https://example.com/?a=1&b=2";
    let cfg: Config = input.parse().unwrap();
    assert_eq!(cfg.get_str("url"), Some("https://example.com/?a=1&b=2"),);
}

#[test]
fn empty_value_is_allowed() {
    let cfg: Config = "key =".parse().unwrap();
    assert_eq!(cfg.get_str("key"), Some(""));
}

#[test]
fn last_assignment_wins_for_leaf() {
    let input = "\
key = first
key = second
";
    let cfg: Config = input.parse().unwrap();
    assert_eq!(cfg.get_str("key"), Some("second"));
}

#[test]
fn deeply_nested_keys_round_trip() {
    let cfg: Config = "a.b.c.d = leaf".parse().unwrap();
    assert_eq!(cfg.get_str("a.b.c.d"), Some("leaf"));
    assert!(cfg.get("a").unwrap().is_map());
    assert!(cfg.get("a.b").unwrap().is_map());
    assert!(cfg.get("a.b.c").unwrap().is_map());
}

#[test]
fn leading_dash_marker_is_stripped() {
    let cfg: Config = "-net.ipv4.tcp_syncookies = 1".parse().unwrap();
    assert_eq!(cfg.get_str("net.ipv4.tcp_syncookies"), Some("1"));
}

#[test]
fn blank_lines_are_ignored() {
    let input = "\n\nkey = value\n\n\nother = thing\n";
    let cfg: Config = input.parse().unwrap();
    assert_eq!(cfg.get_str("key"), Some("value"));
    assert_eq!(cfg.get_str("other"), Some("thing"));
}

#[test]
fn missing_equals_is_a_syntax_error() {
    let err = parse_str("just a line").unwrap_err();
    match err {
        ParseError::Syntax { line, .. } => assert_eq!(line, 1),
        other => panic!("expected Syntax, got {other:?}"),
    }
}

#[test]
fn empty_key_is_a_syntax_error() {
    let err = parse_str(" = value").unwrap_err();
    assert!(matches!(err, ParseError::Syntax { line: 1, .. }));
}

#[test]
fn empty_segment_is_a_syntax_error() {
    let err = parse_str("a..b = value").unwrap_err();
    assert!(matches!(err, ParseError::Syntax { line: 1, .. }));
}

#[test]
fn key_conflict_namespace_then_value() {
    let err = parse_str("log.file = x\nlog = y").unwrap_err();
    match err {
        ParseError::KeyConflict { line, key } => {
            assert_eq!(line, 2);
            assert_eq!(key, "log");
        }
        other => panic!("expected KeyConflict, got {other:?}"),
    }
}

#[test]
fn key_conflict_value_then_namespace() {
    let err = parse_str("log = y\nlog.file = x").unwrap_err();
    match err {
        ParseError::KeyConflict { line, key } => {
            assert_eq!(line, 2);
            assert_eq!(key, "log");
        }
        other => panic!("expected KeyConflict, got {other:?}"),
    }
}

#[test]
fn empty_input_yields_empty_config() {
    let cfg: Config = "".parse().unwrap();
    assert!(cfg.is_empty());
    assert_eq!(cfg.len(), 0);
}

#[test]
fn comment_only_input_yields_empty_config() {
    let input = "\
# nothing
; nothing else
";
    let cfg: Config = input.parse().unwrap();
    assert!(cfg.is_empty());
}

#[test]
fn syntax_error_reports_correct_line_number() {
    let input = "\
ok = yes
broken-line
also.ok = sure
";
    let err = parse_str(input).unwrap_err();
    match err {
        ParseError::Syntax { line, .. } => assert_eq!(line, 2),
        other => panic!("expected Syntax, got {other:?}"),
    }
}

#[test]
fn missing_file_returns_io_error() {
    let err = skillcheck::parse_file("does/not/exist.conf").unwrap_err();
    assert!(matches!(err, ParseError::Io(_)));
}
