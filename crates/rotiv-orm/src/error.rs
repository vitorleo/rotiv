use thiserror::Error;

/// Errors from the rotiv-orm crate.
#[derive(Debug, Error)]
pub enum OrmError {
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Script not found: {0}")]
    ScriptNotFound(String),

    #[error("Failed to spawn Node.js process: {0}")]
    SpawnFailed(String),

    #[error("Migration failed: {0}")]
    MigrationFailed(String),

    #[error("Failed to parse script output: {0}")]
    ParseFailed(String),

    #[error("{0} pending migration(s) need to be applied")]
    PendingMigrations(u32),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
