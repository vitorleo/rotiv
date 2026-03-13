use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Structured error type shared across all Rotiv crates.
/// Serializes to JSON for the `--json` output mode.
#[derive(Debug, Clone, Serialize, Deserialize, Error)]
#[error("{code}: {message}")]
pub struct RotivError {
    pub code: String,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub expected: Option<String>,
    pub got: Option<String>,
    pub suggestion: Option<String>,
    pub corrected_code: Option<String>,
}

impl RotivError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            file: None,
            line: None,
            expected: None,
            got: None,
            suggestion: None,
            corrected_code: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    pub fn with_file(mut self, file: impl Into<String>, line: Option<u32>) -> Self {
        self.file = Some(file.into());
        self.line = line;
        self
    }

    pub fn with_expected(mut self, expected: impl Into<String>, got: impl Into<String>) -> Self {
        self.expected = Some(expected.into());
        self.got = Some(got.into());
        self
    }
}
