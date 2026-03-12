use thiserror::Error;

/// ORM error type — stub for Phase 4.
#[derive(Debug, Error)]
pub enum OrmError {
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}
