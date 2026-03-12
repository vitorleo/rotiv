use rotiv_core::RotivError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("{0}")]
    Rotiv(#[from] RotivError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[allow(dead_code)]
    #[error("{0}")]
    Other(String),
}

impl CliError {
    pub fn to_rotiv_error(&self) -> RotivError {
        match self {
            CliError::Rotiv(e) => e.clone(),
            CliError::Io(e) => RotivError::new("E_IO", e.to_string()),
            CliError::Other(msg) => RotivError::new("E_UNKNOWN", msg.clone()),
        }
    }
}
