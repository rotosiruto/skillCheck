//! Error types.

use std::io;

use thiserror::Error;

/// Errors raised by the parser.
#[derive(Debug, Error)]
pub enum ParseError {
    /// I/O failure when reading from a file.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Malformed line (missing `=`, empty key, ...).
    #[error("syntax error at line {line}: {message}")]
    Syntax {
        /// 1-based line number.
        line: usize,
        /// What was wrong.
        message: String,
    },

    /// A key collides with an existing namespace or value
    /// (e.g. `log = x` followed by `log.file = y`).
    #[error("key conflict at line {line}: '{key}' has an incompatible shape")]
    KeyConflict {
        /// 1-based line number.
        line: usize,
        /// Offending key segment.
        key: String,
    },
}
