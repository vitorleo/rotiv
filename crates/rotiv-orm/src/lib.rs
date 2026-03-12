//! rotiv-orm — stub for Phase 4.
pub mod error;

pub use error::OrmError;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orm_error_stub() {
        let err = OrmError::NotImplemented("query builder".to_string());
        assert!(err.to_string().contains("Not implemented"));
    }
}
