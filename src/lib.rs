//! Parser for the [`sysctl.conf(5)`] file format.
//!
//! [`sysctl.conf(5)`]: https://man7.org/linux/man-pages/man5/sysctl.conf.5.html
//!
//! # Syntax
//!
//! - `key = value`. The first `=` is the separator; the rest is the value.
//! - Whitespace around the key, the `=` and the value is stripped.
//! - Lines whose first non-whitespace character is `#` or `;` are
//!   comments. Blank lines are ignored.
//! - A leading `-` on a key (sysctl's "ignore-errors" marker) is stripped.
//! - Keys may contain `.`; each `.` introduces a nested namespace, so
//!   `log.file = x` becomes `{"log": {"file": "x"}}`.
//! - The last assignment wins for a given leaf key. Mixing a leaf and a
//!   namespace under the same key produces [`ParseError::KeyConflict`].
//!
//! # Example
//!
//! ```
//! use skillcheck::Config;
//!
//! // `##` inside the string is rustdoc's escape for a literal `#`.
//! let input = "\
//! endpoint = localhost:3000
//! ## debug = true
//! log.file = /var/log/console.log
//! log.name = default.log
//! ";
//!
//! let cfg: Config = input.parse().unwrap();
//! assert_eq!(cfg.get_str("endpoint"), Some("localhost:3000"));
//! assert_eq!(cfg.get_str("log.file"), Some("/var/log/console.log"));
//! assert_eq!(cfg.get_str("log.name"), Some("default.log"));
//! assert_eq!(cfg.get_str("debug"),    None);
//! ```
//!
//! ```no_run
//! use skillcheck::Config;
//!
//! let cfg = Config::from_file("/etc/sysctl.conf")?;
//! # Ok::<_, skillcheck::ParseError>(())
//! ```

pub mod config;
pub mod error;
pub mod parser;
pub mod schema;
pub mod value;

pub use config::Config;
pub use error::ParseError;
pub use parser::{parse_file, parse_str};
pub use schema::{Schema, SchemaParseError, Type, ValidationError};
pub use value::Value;
