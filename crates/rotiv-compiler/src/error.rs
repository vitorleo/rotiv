use thiserror::Error;

/// Compiler error type — stub for Phase 3.
#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}
