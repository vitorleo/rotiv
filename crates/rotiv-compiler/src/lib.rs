//! rotiv-compiler — stub for Phase 3.
pub mod error;

pub use error::CompilerError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiler_error_stub() {
        let err = CompilerError::NotImplemented("tsx transform".to_string());
        assert!(err.to_string().contains("Not implemented"));
    }
}
