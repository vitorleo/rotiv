use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Failed to spawn build process: {0}")]
    SpawnFailed(String),

    #[error("Build failed: {0}")]
    BuildFailed(String),

    #[error("Build script not found. Set ROTIV_BUILD_SCRIPT_PATH or run from the monorepo: {0}")]
    ScriptNotFound(String),

    #[error("Failed to parse build output: {0}")]
    ParseFailed(String),
}
